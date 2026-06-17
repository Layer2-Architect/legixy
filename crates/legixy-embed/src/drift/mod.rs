// Document ID: SRC-LGX-013
// standalone drift 対比サブシステム: run / resolve_model / parse_against / compute_drift /
// exit_code と関連型（DD-LGX-013 §2.1・§2.2・§3）。
//
// TC[RED] scaffold。UC-013 の standalone drift コマンド（`legixy drift <artifact_id>`）。
// UC-007 の embed エンジン側 detect_drift（drift_detect.rs）とは別サブシステム。
//
// ベースライン解決・整合性検査（次元/model_version 二段）・非有限防御・drift 算出値域 [0,2]・
// 終了コード非対称性（baseline 不在 exit 0 / 壊れた状態 exit 1）は SRC[GREEN] で実装する。

use std::path::Path;

use legixy_core::{Config, Id};
use legixy_graph::TraceGraph;

use crate::error::DriftError;
use crate::store::EmbeddingStore;

/// ONNX モデルの解決経路（DD-LGX-013 §2.1）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModelSource {
    Flag,
    EnvLgx,
    EnvTe,
    ConfigFile,
}

/// 解決済み ONNX モデルの情報（DD-LGX-013 §2.1）。
pub struct ResolvedModel {
    pub model_version: String,
    pub dim: usize,
    pub source: ModelSource,
    /// model.onnx を含むディレクトリ（実推論の Embedder 構築に使用、BUG-004）。
    pub model_dir: std::path::PathBuf,
}

/// 成果物の解決結果（DD-LGX-013 §2.1）。
#[derive(Debug, Clone, PartialEq)]
pub struct ArtifactRef {
    pub artifact_id: Id,
    pub file_path: std::path::PathBuf,
}

/// ONNX から生成した現行 embedding（DD-LGX-013 §2.1。L2 正規化済）。
#[derive(Debug, Clone, PartialEq)]
pub struct CurrentEmbedding {
    pub artifact_id: Id,
    pub vector: Vec<f32>,
    pub dim: usize,
    pub model_version: String,
}

/// DB から読み込んだベースライン embedding（DD-LGX-013 §2.1）。
#[derive(Debug, Clone, PartialEq)]
pub struct BaselineEmbedding {
    pub artifact_id: Id,
    pub vector: Vec<f32>,
    pub dim: usize,
    pub model_version: String,
    pub source: BaselineSource,
}

/// ベースラインの供給元（DD-LGX-013 §2.1。drift --json の baseline_source に対応）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BaselineSource {
    /// embeddings ストアの現行保存行（--against 省略時）。
    Embeddings,
    /// snapshot の snapshot_id（--against snapshot:<token> 解決後）。
    Snapshot(String),
}

/// drift 対比の最終出力（DD-LGX-013 §2.1）。
#[derive(Debug, Clone, PartialEq)]
pub struct DriftResult {
    pub artifact_id: Id,
    /// None = ベースライン不在（baseline_available: false）。
    pub drift: Option<f32>,
    pub baseline_available: bool,
    pub baseline_source: Option<BaselineSource>,
}

/// --against 引数の解析済み表現（DD-LGX-013 §2.1）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AgainstSpec {
    /// --against 省略（現行 embeddings ストア）。
    Embeddings,
    /// snapshot:<token>（label 優先 → snapshot_id フォールバック）。
    SnapshotToken(String),
    /// snapshot:label:<LABEL>（明示 label 形式、失敗は exit 1）。
    SnapshotLabelExplicit(String),
}

/// model_version 複合キー newtype（DD-LGX-013 §2.2、完全一致判定用）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelVersion(pub String);

/// 整合性検査結果（DD-LGX-013 §2.2）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IntegrityCheckResult {
    Ok,
    DimMismatch {
        current_dim: usize,
        baseline_dim: usize,
    },
    ModelVersionMismatch {
        current: String,
        baseline: String,
    },
}

/// 整合性検査（DD-LGX-013 §3）。次元検査が一段目、model_version 完全一致照合が二段目。
pub fn check_integrity(
    current: &CurrentEmbedding,
    baseline: &BaselineEmbedding,
) -> IntegrityCheckResult {
    if current.dim != baseline.dim {
        return IntegrityCheckResult::DimMismatch {
            current_dim: current.dim,
            baseline_dim: baseline.dim,
        };
    }
    if current.model_version != baseline.model_version {
        return IntegrityCheckResult::ModelVersionMismatch {
            current: current.model_version.clone(),
            baseline: baseline.model_version.clone(),
        };
    }
    IntegrityCheckResult::Ok
}

/// drift 対比本体（DD-LGX-013 §3、BUG-004 で実推論化）。同一入力 → 同一 DriftResult。read-only。
/// engine.db 不在時は DriftResult{baseline_available: false}（DB 新規作成しない、REQ.07 v3 差分）。
/// `project_root` はファイル・モデル解決の基準（旧 CWD 相対バグ修正）。`models_dir` は --models-dir。
/// 整合性検査（dim/model_version）は解決済モデル情報で行うため、不一致は実推論前に Err（ONNX 不要）。
/// 整合性 OK のときのみ実 Embedder で現行 embedding を生成する（onnx feature 必須・stub 不可）。
pub fn run(
    graph: &TraceGraph,
    store: &EmbeddingStore,
    config: &Config,
    artifact_id: &Id,
    against: AgainstSpec,
    project_root: &Path,
    models_dir: Option<&Path>,
) -> Result<DriftResult, DriftError> {
    let absent = |source: Option<BaselineSource>| DriftResult {
        artifact_id: artifact_id.clone(),
        drift: None,
        baseline_available: false,
        baseline_source: source,
    };

    // Step 1: artifact が graph.toml に登録済か（非空グラフのみ厳格判定。空グラフは委譲ケース）。
    if graph.node_count() > 0 && graph.node(artifact_id.as_str()).is_none() {
        return Err(DriftError::ArtifactNotFound {
            artifact_id: artifact_id.clone(),
        });
    }

    // Step 2: baseline 解決（AgainstSpec ごと）。明示 label のみ解決失敗で exit 1、他は baseline 不在 exit 0。
    let (baseline_row, baseline_source): (Option<crate::store::EmbeddingRow>, BaselineSource) =
        match &against {
            AgainstSpec::Embeddings => (
                store
                    .load_embedding(artifact_id.as_str())
                    .map_err(DriftError::Db)?,
                BaselineSource::Embeddings,
            ),
            AgainstSpec::SnapshotLabelExplicit(label) => {
                match store
                    .resolve_snapshot_id_by_label(label)
                    .map_err(DriftError::Db)?
                {
                    // snapshot 内の当該ノード行をロード（BUG-004: 以前は未ロードで常に不在だった）。
                    Some(snapshot_id) => {
                        let row = store
                            .load_snapshot_embedding(&snapshot_id, artifact_id.as_str())
                            .map_err(DriftError::Db)?;
                        (row, BaselineSource::Snapshot(snapshot_id))
                    }
                    // 明示 label の解決失敗は exit 1（曖昧形式 token と非対称、DD §2.3）。
                    None => {
                        return Err(DriftError::LabelNotFound {
                            label: label.clone(),
                        })
                    }
                }
            }
            AgainstSpec::SnapshotToken(token) => {
                match store
                    .resolve_snapshot_id_by_label(token)
                    .map_err(DriftError::Db)?
                {
                    Some(snapshot_id) => {
                        let row = store
                            .load_snapshot_embedding(&snapshot_id, artifact_id.as_str())
                            .map_err(DriftError::Db)?;
                        (row, BaselineSource::Snapshot(snapshot_id))
                    }
                    // 曖昧形式 token の未解決は baseline 不在 = exit 0（DB 新規作成なし）。
                    None => return Ok(absent(None)),
                }
            }
        };

    // baseline 行が取得できない（snapshot に当該ノード不在など）→ baseline 不在 exit 0。
    let baseline_row = match baseline_row {
        Some(r) => r,
        None => return Ok(absent(Some(baseline_source))),
    };

    // Step 3: モデル解決（dim/model_version。実推論前に整合性判定に使う）。
    let resolved = resolve_model(config, models_dir)?;

    // Step 4: 整合性検査（次元 → model_version）。実ベクトル不要なので推論前に判定（DimMismatch 等）。
    if resolved.dim != baseline_row.dim {
        return Err(DriftError::DimMismatch {
            current_dim: resolved.dim,
            baseline_dim: baseline_row.dim,
        });
    }
    if resolved.model_version != baseline_row.model_version {
        return Err(DriftError::ModelVersionMismatch {
            current: resolved.model_version.clone(),
            baseline: baseline_row.model_version.clone(),
        });
    }

    // Step 5: 整合性 OK → 実 Embedder で現行 embedding を生成（BUG-004: ゼロベクトル撤廃）。
    let node = graph
        .node(artifact_id.as_str())
        .ok_or_else(|| DriftError::ArtifactNotFound {
            artifact_id: artifact_id.clone(),
        })?;
    // project_root 基準で現物を読む（旧 CWD 相対バグ修正）。
    let content = crate::content::read_current_content_for_node(node, graph, project_root)
        .map_err(|_| DriftError::FileNotFound {
            artifact_id: artifact_id.clone(),
            path: project_root.join(&node.path),
        })?;
    // 実 ONNX 推論（onnx feature 必須。無効ビルドは Embedder::new が Err → drift も Err）。
    let embedder = crate::embedder::Embedder::new(&resolved.model_dir, &resolved.model_version)
        .map_err(|e| DriftError::ModelLoad(e.to_string()))?;
    let embed_result = embedder
        .embed_node(&content, None, artifact_id.as_str())
        .map_err(|e| DriftError::ModelLoad(e.to_string()))?;
    let current = CurrentEmbedding {
        artifact_id: artifact_id.clone(),
        vector: embed_result.embedding,
        dim: embed_result.dim,
        model_version: embed_result.model_version,
    };

    let baseline = BaselineEmbedding {
        artifact_id: artifact_id.clone(),
        vector: baseline_row.embedding.clone(),
        dim: baseline_row.dim,
        model_version: baseline_row.model_version.clone(),
        source: baseline_source.clone(),
    };

    // 実推論後の次元再照合（防御。モデル実 dim が解決値と乖離した場合）。
    if current.dim != baseline.dim {
        return Err(DriftError::DimMismatch {
            current_dim: current.dim,
            baseline_dim: baseline.dim,
        });
    }

    // Step 6: drift 算出（compute_drift が非有限ガード）。
    let drift = compute_drift(&current, &baseline)?;
    Ok(DriftResult {
        artifact_id: artifact_id.clone(),
        drift: Some(drift),
        baseline_available: true,
        baseline_source: Some(baseline_source),
    })
}

/// モデル解決（DD-LGX-013 §3）。解決順: override → LGX_MODELS_DIR → TE_MODELS_DIR → config。
/// 全経路失敗は ModelNotFound{tried_paths}。
pub fn resolve_model(
    config: &Config,
    models_dir_override: Option<&Path>,
) -> Result<ResolvedModel, DriftError> {
    let mut tried_paths: Vec<String> = Vec::new();

    let mut candidates: Vec<(std::path::PathBuf, ModelSource)> = Vec::new();
    if let Some(dir) = models_dir_override {
        candidates.push((dir.to_path_buf(), ModelSource::Flag));
    }
    if let Ok(dir) = std::env::var("LGX_MODELS_DIR") {
        candidates.push((std::path::PathBuf::from(dir), ModelSource::EnvLgx));
    }
    if let Ok(dir) = std::env::var("TE_MODELS_DIR") {
        candidates.push((std::path::PathBuf::from(dir), ModelSource::EnvTe));
    }
    // ConfigFile 経路: 設定 `[semantic] model_dir` があれば最優先（BUG-003/004 連動）。
    if let Some(m) = &config.semantic.model_dir {
        candidates.push((std::path::PathBuf::from(m), ModelSource::ConfigFile));
    }
    // 最終段: 慣例パス。
    candidates.push((
        std::path::PathBuf::from("models/paraphrase-multilingual-MiniLM-L12-v2"),
        ModelSource::ConfigFile,
    ));

    for (dir, source) in candidates {
        let onnx = dir.join("model.onnx");
        let tok = dir.join("tokenizer.json");
        tried_paths.push(onnx.to_string_lossy().into_owned());
        if onnx.exists() && tok.exists() {
            // 実 shape は ONNX backend で確定（TS-007 委譲）。既定次元 384 を採用。
            let model_version = crate::model_version::compute_model_version(
                "model",
                &onnx,
                crate::model_version::PreprocessProfile::Plain,
                384,
            )
            .map_err(|e| DriftError::ModelLoad(e.to_string()))?;
            return Ok(ResolvedModel {
                model_version,
                dim: 384,
                source,
                model_dir: dir.clone(),
            });
        }
    }

    Err(DriftError::ModelNotFound { tried_paths })
}

/// --against の解析（DD-LGX-013 §3）。
/// None → Embeddings。"snapshot:label:<L>" → SnapshotLabelExplicit。"snapshot:<t>" → SnapshotToken。
/// その他 → InvalidAgainstFormat。
pub fn parse_against(raw: Option<&str>) -> Result<AgainstSpec, DriftError> {
    match raw {
        None => Ok(AgainstSpec::Embeddings),
        Some(value) => {
            if let Some(rest) = value.strip_prefix("snapshot:") {
                if let Some(label) = rest.strip_prefix("label:") {
                    Ok(AgainstSpec::SnapshotLabelExplicit(label.to_string()))
                } else {
                    Ok(AgainstSpec::SnapshotToken(rest.to_string()))
                }
            } else {
                Err(DriftError::InvalidAgainstFormat {
                    value: value.to_string(),
                })
            }
        }
    }
}

/// `1.0 − cosine(current.vector, baseline.vector)`。値域 [0.0, 2.0]（DD-LGX-013 §3、REQ.03）。
/// 非有限値（ゼロベクトル混入のゼロ除算等）→ DriftError::NonFiniteScore。
pub fn compute_drift(
    current: &CurrentEmbedding,
    baseline: &BaselineEmbedding,
) -> Result<f32, DriftError> {
    let a = &current.vector;
    let b = &baseline.vector;
    let mut dot = 0.0_f32;
    let mut norm_a = 0.0_f32;
    let mut norm_b = 0.0_f32;
    for (x, y) in a.iter().zip(b.iter()) {
        dot += x * y;
        norm_a += x * x;
        norm_b += y * y;
    }
    // ゼロノルムはゼロ除算 → NaN（v3 のガードを通さず非有限として捕捉する）。
    let cosine = dot / (norm_a.sqrt() * norm_b.sqrt());
    if !cosine.is_finite() {
        return Err(DriftError::NonFiniteScore);
    }
    let drift = 1.0 - cosine.clamp(-1.0, 1.0);
    if !drift.is_finite() {
        return Err(DriftError::NonFiniteScore);
    }
    Ok(drift)
}

/// 終了コード判定（DD-LGX-013 §3）。Ok(baseline あり/なし)→0、Err→1。
pub fn exit_code(result: &Result<DriftResult, DriftError>) -> i32 {
    match result {
        Ok(_) => 0,
        Err(_) => 1,
    }
}
