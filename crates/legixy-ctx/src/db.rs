// (module of SRC-LGX-002; anchor: compiler.rs)
// legixy-ctx::db — engine.db 接続ハンドル（context_log 監査ログ書込用）
//
// SHARED-NEED 解消（統合）: DD-LGX-002 §2.1 の `db` を実 rusqlite::Connection でラップする。
// 接続層の正準所有は legixy-db（ADR-LGX-015/021）だが scaffold 段階のため本 crate でラップする。
//   - `new()`: 接続を持たない no-op ハンドル（DB 不在相当）。AuditLogger は書込不能を
//     ベストエフォートで握り潰す（REQ.19、TS-LGX-004 ケース21 の挙動を維持）。
//   - `open(path)`: 実 engine.db を開き WAL + busy_timeout=5000 + context_log スキーマを用意。
//     context コマンド（CLI 統合層）が監査ログを書く経路。

use std::fmt;
use std::time::Duration;

use rusqlite::Connection;

/// engine.db への接続ハンドル（context_log 書込）。conn=None は no-op（DB 不在相当）。
pub struct DbConn {
    conn: Option<Connection>,
}

impl DbConn {
    /// 接続を持たない no-op ハンドル（DB 不在相当。AuditLogger は書込不能を握り潰す）。
    pub fn new() -> Self {
        DbConn { conn: None }
    }

    /// 実 engine.db を開き、PRAGMA（WAL + busy_timeout=5000、S2-04）と context_log スキーマを用意する。
    /// context_log は legixy-feedback の同名テーブルと同一スキーマ（FB の ContextAuditReader が読む）。
    pub fn open(path: &str) -> Result<Self, DbConnError> {
        let conn = Connection::open(path).map_err(|e| DbConnError(e.to_string()))?;
        conn.busy_timeout(Duration::from_millis(5000))
            .map_err(|e| DbConnError(e.to_string()))?;
        let _ = conn.pragma_update(None, "journal_mode", "WAL");
        let _ = conn.pragma_update(None, "synchronous", "NORMAL");
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS context_log (\
                 id INTEGER PRIMARY KEY AUTOINCREMENT, \
                 target_id TEXT NOT NULL, \
                 granularity TEXT NULL, \
                 payload TEXT NOT NULL, \
                 created_at TEXT DEFAULT (datetime('now'))\
             );",
        )
        .map_err(|e| DbConnError(e.to_string()))?;
        Ok(DbConn { conn: Some(conn) })
    }

    /// 内部 rusqlite::Connection への借用（接続あり時のみ Some）。AuditLogger が使用。
    pub(crate) fn conn(&self) -> Option<&Connection> {
        self.conn.as_ref()
    }
}

impl Default for DbConn {
    fn default() -> Self {
        Self::new()
    }
}

/// DB 操作失敗（SHARED-NEED: 本来 `rusqlite::Error`、legixy-db 所有）。
#[derive(Debug)]
pub struct DbConnError(pub String);

impl fmt::Display for DbConnError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "db conn error: {}", self.0)
    }
}

impl std::error::Error for DbConnError {}
