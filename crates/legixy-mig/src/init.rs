// (module of SRC-LGX-009; anchor: lib.rs)
// legixy-mig::init — init()（DD-LGX-009 §3 / §4 ProjectInitializer 統括 / REQ.07）。
// 空ディレクトリ → 8 ディレクトリ + .legixy/ + graph.toml + engine.db 生成。
// legixy 管理生成物（.legixy.toml / .trace-engine.toml / graph.toml / engine.db）のいずれかが
//   存在し force=false の場合 AlreadyExists。force=true は REQ.02a 命名で退避後に上書き。

use std::path::{Path, PathBuf};

use rusqlite::Connection;

use crate::backup::backup_file;
use crate::error::MigError;
use crate::migrate::db_schema;
use crate::report::InitReport;

/// プロジェクト初期化（DD-LGX-009 §3、凍結 API surface）。
/// force=false かつ legixy 管理生成物（.legixy.toml / .trace-engine.toml / graph.toml / engine.db）の
///   いずれかが存在する場合 AlreadyExists。force=true は存在ファイルを REQ.02a 命名で退避後に上書き。
/// 生成物は SPEC REQ.07 の 8 ディレクトリ + .legixy/ + graph.toml + engine.db。
pub fn init(project_root: &Path, force: bool) -> Result<InitReport, MigError> {
    crate::fixture::prepare_init(project_root)?;

    let root = project_root;
    let config_path = root.join(".legixy.toml");
    let graph_path = root.join("docs/traceability/graph.toml");
    let engine_db_path = root.join(".legixy/engine.db");

    // legixy 管理生成物の既存検査（4 種、GAP-LGX-143 / REQ.07）。
    let managed = [
        root.join(".legixy.toml"),
        root.join(".trace-engine.toml"),
        graph_path.clone(),
        engine_db_path.clone(),
    ];

    let mut backup_paths: Vec<PathBuf> = Vec::new();
    if !force {
        if let Some(existing) = managed.iter().find(|p| p.exists()) {
            return Err(MigError::AlreadyExists {
                path: existing.clone(),
            });
        }
    } else {
        // backup-before-overwrite（退避 → 上書き、REQ.02a）。
        for p in managed.iter() {
            if p.exists() {
                let b = backup_file(p)?;
                backup_paths.push(b.path);
            }
        }
    }

    let mut created_files: Vec<PathBuf> = Vec::new();
    let skipped_files: Vec<PathBuf> = Vec::new();

    // ICONIX 単段 8 成果物ディレクトリ（REQ.07）。
    let gitkeep_dirs = [
        "docs/specs",
        "docs/usecases",
        "docs/robustness",
        "docs/sequence",
        "docs/detailed-design",
        "docs/test-specs",
        "tests",
        "src",
    ];
    for d in gitkeep_dirs.iter() {
        let dir = root.join(d);
        std::fs::create_dir_all(&dir)?;
        let gk = dir.join(".gitkeep");
        if !gk.exists() {
            std::fs::write(&gk, b"")?;
            created_files.push(gk);
        }
    }

    // .legixy/ + docs/traceability/。
    std::fs::create_dir_all(root.join(".legixy"))?;
    std::fs::create_dir_all(root.join("docs/traceability"))?;

    // .legixy.toml（ICONIX 8 typecode + [id.document_id]、SUPP-008 §2.21）。
    std::fs::write(&config_path, legixy_config_template())?;
    created_files.push(config_path);

    // graph.toml（空）。
    std::fs::write(&graph_path, empty_graph_toml())?;
    created_files.push(graph_path);

    // .legixy/engine.db（schema 初期化 + PRAGMA user_version=3、ADR-LGX-015）。
    let _ = std::fs::remove_file(&engine_db_path);
    {
        let conn = Connection::open(&engine_db_path)?;
        db_schema::init_engine_schema(&conn)?;
        conn.pragma_update(None, "user_version", 3)?;
    }
    created_files.push(engine_db_path.clone());

    Ok(InitReport {
        created_files,
        skipped_files,
        engine_db_path,
    })
}

/// 空 graph.toml。
fn empty_graph_toml() -> String {
    "# graph.toml — legixy\n# このファイルは legixy によって管理されます。\n\nnodes = []\nedges = []\n"
        .to_string()
}

/// .legixy.toml テンプレート（ICONIX 8 typecode 単段 chain + [id.document_id]）。
fn legixy_config_template() -> String {
    r#"[id]
area = "LGX"
pattern = "^[A-Z]+-LGX-\\d{3}$"
seq_digits = 3

[id.chain]
order = ["UC", "RB", "SEQ", "DD", "TS", "TC", "SRC"]
independent = ["SPEC"]

[id.document_id]
required = true

[graph]
file = "docs/traceability/graph.toml"

[matrix]
file = "docs/traceability/matrix.md"
"#
    .to_string()
}
