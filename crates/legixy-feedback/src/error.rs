// (module of SRC-LGX-008; anchor: manager.rs)（error 部）
// DD-LGX-008 §2.3 FeedbackError（thiserror、8 variant）。終了コード規約 LGX-COMPAT-001 §3。

use crate::db::DbStubError;

/// フィードバックループの失敗（DD-LGX-008 §2.3、HR7 凍結）。
///
/// 終了コード規約（DD §2.3 / LGX-COMPAT-001 §3）:
/// - `Db` / `DbCorrupted` / `ProposalNotFound` / `InvalidProposalStatus` /
///   `EmptyRejectReason` / `EmptyObservationMessage` / `AnalyzeFailed` → exit 1
/// - category 不正値（CLI 層の clap ValueEnum 違反）→ exit 2（本 enum では表現せず CLI 層が担う）
/// - CAS 敗者（approve/reject 競合で行数 0）→ `InvalidProposalStatus` → exit 1
#[derive(Debug, thiserror::Error)]
pub enum FeedbackError {
    /// SHARED-NEED: 本来 `#[from] rusqlite::Error`（DD §2.3）。legixy-db 移設まで局所 stub。
    #[error("database error: {0}")]
    Db(#[from] DbStubError),

    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    /// Proposal が存在しない（approve/reject 対象 id 不在 → exit 1）。
    #[error("proposal {id} not found")]
    ProposalNotFound { id: i64 },

    /// Proposal が終端状態（approved/rejected）への再操作（CAS 失敗 → exit 1）。
    #[error("proposal {id} expected status {expected:?}, found {actual:?}")]
    InvalidProposalStatus {
        id: i64,
        expected: &'static str,
        actual: String,
    },

    /// reject の --reason が trim 後 0 文字（GAP-LGX-124 → exit 1）。
    /// 【v3 差分】v3 は is_empty() のみ。legixy は trim().is_empty() で拒否。
    #[error("reject reason must not be empty")]
    EmptyRejectReason,

    /// observe の message が trim 後 0 文字（GAP-LGX-121 → exit 1）。
    /// 【v3 差分】v3 にこの検証は存在しない。legixy で新規実装。
    #[error("observation message must not be empty")]
    EmptyObservationMessage,

    /// analyze 中に単一 Observation の処理が失敗（Claim Release で pending に戻す）。
    #[error("analyze failed for observation {observation_id}: {detail}")]
    AnalyzeFailed { observation_id: i64, detail: String },

    /// engine.db が破損（不在とは区別、REQ.09 GAP-LGX-126 → exit 1）。
    #[error("engine.db is corrupted; restore from backup or remove to reinitialize: {detail}")]
    DbCorrupted { detail: String },
}
