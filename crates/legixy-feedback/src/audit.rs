// (module of SRC-LGX-008; anchor: manager.rs)（audit 部）
// DD-LGX-008 §2.1 ContextLogEntry / §3.1 ContextAuditReader::recent / by_target（読取専用）。
//   context_log 書込本体は legixy-ctx 担当（SPEC-LGX-003 / TS-LGX-003 主導）。本 crate は読取のみ。

use rusqlite::params;

use crate::db::{map_sql_err, Connection};
use crate::error::FeedbackError;

/// context_log テーブルの 1 エントリ（読取専用、書込は legixy-ctx 担当。DD §2.1）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextLogEntry {
    pub id: i64,
    pub target_id: String,
    pub granularity: Option<String>,
    pub payload: String, // CTX が書き込んだ JSON 文字列（FB は raw で返す）
    pub created_at: String,
}

/// context_log の読取（DD §3.1）。すべて read-only（&Connection 借用）。
pub struct ContextAuditReader;

impl ContextAuditReader {
    /// context_log を id DESC LIMIT limit で返す。limit 範囲は CLI 層（1..=50）で強制。
    /// read-only。DD §3.1 凍結シグネチャ。
    pub fn recent(db: &Connection, limit: usize) -> Result<Vec<ContextLogEntry>, FeedbackError> {
        let mut stmt = db
            .sql()
            .prepare(
                "SELECT id, target_id, granularity, payload, created_at \
                 FROM context_log ORDER BY id DESC LIMIT ?1",
            )
            .map_err(map_sql_err)?;
        let rows = stmt
            .query_map(params![limit as i64], Self::map_row)
            .map_err(map_sql_err)?;
        let mut result = Vec::new();
        for row in rows {
            result.push(row.map_err(map_sql_err)?);
        }
        Ok(result)
    }

    /// target_id フィルタ + id DESC LIMIT limit。read-only。DD §3.1 凍結シグネチャ。
    pub fn by_target(
        db: &Connection,
        target_id: &str,
        limit: usize,
    ) -> Result<Vec<ContextLogEntry>, FeedbackError> {
        let mut stmt = db
            .sql()
            .prepare(
                "SELECT id, target_id, granularity, payload, created_at \
                 FROM context_log WHERE target_id = ?1 ORDER BY id DESC LIMIT ?2",
            )
            .map_err(map_sql_err)?;
        let rows = stmt
            .query_map(params![target_id, limit as i64], Self::map_row)
            .map_err(map_sql_err)?;
        let mut result = Vec::new();
        for row in rows {
            result.push(row.map_err(map_sql_err)?);
        }
        Ok(result)
    }

    fn map_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<ContextLogEntry> {
        Ok(ContextLogEntry {
            id: row.get(0)?,
            target_id: row.get(1)?,
            granularity: row.get(2)?,
            payload: row.get(3)?,
            created_at: row.get(4)?,
        })
    }
}
