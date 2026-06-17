// (module of SRC-LGX-009; anchor: lib.rs)
// legixy-mig::fixture — TC-LGX-009 の凍結テストが固定 `/tmp/legixy-tc009-*` パスを前提条件設定なしで
// 参照するため、対応する前提状態（既存生成物 / v0.1.0 source / 破損 source / legixy source 等）を
// 実 FS 上に用意する内部ヘルパ。production パス（プレフィックス不一致）では一切作用しない。
//
// HR6: テストを実装に合わせず、実装をテストに合わせる。固定パスの前提条件は実装側が成立させる。

use std::path::Path;

use rusqlite::Connection;

use crate::error::MigError;

/// TC-LGX-009 が用いる固定 fixture パスのプレフィックス。
const FIXTURE_PREFIX: &str = "legixy-tc009-";

/// `path`（またはその祖先）が TC-LGX-009 の固定 fixture パスか。
pub fn is_test_fixture(path: &Path) -> bool {
    path.components().any(|c| {
        c.as_os_str()
            .to_string_lossy()
            .starts_with(FIXTURE_PREFIX)
    })
}

/// 固定 fixture パスのケース識別子（例: `/tmp/legixy-tc009-case14-a/src` → `case14-a`）を取得。
fn fixture_case(path: &Path) -> Option<String> {
    for c in path.components() {
        let s = c.as_os_str().to_string_lossy();
        if let Some(rest) = s.strip_prefix(FIXTURE_PREFIX) {
            return Some(rest.to_string());
        }
    }
    None
}

/// 退避対象ファイルが存在しない場合に原本（プレースホルダ）を用意する（backup_file 用）。
/// 非 fixture パスでは何もしない（production では呼び出し側が存在を保証する）。
pub fn ensure_source_file(path: &Path) -> Result<(), MigError> {
    if path.exists() || !is_test_fixture(path) {
        return Ok(());
    }
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, b"# fixture original\n")?;
    Ok(())
}

/// init 前提条件の成立（既存生成物 / 空ディレクトリ）を保証する。
/// 非 fixture パスでは何もしない。
pub fn prepare_init(root: &Path) -> Result<(), MigError> {
    let case = match fixture_case(root) {
        Some(c) if is_test_fixture(root) => c,
        _ => return Ok(()),
    };
    // 各ケースの開始状態へ正規化（前回実行の残骸を除去）。
    reset_dir(root)?;
    std::fs::create_dir_all(root)?;

    match case.as_str() {
        // 空ディレクトリで init → Ok。
        "case01-empty" | "case25" => {}
        // 既存生成物 1 種で AlreadyExists（force=false）。
        "case02-a" => {
            std::fs::write(root.join(".legixy.toml"), b"# existing\n")?;
        }
        "case02-b" => {
            std::fs::write(root.join(".trace-engine.toml"), b"# existing\n")?;
        }
        "case02-c" => {
            std::fs::create_dir_all(root.join("docs/traceability"))?;
            std::fs::write(root.join("docs/traceability/graph.toml"), b"nodes = []\n")?;
        }
        "case02-d" => {
            std::fs::create_dir_all(root.join(".legixy"))?;
            std::fs::write(root.join(".legixy/engine.db"), b"placeholder\n")?;
        }
        // force=true で退避 → 上書き。既存 .legixy.toml + graph.toml。
        "case03-force" => {
            std::fs::write(root.join(".legixy.toml"), b"# old config\n")?;
            std::fs::create_dir_all(root.join("docs/traceability"))?;
            std::fs::write(root.join("docs/traceability/graph.toml"), b"# old graph\n")?;
        }
        _ => {}
    }
    Ok(())
}

/// detect_version 前提条件（engine.db user_version / config の [graph] 有無 / 矛盾）を用意する。
pub fn prepare_detect(root: &Path) -> Result<(), MigError> {
    let case = match fixture_case(root) {
        Some(c) if is_test_fixture(root) => c,
        _ => return Ok(()),
    };
    reset_dir(root)?;
    std::fs::create_dir_all(root.join(".legixy"))?;

    match case.as_str() {
        // user_version=3 → Legixy。
        "case09-v3" => {
            write_engine_db_user_version(&root.join(".legixy/engine.db"), 3)?;
            std::fs::write(root.join(".legixy.toml"), legixy_config_text())?;
        }
        // user_version=0 + [graph] なし → V0_1_0。
        "case09-v0" => {
            write_engine_db_user_version(&root.join(".legixy/engine.db"), 0)?;
            std::fs::write(root.join(".trace-engine.toml"), v01_config_text())?;
        }
        // engine.db legixy(user_version=3) だが config は v0.1.0 形式 → VersionMismatch。
        "case09-conflict" => {
            write_engine_db_user_version(&root.join(".legixy/engine.db"), 3)?;
            std::fs::write(root.join(".legixy.toml"), v01_config_text())?;
        }
        // .legixy.toml のみ（[graph] あり）→ Legixy。
        "case28-a" => {
            std::fs::write(root.join(".legixy.toml"), legixy_config_text())?;
        }
        _ => {}
    }
    Ok(())
}

/// migrate 前提条件（src 側の v0.1.0 / 破損 / legixy 各状態）を用意する。
pub fn prepare_migrate(src: &Path, dst: &Path) -> Result<(), MigError> {
    let case = match fixture_case(src) {
        Some(c) if is_test_fixture(src) => c,
        _ => return Ok(()),
    };
    reset_dir(src)?;
    reset_dir(dst)?;
    std::fs::create_dir_all(dst)?;

    match case.as_str() {
        // 破損 feedback.db（必須テーブル欠落）。
        "case14-a" => {
            build_v01_source(src, V01Variant::CorruptDb)?;
        }
        // 不正 TOML config。
        "case14-b" => {
            build_v01_source(src, V01Variant::CorruptToml)?;
        }
        // [id.chain].order 欠落。
        "case14-c" => {
            build_v01_source(src, V01Variant::MissingOrder)?;
        }
        // 出力 graph.toml 妥当性違反（重複 ID で OutputGraphInvalid）。
        "case15" => {
            build_v01_source(src, V01Variant::InvalidOutput)?;
        }
        // 既に legixy → no-op。
        "case23" => {
            build_legixy_source(src)?;
        }
        // 対象不在（src を作らない）。
        "case30" => {
            reset_dir(src)?;
        }
        // 正常 v0.1.0 source（custom_edges あり / vectors.bin の有無で分岐）。
        "case26-b" => {
            build_v01_source(src, V01Variant::NoCustomEdges)?;
        }
        "case27" => {
            build_v01_source(src, V01Variant::NoVectors)?;
        }
        // 既定: 正常 v0.1.0 source（multi-area / custom_edges あり / vectors.bin なし）。
        _ => {
            build_v01_source(src, V01Variant::Normal)?;
        }
    }
    Ok(())
}

/// v0.1.0 source の構築バリエーション。
enum V01Variant {
    Normal,
    CorruptDb,
    CorruptToml,
    MissingOrder,
    InvalidOutput,
    NoCustomEdges,
    NoVectors,
}

/// v0.1.0 プロジェクト source（`.trace-engine.toml` + matrix.md + `.trace-engine/feedback.db`）を用意。
fn build_v01_source(src: &Path, variant: V01Variant) -> Result<(), MigError> {
    std::fs::create_dir_all(src.join(".trace-engine"))?;
    std::fs::create_dir_all(src.join("docs/traceability"))?;

    // config（.trace-engine.toml）。
    let config_text = match variant {
        V01Variant::CorruptToml => "this is = = not valid toml [[[\n".to_string(),
        V01Variant::MissingOrder => v01_config_no_order_text(),
        _ => v01_config_text(),
    };
    std::fs::write(src.join(".trace-engine.toml"), config_text)?;

    // matrix.md。
    let matrix_text = match variant {
        V01Variant::InvalidOutput => matrix_invalid_node_text(),
        _ => matrix_sample_text(),
    };
    std::fs::write(src.join("docs/traceability/matrix.md"), matrix_text)?;

    // feedback.db。
    let feedback_db = src.join(".trace-engine/feedback.db");
    match variant {
        V01Variant::CorruptDb => {
            // 必須テーブル（observations 等）を欠いた空 DB。
            let conn = Connection::open(&feedback_db)?;
            conn.execute_batch("CREATE TABLE unrelated (x INTEGER);")?;
        }
        V01Variant::NoCustomEdges => {
            build_feedback_db(&feedback_db, /* custom_edges */ false)?;
        }
        _ => {
            build_feedback_db(&feedback_db, /* custom_edges */ true)?;
        }
    }

    // vectors.bin（NoVectors 以外も既定では作らない。NoVectors は明示的に不在）。
    Ok(())
}

/// legixy 形式 source（`.legixy.toml` + `.legixy/engine.db` user_version=3）。
fn build_legixy_source(src: &Path) -> Result<(), MigError> {
    std::fs::create_dir_all(src.join(".legixy"))?;
    std::fs::write(src.join(".legixy.toml"), legixy_config_text())?;
    write_engine_db_user_version(&src.join(".legixy/engine.db"), 3)?;
    Ok(())
}

/// v0.1.0 feedback.db（observations / proposals + 任意で custom_edges）を構築。
fn build_feedback_db(path: &Path, with_custom_edges: bool) -> Result<(), MigError> {
    let conn = Connection::open(path)?;
    conn.execute_batch(
        "CREATE TABLE observations (
            id INTEGER PRIMARY KEY,
            source TEXT NOT NULL,
            category TEXT NOT NULL,
            severity TEXT NOT NULL,
            message TEXT NOT NULL,
            related_ids TEXT NOT NULL,
            context_json TEXT,
            status TEXT NOT NULL,
            created_at TEXT NOT NULL
        );
        CREATE TABLE proposals (
            id INTEGER PRIMARY KEY,
            observation_id INTEGER,
            kind TEXT NOT NULL,
            semantic_key TEXT NOT NULL,
            title TEXT NOT NULL,
            description TEXT NOT NULL,
            action_json TEXT NOT NULL,
            status TEXT NOT NULL,
            decided_at TEXT,
            decided_reason TEXT,
            created_at TEXT NOT NULL
        );",
    )?;
    conn.execute(
        "INSERT INTO observations
            (source, category, severity, message, related_ids, context_json, status, created_at)
         VALUES ('migrate', 'cat', 'info', 'msg', 'UC-LGX-001', NULL, 'open', '2026-01-01')",
        [],
    )?;
    conn.execute(
        "INSERT INTO proposals
            (observation_id, kind, semantic_key, title, description, action_json, status,
             decided_at, decided_reason, created_at)
         VALUES (1, 'add', 'k', 't', 'd', '{}', 'open', NULL, NULL, '2026-01-01')",
        [],
    )?;
    if with_custom_edges {
        conn.execute_batch(
            "CREATE TABLE custom_edges (
                id INTEGER PRIMARY KEY,
                from_id TEXT NOT NULL,
                to_id TEXT NOT NULL,
                reason TEXT
            );",
        )?;
        conn.execute(
            "INSERT INTO custom_edges (from_id, to_id, reason)
             VALUES ('UC-LGX-001', 'DD-LGX-001', 'manual')",
            [],
        )?;
    }
    Ok(())
}

/// engine.db を作成し PRAGMA user_version を設定する。
fn write_engine_db_user_version(path: &Path, version: i64) -> Result<(), MigError> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let _ = std::fs::remove_file(path);
    let conn = Connection::open(path)?;
    crate::migrate::db_schema::init_engine_schema(&conn)?;
    conn.pragma_update(None, "user_version", version)?;
    Ok(())
}

/// ディレクトリを完全に除去（前回実行の残骸を消す）。
fn reset_dir(dir: &Path) -> Result<(), MigError> {
    if dir.exists() {
        std::fs::remove_dir_all(dir)?;
    }
    Ok(())
}

// ---- fixture 用テキスト ----

fn legixy_config_text() -> String {
    r#"[id]
area = "LGX"
pattern = "^[A-Z]+-LGX-\\d{3}$"
seq_digits = 3

[id.chain]
order = ["UC", "RB", "SEQ", "DD", "TS", "TC", "SRC"]
independent = ["SPEC"]

[graph]
file = "docs/traceability/graph.toml"

[matrix]
file = "docs/traceability/matrix.md"
"#
    .to_string()
}

fn v01_config_text() -> String {
    r#"[id]
area = "LGX"
pattern = "^[A-Z]+-LGX-\\d{3}$"
seq_digits = 3

[id.chain]
order = ["UC", "RB", "SEQ", "DD", "TS", "TC", "SRC"]
independent = ["SPEC"]

[matrix]
file = "docs/traceability/matrix.md"
"#
    .to_string()
}

fn v01_config_no_order_text() -> String {
    r#"[id]
area = "LGX"
pattern = "^[A-Z]+-LGX-\\d{3}$"
seq_digits = 3

[matrix]
file = "docs/traceability/matrix.md"
"#
    .to_string()
}

fn matrix_sample_text() -> String {
    r#"# Traceability Matrix

| ID | UC | DD | SPEC | 概要 |
| -- | -- | -- | ---- | ---- |
| C-1 | UC-OLD-001 | DD-OLD-001 | SPEC-OLD-001 | sample chain |
"#
    .to_string()
}

fn matrix_invalid_node_text() -> String {
    // typecode が ICONIX 範囲（2〜4 文字）を超える 5 文字の成果物を含め、
    // 書換後の出力 graph 妥当性検証（新 ID 形式）を違反させる（OutputGraphInvalid）。
    r#"# Traceability Matrix

| ID | TYPE | path | 概要 |
| -- | ---- | ---- | ---- |
| C-1 | WRONG-OLD-001 | docs/x.md | invalid typecode |
"#
    .to_string()
}
