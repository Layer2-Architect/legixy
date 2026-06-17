// Document ID: SRC-CLI-001
// SRC: legixy CLI 統合層（実行ファイル `legixy`）。配送軸 area=CLI（DevProc_V4.1 §12、CTR-CLI-001 系）。
// 親: LGX-COMPAT-001 §3（19 サブコマンド凍結契約・終了コード）+ 各 UC の DD（§4 module / §8 integration）。
//
// 本ビルドの配線状況（増分 1）:
//   - 配線済み（純グラフ系・engine.db 不要）: check / impact / investigate
//   - サーフェス宣言済み・dispatch 未配線（次段で engine.db 接続層 + 各ライブラリ wiring）:
//     init / migrate / embed / drift / report / calibrate / snapshot / refresh-subnodes /
//     context / feedback / observe / audit / analyze / proposals / approve / reject
//   clap によるサブコマンド名・引数・終了コード 2（使用法誤り）は全 19 で契約準拠（パース fidelity）。
//
// 終了コード（LGX-COMPAT-001 §3 / SPEC-LGX-004.REQ.04）: 0=成功、1=実行時失敗 or check error>0、2=clap 構文誤り。

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::{Parser, Subcommand, ValueEnum};

mod refresh;

use legixy_check::{
    exit_code as check_exit_code, run_with_root as check_run_with_root, CheckMode,
    CheckReport,
};
use legixy_core::{Config, Id, Severity};
use legixy_ctx::db::DbConn as CtxDbConn;
use legixy_embed::{
    compute_model_version, BucketCount, EmbedOptions, Embedder, EmbeddingStore, NodeFilter,
    PreprocessProfile,
};
use legixy_ctx::{CompileInput, ContextCompiler, Granularity as CtxGranularity};
use legixy_feedback::observer::NewObservation;
use legixy_mig::{MigOutputFormat, MigrateOpts, MigrationReport, UnmappedIdPolicy};
use legixy_feedback::{
    Connection, ContextAuditReader, FeedbackCli, ObservationRecorder, ProposalStatus,
};
use legixy_graph::subnode::{parse_graph, GraphParseError};
use legixy_graph::TraceGraph;
use legixy_nav::{impact, investigate_with_depth, render_multi, render_outcome, ReportFormat};

#[derive(Parser)]
#[command(
    name = "legixy",
    version,
    about = "legixy — 有向グラフ主体のトレーサビリティエンジン（traceability-engine 互換）"
)]
struct Cli {
    /// プロジェクトルート（graph.toml / engine.db / 設定の基準）。MCP は常に明示指定する。
    #[arg(long, global = true, default_value = ".")]
    project_root: PathBuf,

    /// JSON 出力モード（全コマンド共通、LGX-COMPAT-001 §3）。
    #[arg(long, global = true)]
    json: bool,

    /// ONNX モデルディレクトリ（既定は設定の `model_dir`、§3）。
    #[arg(long, global = true)]
    models_dir: Option<PathBuf>,

    #[command(subcommand)]
    command: Command,
}

#[derive(Clone, Copy, ValueEnum)]
enum FormatArg {
    Markdown,
    Json,
}

#[derive(Clone, Copy, ValueEnum)]
enum GranularityArg {
    Document,
    Subnode,
}

#[derive(Clone, Copy, ValueEnum)]
enum ObserveCategoryArg {
    #[value(name = "compile_miss")]
    CompileMiss,
    #[value(name = "review_correction")]
    ReviewCorrection,
    #[value(name = "manual_note")]
    ManualNote,
}

impl ObserveCategoryArg {
    fn as_str(self) -> &'static str {
        match self {
            ObserveCategoryArg::CompileMiss => "compile_miss",
            ObserveCategoryArg::ReviewCorrection => "review_correction",
            ObserveCategoryArg::ManualNote => "manual_note",
        }
    }
}

#[derive(Clone, Copy, ValueEnum)]
enum ProposalStatusArg {
    Pending,
    Approved,
    Rejected,
}

impl ProposalStatusArg {
    fn to_status(self) -> ProposalStatus {
        match self {
            ProposalStatusArg::Pending => ProposalStatus::Pending,
            ProposalStatusArg::Approved => ProposalStatus::Approved,
            ProposalStatusArg::Rejected => ProposalStatus::Rejected,
        }
    }
}

#[derive(Subcommand)]
enum Command {
    /// `.trace-engine.toml` を `.bak` 退避し上書き生成（LGX-COMPAT-001 #1）。
    Init {
        #[arg(long)]
        force: bool,
    },
    /// v0.1.0 → 現行への移行（#2）。
    Migrate {
        #[arg(long)]
        from: PathBuf,
        #[arg(long)]
        to: Option<PathBuf>,
        #[arg(long)]
        dry_run: bool,
        #[arg(long, value_enum, default_value_t = FormatArg::Markdown)]
        format: FormatArg,
    },
    /// chain 整合・DAG 健全性の検証（#3、G1 ゲート）。
    Check {
        /// 形式層のみ（意味層スキップ）。
        #[arg(long)]
        formal: bool,
    },
    /// embedding 再生成（#4）。
    Embed {
        /// 全ノード対象（既定）。`--node` と排他（契約 §4#4、BUG-008）。
        #[arg(long, conflicts_with = "node")]
        all: bool,
        #[arg(long = "node")]
        node: Vec<String>,
        #[arg(long)]
        force: bool,
    },
    /// standalone ドリフト対比（#5）。
    Drift {
        artifact_id: String,
        #[arg(long)]
        against: Option<String>,
    },
    /// 全リンク類似度＋候補一覧（#6）。
    Report,
    /// 類似度分布／推奨閾値（#7）。
    Calibrate {
        #[arg(long, default_value_t = 10)]
        buckets: usize,
        #[arg(long)]
        recommend: bool,
    },
    /// ベースライン凍結管理（#8）。
    Snapshot {
        #[command(subcommand)]
        cmd: SnapshotCmd,
    },
    /// 見出しリネーム時のサブノード ID 連鎖反映（#9）。
    RefreshSubnodes {
        #[arg(long)]
        dry_run: bool,
        /// 変更を適用。`--dry-run` と排他（使用法誤りは clap が exit 2、契約 §3、BUG-009）。
        #[arg(long, conflicts_with = "dry_run")]
        apply: bool,
    },
    /// コンテキスト解決（MCP compile_context の下位層、#10）。
    Context {
        #[arg(required = true)]
        target_files: Vec<String>,
        #[arg(long)]
        command: Option<String>,
        #[arg(long, value_enum)]
        granularity: Option<GranularityArg>,
        #[arg(long)]
        outline_only: bool,
        #[arg(long)]
        sections: Option<String>,
        #[arg(long)]
        depth: Option<usize>,
    },
    /// 順方向探索（#11）。
    Impact {
        start: String,
        #[arg(long)]
        max_depth: Option<usize>,
    },
    /// 逆方向探索（#12）。
    Investigate {
        start: String,
        #[arg(long)]
        max_depth: Option<usize>,
    },
    /// check 結果から Observation 自動生成（#13）。
    Feedback,
    /// 気づきの記録（MCP observe の下位層、#14）。
    Observe {
        #[arg(value_enum)]
        category: ObserveCategoryArg,
        message: String,
        #[arg(long)]
        severity: Option<String>,
        #[arg(long = "related-id")]
        related_id: Vec<String>,
        #[arg(long = "target-file")]
        target_file: Vec<String>,
        #[arg(long = "missing-doc")]
        missing_doc: Option<String>,
        #[arg(long = "source-glob")]
        source_glob: Option<String>,
    },
    /// コンテキスト解決履歴（MCP get_compile_audit の下位層、#15）。
    Audit {
        #[arg(long, default_value_t = 10, value_parser = clap::value_parser!(u32).range(1..=50))]
        limit: u32,
    },
    /// pending Observation → Proposal 生成（#16）。
    Analyze,
    /// Proposal 一覧（#17）。
    Proposals {
        #[arg(long, value_enum)]
        status: Option<ProposalStatusArg>,
    },
    /// Proposal 承認（人間のみ、#18）。
    Approve {
        id: i64,
    },
    /// Proposal 却下（#19）。
    Reject {
        id: i64,
        #[arg(long)]
        reason: String,
    },
}

#[derive(Subcommand)]
enum SnapshotCmd {
    Create {
        #[arg(long)]
        label: Option<String>,
    },
    List,
    Delete {
        target: String,
    },
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    match dispatch(&cli) {
        Ok(code) => code,
        Err(msg) => {
            eprintln!("{msg}");
            ExitCode::from(1)
        }
    }
}

/// graph.toml の正準配置（プロジェクトルート相対、`.trace-engine.toml` の graph location 既定）。
fn graph_path(project_root: &Path) -> PathBuf {
    project_root.join("docs/traceability/graph.toml")
}

fn load(project_root: &Path) -> Result<TraceGraph, String> {
    let p = graph_path(project_root);
    // サブノード自動抽出を含む正準ローダ（DD-LGX-003 §1/§3「上位呼び出しは legixy-cli が
    // parse_graph を呼ぶことで間接的に UC-LGX-003 を起動する」）。graph.toml の宣言ノードに加え、
    // .md ドキュメントの h2/h3 見出しを AutoGenerated サブノードとして materialize し、明示 `#s:`
    // を含む全サブノードに ParentChild エッジを張る（BUG-007 R-1〜3,7: --granularity subnode の
    // 削減・#s: 明示上流の非潰し・SubnodeIdFormat 検出を E2E で有効化）。
    parse_graph(&p, project_root).map_err(|e| match e {
        GraphParseError::Io(m) => format!("graph load failed (io): {} — {m}", p.display()),
        other => format!("graph parse failed: {} — {other}", p.display()),
    })
}

/// 設定を解決する（BUG-003、LGX-COMPAT-001 §6 / SPEC-LGX-008.REQ.13）。
/// `.legixy.toml` → `.trace-engine.toml` の順に探索。不在は既定、不正 TOML は exit 1。
/// 旧名 fallback 時は移行 Info を stderr に出す。
fn load_config(project_root: &Path) -> Result<Config, String> {
    let loaded = Config::load(project_root).map_err(|e| format!("config load failed: {e}"))?;
    if loaded.legacy_fallback {
        if let Some(p) = &loaded.source {
            eprintln!(
                "info: 旧名 config を読み込みました（{}）。`.legixy.toml` への移行を推奨します。",
                p.display()
            );
        }
    }
    Ok(loaded.config)
}

/// engine.db のパスを解決する（ADR-LGX-015）。
/// 正準 `.legixy/engine.db` を優先。**書込は常に正準パスへ**（write=true は `.trace-engine/` へ
/// フォールバックしない）。read-only（write=false）かつ正準不在のときのみ `.trace-engine/engine.db`
/// を読取フォールバック（v3 データ相互運用）。最終的に正準を使う場合は親 dir を作成する。
fn engine_db_path(project_root: &Path, write: bool) -> Result<PathBuf, String> {
    let canonical = project_root.join(".legixy/engine.db");
    if canonical.exists() {
        return Ok(canonical);
    }
    if !write {
        let fallback = project_root.join(".trace-engine/engine.db");
        if fallback.exists() {
            return Ok(fallback);
        }
    }
    if let Some(parent) = canonical.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("create {} failed: {e}", parent.display()))?;
    }
    Ok(canonical)
}

/// engine.db が既に存在するか（read-only check が db を新規作成しないための事前判定、BUG-005）。
fn engine_db_exists(project_root: &Path) -> bool {
    project_root.join(".legixy/engine.db").exists()
        || project_root.join(".trace-engine/engine.db").exists()
}

/// engine.db を開く（スキーマ初期化 + WAL + busy_timeout は legixy-feedback::Connection が担う）。
fn open_db(project_root: &Path, write: bool) -> Result<Connection, String> {
    let path = engine_db_path(project_root, write)?;
    Connection::open_path(&path.to_string_lossy())
        .map_err(|e| format!("engine.db open failed: {} — {e}", path.display()))
}

/// context 監査ログ用に engine.db を開く（書込。失敗時 None = audit no-op、context は継続）。
fn open_ctx_db(project_root: &Path) -> Option<CtxDbConn> {
    let path = engine_db_path(project_root, true).ok()?;
    CtxDbConn::open(&path.to_string_lossy()).ok()
}

/// embed 系（embeddings / snapshots テーブル）用に engine.db を開く（EmbeddingStore、ADR-LGX-015）。
fn open_embed_store(project_root: &Path, write: bool) -> Result<EmbeddingStore, String> {
    let path = engine_db_path(project_root, write)?;
    EmbeddingStore::open_on_disk(&path, 5000)
        .map_err(|e| format!("engine.db open failed: {} — {e}", path.display()))
}

/// ONNX モデルディレクトリを解決する（embed/drift 用、LGX-COMPAT-001 §3 / ADR-LGX-016）。
/// 優先順: `--models-dir`（CLI 明示）→ `LGX_MODELS_DIR` env → 設定 `model_dir` → `<root>/models/...`。
fn resolve_model_dir(
    project_root: &Path,
    override_dir: Option<&Path>,
    config: &Config,
) -> Result<PathBuf, String> {
    // model.onnx を含む候補を順に検査。
    let mut candidates: Vec<PathBuf> = Vec::new();
    if let Some(d) = override_dir {
        candidates.push(d.to_path_buf());
    }
    if let Ok(dir) = std::env::var("LGX_MODELS_DIR") {
        candidates.push(PathBuf::from(dir));
    }
    if let Some(m) = &config.semantic.model_dir {
        // 相対指定は project_root 基準で解決。
        let p = PathBuf::from(m);
        candidates.push(if p.is_absolute() {
            p
        } else {
            project_root.join(p)
        });
    }
    candidates.push(project_root.join("models/paraphrase-multilingual-MiniLM-L12-v2"));

    for c in &candidates {
        if c.join("model.onnx").exists() {
            return Ok(c.clone());
        }
    }
    Err("ONNX モデルが見つかりません（--models-dir / LGX_MODELS_DIR / 設定 model_dir / <root>/models/paraphrase-multilingual-MiniLM-L12-v2 のいずれにも model.onnx がありません）".to_string())
}

/// observe の `--target-file` / `--missing-doc` / `--source-glob` を context_json へ畳む。
/// いずれも未指定なら None（NewObservation.context_json）。
fn build_context_json(
    target_files: &[String],
    missing_doc: Option<&str>,
    source_glob: Option<&str>,
) -> Option<String> {
    if target_files.is_empty() && missing_doc.is_none() && source_glob.is_none() {
        return None;
    }
    let mut obj = serde_json::Map::new();
    if !target_files.is_empty() {
        obj.insert("target_files".to_string(), serde_json::json!(target_files));
    }
    if let Some(d) = missing_doc {
        obj.insert("missing_doc".to_string(), serde_json::json!(d));
    }
    if let Some(g) = source_glob {
        obj.insert("source_glob".to_string(), serde_json::json!(g));
    }
    Some(serde_json::Value::Object(obj).to_string())
}

fn dispatch(cli: &Cli) -> Result<ExitCode, String> {
    let root = cli.project_root.as_path();
    let json = cli.json; // グローバル --json（全コマンド共通、§3）。
    let models_dir = cli.models_dir.as_deref(); // グローバル --models-dir（§3）。
    // nav 系の出力フォーマット（--json → JSON Lines）。
    let nav_format = if json {
        ReportFormat::JsonLines
    } else {
        ReportFormat::Text
    };
    match &cli.command {
        Command::Check { formal } => {
            let graph = load(root)?;
            let config = load_config(root)?;
            let mode = if *formal {
                CheckMode::Formal
            } else {
                CheckMode::Full
            };
            // 意味層ストア（BUG-005）: Full かつ engine.db 在のとき配線（read-only check が db を
            // 新規作成しないよう存在判定を先に行う）。db 不在 or 形式層のみは None（Info 1 件 or 省略）。
            let store = if matches!(mode, CheckMode::Full) && engine_db_exists(root) {
                open_embed_store(root, false).ok()
            } else {
                None
            };
            // FileExistence/OrphanFile は project_root 必須（BUG-006）。
            let report = check_run_with_root(&graph, &config, mode, store.as_ref(), root)
                .map_err(|e| format!("check failed: {e}"))?;
            if json {
                // JSON Lines（1 finding = 1 行）。0 件は 0 レコード = 無出力（余分な空行を出さない）。
                let body = report.to_json();
                if !body.is_empty() {
                    println!("{body}");
                }
            } else {
                print!("{}", render_check_report(&report));
            }
            Ok(ExitCode::from(check_exit_code(&report) as u8))
        }
        Command::Impact { start, max_depth } => {
            let graph = load(root)?;
            let result = impact(&graph, std::slice::from_ref(start), *max_depth)
                .map_err(|e| format!("impact failed: {e}"))?;
            print!("{}", render_multi(&result, nav_format));
            Ok(ExitCode::SUCCESS)
        }
        Command::Investigate { start, max_depth } => {
            let graph = load(root)?;
            let config = load_config(root)?;
            // 増分 1: engine.db 未配線 → db=None（意味的ドリフト剪定なし）。打ち切りでも exit 0。
            // 剪定しきい値は config 由来（BUG-003: 旧来の固定 0.25 を撤廃、SD-THR-010）。
            let outcome = investigate_with_depth(
                &graph,
                std::slice::from_ref(start),
                None,
                config.semantic.drift_threshold,
                *max_depth,
            )
            .map_err(|e| format!("investigate failed: {e}"))?;
            print!("{}", render_outcome(&outcome, nav_format));
            Ok(ExitCode::SUCCESS)
        }
        // ── init / migrate（legixy-mig。プロジェクト初期化と v0.1.0 移行、DD-LGX-009） ──
        Command::Init { force } => {
            let report = legixy_mig::init(root, *force).map_err(|e| format!("init failed: {e}"))?;
            if json {
                println!(
                    "{}",
                    serde_json::json!({
                        "created": report.created_files.iter().map(|f| f.display().to_string()).collect::<Vec<_>>(),
                        "skipped": report.skipped_files.iter().map(|f| f.display().to_string()).collect::<Vec<_>>(),
                        "engine_db": report.engine_db_path.display().to_string(),
                    })
                );
            } else {
                for f in &report.created_files {
                    println!("created: {}", f.display());
                }
                for f in &report.skipped_files {
                    println!("skipped (exists): {}", f.display());
                }
                println!("engine.db: {}", report.engine_db_path.display());
            }
            Ok(ExitCode::SUCCESS)
        }
        Command::Migrate {
            from,
            to,
            dry_run,
            format,
        } => {
            // --to 既定は --project-root（LGX-COMPAT-001 #2）。
            let dst = to.clone().unwrap_or_else(|| root.to_path_buf());
            let opts = MigrateOpts {
                dry_run: *dry_run,
                format: match format {
                    FormatArg::Json => MigOutputFormat::Json,
                    FormatArg::Markdown => MigOutputFormat::Markdown,
                },
                unmapped_policy: UnmappedIdPolicy::Abort, // 既定: 非破壊性優先（REQ.11）。
            };
            let report =
                legixy_mig::migrate(from, &dst, opts).map_err(|e| format!("migrate failed: {e}"))?;
            // グローバル --json も JSON を選択（--format json と等価、§3）。
            if json || matches!(format, FormatArg::Json) {
                println!("{}", report.to_json());
            } else {
                print!("{}", render_migration_text(&report));
            }
            Ok(ExitCode::SUCCESS)
        }
        // ── context（graph + engine.db audit log、legixy-ctx。MCP compile_context の下位層、DD-LGX-002） ──
        Command::Context {
            target_files,
            command,
            granularity,
            outline_only,
            sections,
            depth,
        } => {
            let graph = load(root)?;
            let config = load_config(root)?;
            // 監査ログ用に engine.db を開く（ベストエフォート: 失敗時は None=audit no-op、context は継続）。
            let db = open_ctx_db(root);
            let compiler = ContextCompiler::new(&graph, &config, db.as_ref(), root);
            let input = CompileInput {
                target_files: target_files.iter().map(PathBuf::from).collect(),
                granularity: match granularity {
                    Some(GranularityArg::Subnode) => CtxGranularity::Subnode,
                    _ => CtxGranularity::Document,
                },
                command: command.clone(),
                outline_only: *outline_only,
                sections: sections.as_ref().map(|s| {
                    s.split(',')
                        .map(|x| x.trim().to_string())
                        .filter(|x| !x.is_empty())
                        .collect()
                }),
                depth_limit: *depth,
            };
            let result = compiler
                .compile(&input)
                .map_err(|e| format!("context failed: {e}"))?;
            // ResultTooLarge は render 経路で Err → stderr + exit 1（REQ.13 文言、DD-freeze B-1）。
            let rendered = compiler.render(&result).map_err(|e| format!("{e}"))?;
            if json {
                // MCP compile_context は markdown 本文を返す。CLI --json はそれを JSON で包む。
                println!(
                    "{}",
                    serde_json::json!({ "markdown": rendered })
                );
            } else {
                print!("{rendered}");
            }
            Ok(ExitCode::SUCCESS)
        }
        // ── feedback 群（engine.db、legixy-feedback。Admin/Agent Surface、DD-LGX-008） ──
        Command::Feedback => {
            let graph = load(root)?;
            let config = load_config(root)?;
            // engine.db が事前に存在したか（open_db は正準パスに db を新規作成するため、
            // 意味層ストアの存在判定は作成前に行う。BUG-005 と同じ read-only 配慮）。
            let had_db = engine_db_exists(root);
            let db = open_db(root, true)?;
            // check の結果「や embedding」から Observation 生成（SPEC-LGX-007.REQ.02 / LGX-COMPAT-001 #13）。
            // 意味層（SemanticSimilarity = link_candidate / drift / 低類似度エッジ）は engine.db に
            // embeddings がある場合のみ生成される（R-4/R-5）。db 事前不在なら形式層のみ。
            let store = if had_db {
                open_embed_store(root, false).ok()
            } else {
                None
            };
            let mode = if store.is_some() {
                CheckMode::Full
            } else {
                CheckMode::Formal
            };
            // FileExistence/OrphanFile・drift は project_root 必須（BUG-006 / 意味層 drift）。
            let report = check_run_with_root(&graph, &config, mode, store.as_ref(), root)
                .map_err(|e| format!("check failed: {e}"))?;
            let fr = FeedbackCli::run_feedback(&db, &report)
                .map_err(|e| format!("feedback failed: {e}"))?;
            if json {
                println!(
                    "{}",
                    serde_json::json!({
                        "observations_created": fr.observations_created,
                        "observations_skipped": fr.observations_skipped,
                    })
                );
            } else {
                println!(
                    "feedback: observations created={}, skipped={}",
                    fr.observations_created, fr.observations_skipped
                );
            }
            Ok(ExitCode::SUCCESS)
        }
        Command::Observe {
            category,
            message,
            severity,
            related_id,
            target_file,
            missing_doc,
            source_glob,
        } => {
            let db = open_db(root, true)?;
            let obs = NewObservation {
                source: "manual".to_string(),
                category: category.as_str().to_string(),
                severity: severity.clone().unwrap_or_else(|| "info".to_string()),
                message: message.clone(),
                related_ids: related_id.clone(),
                context_json: build_context_json(
                    target_file,
                    missing_doc.as_deref(),
                    source_glob.as_deref(),
                ),
            };
            // message 空（trim 後 0 文字）は EmptyObservationMessage → exit 1（SPEC-LGX-007.REQ.04）。
            let r = ObservationRecorder::record(&obs, &db)
                .map_err(|e| format!("observe failed: {e}"))?;
            if json {
                println!(
                    "{}",
                    serde_json::json!({ "id": r.id, "skipped": r.skipped })
                );
            } else {
                // 凍結 stdout 形式（LGX-COMPAT-001 §4.1、MCP observe が parse する）。
                println!("observation: id={} skipped={}", r.id, r.skipped);
            }
            Ok(ExitCode::SUCCESS)
        }
        Command::Audit { limit } => {
            let db = open_db(root, false)?;
            let entries = ContextAuditReader::recent(&db, *limit as usize)
                .map_err(|e| format!("audit failed: {e}"))?;
            let arr: Vec<serde_json::Value> = entries
                .iter()
                .map(|e| {
                    serde_json::json!({
                        "id": e.id,
                        "target_id": e.target_id,
                        "granularity": e.granularity,
                        "payload": e.payload,
                        "created_at": e.created_at,
                    })
                })
                .collect();
            println!(
                "{}",
                serde_json::to_string(&arr).unwrap_or_else(|_| "[]".to_string())
            );
            Ok(ExitCode::SUCCESS)
        }
        Command::Analyze => {
            let db = open_db(root, true)?;
            let proposals =
                FeedbackCli::run_analyze(&db).map_err(|e| format!("analyze failed: {e}"))?;
            if json {
                let arr: Vec<serde_json::Value> = proposals
                    .iter()
                    .map(|p| serde_json::json!({ "id": p.id, "kind": p.kind, "title": p.title }))
                    .collect();
                println!("{}", serde_json::Value::Array(arr));
            } else {
                println!("analyze: {} proposal(s) generated", proposals.len());
                for p in &proposals {
                    println!("  #{} [{}] {}", p.id, p.kind, p.title);
                }
            }
            Ok(ExitCode::SUCCESS)
        }
        Command::Proposals { status } => {
            let db = open_db(root, false)?;
            let filter = status.as_ref().map(|s| s.to_status());
            let list = FeedbackCli::list_proposals(&db, filter)
                .map_err(|e| format!("proposals failed: {e}"))?;
            if json {
                let arr: Vec<serde_json::Value> = list
                    .iter()
                    .map(|p| {
                        serde_json::json!({
                            "id": p.id,
                            "status": p.status.as_str(),
                            "kind": p.kind,
                            "title": p.title,
                            "created_at": p.created_at,
                        })
                    })
                    .collect();
                println!("{}", serde_json::Value::Array(arr));
            } else if list.is_empty() {
                println!("(no proposals)");
            } else {
                for p in &list {
                    println!(
                        "#{} [{}] {} — {} ({})",
                        p.id,
                        p.status.as_str(),
                        p.kind,
                        p.title,
                        p.created_at
                    );
                }
            }
            Ok(ExitCode::SUCCESS)
        }
        Command::Approve { id } => {
            let db = open_db(root, true)?;
            FeedbackCli::approve(&db, *id).map_err(|e| format!("approve failed: {e}"))?;
            if json {
                println!("{}", serde_json::json!({ "id": id, "status": "approved" }));
            } else {
                println!("approved proposal #{id}");
            }
            Ok(ExitCode::SUCCESS)
        }
        Command::Reject { id, reason } => {
            let db = open_db(root, true)?;
            // reason 空（trim 後 0 文字）は EmptyRejectReason → exit 1（GAP-LGX-124）。
            FeedbackCli::reject(&db, *id, reason).map_err(|e| format!("reject failed: {e}"))?;
            if json {
                println!("{}", serde_json::json!({ "id": id, "status": "rejected" }));
            } else {
                println!("rejected proposal #{id}");
            }
            Ok(ExitCode::SUCCESS)
        }
        // ── embed 系（legixy-embed、engine.db embeddings。DD-LGX-007/010/011/012/013） ──
        Command::Embed { all, node, force } => {
            let _ = all; // --all は既定（NodeFilter::All）。--node 指定時のみ Ids。
            let graph = load(root)?;
            let config = load_config(root)?;
            let store = open_embed_store(root, true)?;
            let model_dir = resolve_model_dir(root, models_dir, &config)?;
            let model_version = compute_model_version(
                "model",
                &model_dir.join("model.onnx"),
                PreprocessProfile::Plain,
                384,
            )
            .map_err(|e| format!("model_version 計算失敗: {e}"))?;
            // ONNX feature 無効ビルドでは Embedder::new が ModelShapeInvalid を返す（embed 未対応）。
            let embedder = Embedder::new(&model_dir, &model_version)
                .map_err(|e| format!("embedder 初期化失敗（onnx feature ビルドが必要）: {e}"))?;
            let filter = if node.is_empty() {
                NodeFilter::All
            } else {
                NodeFilter::Ids(node.clone())
            };
            // Contextual Retrieval（CACHE-CR-002）: `[contextual_retrieval] enabled=true` のとき
            // ContextualConfig を渡す。これにより各 embedding に context/context_hash が生成・格納される。
            let contextual = if config.contextual_retrieval_enabled {
                Some(legixy_embed::ContextualConfig {
                    opts: legixy_embed::CrOptions::default(),
                })
            } else {
                None
            };
            let options = EmbedOptions {
                force: *force,
                node_filter: filter,
                project_root: Some(root.to_path_buf()),
                contextual,
                ..Default::default()
            };
            let report = legixy_embed::embed_all(&graph, &store, &embedder, options)
                .map_err(|e| format!("embed failed: {e}"))?;
            if json {
                println!(
                    "{}",
                    serde_json::to_string(&report).unwrap_or_else(|_| "{}".to_string())
                );
            } else {
                println!(
                    "embed: generated={} skipped={} failed={}",
                    report.generated, report.skipped, report.failed
                );
                for e in &report.errors {
                    eprintln!("  error {}: {}", e.node_id, e.message);
                }
            }
            Ok(ExitCode::SUCCESS)
        }
        Command::Drift {
            artifact_id,
            against,
        } => {
            let graph = load(root)?;
            let store = open_embed_store(root, false)?;
            let config = load_config(root)?;
            let against_spec = legixy_embed::drift::parse_against(against.as_deref())
                .map_err(|e| format!("--against parse 失敗: {e}"))?;
            // project_root 基準でモデル/現物を解決し実推論（BUG-004）。--models-dir 反映。
            let result = legixy_embed::drift::run(
                &graph,
                &store,
                &config,
                &Id::new(artifact_id),
                against_spec,
                root,
                models_dir,
            );
            let code = legixy_embed::drift::exit_code(&result);
            match &result {
                Ok(r) => {
                    if json {
                        println!(
                            "{}",
                            serde_json::json!({
                                "artifact_id": artifact_id,
                                "drift": r.drift,
                                "baseline_available": r.drift.is_some(),
                                "baseline_source": format!("{:?}", r.baseline_source),
                            })
                        );
                    } else {
                        match r.drift {
                            Some(d) => println!(
                                "drift: {artifact_id} = {d:.4} (baseline_source={:?})",
                                r.baseline_source
                            ),
                            None => println!(
                                "drift: {artifact_id} = baseline なし (baseline_available=false)"
                            ),
                        }
                    }
                }
                Err(e) => {
                    if json {
                        eprintln!("{}", serde_json::json!({ "error": format!("{e}") }));
                    } else {
                        eprintln!("drift error: {e}");
                    }
                }
            }
            Ok(ExitCode::from(code as u8))
        }
        Command::Report => {
            let graph = load(root)?;
            let store = open_embed_store(root, false)?;
            let config = load_config(root)?;
            let output = legixy_embed::run_report(&graph, &store, &config)
                .map_err(|e| format!("report failed: {e}"))?;
            if json {
                println!("{}", output.to_json());
            } else {
                print!("{}", output.to_text());
            }
            Ok(ExitCode::SUCCESS)
        }
        Command::Calibrate { buckets, recommend } => {
            let store = open_embed_store(root, false)?;
            let config = load_config(root)?;
            let bc = BucketCount::new(*buckets).map_err(|e| format!("buckets 不正: {e}"))?;
            let report = legixy_embed::calibrate(&store, &config, bc, *recommend)
                .map_err(|e| format!("calibrate failed: {e}"))?;
            if json {
                println!(
                    "{}",
                    serde_json::json!({
                        "pairs": report.pairs,
                        "buckets": report.distribution.len(),
                        "recommended": report.recommended.is_some(),
                    })
                );
            } else {
                println!("{report:#?}");
            }
            Ok(ExitCode::SUCCESS)
        }
        Command::Snapshot { cmd } => {
            // create / delete は書込、list は読取（ADR-LGX-015）。
            let store = open_embed_store(root, !matches!(cmd, SnapshotCmd::List))?;
            match cmd {
                SnapshotCmd::Create { label } => {
                    let id = legixy_embed::snapshot::generate_snapshot_id();
                    let r = legixy_embed::snapshot::create(&store, &id, label.as_deref())
                        .map_err(|e| format!("snapshot create failed: {e}"))?;
                    // 空ストア（node_count=0）は非永続。明示メッセージ（UC-LGX-012、BUG-010）。
                    let persisted = r.node_count > 0;
                    if json {
                        println!(
                            "{}",
                            serde_json::json!({
                                "snapshot_id": r.snapshot_id,
                                "label": r.label,
                                "node_count": r.node_count,
                                "persisted": persisted,
                            })
                        );
                    } else if persisted {
                        println!(
                            "snapshot created: {} (label={:?}, nodes={})",
                            r.snapshot_id, r.label, r.node_count
                        );
                    } else {
                        println!(
                            "snapshot 未作成: embeddings がありません（非永続）。`legixy embed --all` を先に実行してください"
                        );
                    }
                }
                SnapshotCmd::List => {
                    let metas = legixy_embed::snapshot::list(&store)
                        .map_err(|e| format!("snapshot list failed: {e}"))?;
                    if json {
                        let arr: Vec<serde_json::Value> = metas
                            .iter()
                            .map(|m| {
                                serde_json::json!({
                                    "snapshot_id": m.snapshot_id,
                                    "label": m.label,
                                    "node_count": m.node_count,
                                    "taken_at": m.taken_at,
                                })
                            })
                            .collect();
                        println!("{}", serde_json::Value::Array(arr));
                    } else if metas.is_empty() {
                        println!("(no snapshots)");
                    } else {
                        for m in &metas {
                            println!(
                                "{} label={:?} nodes={} taken_at={}",
                                m.snapshot_id, m.label, m.node_count, m.taken_at
                            );
                        }
                    }
                }
                SnapshotCmd::Delete { target } => {
                    // target = snapshot_id または "label:<L>"（LGX-COMPAT-001 #8）。
                    let snapshot_id = if let Some(label) = target.strip_prefix("label:") {
                        match legixy_embed::snapshot::resolve_label(&store, label)
                            .map_err(|e| format!("resolve label failed: {e}"))?
                        {
                            legixy_embed::LabelResolveResult::Resolved(id) => id,
                            // delete の label 不在は exit 1（SPEC-LGX-010.REQ.03 6c）。
                            legixy_embed::LabelResolveResult::NotFound => {
                                return Err(format!(
                                    "label '{label}' に対応する snapshot がありません"
                                ))
                            }
                        }
                    } else {
                        target.clone()
                    };
                    let r = legixy_embed::snapshot::delete(&store, &snapshot_id)
                        .map_err(|e| format!("snapshot delete failed: {e}"))?;
                    if json {
                        println!(
                            "{}",
                            serde_json::json!({
                                "snapshot_id": r.snapshot_id,
                                "deleted_rows": r.deleted_rows,
                            })
                        );
                    } else {
                        println!(
                            "snapshot deleted: {} (rows={})",
                            r.snapshot_id, r.deleted_rows
                        );
                    }
                }
            }
            Ok(ExitCode::SUCCESS)
        }
        // ── refresh-subnodes（見出しリネーム連鎖反映、ADR-LGX-023、LGX-COMPAT-001 #9） ──
        Command::RefreshSubnodes { dry_run, apply } => {
            // --dry-run / --apply の排他は clap が exit 2 で弾く（BUG-009）。ここに来る時点で非両立。
            let _ = dry_run;
            let do_apply = *apply; // 既定（どちらも未指定）= dry-run。
            let graph = load(root)?;
            let db_path = engine_db_path(root, do_apply)?;
            let store = EmbeddingStore::open_on_disk(&db_path, 5000)
                .map_err(|e| format!("engine.db open failed: {} — {e}", db_path.display()))?;
            let report = refresh::detect_changes(&graph, &store, root)?;
            if do_apply && !report.renames.is_empty() {
                // --apply 前に engine.db をバックアップ（LGX-COMPAT-001 #9: .refresh-bak.{epoch}）。
                let epoch = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_secs())
                    .unwrap_or(0);
                let backup = format!("{}.refresh-bak.{epoch}", db_path.display());
                std::fs::copy(&db_path, &backup)
                    .map_err(|e| format!("engine.db backup failed: {e}"))?;
                eprintln!("backup: {backup}");
                refresh::apply_renames(&store, &report)?;
            }
            if json {
                println!(
                    "{}",
                    serde_json::json!({
                        "applied": do_apply,
                        "parents_scanned": report.parents_scanned,
                        "renames": report.renames.iter().map(|r| serde_json::json!({
                            "old_id": r.old_id, "new_id": r.new_id, "parent_id": r.parent_id,
                        })).collect::<Vec<_>>(),
                        "orphans": report.orphans.iter().map(|o| serde_json::json!({
                            "id": o.id, "parent_id": o.parent_id,
                        })).collect::<Vec<_>>(),
                    })
                );
            } else {
                print!("{}", refresh::render(&report, do_apply));
            }
            Ok(ExitCode::SUCCESS)
        } // 全 19 サブコマンド配線済み（catch-all なし = コンパイラが網羅性を保証）。
    }
}

/// MigrationReport を人間可読テキストへ整形（--format markdown）。
fn render_migration_text(r: &MigrationReport) -> String {
    let mut s = String::from("=== Migration Report ===\n");
    s.push_str(&format!("files_written: {}\n", r.files_written.len()));
    for f in &r.files_written {
        s.push_str(&format!("  {}\n", f.display()));
    }
    s.push_str(&format!("ids_rewritten: {}\n", r.ids_rewritten_count));
    s.push_str(&format!("id_map: {}\n", r.id_map_path.display()));
    s.push_str(&format!(
        "tables_copied: {} (rows={})\n",
        r.tables_copied.join(", "),
        r.rows_copied
    ));
    if !r.backup_paths.is_empty() {
        s.push_str(&format!("backups: {}\n", r.backup_paths.len()));
    }
    for w in &r.warnings {
        s.push_str(&format!("warning: {w}\n"));
    }
    s
}

/// CheckReport を人間可読テキストへ整形（v3 互換の簡潔書式）。
fn render_check_report(report: &CheckReport) -> String {
    let mut s = String::new();
    for f in &report.findings {
        let sev = match f.severity {
            Severity::Error => "ERROR",
            Severity::Warning => "WARNING",
            Severity::Info => "INFO",
            Severity::Ok => "OK",
        };
        s.push_str(&format!("[{sev}] {:?}: {}", f.category, f.message));
        if !f.related_ids.is_empty() {
            let ids: Vec<&str> = f.related_ids.iter().map(|i| i.as_str()).collect();
            s.push_str(&format!(" [{}]", ids.join(", ")));
        }
        s.push('\n');
    }
    let c = &report.counts;
    s.push_str(&format!(
        "\nSummary: {} error, {} warning, {} info\n",
        c.error, c.warning, c.info
    ));
    s
}
