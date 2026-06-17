// (module of SRC-LGX-008; anchor: manager.rs)（recorder 部）
// DD-LGX-008 §2.1 RecordResult / §3.1 ObservationRecorder::record。
//   dedup INSERT・UNIQUE 制約 fallback・distinct→昇順 sort→JSON 正準化（REQ.11 / FB-INV-1）。

use rusqlite::{params, OptionalExtension};

use crate::db::{map_sql_err, Connection};
use crate::error::FeedbackError;
use crate::observer::NewObservation;

/// ObservationRecorder::record の戻り値（DD §2.1）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RecordResult {
    pub id: i64,
    pub skipped: bool, // true = 既存 pending/analyzing との dedup で INSERT スキップ
}

/// Observation の dedup INSERT 担当（DD §3.1）。
pub struct ObservationRecorder;

impl ObservationRecorder {
    /// related_ids を distinct→昇順 sort→JSON 化して dedup キーとする。同一
    /// (category, related_ids_json) かつ status IN ('pending','analyzing') が存在すれば INSERT
    /// スキップ（FB-INV-1 / REQ.11）。並行 INSERT 競合は UNIQUE 制約違反 → SELECT fallback で吸収。
    /// 同期。DD §3.1 凍結シグネチャ。
    pub fn record(
        obs: &NewObservation,
        db: &Connection,
    ) -> Result<RecordResult, FeedbackError> {
        // 【v3 差分】message は trim 後 1 文字以上必須（EmptyObservationMessage、GAP-LGX-121）。
        // INSERT は発生しない（早期 return）。
        if obs.message.trim().is_empty() {
            return Err(FeedbackError::EmptyObservationMessage);
        }

        // distinct→昇順 sort→JSON 正準化（【v3 差分】v3 は sort のみ、REQ.11 正準定義）。
        let related_ids_json = Self::canonicalize_related_ids(&obs.related_ids)?;

        // 既存 pending/analyzing に同一 (category, related_ids_json) があれば INSERT スキップ。
        if let Some(existing_id) = Self::find_existing(db, &obs.category, &related_ids_json)? {
            return Ok(RecordResult {
                id: existing_id,
                skipped: true,
            });
        }

        // INSERT 試行。UNIQUE INDEX idx_obs_dedup 違反（並行レース）時は SELECT fallback で吸収。
        let insert_result = db.sql().execute(
            "INSERT INTO observations \
             (source, category, severity, message, related_ids, context_json, status, created_at) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, 'pending', datetime('now'))",
            params![
                obs.source,
                obs.category,
                obs.severity,
                obs.message,
                related_ids_json,
                obs.context_json,
            ],
        );

        match insert_result {
            Ok(_) => Ok(RecordResult {
                id: db.sql().last_insert_rowid(),
                skipped: false,
            }),
            Err(e) if is_unique_constraint(&e) => {
                let existing_id = Self::find_existing(db, &obs.category, &related_ids_json)?
                    .ok_or_else(|| map_sql_err(e))?;
                Ok(RecordResult {
                    id: existing_id,
                    skipped: true,
                })
            }
            Err(e) => Err(map_sql_err(e)),
        }
    }

    /// related_ids の正準化（distinct→昇順 sort→JSON）。dedup キー生成の純関数部。
    /// DD §2.1 / §7.1（distinct→sort→JSON の順、【v3 差分】v3 は sort のみ）。
    /// property テスト（ケース 4）が決定性を検証する。
    pub fn canonicalize_related_ids(
        related_ids: &[String],
    ) -> Result<String, FeedbackError> {
        let mut ids = related_ids.to_vec();
        ids.sort();
        ids.dedup(); // 隣接重複除去（sort 済なので全重複が隣接化される＝distinct）
        let json = serde_json::to_string(&ids)?;
        Ok(json)
    }

    fn find_existing(
        db: &Connection,
        category: &str,
        related_ids_json: &str,
    ) -> Result<Option<i64>, FeedbackError> {
        db.sql()
            .prepare(
                "SELECT id FROM observations \
                 WHERE category = ?1 AND related_ids = ?2 \
                   AND status IN ('pending', 'analyzing') \
                 LIMIT 1",
            )
            .map_err(map_sql_err)?
            .query_row(params![category, related_ids_json], |row| {
                row.get::<_, i64>(0)
            })
            .optional()
            .map_err(map_sql_err)
    }
}

fn is_unique_constraint(err: &rusqlite::Error) -> bool {
    matches!(
        err,
        rusqlite::Error::SqliteFailure(ffi, _)
            if ffi.code == rusqlite::ErrorCode::ConstraintViolation
    )
}
