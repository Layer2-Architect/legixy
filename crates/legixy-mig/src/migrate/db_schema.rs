// (module of SRC-LGX-009; anchor: lib.rs)
// legixy-mig::migrate::db_schema — engine.db スキーマ初期化（ADR-LGX-015）。
// legixy-db は scaffold のため、移行先 `.legixy/engine.db` のスキーマは本 crate 内で完結させる
// （共通指示: 不足する共有型は自 crate 内で完結。DB は rusqlite を直接使用）。

use rusqlite::Connection;

use crate::error::MigError;

/// engine.db の observations / proposals / custom_edges スキーマを初期化する（冪等）。
/// feedback.db → engine.db 移行先のテーブル定義（DD-LGX-009 §1 / M-3）。
pub fn init_engine_schema(conn: &Connection) -> Result<(), MigError> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS observations (
            id INTEGER PRIMARY KEY,
            source TEXT NOT NULL,
            category TEXT NOT NULL,
            severity TEXT NOT NULL,
            message TEXT NOT NULL,
            related_ids TEXT NOT NULL,
            context_json TEXT,
            status TEXT NOT NULL,
            created_at TEXT NOT NULL,
            UNIQUE(source, category, severity, message, related_ids, created_at)
        );
        CREATE TABLE IF NOT EXISTS proposals (
            id INTEGER PRIMARY KEY,
            observation_id INTEGER,
            kind TEXT NOT NULL,
            semantic_key TEXT NOT NULL,
            title TEXT NOT NULL,
            description TEXT NOT NULL,
            action_json TEXT NOT NULL,
            status TEXT NOT NULL,
            decided_at TEXT,
            decided_reason TEXT,
            created_at TEXT NOT NULL
        );
        CREATE TABLE IF NOT EXISTS custom_edges (
            id INTEGER PRIMARY KEY,
            from_id TEXT NOT NULL,
            to_id TEXT NOT NULL,
            reason TEXT,
            UNIQUE(from_id, to_id)
        );",
    )?;
    Ok(())
}
