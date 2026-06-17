// (module of SRC-LGX-005; anchor: investigate.rs)
// legixy-nav: グラフ走査 crate（investigate 逆方向 / impact 順方向）。ADR-LGX-020 crate 境界。
//
// TC[RED] フェーズの scaffold。公開 API surface（型・シグネチャ）は DD-LGX-005 §2/§3（共有型・
// investigate 系の正典）と DD-LGX-006 §2/§3（impact 系・打ち切り可観測性）に凍結整合させる（HR7）。
// データ型は実体を持ち、走査・枝刈り・整形ロジックは todo!() として TC[RED] を失敗させる。
// SRC[GREEN] で最小実装に置換する。
//
// 共有型（VisitedNode / MultiTraversalResult / NavError / ReportFormat / render_multi）は
// DD-LGX-005 を正典として本 crate 内で定義し、DD-LGX-006 は参照（ADR-LGX-021 §2.2）。
// NodeId = String（ADR-LGX-021）。共有グラフ型は legixy-graph、共通型は legixy-core を参照。
// 親 chain: TS-LGX-005 → TC-LGX-005 → SRC-LGX-005、TS-LGX-006 → TC-LGX-006 → SRC-LGX-006。

mod drift_pruner;
mod error;
mod impact;
mod investigate;
mod multi_traverser;
mod reporter;
mod result;

// --- 公開 re-export（DD-LGX-006 §4 lib.rs 列挙）---

// 逆方向探索（UC-LGX-005 / DD-LGX-005）
pub use investigate::{investigate, investigate_with_depth};

// 順方向探索（UC-LGX-006 / DD-LGX-006）
pub use impact::impact;

// 多起点ラッパー
pub use multi_traverser::MultiTraverser;

// Reporter
pub use reporter::{
    detect_truncation, emit_truncation_info, render_multi, render_outcome, render_pruned,
    ReportFormat,
};

// 結果型・データ型
pub use result::{
    InvestigateOutcome, MultiTraversalResult, PrunedTraversalResult, SuspiciousNode,
    TraversalResult, TruncationInfo, VisitedNode,
};

// エラー型
pub use error::NavError;

// 内部だが drift 経路テストから参照される枝刈り器（DD-LGX-005 §4）
pub use drift_pruner::DriftPruner;
