// (module of SRC-LGX-002; anchor: compiler.rs)
// legixy-ctx::error — ContextError（DD-LGX-002 §2.3 / DD-LGX-004 §2.3、thiserror）
//
// TC[RED] scaffold。エラー型は実体（バリアント・フィールド・Display）を持つ。
// Display 文言は DD-LGX-002 §2.3 の凍結書式に厳密整合（v3 lx-ctx/src/error.rs 底本）。
// 終了コード分類（LGX-COMPAT-001 §3 / DD-freeze 裁定 B-1）:
//   InvalidInput / Io / Graph / Db / Serde / ResultTooLarge → すべて exit 1。
//   ResultTooLarge は呼び出し側 CLI が stderr へ Display を出力し exit 1（v3 互換、B-1）。

use crate::db::DbConnError;

/// コンテキスト解決の実行時失敗（DD-LGX-002 §2.3 / DD-LGX-004 §2.3）。
///
/// SHARED-NEED: `Db` の元エラー `rusqlite::Error` は本来 legixy-db 所有（DD は
/// `#[from] rusqlite::Error`）。TC[RED] scaffold では rusqlite を持ち込まず、
/// legixy-db 由来の不透明 DB エラー（`crate::db::DbConnError` = SHARED-NEED）で代替する。
/// 統合時に legixy-db の実 DB エラー型へ移設する。
#[derive(Debug, thiserror::Error)]
pub enum ContextError {
    /// SPEC-LGX-003.REQ.13 / CACHE-INV-3 / NFR-LGX-001.PERF.09。
    /// 返却本文が `RESULT_SIZE_LIMIT_CHARS` (500,000 コードポイント) を超過した場合。
    /// 切り捨て・要約禁止。文言は DD-LGX-002 §2.3（v3 error.rs:11-14 整合）。
    #[error(
        "compile_context result exceeds {limit} characters.\nCurrent size: {current} characters.\nSuggested action:\n  - Try --granularity subnode for finer-grained retrieval.\n  - Narrow the target scope."
    )]
    ResultTooLarge { current: usize, limit: usize },

    /// I/O 失敗（exit 1）。
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    /// DB 操作失敗（exit 1）。SHARED-NEED: 本来 `rusqlite::Error`（legixy-db 所有）。
    #[error("db error: {0}")]
    Db(#[from] DbConnError),

    /// graph.toml パース・参照失敗（exit 1）。
    #[error("graph error: {0}")]
    Graph(String),

    /// 受理済み引数の意味的不正（granularity 不正値など、exit 1）。
    #[error("invalid input: {0}")]
    InvalidInput(String),

    /// シリアライズ失敗（exit 1）。
    #[error("serialization error: {0}")]
    Serde(#[from] serde_json::Error),
}
