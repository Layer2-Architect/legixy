// (module of SRC-LGX-002; anchor: compiler.rs)
// Document ID: SRC-LGX-004
// legixy-ctx: compile_context・コンテキスト解決・粒度制御・キャッシュ整列（DD-LGX-002 / DD-LGX-004）
//
// TC[RED] フェーズの scaffold。公開 API surface（型・シグネチャ）は DD-LGX-002 §2/§3 および
// DD-LGX-004 §2/§3 に凍結整合させる（HR7）。データ型は実体を持ち、ロジック
// （compile / render / enforce_size_limit / walk / log / build_outline）は `todo!()` として
// TC[RED] を失敗させる。SRC[GREEN] で最小実装に置換する。
//
// 親 chain: TS-LGX-002 → TC-LGX-002 → 本 SRC-LGX-002、TS-LGX-004 → TC-LGX-004 → SRC-LGX-004。
// crate 境界・共有型は ADR-LGX-020。
//
// 上流裁定の反映:
//  - B-1: `ContextError::ResultTooLarge` は exit 1 / stderr（DD-LGX-002 v1.2）。
//  - A-1: `SectionFormatter::upstream_sort_rule(Subnode)` = "parent_id-asc,anchor-appearance-order"
//         （アンカー出現順、DD-LGX-004 v1.1）。
//
// SHARED-NEEDS（共有 crate へ移設すべき型 — 統合時に差し替え。本 crate のローカル stub で代替中）:
//  - `TraceConfig`（DD §2.1）= legixy-core 所有。本 scaffold は `legixy_core::Config` を別名参照。
//  - DB 接続/エラー（DD §2.1 `rusqlite::Connection` / §2.3 `rusqlite::Error`）= legixy-db 所有。
//    本 scaffold は `crate::db::{DbConn, DbConnError}`（不透明 stub）で代替。

mod audit_logger;
mod compiler;
pub mod db; // SHARED-NEED: DbConn / DbConnError は本来 legixy-db 所有（統合時に移設）
mod error;
mod result;
mod section_formatter;
mod upstream_walker;

pub use audit_logger::AuditLogger;
pub use compiler::{build_outline, CompileInput, ContextCompiler, Granularity, TraceConfig};
pub use error::ContextError;
pub use result::{
    ContextResult, CustomDocument, LayerDocument, ResolvedTarget, TargetNodeMetadata,
    UpstreamArtifact,
};
pub use section_formatter::SectionFormatter;
pub use upstream_walker::UpstreamWalker;

/// SPEC-LGX-003.REQ.13 / CACHE-INV-3 / NFR-LGX-001.PERF.09。返却本文の文字数上限。
pub const RESULT_SIZE_LIMIT_CHARS: usize = 500_000;

/// REQ.12 / CACHE-INV-2。キャッシュブレーク点マーカ（Additional Guidelines と Upstream の間）。
pub const CACHE_BREAKPOINT_MARKER: &str = "<!-- cache-breakpoint: stable-end -->";

/// S2-04 / NFR-LGX-001.REL.07。context_log 書込の SQLite busy_timeout（ms）。
pub const CONTEXT_LOG_BUSY_TIMEOUT_MS: u64 = 5_000;
