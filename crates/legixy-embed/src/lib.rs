// (module of SRC-LGX-007; anchor: orchestrator.rs)
// legixy-embed: embedding 生成・bulk similarity・report・calibrate・snapshot・drift（ADR-LGX-020）。
//
// TC[RED] scaffold。本 crate は 5 UC（UC-007/010/011/012/013）の SRC stub を集約する。
// 各 module 先頭の `// Document ID:` は所有 UC を示す:
//   - SRC-LGX-007: embedder / content / model_version / orchestrator / contextual / similarity /
//                  drift_detect / store / error（embed 生成・bulk エンジン・EmbeddingStore）
//   - SRC-LGX-010: report（run_report / ReportOutput / to_text / to_json）
//   - SRC-LGX-011: calibrate（calibrate / histogram / compute_recommended / 関連型）
//   - SRC-LGX-012: snapshot（create / list / delete / resolve_label / generate_snapshot_id）
//   - SRC-LGX-013: drift（standalone drift::run / resolve_model / parse_against / compute_drift）
//
// 親 chain: 各 UC の TS-LGX-NNN → TC-LGX-NNN → 本 SRC 群。crate 境界は ADR-LGX-020。

pub mod calibrate;
pub mod content;
pub mod contextual;
pub mod drift;
pub mod drift_detect;
pub mod embedder;
pub mod error;
pub mod model_version;
pub mod orchestrator;
pub mod report;
pub mod similarity;
pub mod snapshot;
pub mod store;

// ── 共有エラー型（再エクスポート、各 DD §2.3）──
pub use error::{CalibrateError, DriftError, EmbedError, ReportError, SnapshotError};

// ── SRC-LGX-007: embed 生成・bulk エンジン・store ──
pub use content::{content_hash_for, normalize_content, read_current_content_for_node};
pub use contextual::{
    synthesize_with_fallback, ContextualConfig, CrOptions, DeterministicContextClient, LlmClient,
};
pub use drift_detect::{detect_drift, DriftFinding, DriftKind};
pub use embedder::{EmbedResult, Embedder};
pub use model_version::{compute_model_version, PreprocessProfile, ShapeValidation};
pub use orchestrator::{
    embed_all, EmbedErrorItem, EmbedOptions, EmbedReport, HashMatchState, NodeFilter,
};
pub use similarity::{
    compute_all_pair_scores, compute_edge_scores, compute_link_candidates, cosine_similarity,
    AggregatedWarnings, CandidateScore, EdgeScore,
};
pub use store::{EmbeddingRow, EmbeddingStore, SnapshotMeta, SnapshotRow, SubnodeRef};

// ── SRC-LGX-010: report ──
pub use report::{
    run_report, EdgeKindJson, ReportFormat, ReportOutput, ReportSummary, SkipReasonSummary,
    SkipWarning,
};

// ── SRC-LGX-011: calibrate ──
pub use calibrate::{
    calibrate, compute_all_pair_scores_calibrate, compute_recommended, histogram, AllPairScores,
    Bucket, BucketCount, CalibrateReport, CurrentThresholds, EarlyExit, Histogram, HistogramStats,
    Percentiles, RecommendedThresholds, SkipSummary,
};

// ── SRC-LGX-012: snapshot（名前空間で公開: legixy_embed::snapshot::create 等）──
pub use snapshot::{LabelResolveResult, SnapshotCreateResult, SnapshotDeleteResult};

// ── SRC-LGX-013: drift（名前空間で公開: legixy_embed::drift::run 等）──
pub use drift::{
    AgainstSpec, ArtifactRef, BaselineEmbedding, BaselineSource, CurrentEmbedding,
    DriftResult, IntegrityCheckResult, ModelSource, ModelVersion, ResolvedModel,
};
