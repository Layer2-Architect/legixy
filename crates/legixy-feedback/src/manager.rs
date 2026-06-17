// Document ID: SRC-LGX-008
// DD-LGX-008 §3.1 ProposalManager::approve / reject。
//   CAS UPDATE（更新行数判定）+ approve→observation Resolved 連動 + reject→observation pending 復帰。

use rusqlite::{params, OptionalExtension};

use crate::db::{map_sql_err, Connection};
use crate::error::FeedbackError;

/// fetch_proposal_core の戻り（CAS 前の現状把握）。
struct ProposalCore {
    observation_id: Option<i64>,
    kind: String,
    action_json: String,
    status: String,
}

/// Proposal の承認 / 却下担当（CAS、DD §3.1 / FB-INV-2）。
pub struct ProposalManager;

impl ProposalManager {
    /// 単一 tx: CAS `UPDATE proposals SET status='approved' WHERE id=? AND status='pending'`
    /// （更新行数判定）→ `kind == "add_custom_edge"` の場合のみ custom_edges に INSERT →
    /// approve tx 内で対応 observation を resolved へ UPDATE（FB-INV-2）。
    /// 終端状態への再操作は `InvalidProposalStatus` → exit 1。同期。DD §3.1 凍結シグネチャ。
    pub fn approve(proposal_id: i64, db: &Connection) -> Result<(), FeedbackError> {
        let conn = db.sql();
        let tx = conn.unchecked_transaction().map_err(map_sql_err)?;

        let core = match Self::fetch_proposal_core(&tx, proposal_id)? {
            Some(c) => c,
            None => return Err(FeedbackError::ProposalNotFound { id: proposal_id }),
        };

        // CAS: pending のときのみ approved へ。更新行数 0 = 終端状態への再操作（敗者）。
        let updated = tx
            .execute(
                "UPDATE proposals \
                 SET status = 'approved', decided_at = datetime('now'), decided_reason = NULL \
                 WHERE id = ?1 AND status = 'pending'",
                params![proposal_id],
            )
            .map_err(map_sql_err)?;

        if updated == 0 {
            return Err(FeedbackError::InvalidProposalStatus {
                id: proposal_id,
                expected: "pending",
                actual: core.status,
            });
        }

        // add_custom_edge の場合のみ custom_edges に INSERT（FB-INV-2、同一 tx）。
        if core.kind == "add_custom_edge" {
            let action: serde_json::Value = serde_json::from_str(&core.action_json)?;
            let from_id = action.get("from_id").and_then(|v| v.as_str()).unwrap_or("");
            let to_id = action.get("to_id").and_then(|v| v.as_str()).unwrap_or("");
            let reason = action.get("reason").and_then(|v| v.as_str()).unwrap_or("");
            tx.execute(
                "INSERT INTO custom_edges (from_id, to_id, reason) VALUES (?1, ?2, ?3)",
                params![from_id, to_id, reason],
            )
            .map_err(map_sql_err)?;
        }

        // approve 連動: 対応 observation を resolved へ（DD §2.2、【v3 差分】）。
        if let Some(obs_id) = core.observation_id {
            tx.execute(
                "UPDATE observations SET status = 'resolved' WHERE id = ?1",
                params![obs_id],
            )
            .map_err(map_sql_err)?;
        }

        tx.commit().map_err(map_sql_err)?;
        Ok(())
    }

    /// reason trim 後 0 文字は `EmptyRejectReason` → exit 1（GAP-LGX-124。【v3 差分】v3 は is_empty()）。
    /// CAS `UPDATE proposals SET status='rejected' WHERE id=? AND status='pending'`（行数 0 →
    /// `InvalidProposalStatus`）。observation 状態は pending に戻す（SPEC REQ.08）。
    /// 同期。DD §3.1 凍結シグネチャ。
    pub fn reject(
        proposal_id: i64,
        reason: &str,
        db: &Connection,
    ) -> Result<(), FeedbackError> {
        // 【v3 差分】trim 後 0 文字を拒否（CAS 未実行で早期 return）。
        if reason.trim().is_empty() {
            return Err(FeedbackError::EmptyRejectReason);
        }

        let conn = db.sql();
        let tx = conn.unchecked_transaction().map_err(map_sql_err)?;

        let core = match Self::fetch_proposal_core(&tx, proposal_id)? {
            Some(c) => c,
            None => return Err(FeedbackError::ProposalNotFound { id: proposal_id }),
        };

        let updated = tx
            .execute(
                "UPDATE proposals \
                 SET status = 'rejected', decided_at = datetime('now'), decided_reason = ?1 \
                 WHERE id = ?2 AND status = 'pending'",
                params![reason, proposal_id],
            )
            .map_err(map_sql_err)?;

        if updated == 0 {
            return Err(FeedbackError::InvalidProposalStatus {
                id: proposal_id,
                expected: "pending",
                actual: core.status,
            });
        }

        // reject 連動: 対応 observation を pending に戻す（approve の resolved と非対称、SPEC REQ.08）。
        if let Some(obs_id) = core.observation_id {
            tx.execute(
                "UPDATE observations SET status = 'pending' WHERE id = ?1",
                params![obs_id],
            )
            .map_err(map_sql_err)?;
        }

        tx.commit().map_err(map_sql_err)?;
        Ok(())
    }

    fn fetch_proposal_core(
        tx: &rusqlite::Transaction<'_>,
        id: i64,
    ) -> Result<Option<ProposalCore>, FeedbackError> {
        tx.prepare("SELECT observation_id, kind, action_json, status FROM proposals WHERE id = ?1")
            .map_err(map_sql_err)?
            .query_row(params![id], |row| {
                Ok(ProposalCore {
                    observation_id: row.get(0)?,
                    kind: row.get(1)?,
                    action_json: row.get(2)?,
                    status: row.get(3)?,
                })
            })
            .optional()
            .map_err(map_sql_err)
    }
}
