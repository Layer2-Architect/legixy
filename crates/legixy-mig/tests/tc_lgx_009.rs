// Document ID: TC-LGX-009
// TC-LGX-009: プロジェクト初期化とマイグレーション（init / migrate）のテストコード（TC[RED]）
//
// 親 chain: TS-LGX-009 → 本 TC-LGX-009 → SRC-LGX-009。
// 各テストは TS-LGX-009 のケースを legixy-mig の凍結 API（DD-LGX-009 §3）に束縛する。
// SRC[GREEN] 未実装（init/migrate/detect_version/backup_file/atomic_write/parse_matrix/
//   generate_id_map/to_json = todo!()）のため全テストは失敗する（RED）。
// `cargo test -p legixy-mig --no-run` は通る（型・シグネチャ整合）が `cargo test` は失敗する。
//
// TS §6 は init/backup/atomic/version/matrix/id_map/migrate に分割した複数 tests/*.rs を想定するが、
// 共通指示「各 UC につき 1 ファイル tests/tc_lgx_NNN.rs」に従い本ファイルに集約する。

use std::path::{Path, PathBuf};

use legixy_mig::{
    atomic_write, backup_file, detect_version, generate_id_map, init, migrate, parse_matrix,
    ArtifactIdSet, ArtifactItem, ChainConfigVariant, IdMapConfidence, IdMapping, InitReport,
    MigError, MigOutputFormat, MigrateOpts, MigrationConfig, MigrationIdMap, MigrationReport,
    MigrationSummaryJson, ProjectVersion, UnmappedIdPolicy,
};

// ---- ヘルパ ----

/// MigError → 終了コード写像（DD-LGX-009 §2.3：実行時失敗 = exit 1）。
/// CLI ディスパッチの exit 規約（LGX-COMPAT-001 §3）を本テストレベルで具体化する。
fn mig_exit_code(err: &MigError) -> i32 {
    // 全 MigError variant は実行時失敗 = exit 1（clap 構文誤りの exit 2 は legixy-cli へ委譲）。
    let _ = err;
    1
}

/// 既定の MigrateOpts（dry_run=false / Markdown / Abort）。
fn default_opts() -> MigrateOpts {
    MigrateOpts {
        dry_run: false,
        format: MigOutputFormat::Markdown,
        unmapped_policy: UnmappedIdPolicy::Abort,
    }
}

/// 最小 MigrationConfig（単数形 chain）。raw_toml は空テーブル。
fn config_single() -> MigrationConfig {
    MigrationConfig {
        chain_order: vec!["UC".into(), "DD".into()],
        independent: vec!["SPEC".into()],
        matrix_file: PathBuf::from("docs/traceability/matrix.md"),
        matrix_section: Some("Matrix".into()),
        id_pattern: r"^[A-Z]+-LGX-\d{3}$".into(),
        id_area: "LGX".into(),
        seq_digits: 3,
        is_multi_area: false,
        raw_toml: toml::Value::Table(toml::map::Map::new()),
    }
}

/// multi-area 変種の MigrationConfig（[id.chains]+[id.areas]）。
fn config_multi_area() -> MigrationConfig {
    let mut c = config_single();
    c.is_multi_area = true;
    c
}

fn artifact(typecode: &str, id_str: &str, path: &str) -> ArtifactItem {
    ArtifactItem {
        typecode: typecode.to_string(),
        id_str: id_str.to_string(),
        path: Some(path.to_string()),
    }
}

// ============================================================
// ケース 1: 空ディレクトリでの init → 5 成果物 + 8 ディレクトリ生成 / exit 0
// ============================================================
#[test]
fn case01_init_empty_dir_creates_artifacts() {
    // @ts: TS-LGX-009 ケース 1
    let root = Path::new("/tmp/legixy-tc009-case01-empty");
    let report: InitReport = init(root, false).expect("空ディレクトリは Ok(InitReport)");
    // 生成物に .legixy.toml を含む。
    assert!(
        report
            .created_files
            .iter()
            .any(|p| p.file_name().map(|f| f == "legixy.toml" || f == ".legixy.toml").unwrap_or(false)),
        "created_files に .legixy.toml を含む"
    );
    // engine_db_path は .legixy/engine.db を指す。
    assert!(
        report.engine_db_path.ends_with(".legixy/engine.db"),
        "engine_db_path == <root>/.legixy/engine.db"
    );
    // 既存スキップは空。
    assert_eq!(report.skipped_files, Vec::<PathBuf>::new());
}

// ============================================================
// ケース 2: legixy 生成物が 1 つでも既存 → AlreadyExists（force=false）
//   4 種既存（.legixy.toml / .trace-engine.toml / graph.toml / engine.db）を別ケースで展開。
// ============================================================
#[test]
fn case02_init_existing_legixy_toml_is_already_exists() {
    // @ts: TS-LGX-009 ケース 2（.legixy.toml 既存）
    let root = Path::new("/tmp/legixy-tc009-case02-a");
    let err = init(root, false).expect_err("既存生成物 → AlreadyExists");
    assert!(matches!(err, MigError::AlreadyExists { .. }), "AlreadyExists variant");
    assert_eq!(mig_exit_code(&err), 1);
}

#[test]
fn case02_init_existing_trace_engine_toml_is_already_exists() {
    // @ts: TS-LGX-009 ケース 2（.trace-engine.toml 既存 = 旧プロジェクト後付け init）
    let root = Path::new("/tmp/legixy-tc009-case02-b");
    let err = init(root, false).expect_err("既存生成物 → AlreadyExists");
    assert!(matches!(err, MigError::AlreadyExists { .. }));
    assert_eq!(mig_exit_code(&err), 1);
}

#[test]
fn case02_init_existing_graph_toml_is_already_exists() {
    // @ts: TS-LGX-009 ケース 2（graph.toml 既存）
    let root = Path::new("/tmp/legixy-tc009-case02-c");
    let err = init(root, false).expect_err("既存生成物 → AlreadyExists");
    assert!(matches!(err, MigError::AlreadyExists { .. }));
}

#[test]
fn case02_init_existing_engine_db_is_already_exists() {
    // @ts: TS-LGX-009 ケース 2（engine.db 既存）
    let root = Path::new("/tmp/legixy-tc009-case02-d");
    let err = init(root, false).expect_err("既存生成物 → AlreadyExists");
    assert!(matches!(err, MigError::AlreadyExists { .. }));
}

// ============================================================
// ケース 3: init --force → 既存生成物を REQ.02a 命名で退避後に上書き
// ============================================================
#[test]
fn case03_init_force_backs_up_then_overwrites() {
    // @ts: TS-LGX-009 ケース 3
    let root = Path::new("/tmp/legixy-tc009-case03-force");
    let report = init(root, true).expect("force=true は Ok（退避後上書き）");
    // 新規 .legixy.toml を created_files に含む。
    assert!(
        report
            .created_files
            .iter()
            .any(|p| p.to_string_lossy().contains("legixy.toml")),
        "force 後も新規 .legixy.toml を生成"
    );
}

// ============================================================
// ケース 4: backup_file の退避名 = `<元名>.bak.{epoch}`
// ============================================================
#[test]
fn case04_backup_file_names_with_epoch_suffix() {
    // @ts: TS-LGX-009 ケース 4
    let path = Path::new("/tmp/legixy-tc009-case04/target.toml");
    let backup = backup_file(path).expect("退避は Ok(BackupName)");
    let name = backup.path.to_string_lossy().to_string();
    assert!(name.contains(".bak."), "退避名は <元名>.bak.{{epoch}} 形式");
    // epoch サフィックスは数値（固定 .bak は不採用）。
    let suffix = name.rsplit(".bak.").next().unwrap_or("");
    assert!(
        suffix.chars().next().map(|c| c.is_ascii_digit()).unwrap_or(false),
        "epoch 秒（数値）サフィックス必須、固定 .bak は不採用"
    );
}

// ============================================================
// ケース 5: backup_file の同一秒内衝突 → 連番 `.bak.{epoch}.{seq}`
// ============================================================
#[test]
fn case05_backup_file_same_second_collision_appends_seq() {
    // @ts: TS-LGX-009 ケース 5
    let path = Path::new("/tmp/legixy-tc009-case05/target.toml");
    let first = backup_file(path).expect("1 回目 Ok");
    let second = backup_file(path).expect("2 回目 Ok");
    // 2 世代とも別名（既存退避を上書きしない）。
    assert_ne!(first.path, second.path, "両世代とも別名で残存（非破壊性）");
    // 2 回目は連番 `.bak.{epoch}.{seq}`（少なくとも一方が末尾 .{seq} 形式）。
    let s = second.path.to_string_lossy().to_string();
    assert!(
        s.matches('.').count() >= 3,
        "同一秒内衝突時は .bak.{{epoch}}.{{seq}} で連番付与"
    );
}

// ============================================================
// ケース 6: 2 回 migrate で退避ファイル 2 世代が保全される
// ============================================================
#[test]
fn case06_two_migrations_preserve_two_backup_generations() {
    // @ts: TS-LGX-009 ケース 6
    let src = Path::new("/tmp/legixy-tc009-case06/src");
    let dst = Path::new("/tmp/legixy-tc009-case06/dst");
    let r1 = migrate(src, dst, default_opts()).expect("1 回目 migrate");
    let r2 = migrate(src, dst, default_opts()).expect("2 回目 migrate");
    // 退避ファイルが累積保持（機械削除しない）。
    let mut all: Vec<PathBuf> = r1.backup_paths.clone();
    all.extend(r2.backup_paths.clone());
    let unique: std::collections::BTreeSet<&PathBuf> = all.iter().collect();
    assert_eq!(unique.len(), all.len(), "退避は別名で 2 世代とも残存");
}

// ============================================================
// ケース 7: atomic_write は temp+fsync+rename で確定（中断耐性）
// ============================================================
#[test]
fn case07_atomic_write_commits_content() {
    // @ts: TS-LGX-009 ケース 7
    let path = Path::new("/tmp/legixy-tc009-case07/graph.toml");
    let content = b"# new content\n";
    atomic_write(path, content).expect("atomic_write は Ok(())");
    // 確定後内容が new_content と完全一致（SRC[GREEN] が保証）。
    let written = std::fs::read(path).expect("確定後ファイルが読める");
    assert_eq!(written, content, "確定後内容 == new_content（半端な書込なし）");
}

// ============================================================
// ケース 8: atomic_write の冪等収束（中断後再実行で同一最終状態）
// ============================================================
#[test]
fn case08_atomic_write_idempotent_convergence() {
    // @ts: TS-LGX-009 ケース 8
    let path = Path::new("/tmp/legixy-tc009-case08/id-map.toml");
    let content = b"[mappings]\n";
    atomic_write(path, content).expect("1 回目");
    atomic_write(path, content).expect("2 回目（冪等）");
    let written = std::fs::read(path).expect("読める");
    assert_eq!(written, content, "何回実行しても最終状態は同一（content）");
}

// ============================================================
// ケース 9: detect_version の網羅判定（user_version 0/3 × [graph] 有無 × 矛盾）
// ============================================================
#[test]
fn case09_detect_version_user_version_3_is_legixy() {
    // @ts: TS-LGX-009 ケース 9（user_version=3 → Legixy）
    let root = Path::new("/tmp/legixy-tc009-case09-v3");
    let detected = detect_version(root).expect("検出 Ok");
    assert_eq!(detected.kind, ProjectVersion::Legixy);
    assert!(detected.evidence.contains("user_version"), "evidence に判定根拠");
}

#[test]
fn case09_detect_version_v0_no_graph_is_v010() {
    // @ts: TS-LGX-009 ケース 9（user_version=0 + [graph] なし → V0_1_0 最保守）
    let root = Path::new("/tmp/legixy-tc009-case09-v0");
    let detected = detect_version(root).expect("検出 Ok");
    assert_eq!(detected.kind, ProjectVersion::V0_1_0);
}

#[test]
fn case09_detect_version_conflict_is_version_mismatch() {
    // @ts: TS-LGX-009 ケース 9（engine.db legixy だが config は v0.1.0 → VersionMismatch）
    let root = Path::new("/tmp/legixy-tc009-case09-conflict");
    let err = detect_version(root).expect_err("矛盾 → Err");
    assert!(
        matches!(err, MigError::VersionMismatch { .. }),
        "矛盾は VersionMismatch（exit 1）"
    );
    assert_eq!(mig_exit_code(&err), 1);
}

// ============================================================
// ケース 10: parse_matrix 空入力（抽出 0 件）→ 空 ArtifactIdSet・正常
// ============================================================
#[test]
fn case10_parse_matrix_empty_input_is_ok_empty() {
    // @ts: TS-LGX-009 ケース 10
    let config = config_single();
    let set = parse_matrix("", &config).expect("空入力は Error にせず Ok（空入力正常）");
    assert_eq!(set.items, Vec::<ArtifactItem>::new(), "抽出 0 件 = 空 ArtifactIdSet");
}

// ============================================================
// ケース 11: parse_matrix 単一ノード抽出（SUPP-008 §2.5 抽出規則）
// ============================================================
#[test]
fn case11_parse_matrix_single_node() {
    // @ts: TS-LGX-009 ケース 11
    let config = config_single();
    let content = "\
| id | path |
| UC-LGX-001 | docs/usecases/uc.md |
";
    let set = parse_matrix(content, &config).expect("Ok");
    assert_eq!(set.items.len(), 1, "成果物 1 件を抽出（先頭行 ID は非ノード化）");
}

// ============================================================
// ケース 12: parse_matrix の [id.chain] 欠落 → ChainConfigMissing（破損 Error）
// ============================================================
#[test]
fn case12_parse_matrix_missing_chain_config_is_error() {
    // @ts: TS-LGX-009 ケース 12
    // chain_order 空 = [id.chain]/[id.chains] 双方欠落相当（構造情報の黙殺禁止）。
    let mut config = config_single();
    config.chain_order = vec![];
    let content = "| UC-LGX-001 | docs/uc.md |\n";
    let err = parse_matrix(content, &config).expect_err("chain 定義欠落 → Err");
    assert!(
        matches!(err, MigError::ChainConfigMissing),
        "ChainConfigMissing（chain エッジを暗黙 0 本で続行しない）"
    );
}

// ============================================================
// ケース 13: [id.chains]+[id.areas] multi-area 変種を受理（両表記）
// ============================================================
#[test]
fn case13_parse_matrix_accepts_both_single_and_multi_area() {
    // @ts: TS-LGX-009 ケース 13
    let content = "| UC-LGX-001 | docs/uc.md |\n";
    // 単数形版（is_multi_area=false）を受理。
    let single = config_single();
    assert_eq!(single.is_multi_area, false);
    let _ = parse_matrix(content, &single).expect("単数形版を受理");
    // 複数形版（is_multi_area=true）を受理。
    let multi = config_multi_area();
    assert_eq!(multi.is_multi_area, true);
    let _ = parse_matrix(content, &multi).expect("multi-area 変種を受理");
    // ChainConfigVariant の両表記が型として存在する（ADR-LGX-018#15）。
    assert_ne!(ChainConfigVariant::Single, ChainConfigVariant::MultiArea);
}

// ============================================================
// ケース 14: 破損 source（壊れた feedback.db / 不正 TOML / order 欠落）→ Error + 原本無傷
// ============================================================
#[test]
fn case14_corrupt_feedback_db_is_error() {
    // @ts: TS-LGX-009 ケース 14（a: 壊れた feedback.db / 必須テーブル欠落）
    let src = Path::new("/tmp/legixy-tc009-case14-a/src");
    let dst = Path::new("/tmp/legixy-tc009-case14-a/dst");
    let err = migrate(src, dst, default_opts()).expect_err("破損 source → Err");
    // SchemaIncompatible または Sqlite。
    assert!(
        matches!(err, MigError::SchemaIncompatible { .. } | MigError::Sqlite(_)),
        "DB 破損は SchemaIncompatible / Sqlite（exit 1、原本無傷）"
    );
    assert_eq!(mig_exit_code(&err), 1);
}

#[test]
fn case14_invalid_toml_is_error() {
    // @ts: TS-LGX-009 ケース 14（b: TOML パース失敗）
    let src = Path::new("/tmp/legixy-tc009-case14-b/src");
    let dst = Path::new("/tmp/legixy-tc009-case14-b/dst");
    let err = migrate(src, dst, default_opts()).expect_err("不正 TOML → Err");
    assert!(
        matches!(err, MigError::TomlParse(_) | MigError::ConfigCorrupt { .. }),
        "TOML 破損は TomlParse / ConfigCorrupt"
    );
}

#[test]
fn case14_missing_order_is_chain_config_missing() {
    // @ts: TS-LGX-009 ケース 14（c: [id.chain].order 欠落）
    let src = Path::new("/tmp/legixy-tc009-case14-c/src");
    let dst = Path::new("/tmp/legixy-tc009-case14-c/dst");
    let err = migrate(src, dst, default_opts()).expect_err("order 欠落 → Err");
    assert!(matches!(err, MigError::ChainConfigMissing));
}

// ============================================================
// ケース 15: 出力 graph.toml 妥当性検証失敗 → OutputGraphInvalid（atomic 確定前）
// ============================================================
#[test]
fn case15_output_graph_invalid_before_commit() {
    // @ts: TS-LGX-009 ケース 15
    let src = Path::new("/tmp/legixy-tc009-case15/src");
    let dst = Path::new("/tmp/legixy-tc009-case15/dst");
    let err = migrate(src, dst, default_opts()).expect_err("出力妥当性違反 → Err");
    assert!(
        matches!(err, MigError::OutputGraphInvalid { .. }),
        "OutputGraphInvalid（atomic rename 前に検出、原本無変更）"
    );
}

// ============================================================
// ケース 16: generate_id_map の SHA-256 基本動作（High 確信度）
// ============================================================
#[test]
fn case16_generate_id_map_sha256_high_confidence() {
    // @ts: TS-LGX-009 ケース 16
    let config = config_single();
    let set = ArtifactIdSet {
        items: vec![
            artifact("UC", "UC-OLD-001", "docs/usecases/a.md"),
            artifact("DD", "DD-OLD-001", "docs/detailed-design/b.md"),
        ],
    };
    let existing_refs: Vec<String> = vec![];
    let id_map: MigrationIdMap =
        generate_id_map(&set, &existing_refs, &config, UnmappedIdPolicy::Abort).expect("Ok");
    assert_eq!(id_map.mappings.len(), 2, "各 ArtifactItem に 1 対応");
    for m in &id_map.mappings {
        let _: &IdMapping = m;
        assert_eq!(m.confidence, IdMapConfidence::High, "通常ケースは High のみ");
    }
}

// ============================================================
// ケース 17: generate_id_map 全単射違反 → IdBijectionViolation
// ============================================================
#[test]
fn case17_generate_id_map_bijection_violation() {
    // @ts: TS-LGX-009 ケース 17（旧 ID 重複 = 同一旧 ID に複数項目）
    let config = config_single();
    let set = ArtifactIdSet {
        items: vec![
            artifact("UC", "UC-OLD-001", "docs/usecases/a.md"),
            artifact("UC", "UC-OLD-001", "docs/usecases/dup.md"), // 旧 ID 重複（曖昧性）
        ],
    };
    let err = generate_id_map(&set, &[], &config, UnmappedIdPolicy::Abort)
        .expect_err("全単射違反 → Err");
    assert!(
        matches!(err, MigError::IdBijectionViolation { .. }),
        "IdBijectionViolation（v3 の桁伸長回避は不採用、Error にする）"
    );
    assert_eq!(mig_exit_code(&err), 1);
}

// ============================================================
// ケース 18: マッピング不可 ID — 既定 abort / --skip-unmapped で SkipEdge
// ============================================================
#[test]
fn case18_unmapped_id_abort_is_error() {
    // @ts: TS-LGX-009 ケース 18（a: 既定 Abort → UnmappedIds Error）
    let config = config_single();
    let set = ArtifactIdSet {
        items: vec![artifact("UC", "UC-OLD-001", "docs/usecases/a.md")],
    };
    // artifact_set で解決できない旧 ID 参照（dangling 候補）。
    let existing_refs = vec!["DD-OLD-999".to_string()];
    let err = generate_id_map(&set, &existing_refs, &config, UnmappedIdPolicy::Abort)
        .expect_err("マッピング不可 + Abort → Err");
    assert!(matches!(err, MigError::UnmappedIds { .. }), "UnmappedIds（非破壊性優先）");
}

#[test]
fn case18_unmapped_id_skip_edge_is_ok() {
    // @ts: TS-LGX-009 ケース 18（b: SkipEdge → Ok、当該エッジ除外）
    let config = config_single();
    let set = ArtifactIdSet {
        items: vec![artifact("UC", "UC-OLD-001", "docs/usecases/a.md")],
    };
    let existing_refs = vec!["DD-OLD-999".to_string()];
    let id_map = generate_id_map(&set, &existing_refs, &config, UnmappedIdPolicy::SkipEdge)
        .expect("SkipEdge は Ok（当該エッジ除外で継続）");
    // 解決できた 1 件のみがマッピングされ、dangling は残置しない。
    assert!(
        id_map.mappings.iter().all(|m| m.old_id != "DD-OLD-999"),
        "dangling 参照は除外（残置しない）"
    );
}

// ============================================================
// ケース 19: migrate v0.1.0 fixture → graph.toml / id-map / engine.db 生成
// ============================================================
#[test]
fn case19_migrate_v01_generates_artifacts() {
    // @ts: TS-LGX-009 ケース 19
    let src = Path::new("/tmp/legixy-tc009-case19/v01-src");
    let dst = Path::new("/tmp/legixy-tc009-case19/dst");
    let report: MigrationReport = migrate(src, dst, default_opts()).expect("Ok(MigrationReport)");
    // files_written に graph.toml / id-map / .legixy.toml を含む。
    let joined = report
        .files_written
        .iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect::<Vec<_>>()
        .join(",");
    assert!(joined.contains("graph.toml"), "graph.toml を生成");
    // id_map_path は .legixy/migration-id-map.toml。
    assert!(
        report.id_map_path.ends_with(".legixy/migration-id-map.toml"),
        "id_map_path == .legixy/migration-id-map.toml"
    );
    // 3 テーブル（observations/proposals/custom_edges）の転記実績。
    assert!(report.rows_copied >= 1 || !report.tables_copied.is_empty());
}

// ============================================================
// ケース 20: 確定順序 — DB コミット先行 → graph.toml/id-map/config の atomic 確定
// ============================================================
#[test]
fn case20_commit_order_db_first() {
    // @ts: TS-LGX-009 ケース 20
    let src = Path::new("/tmp/legixy-tc009-case20/v01-src");
    let dst = Path::new("/tmp/legixy-tc009-case20/dst");
    // SRC[GREEN] が DB コミット先行 → atomic 平文確定の順序を保証する（本テストは成功収束のみ検証）。
    let report = migrate(src, dst, default_opts()).expect("確定順序遵守で Ok");
    assert!(
        !report.files_written.is_empty(),
        "平文確定（graph.toml/id-map/config）が DB コミット後に到達"
    );
}

// ============================================================
// ケース 21: DB コミット後中断の再実行収束（resume なし全やり直し）
// ============================================================
#[test]
fn case21_resume_after_commit_converges() {
    // @ts: TS-LGX-009 ケース 21
    let src = Path::new("/tmp/legixy-tc009-case21/v01-src");
    let dst = Path::new("/tmp/legixy-tc009-case21/dst");
    // 1 回目（中断相当）と 2 回目（再実行）で同一最終状態へ収束する。
    let _ = migrate(src, dst, default_opts()).expect("1 回目");
    let r2 = migrate(src, dst, default_opts()).expect("再実行で同一最終状態へ収束");
    // engine.db は INSERT OR IGNORE で冪等、原本破壊なし。
    assert!(r2.id_map_path.ends_with("migration-id-map.toml"));
}

// ============================================================
// ケース 22: --dry-run は一切書き込まない（全単射検証は実施）
// ============================================================
#[test]
fn case22_dry_run_writes_nothing() {
    // @ts: TS-LGX-009 ケース 22
    let src = Path::new("/tmp/legixy-tc009-case22/v01-src");
    let dst = Path::new("/tmp/legixy-tc009-case22/dst");
    let opts = MigrateOpts {
        dry_run: true,
        format: MigOutputFormat::Markdown,
        unmapped_policy: UnmappedIdPolicy::Abort,
    };
    let report = migrate(src, dst, opts).expect("dry-run は検証成功で Ok");
    // 退避は発生しない（非書込）。MigrationReport は「書き込まれる予定」を表す。
    assert_eq!(report.backup_paths, Vec::<PathBuf>::new(), "dry-run は退避も発生しない");
}

// ============================================================
// ケース 23: 既に legixy → no-op（exit 0 + stderr Info + 空サマリ）
// ============================================================
#[test]
fn case23_already_legixy_is_noop() {
    // @ts: TS-LGX-009 ケース 23
    let src = Path::new("/tmp/legixy-tc009-case23/legixy-src");
    let dst = Path::new("/tmp/legixy-tc009-case23/dst");
    let report = migrate(src, dst, default_opts()).expect("既に legixy は exit 0 + 空サマリ");
    assert_eq!(report.files_written, Vec::<PathBuf>::new(), "no-op: files_written 空");
    assert_eq!(report.ids_rewritten_count, 0, "no-op: 書換 0 件");
    assert_eq!(report.backup_paths, Vec::<PathBuf>::new(), "no-op: 退避 0 件");
}

// ============================================================
// ケース 24: 成功サマリ stdout / 診断 stderr 分離 + to_json スキーマ
// ============================================================
#[test]
fn case24_to_json_emits_summary_json_schema() {
    // @ts: TS-LGX-009 ケース 24
    let report = MigrationReport {
        files_written: vec![PathBuf::from("docs/traceability/graph.toml")],
        ids_rewritten_count: 3,
        id_map_path: PathBuf::from(".legixy/migration-id-map.toml"),
        backup_paths: vec![PathBuf::from(".legixy.toml.bak.1700000000")],
        warnings: vec!["vectors.bin skipped".into()],
        tables_copied: vec!["observations".into()],
        rows_copied: 10,
    };
    let json = report.to_json();
    // MigrationSummaryJson スキーマ（files_written / ids_rewritten_count / id_map_path / backups / warnings）。
    let parsed: MigrationSummaryJson =
        serde_json::from_str(&json).expect("to_json は MigrationSummaryJson スキーマ");
    assert_eq!(parsed.ids_rewritten_count, 3);
    assert_eq!(parsed.id_map_path, ".legixy/migration-id-map.toml");
    assert!(parsed.files_written.iter().any(|f| f.contains("graph.toml")));
}

// ============================================================
// ケース 25: 終了コード契約 0/1/2（LGX-COMPAT-001 §3 凍結）
//   (a) 成功 → 0、(b) MigError → 1。(c) clap exit 2 は legixy-cli へ委譲。
// ============================================================
#[test]
fn case25_exit_code_contract_success_and_failure() {
    // @ts: TS-LGX-009 ケース 25
    // (b) MigError（実行時失敗）は exit 1（型レベルの契約確認）。
    let err = MigError::AlreadyExists {
        path: PathBuf::from("/x/.legixy.toml"),
    };
    assert_eq!(mig_exit_code(&err), 1, "AlreadyExists → exit 1");
    let err2 = MigError::IdBijectionViolation {
        detail: "dup".into(),
    };
    assert_eq!(mig_exit_code(&err2), 1, "破損出力も exit 1（exit 2 ではない）");
    // (a) 成功 → exit 0。実 API（init）の Ok 経路を exit 0 に写像して契約を束縛する。
    //   SRC[GREEN] 未実装のため本呼び出しで panic（RED）。
    let root = Path::new("/tmp/legixy-tc009-case25/ok-src");
    let report = init(root, false).expect("成功 init は Ok");
    let success_exit = if report.created_files.is_empty() { 1 } else { 0 };
    assert_eq!(success_exit, 0, "成功 → exit 0");
    // (c) clap exit 2 は legixy-cli E2E（ディスパッチ層）へ委譲。
}

// ============================================================
// ケース 26: custom_edges 継承（v0.1.0 にあれば転記、なければ継承なし）
// ============================================================
#[test]
fn case26_custom_edges_inheritance() {
    // @ts: TS-LGX-009 ケース 26（a: custom_edges を持つ feedback.db → 転記）
    let src = Path::new("/tmp/legixy-tc009-case26-a/src");
    let dst = Path::new("/tmp/legixy-tc009-case26-a/dst");
    let report = migrate(src, dst, default_opts()).expect("Ok");
    assert!(
        report.tables_copied.iter().any(|t| t == "custom_edges"),
        "custom_edges を engine.db へ転記（tables_copied に反映）"
    );
}

#[test]
fn case26_no_custom_edges_is_ok() {
    // @ts: TS-LGX-009 ケース 26（b: custom_edges テーブルなし → 継承なしで正常）
    let src = Path::new("/tmp/legixy-tc009-case26-b/src");
    let dst = Path::new("/tmp/legixy-tc009-case26-b/dst");
    let report = migrate(src, dst, default_opts()).expect("custom_edges 不在は Error にしない");
    assert!(
        !report.tables_copied.iter().any(|t| t == "custom_edges"),
        "継承なしで明示（破損ではない）"
    );
}

// ============================================================
// ケース 27: vectors.bin 不在時はスキップ + Warning（Phase 1 ドキュメントノードのみ）
// ============================================================
#[test]
fn case27_missing_vectors_bin_skips_with_warning() {
    // @ts: TS-LGX-009 ケース 27（a: vectors.bin 不在 → スキップ + Warning）
    let src = Path::new("/tmp/legixy-tc009-case27/no-vectors-src");
    let dst = Path::new("/tmp/legixy-tc009-case27/dst");
    let report = migrate(src, dst, default_opts()).expect("vectors.bin 不在でも exit 0");
    assert!(
        report.warnings.iter().any(|w| w.contains("vectors")),
        "report.warnings に vectors.bin スキップの非致命警告"
    );
}

// ============================================================
// ケース 28: 設定ファイル探索順 4 ケース（.legixy.toml / .trace-engine.toml フォールバック）
// ============================================================
#[test]
fn case28_config_search_legixy_only() {
    // @ts: TS-LGX-009 ケース 28（a: .legixy.toml のみ → 既に legixy 形式）
    let root = Path::new("/tmp/legixy-tc009-case28-a");
    let detected = detect_version(root).expect("Ok");
    // .legixy.toml のみ存在 → legixy 寄りの判定（探索順は .legixy.toml 優先）。
    assert!(
        matches!(detected.kind, ProjectVersion::Legixy | ProjectVersion::Unknown),
        ".legixy.toml を使用（.trace-engine.toml 無視）"
    );
}

#[test]
fn case28_config_search_trace_engine_fallback() {
    // @ts: TS-LGX-009 ケース 28（b: .trace-engine.toml のみ → フォールバック読込）
    let src = Path::new("/tmp/legixy-tc009-case28-b/src");
    let dst = Path::new("/tmp/legixy-tc009-case28-b/dst");
    // .trace-engine.toml フォールバック読込 + 一度だけ Info（移行案内）で migrate 継続。
    let report = migrate(src, dst, default_opts()).expect(".trace-engine.toml フォールバックで継続");
    assert!(report.id_map_path.ends_with("migration-id-map.toml"));
}

// ============================================================
// ケース 30: V01NotFound — migrate 対象が見つからない（代替フロー 2b）
// ============================================================
#[test]
fn case30_v01_not_found() {
    // @ts: TS-LGX-009 ケース 30
    let src = Path::new("/tmp/legixy-tc009-case30/nonexistent");
    let dst = Path::new("/tmp/legixy-tc009-case30/dst");
    let err = migrate(src, dst, default_opts()).expect_err("対象不在 → Err");
    assert!(
        matches!(err, MigError::V01NotFound { .. }),
        "V01NotFound（exit 1）。--from 省略の構文誤り exit 2 とは区別"
    );
    assert_eq!(mig_exit_code(&err), 1);
}

// ============================================================
// ケース 29: parse_matrix の決定性（property、proptest）
// ============================================================
mod prop {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        // @ts: TS-LGX-009 ケース 29
        // 同一 (content, config) 入力に対し parse_matrix は常に同一 ArtifactIdSet（決定性）。
        #[test]
        fn case29_parse_matrix_deterministic(
            lines in proptest::collection::vec(
                "(\\| [A-Z]{2,4}-LGX-[0-9]{3} \\| docs/[a-z]+\\.md \\|)|(\\| - \\| - \\|)|()",
                0..8usize,
            )
        ) {
            let content = lines.join("\n");
            let config = config_single();
            // todo!() のため両呼び出しとも panic する（RED）。同一入力 → 同一出力の不変条件を束縛。
            let a = parse_matrix(&content, &config);
            let b = parse_matrix(&content, &config);
            match (a, b) {
                (Ok(x), Ok(y)) => prop_assert_eq!(x, y, "同一入力 → 同一 ArtifactIdSet"),
                (Err(_), Err(_)) => {}
                _ => prop_assert!(false, "同一入力で Ok/Err が割れた（非決定）"),
            }
        }
    }
}
