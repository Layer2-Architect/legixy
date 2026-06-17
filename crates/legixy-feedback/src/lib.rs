// (module of SRC-LGX-008; anchor: manager.rs)
// legixy-feedback: フィードバックループ（observe / feedback / analyze / approve / reject /
//   proposals / audit）の crate 公開 API surface。
//
// TC[RED] フェーズの scaffold。公開型・シグネチャは DD-LGX-008 §2/§3 に凍結整合させる（HR7）。
// データ型は実体を持ち（テストが構築できるよう pub フィールド付き）、ロジックを持つ公開関数は
// `todo!()` で TC[RED] を失敗させる。SRC[GREEN] で最小実装に置換する。
//
// 親 chain: TS-LGX-008 → TC-LGX-008 → 本 SRC-LGX-008。crate 境界は ADR-LGX-020。
//
// SHARED-NEEDS（統合時に共有 crate へ移設、本 crate では局所 stub）:
//   - `db::Connection` / `db::DbStubError`        ← legixy-db（rusqlite::Connection / エラー型）
//   - `embed::EmbedError` / `embed::mask_api_key` ← legixy-embed
//   詳細は各 module 先頭 `// SHARED-NEED:` コメント参照。
//
// MCP 転送層（observe / get_compile_audit）は TypeScript（ts-mcp）の責務であり本 Rust crate の
// 対象外（DD-LGX-008 §3.2 / §4.2）。TS-008 ケース 28〜32 は TC-LGX-009（ts-mcp）が担う。

pub mod analyzer;
pub mod audit;
pub mod cli;
pub mod db;
pub mod embed;
pub mod error;
pub mod manager;
pub mod observer;
pub mod recorder;

pub use analyzer::{Proposal, ProposalAnalyzer, ProposalStatus};
pub use audit::{ContextAuditReader, ContextLogEntry};
pub use cli::{FeedbackCli, FeedbackReport, ProposalSummary};
pub use db::Connection;
pub use embed::{mask_api_key, EmbedError};
pub use error::FeedbackError;
pub use manager::ProposalManager;
pub use observer::{
    drift_from_embed_error, AutoObserver, FeedbackCategory, NewObservation, Observation,
    ObservationStatus, ObserveCategoryInput,
};
pub use recorder::{ObservationRecorder, RecordResult};
