// (module of SRC-LGX-008; anchor: manager.rs)（cli ファサード部）
// DD-LGX-008 §2.1 FeedbackReport / ProposalSummary / §3.1 FeedbackCli ファサード群。
//   legixy-cli が CLI サブコマンドに組み込む薄いファサード（stdout 出力は legixy-cli 層が担う）。

use rusqlite::params;

use legixy_check::CheckReport;

use crate::analyzer::{Proposal, ProposalAnalyzer, ProposalStatus};
use crate::db::{map_sql_err, Connection};
use crate::error::FeedbackError;
use crate::manager::ProposalManager;
use crate::observer::AutoObserver;
use crate::recorder::ObservationRecorder;

/// run_feedback の実行結果サマリ（DD §2.1）。
/// stdout 契約: `feedback: {observations_created} created, {observations_skipped} skipped`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FeedbackReport {
    pub observations_created: usize,
    pub observations_skipped: usize,
}

/// proposals 一覧表示用（FeedbackCli::list_proposals の戻り。DD §2.1）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProposalSummary {
    pub id: i64,
    pub kind: String,
    pub semantic_key: String,
    pub title: String,
    pub status: ProposalStatus,
    pub created_at: String,
}

/// legixy-cli が組み込むファサード（DD §3.1）。各メソッドは下位モジュールの薄いラッパ。
pub struct FeedbackCli;

impl FeedbackCli {
    /// AutoObserver でフィルタ → 各 NewObservation を record → FeedbackReport 集計。
    /// stdout 出力は legixy-cli 層が担う。DD §3.1 凍結シグネチャ。
    pub fn run_feedback(
        db: &Connection,
        check_report: &CheckReport,
    ) -> Result<FeedbackReport, FeedbackError> {
        let candidates = AutoObserver::from_check_results(check_report);
        let mut report = FeedbackReport {
            observations_created: 0,
            observations_skipped: 0,
        };
        for obs in &candidates {
            let result = ObservationRecorder::record(obs, db)?;
            if result.skipped {
                report.observations_skipped += 1;
            } else {
                report.observations_created += 1;
            }
        }
        Ok(report)
    }

    /// ProposalAnalyzer::analyze の薄いラッパ。DD §3.1 凍結シグネチャ。
    pub fn run_analyze(db: &Connection) -> Result<Vec<Proposal>, FeedbackError> {
        ProposalAnalyzer::analyze(db)
    }

    /// status_filter が None の場合は全件（`--status` 省略時相当）。ORDER BY id。read-only。
    /// DD §3.1 凍結シグネチャ。
    pub fn list_proposals(
        db: &Connection,
        status_filter: Option<ProposalStatus>,
    ) -> Result<Vec<ProposalSummary>, FeedbackError> {
        let map_row = |row: &rusqlite::Row<'_>| -> rusqlite::Result<ProposalSummary> {
            let status_str: String = row.get(4)?;
            Ok(ProposalSummary {
                id: row.get(0)?,
                kind: row.get(1)?,
                semantic_key: row.get(2)?,
                title: row.get(3)?,
                status: ProposalStatus::from_str(&status_str).unwrap_or(ProposalStatus::Pending),
                created_at: row.get(5)?,
            })
        };

        let rows: Vec<ProposalSummary> = match status_filter {
            Some(s) => {
                let mut stmt = db
                    .sql()
                    .prepare(
                        "SELECT id, kind, semantic_key, title, status, created_at \
                         FROM proposals WHERE status = ?1 ORDER BY id",
                    )
                    .map_err(map_sql_err)?;
                let collected = stmt
                    .query_map(params![s.as_str()], map_row)
                    .map_err(map_sql_err)?
                    .collect::<rusqlite::Result<_>>()
                    .map_err(map_sql_err)?;
                collected
            }
            None => {
                let mut stmt = db
                    .sql()
                    .prepare(
                        "SELECT id, kind, semantic_key, title, status, created_at \
                         FROM proposals ORDER BY id",
                    )
                    .map_err(map_sql_err)?;
                let collected = stmt
                    .query_map([], map_row)
                    .map_err(map_sql_err)?
                    .collect::<rusqlite::Result<_>>()
                    .map_err(map_sql_err)?;
                collected
            }
        };
        Ok(rows)
    }

    /// ProposalManager::approve の薄いラッパ。DD §3.1 凍結シグネチャ。
    pub fn approve(db: &Connection, proposal_id: i64) -> Result<(), FeedbackError> {
        ProposalManager::approve(proposal_id, db)
    }

    /// ProposalManager::reject の薄いラッパ。DD §3.1 凍結シグネチャ。
    pub fn reject(
        db: &Connection,
        proposal_id: i64,
        reason: &str,
    ) -> Result<(), FeedbackError> {
        ProposalManager::reject(proposal_id, reason, db)
    }
}
