// (module of SRC-LGX-008; anchor: manager.rs)（analyzer 部）
// DD-LGX-008 §2.1 Proposal / §2.2 ProposalStatus / §3.1 ProposalAnalyzer::analyze。
//   Pessimistic Claim / カテゴリ別変換 / semantic_key dedup / observation 状態更新（FB-INV-5）。

use rusqlite::{params, OptionalExtension};

use crate::db::{map_sql_err, Connection};
use crate::error::FeedbackError;

/// Proposal の永続化状態（SPEC-LGX-007 REQ.09 の 3 値モデル / DD §2.2）。
///
/// 遷移グラフ（HR7 凍結）: (無) → Pending → { Approved | Rejected }。
/// Approved / Rejected は終端・不可逆。終端状態への再 approve/reject は CAS 行数 0 → exit 1。
/// 【v3 差分】v3 enum に Skipped が存在するが DB 行化経路はない。legixy は 3 値で実装。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProposalStatus {
    Pending,
    Approved,
    Rejected,
}

impl ProposalStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "pending" => Some(Self::Pending),
            "approved" => Some(Self::Approved),
            "rejected" => Some(Self::Rejected),
            _ => None,
        }
    }
}

/// engine.db に永続化済みの Proposal（DD §2.1）。
/// 列名は v3 実測（decided_at / decided_reason）を踏襲（SUPP-LGX-007 §2-2）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Proposal {
    pub id: i64,
    pub observation_id: i64,
    pub kind: String,         // "add_chain_entry" | "add_link" | "update_doc"
    pub semantic_key: String, // REQ.09 の 3 形式で生成
    pub title: String,        // "{kind}: {message}"
    pub description: String,   // message そのまま
    pub action_json: String,  // JSON 文字列
    pub status: ProposalStatus,
    pub decided_at: Option<String>,     // approve/reject 時の datetime('now')
    pub decided_reason: Option<String>, // reject 理由（approve 時は NULL）
    pub created_at: String,
}

/// claim_pending で取り込んだ analyzing 状態の observation（process_one の入力）。
#[derive(Debug, Clone)]
struct ClaimedObservation {
    id: i64,
    category: String,
    message: String,
    related_ids: Vec<String>,
}

/// pending observation → Proposal 変換器（DD §3.1）。
pub struct ProposalAnalyzer;

impl ProposalAnalyzer {
    /// Pessimistic Claim（single tx で pending→analyzing）→ カテゴリ別変換 → semantic_key dedup →
    /// INSERT → observation 状態更新（resolved/skipped/pending Claim Release）。戻り値は新規 INSERT
    /// した Proposal のみ（重複排除スキップ分を含まない）。FB-INV-5 維持。同期。DD §3.1 凍結シグネチャ。
    ///
    /// 【v3 差分】legixy では Proposal 生成成功時 observation は analyzing のまま据え置き、
    /// approve 連動で resolved 化する（DD §2.2）。変換規則なしカテゴリは skipped 終端、
    /// 処理失敗は pending 復帰（Claim Release）。
    pub fn analyze(db: &Connection) -> Result<Vec<Proposal>, FeedbackError> {
        let claimed = Self::claim_pending(db)?;
        let mut created = Vec::new();

        for obs in &claimed {
            match Self::process_one(db, obs) {
                Ok(Some(proposal)) => {
                    // Proposal 生成成功 → observation は analyzing のまま（approve で resolved 化）。
                    created.push(proposal);
                }
                Ok(None) => {
                    // semantic_key 重複で新規 INSERT 抑止（FB-INV-5）。既存 proposal の lifecycle に
                    // 委ねるため observation は analyzing のまま据え置く（再 claim 対象外）。
                }
                Err(ProcessError::Unconvertible) => {
                    // 変換規則なしカテゴリ → skipped 終端（永久再 claim 解消、ADR-LGX-019）。
                    Self::update_observation_status(db, obs.id, "skipped")?;
                }
                Err(ProcessError::Db(e)) => {
                    // Claim Release: pending に戻して再試行可能にする。AnalyzeFailed を返す。
                    Self::update_observation_status(db, obs.id, "pending")?;
                    return Err(FeedbackError::AnalyzeFailed {
                        observation_id: obs.id,
                        detail: e.to_string(),
                    });
                }
            }
        }

        Ok(created)
    }

    /// Pessimistic Claim を単一トランザクション内で実行する（DD §3.1 step 1）。
    fn claim_pending(db: &Connection) -> Result<Vec<ClaimedObservation>, FeedbackError> {
        let conn = db.sql();
        let tx = conn.unchecked_transaction().map_err(map_sql_err)?;
        tx.execute(
            "UPDATE observations SET status = 'analyzing' WHERE status = 'pending'",
            [],
        )
        .map_err(map_sql_err)?;

        let mut stmt = tx
            .prepare(
                "SELECT id, category, message, related_ids \
                 FROM observations WHERE status = 'analyzing' ORDER BY id",
            )
            .map_err(map_sql_err)?;

        let rows = stmt
            .query_map([], |row| {
                let related_ids_str: String = row.get(3)?;
                let related_ids: Vec<String> =
                    serde_json::from_str(&related_ids_str).unwrap_or_default();
                Ok(ClaimedObservation {
                    id: row.get(0)?,
                    category: row.get(1)?,
                    message: row.get(2)?,
                    related_ids,
                })
            })
            .map_err(map_sql_err)?;

        let mut claimed = Vec::new();
        for row in rows {
            claimed.push(row.map_err(map_sql_err)?);
        }
        drop(stmt);
        tx.commit().map_err(map_sql_err)?;
        Ok(claimed)
    }

    /// 単一 Observation を Proposal に変換する。
    /// - `Ok(Some)`: 新規 INSERT 成功
    /// - `Ok(None)`: semantic_key 重複で新規 INSERT 抑止（FB-INV-5）
    /// - `Err(Unconvertible)`: 既知だが変換規則なし → skipped 化
    /// - `Err(Db)`: その他 DB エラー → pending へ Claim Release
    fn process_one(
        db: &Connection,
        obs: &ClaimedObservation,
    ) -> Result<Option<Proposal>, ProcessError> {
        let (kind, action_value) =
            Self::generate_proposal_action(obs).ok_or(ProcessError::Unconvertible)?;

        let semantic_key = Self::build_semantic_key(&kind, &action_value);

        // FB-INV-5: 同一 semantic_key の pending Proposal があれば新規挿入を抑止。
        let existing: Option<i64> = db
            .sql()
            .prepare("SELECT id FROM proposals WHERE semantic_key = ?1 AND status = 'pending' LIMIT 1")
            .map_err(ProcessError::from_sql)?
            .query_row(params![&semantic_key], |row| row.get(0))
            .optional()
            .map_err(ProcessError::from_sql)?;

        if existing.is_some() {
            return Ok(None);
        }

        let action_json = action_value.to_string();
        let title = format!("{}: {}", kind, obs.message);
        let description = obs.message.clone();

        db.sql()
            .execute(
                "INSERT INTO proposals \
                 (observation_id, kind, semantic_key, title, description, action_json, \
                  status, created_at) \
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, 'pending', datetime('now'))",
                params![obs.id, &kind, &semantic_key, &title, &description, &action_json],
            )
            .map_err(ProcessError::from_sql)?;
        let proposal_id = db.sql().last_insert_rowid();

        let proposal = db
            .sql()
            .prepare(
                "SELECT id, observation_id, kind, semantic_key, title, description, \
                        action_json, status, decided_at, decided_reason, created_at \
                 FROM proposals WHERE id = ?1",
            )
            .map_err(ProcessError::from_sql)?
            .query_row(params![proposal_id], |row| {
                let status_str: String = row.get(7)?;
                Ok(Proposal {
                    id: row.get(0)?,
                    observation_id: row.get(1)?,
                    kind: row.get(2)?,
                    semantic_key: row.get(3)?,
                    title: row.get(4)?,
                    description: row.get(5)?,
                    action_json: row.get(6)?,
                    status: ProposalStatus::from_str(&status_str).unwrap_or(ProposalStatus::Pending),
                    decided_at: row.get(8)?,
                    decided_reason: row.get(9)?,
                    created_at: row.get(10)?,
                })
            })
            .map_err(ProcessError::from_sql)?;

        Ok(Some(proposal))
    }

    /// カテゴリ別変換ルール（DD §3.1）。`None` なら既知だが変換規則なし（skipped 扱い）。
    /// 【legixy 差分】orphan_file / semantic_similarity は変換せず skipped 終端（DD §2.2）。
    fn generate_proposal_action(
        obs: &ClaimedObservation,
    ) -> Option<(String, serde_json::Value)> {
        match obs.category.as_str() {
            "chain_integrity" => {
                let missing_id = obs.related_ids.first().cloned().unwrap_or_default();
                Some((
                    "add_chain_entry".to_string(),
                    serde_json::json!({ "missing_id": missing_id }),
                ))
            }
            "link_candidate" => {
                let mut ids = obs.related_ids.clone();
                ids.sort();
                let source_id = ids.first().cloned().unwrap_or_default();
                let target_id = ids.get(1).cloned().unwrap_or_default();
                Some((
                    "add_link".to_string(),
                    serde_json::json!({ "source_id": source_id, "target_id": target_id }),
                ))
            }
            "drift" => {
                let changed_id = obs.related_ids.first().cloned().unwrap_or_default();
                Some((
                    "update_doc".to_string(),
                    serde_json::json!({
                        "changed_id": changed_id,
                        "review_targets": obs.related_ids,
                    }),
                ))
            }
            // orphan_file / semantic_similarity / observe 3 値 → 変換規則なし（skipped 終端）。
            _ => None,
        }
    }

    /// kind ごとのテキスト連結方式で semantic_key を生成する（DD §3.1 / v3 底本）。
    fn build_semantic_key(kind: &str, action: &serde_json::Value) -> String {
        match kind {
            "add_chain_entry" => {
                let missing_id = action.get("missing_id").and_then(|v| v.as_str()).unwrap_or("");
                format!("add_chain_entry:{}", missing_id)
            }
            "add_link" => {
                let source_id = action.get("source_id").and_then(|v| v.as_str()).unwrap_or("");
                let target_id = action.get("target_id").and_then(|v| v.as_str()).unwrap_or("");
                let mut pair = [source_id, target_id];
                pair.sort();
                format!("add_link:{}:{}", pair[0], pair[1])
            }
            "update_doc" => {
                let changed_id = action.get("changed_id").and_then(|v| v.as_str()).unwrap_or("");
                format!("update_doc:{}", changed_id)
            }
            _ => format!("unknown:{}", kind),
        }
    }

    fn update_observation_status(
        db: &Connection,
        id: i64,
        status: &str,
    ) -> Result<(), FeedbackError> {
        db.sql()
            .execute(
                "UPDATE observations SET status = ?1 WHERE id = ?2",
                params![status, id],
            )
            .map_err(map_sql_err)?;
        Ok(())
    }
}

/// process_one の内部エラー分類（analyze の observation 状態更新を分岐させる）。
enum ProcessError {
    /// 既知だが変換規則なし → skipped 化対象。
    Unconvertible,
    /// その他 DB エラー → pending Claim Release 対象。
    Db(FeedbackError),
}

impl ProcessError {
    fn from_sql(err: rusqlite::Error) -> Self {
        ProcessError::Db(map_sql_err(err))
    }
}
