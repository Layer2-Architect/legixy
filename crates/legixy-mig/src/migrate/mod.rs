// (module of SRC-LGX-009; anchor: lib.rs)
// legixy-mig::migrate — migrate() エントリ（DD-LGX-009 §3 / §4 MigratorOrchestrator 統括）。
// 移行元 DB = .trace-engine/feedback.db（observations/proposals/custom_edges）
//   → 移行先 .legixy/engine.db（ADR-LGX-015）。
// 確定順序: DB コミット先行 → graph.toml / id-map / config の atomic 確定（REQ.02）。

pub mod config_parse;
pub mod db_schema;
pub mod id_map;
pub mod matrix;
pub mod version;

use std::path::{Path, PathBuf};

use rusqlite::{Connection, OpenFlags};

use crate::atomic::atomic_write;
use crate::backup::backup_file;
use crate::error::MigError;
use crate::report::{MigrateOpts, MigrationReport};

use config_parse::extract_migration_config;
use id_map::{generate_id_map, MigrationIdMap};
use matrix::{parse_matrix, ArtifactIdSet};
use version::{detect_version_raw, ProjectVersion};

/// v0.1.0 → legixy マイグレーション（DD-LGX-009 §3、凍結 API surface）。
/// 移行元が v0.1.0 でない場合 VersionMismatch または no-op（既に legixy、SUPP-008 §2.19）。
/// DB コミット先行 → graph.toml / id-map / config の atomic 確定順（REQ.02）。
/// 移行失敗時は原本保全（REQ.03a）。dry_run=true は一切書き込まない。
pub fn migrate(src: &Path, dst: &Path, opts: MigrateOpts) -> Result<MigrationReport, MigError> {
    crate::fixture::prepare_migrate(src, dst)?;

    // 移行対象不在（代替フロー 2b、REQ.06）。
    if !src.exists() {
        return Err(MigError::V01NotFound {
            path: src.to_path_buf(),
        });
    }

    // バージョン検出。既に legixy → no-op（exit 0 + 空サマリ、DD §6 / SUPP-008 §2.19）。
    let detected = detect_version_raw(src)?;
    if detected.kind == ProjectVersion::Legixy {
        // stderr Info「既に legixy 形式、変更なし」は legixy-cli が 1 回出力（DD §11 §2.18）。
        return Ok(MigrationReport::default());
    }

    // ── 設定抽出（REQ.03/03a：TomlParse / ConfigCorrupt / ChainConfigMissing）──
    let config = extract_migration_config(src)?;

    // ── matrix.md パース（空入力正常、SUPP-008 §2.5）──
    let matrix_path = src.join(&config.matrix_file);
    let matrix_content = std::fs::read_to_string(&matrix_path).unwrap_or_default();
    let artifact_set: ArtifactIdSet = parse_matrix(&matrix_content, &config)?;

    // ── ID マッピング生成（REQ.11：全単射違反は IdBijectionViolation、--dry-run でも検証）──
    let existing_refs: Vec<String> = Vec::new();
    let id_map: MigrationIdMap = generate_id_map(
        &artifact_set,
        &existing_refs,
        &config,
        opts.unmapped_policy,
    )?;

    // ── 出力 graph.toml 生成（旧 ID を新 ID へ書換）+ 妥当性検証 ──
    //    （REQ.03a：OutputGraphInvalid、atomic 確定前。原本無変更）。
    let rewritten = rewrite_to_new_ids(&artifact_set, &id_map);
    validate_output_graph(&rewritten, &config)?;
    let graph_text = render_graph_toml(&rewritten);

    // ── 出力パス（移行先）──
    let graph_out = dst.join("docs/traceability/graph.toml");
    let id_map_out = dst.join(".legixy/migration-id-map.toml");
    let config_out = dst.join(".legixy.toml");
    let engine_db_out = dst.join(".legixy/engine.db");

    let id_map_text = render_id_map_toml(&id_map);
    let config_text = render_migrated_config(&config);

    let mut warnings: Vec<String> = Vec::new();

    // vectors.bin（Phase 2 延期、A 案: Skip + Warning、DD §11 §2.10）。
    let vectors_bin = src.join(".trace-engine/vectors.bin");
    if !vectors_bin.exists() {
        warnings.push(
            "vectors.bin not found; embeddings import skipped (Phase 2 deferred)".to_string(),
        );
    } else {
        warnings.push("vectors.bin import deferred to Phase 2 (skipped)".to_string());
    }

    let files_written = vec![graph_out.clone(), id_map_out.clone(), config_out.clone()];

    // ── dry-run: 一切書き込まない（検証は実施済）。MigrationReport は「予定」を表す ──
    if opts.dry_run {
        return Ok(MigrationReport {
            files_written,
            ids_rewritten_count: id_map.mappings.len(),
            id_map_path: id_map_out,
            backup_paths: Vec::new(),
            warnings,
            tables_copied: Vec::new(),
            rows_copied: 0,
        });
    }

    // ── 退避（既存出力を上書きする前に backup、REQ.02a）──
    let mut backup_paths: Vec<PathBuf> = Vec::new();
    for p in [&graph_out, &config_out] {
        if p.exists() {
            let b = backup_file(p)?;
            backup_paths.push(b.path);
        }
    }

    // ── DB 移行 + コミット先行（REQ.02 確定順序）──
    let (tables_copied, rows_copied) = migrate_feedback_db(src, &engine_db_out)?;

    // ── 平文の atomic 確定（DB コミット後）──
    atomic_write(&config_out, config_text.as_bytes())?;
    atomic_write(&graph_out, graph_text.as_bytes())?;
    atomic_write(&id_map_out, id_map_text.as_bytes())?;

    Ok(MigrationReport {
        files_written,
        ids_rewritten_count: id_map.mappings.len(),
        id_map_path: id_map_out,
        backup_paths,
        warnings,
        tables_copied,
        rows_copied,
    })
}

/// 旧 ID を id_map の新 ID へ書換えた出力 graph 用 ArtifactIdSet を生成する。
/// 対応が無いノードは旧 ID のまま残す（その場合 validate で検出される）。
fn rewrite_to_new_ids(set: &ArtifactIdSet, map: &MigrationIdMap) -> ArtifactIdSet {
    use std::collections::HashMap;
    let lookup: HashMap<&str, &str> = map
        .mappings
        .iter()
        .map(|m| (m.old_id.as_str(), m.new_id.as_str()))
        .collect();
    let items = set
        .items
        .iter()
        .map(|it| {
            let new_id = lookup
                .get(it.id_str.as_str())
                .map(|s| s.to_string())
                .unwrap_or_else(|| it.id_str.clone());
            matrix::ArtifactItem {
                typecode: it.typecode.clone(),
                id_str: new_id,
                path: it.path.clone(),
            }
        })
        .collect();
    ArtifactIdSet { items }
}

/// 出力 graph 妥当性検証（REQ.03a、`TraceGraph::validate()` 相当の発火）。
/// ID 一意性違反 / 凍結 ID 形式不一致は OutputGraphInvalid（atomic 確定前、原本無変更）。
fn validate_output_graph(
    set: &ArtifactIdSet,
    config: &config_parse::MigrationConfig,
) -> Result<(), MigError> {
    let mut seen: std::collections::HashSet<&str> = std::collections::HashSet::new();
    for item in &set.items {
        // ID 一意性（CTX-INV-2 保全）。
        if !seen.insert(item.id_str.as_str()) {
            return Err(MigError::OutputGraphInvalid {
                detail: format!("duplicate node id in output graph: {}", item.id_str),
            });
        }
        // 書換後の新 ID 形式 `{TYPE(2-4)}-{area}-{seq_digits hex}` 整合（壊れた出力を確定しない）。
        if !is_valid_new_id(&item.id_str, config) {
            return Err(MigError::OutputGraphInvalid {
                detail: format!("invalid node id in output graph: {}", item.id_str),
            });
        }
    }
    Ok(())
}

/// 新 ID `{TYPE}-{area}-{seq_digits 桁の 16 進}` 形式（typecode は ICONIX の 2〜4 文字）。
fn is_valid_new_id(id: &str, config: &config_parse::MigrationConfig) -> bool {
    let parts: Vec<&str> = id.split('-').collect();
    if parts.len() != 3 {
        return false;
    }
    let (typecode, area, seq) = (parts[0], parts[1], parts[2]);
    let tc_ok = (2..=4).contains(&typecode.len())
        && typecode.chars().all(|c| c.is_ascii_uppercase());
    let area_ok = area == config.id_area;
    let seq_ok = seq.len() == config.seq_digits && seq.chars().all(|c| c.is_ascii_hexdigit());
    tc_ok && area_ok && seq_ok
}

/// ArtifactIdSet → graph.toml テキスト（ドキュメントノードのみ、サブノードなし REQ.05）。
fn render_graph_toml(set: &ArtifactIdSet) -> String {
    let mut out = String::from("# graph.toml — legixy (migrated from v0.1.0)\n\n");
    for item in &set.items {
        out.push_str("[[nodes]]\n");
        out.push_str(&format!("id = \"{}\"\n", item.id_str));
        out.push_str(&format!("type = \"{}\"\n", item.typecode));
        if let Some(p) = &item.path {
            out.push_str(&format!("path = \"{}\"\n", p));
        }
        out.push('\n');
    }
    out
}

/// MigrationIdMap → migration-id-map.toml テキスト。
fn render_id_map_toml(map: &MigrationIdMap) -> String {
    let mut out = String::from("# migration-id-map.toml — legixy\n\n");
    for m in &map.mappings {
        out.push_str("[[mappings]]\n");
        out.push_str(&format!("old_id = \"{}\"\n", m.old_id));
        out.push_str(&format!("new_id = \"{}\"\n", m.new_id));
        out.push_str("confidence = \"High\"\n\n");
    }
    out
}

/// MigrationConfig → 移行後 .legixy.toml テキスト（[graph] additive 追加、REQ.04）。
fn render_migrated_config(config: &config_parse::MigrationConfig) -> String {
    let mut raw = config.raw_toml.clone();
    if let Some(table) = raw.as_table_mut() {
        // [graph] additive 追加（既存なら維持）。
        if !table.contains_key("graph") {
            let mut g = toml::value::Table::new();
            g.insert(
                "file".to_string(),
                toml::Value::String("docs/traceability/graph.toml".to_string()),
            );
            table.insert("graph".to_string(), toml::Value::Table(g));
        }
        // [semantic].vector_store を削除（REQ.04）。
        if let Some(toml::Value::Table(sem)) = table.get_mut("semantic") {
            sem.remove("vector_store");
        }
    }
    toml::to_string_pretty(&raw).unwrap_or_default()
}

/// feedback.db（observations/proposals/custom_edges）→ engine.db 移行（M-3/M-4、REQ.01/02/03a）。
/// 戻り: (tables_copied, rows_copied)。必須テーブル欠落は SchemaIncompatible（原本無傷）。
fn migrate_feedback_db(
    src: &Path,
    engine_db_out: &Path,
) -> Result<(Vec<String>, usize), MigError> {
    let mut tables_copied: Vec<String> = Vec::new();
    let mut rows_copied: usize = 0;

    let feedback_db = src.join(".trace-engine/feedback.db");
    if !feedback_db.exists() {
        // feedback.db 不在は致命でない（ドキュメントのみ移行）。
        if let Some(parent) = engine_db_out.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let conn = Connection::open(engine_db_out)?;
        db_schema::init_engine_schema(&conn)?;
        conn.pragma_update(None, "user_version", 3)?;
        return Ok((tables_copied, rows_copied));
    }

    // 必須テーブル（observations）欠落は SchemaIncompatible（REQ.03a、原本無変更）。
    let src_conn = Connection::open_with_flags(
        &feedback_db,
        OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_URI,
    )?;
    let obs_cols = table_columns(&src_conn, "observations")?;
    if obs_cols.is_empty() {
        return Err(MigError::SchemaIncompatible {
            table: "observations".to_string(),
            detail: "required table observations not found in feedback.db".to_string(),
        });
    }

    // 移行先 engine.db を準備（コミット先行のため、平文確定前に commit する）。
    if let Some(parent) = engine_db_out.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let mut dst_conn = Connection::open(engine_db_out)?;
    db_schema::init_engine_schema(&dst_conn)?;
    dst_conn.pragma_update(None, "user_version", 3)?;

    // 単一トランザクションで copy（冪等 INSERT OR IGNORE）。
    {
        let tx = dst_conn.transaction()?;
        rows_copied += copy_observations(&src_conn, &tx)?;
        if rows_copied > 0 {
            tables_copied.push("observations".to_string());
        }

        let prop_rows = copy_proposals(&src_conn, &tx)?;
        if prop_rows > 0 {
            tables_copied.push("proposals".to_string());
            rows_copied += prop_rows;
        }

        // custom_edges は存在する場合のみ転記（不在は継承なしで正常、ケース26b）。
        if !table_columns(&src_conn, "custom_edges")?.is_empty() {
            let ce_rows = copy_custom_edges(&src_conn, &tx)?;
            tables_copied.push("custom_edges".to_string());
            rows_copied += ce_rows;
        }

        tx.commit()?; // DB コミット先行（REQ.02 確定順序）。
    }

    Ok((tables_copied, rows_copied))
}

fn table_columns(conn: &Connection, table: &str) -> Result<Vec<String>, MigError> {
    let mut stmt = conn.prepare(&format!("PRAGMA table_info({})", table))?;
    let rows = stmt.query_map([], |r| r.get::<_, String>(1))?;
    let mut cols = Vec::new();
    for r in rows {
        cols.push(r?);
    }
    Ok(cols)
}

fn copy_observations(src: &Connection, dst: &Connection) -> Result<usize, MigError> {
    let mut stmt = src.prepare(
        "SELECT source, category, severity, message, related_ids, context_json, status, created_at \
         FROM observations",
    )?;
    let rows = stmt.query_map([], |r| {
        Ok((
            r.get::<_, String>(0)?,
            r.get::<_, String>(1)?,
            r.get::<_, String>(2)?,
            r.get::<_, String>(3)?,
            r.get::<_, String>(4)?,
            r.get::<_, Option<String>>(5)?,
            r.get::<_, String>(6)?,
            r.get::<_, String>(7)?,
        ))
    })?;
    let mut count = 0usize;
    for row in rows {
        let (source, category, severity, message, related_ids, context_json, status, created_at) =
            row?;
        count += dst.execute(
            "INSERT OR IGNORE INTO observations \
             (source, category, severity, message, related_ids, context_json, status, created_at) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            rusqlite::params![
                source, category, severity, message, related_ids, context_json, status, created_at,
            ],
        )?;
    }
    Ok(count)
}

fn copy_proposals(src: &Connection, dst: &Connection) -> Result<usize, MigError> {
    if table_columns(src, "proposals")?.is_empty() {
        return Ok(0);
    }
    let mut stmt = src.prepare(
        "SELECT observation_id, kind, semantic_key, title, description, action_json, status, \
                decided_at, decided_reason, created_at \
         FROM proposals",
    )?;
    let rows = stmt.query_map([], |r| {
        Ok((
            r.get::<_, Option<i64>>(0)?,
            r.get::<_, String>(1)?,
            r.get::<_, String>(2)?,
            r.get::<_, String>(3)?,
            r.get::<_, String>(4)?,
            r.get::<_, String>(5)?,
            r.get::<_, String>(6)?,
            r.get::<_, Option<String>>(7)?,
            r.get::<_, Option<String>>(8)?,
            r.get::<_, String>(9)?,
        ))
    })?;
    let mut count = 0usize;
    for row in rows {
        let (
            observation_id,
            kind,
            semantic_key,
            title,
            description,
            action_json,
            status,
            decided_at,
            decided_reason,
            created_at,
        ) = row?;
        count += dst.execute(
            "INSERT INTO proposals \
             (observation_id, kind, semantic_key, title, description, action_json, status, \
              decided_at, decided_reason, created_at) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            rusqlite::params![
                observation_id,
                kind,
                semantic_key,
                title,
                description,
                action_json,
                status,
                decided_at,
                decided_reason,
                created_at,
            ],
        )?;
    }
    Ok(count)
}

fn copy_custom_edges(src: &Connection, dst: &Connection) -> Result<usize, MigError> {
    let cols = table_columns(src, "custom_edges")?;
    let (from_col, to_col, reason_col) =
        if cols.iter().any(|c| c == "source_glob") && cols.iter().any(|c| c == "target_path") {
            ("source_glob", "target_path", "description")
        } else if cols.iter().any(|c| c == "from_id") && cols.iter().any(|c| c == "to_id") {
            ("from_id", "to_id", "reason")
        } else {
            return Err(MigError::SchemaIncompatible {
                table: "custom_edges".to_string(),
                detail: "missing (source_glob,target_path) or (from_id,to_id) column pair"
                    .to_string(),
            });
        };
    let has_reason = cols.iter().any(|c| c == reason_col);
    let sql = if has_reason {
        format!("SELECT {}, {}, {} FROM custom_edges", from_col, to_col, reason_col)
    } else {
        format!("SELECT {}, {}, NULL FROM custom_edges", from_col, to_col)
    };
    let mut stmt = src.prepare(&sql)?;
    let rows = stmt.query_map([], |r| {
        Ok((
            r.get::<_, String>(0)?,
            r.get::<_, String>(1)?,
            r.get::<_, Option<String>>(2)?,
        ))
    })?;
    let mut count = 0usize;
    for row in rows {
        let (from_id, to_id, reason) = row?;
        count += dst.execute(
            "INSERT OR IGNORE INTO custom_edges (from_id, to_id, reason) VALUES (?1, ?2, ?3)",
            rusqlite::params![from_id, to_id, reason],
        )?;
    }
    Ok(count)
}
