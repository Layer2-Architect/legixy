// (module of SRC-LGX-007; anchor: orchestrator.rs)
// legixy-embed エラー型階層（DD-LGX-007 §2.3 / DD-LGX-010 §2.3 / DD-LGX-011 §2.3 /
// DD-LGX-012 §2.3 / DD-LGX-013 §2.2）。
//
// TC[RED] scaffold。エラー variant・Display 文言・From 変換はそれぞれの DD §2.3 凍結に整合。
// 終了コードへの写像（exit 0/1/2）はコマンド層が担う（各 TS の exit 契約ケース参照）。

use std::path::PathBuf;

use legixy_db::DbError;

/// embed 生成サブシステムの実行時エラー（DD-LGX-007 §2.3、v3 EmbedError 相当・11 variant）。
#[derive(Debug, thiserror::Error)]
pub enum EmbedError {
    #[error("model load failed: {path:?}: {reason}")]
    ModelLoadFailed { path: PathBuf, reason: String },

    #[error("model shape invalid: {reason}")]
    ModelShapeInvalid { reason: String },

    #[error("tokenizer error: {reason}")]
    TokenizerError { reason: String },

    #[error("onnx inference error: {reason}")]
    OnnxInferenceError { reason: String },

    #[error("db error: {0}")]
    Db(#[from] DbError),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("dimension mismatch: stored={stored_dim}, current={current_dim}")]
    DimensionMismatch { stored_dim: usize, current_dim: usize },

    #[error("node not found: {0}")]
    NodeNotFound(String),

    #[error("invalid content range for {node_id}: {reason}")]
    InvalidContentRange { node_id: String, reason: String },

    #[error("contextual retrieval failed for {node_id}: {reason}")]
    ContextualRetrievalFailed { node_id: String, reason: String },
}

/// report コマンドの実行時失敗（DD-LGX-010 §2.3。exit 1。計測スキップとは別概念）。
#[derive(Debug, thiserror::Error)]
pub enum ReportError {
    #[error("graph load failed: {0}")]
    GraphLoad(legixy_graph::GraphError),
    #[error("config load failed: {0}")]
    ConfigLoad(legixy_core::ConfigError),
    #[error("db error: {0}")]
    Db(legixy_db::DbError),
}

/// calibrate コマンドの実行時失敗（DD-LGX-011 §2.3。exit 1。InvalidBuckets は値の意味的不正）。
#[derive(Debug, thiserror::Error)]
pub enum CalibrateError {
    #[error("--buckets は 1 以上を指定してください")]
    InvalidBuckets,
    #[error("pair score computation failed: {0}")]
    PairScoreFailure(String),
    #[error("db error: {0}")]
    Db(legixy_db::DbError),
    #[error("config load failed: {0}")]
    Config(legixy_core::ConfigError),
}

/// snapshot 操作の実行時失敗（DD-LGX-012 §2.3。exit 1）。
#[derive(Debug, thiserror::Error)]
pub enum SnapshotError {
    #[error("DB エラー: {0}")]
    Db(#[from] DbError),

    #[error("label '{label}' に該当するスナップショットがありません")]
    LabelNotFound { label: String },

    #[error("create トランザクション失敗: {0}")]
    TransactionFailed(DbError),
}

/// drift サブシステムの実行時エラー（DD-LGX-013 §2.2。11 variant。exit 1）。
#[derive(Debug, thiserror::Error)]
pub enum DriftError {
    #[error("モデル解決失敗: {tried_paths:?}")]
    ModelNotFound { tried_paths: Vec<String> },

    #[error("モデル読み込みエラー: {0}")]
    ModelLoad(String),

    #[error("成果物 {artifact_id:?} は graph.toml に未登録です")]
    ArtifactNotFound { artifact_id: legixy_core::Id },

    #[error("成果物 {artifact_id:?} のファイルが見つかりません: {path:?}")]
    FileNotFound {
        artifact_id: legixy_core::Id,
        path: PathBuf,
    },

    #[error("--against 値の形式が不正です（'snapshot:' プレフィクスが必要です）: {value}")]
    InvalidAgainstFormat { value: String },

    #[error("ラベル '{label}' に対応するスナップショットが見つかりません")]
    LabelNotFound { label: String },

    #[error("次元数不一致: 現行 {current_dim} / ベースライン {baseline_dim}")]
    DimMismatch {
        current_dim: usize,
        baseline_dim: usize,
    },

    #[error("モデルバージョン不一致: 現行 {current} / ベースライン {baseline}")]
    ModelVersionMismatch { current: String, baseline: String },

    #[error("非有限スコアが発生しました（NaN/Inf）")]
    NonFiniteScore,

    #[error("DB エラー: {0}")]
    Db(#[from] DbError),

    #[error("グラフ読み込みエラー: {0}")]
    Graph(#[from] legixy_graph::GraphError),

    #[error("I/O エラー: {0}")]
    Io(#[from] std::io::Error),
}

