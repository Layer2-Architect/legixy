// (module of SRC-LGX-002; anchor: compiler.rs)
// legixy-ctx::audit_logger — AuditLogger（DD-LGX-002 §3）
//
// TC[RED] scaffold。new は借用保持の実体、log は todo!()。
// REQ.07/09/19。db=None で no-op。失敗は stderr Warning のみで常に Ok(())（ADR-LGX-004）。
// DB 存在時のみ書込（S2-22）。WAL + busy_timeout 5000ms（S2-04）。

use crate::compiler::CompileInput;
use crate::db::DbConn;
use crate::error::ContextError;
use crate::result::ContextResult;

/// context_log ベストエフォート書込（DD-LGX-002 §3）。db を借用。
pub struct AuditLogger<'a> {
    db: Option<&'a DbConn>,
}

impl<'a> AuditLogger<'a> {
    /// db 借用で構築（None で no-op ロガー）。
    pub fn new(db: Option<&'a DbConn>) -> Self {
        AuditLogger { db }
    }

    /// DD-LGX-002 §3 凍結。db=None で no-op。書込失敗は stderr Warning のみで常に Ok(())。
    /// REQ.07/09/19 / ADR-LGX-004: 監査ログはベストエフォート。書込結果に関わらず
    /// 本処理（compile）を失敗させない（exit 0 維持）。
    pub fn log(
        &self,
        input: &CompileInput,
        result: &ContextResult,
    ) -> Result<(), ContextError> {
        // db=None は no-op（engine.db 不在、S2-22）。
        let Some(db) = self.db else {
            return Ok(());
        };
        // 書込試行はベストエフォート。失敗は stderr Warning のみで握り潰し常に Ok(())。
        if let Err(e) = self.try_write(db, input, result) {
            eprintln!("Warning: context_log への監査ログ書込に失敗しました（処理は継続）: {e}");
        }
        Ok(())
    }

    /// context_log への実書込（SHARED-NEED: 本来 rusqlite で WAL + busy_timeout 5000ms）。
    /// 接続ハンドル不在（DbConn::new / DB 不在）時は書込不能を返し、上位 `log` がベストエフォートで
    /// 握り潰す（REQ.19、TS-LGX-004 ケース21）。接続あり時は context_log へ 1 行 INSERT する
    /// （WAL + busy_timeout=5000 は DbConn::open 設定済み）。
    fn try_write(
        &self,
        db: &DbConn,
        input: &CompileInput,
        result: &ContextResult,
    ) -> Result<(), String> {
        let Some(conn) = db.conn() else {
            return Err("audit log backend unavailable (no engine.db connection)".to_string());
        };
        // target_id: 先頭ターゲットの解決済み artifact_id（未解決ならパス文字列）。schema は NOT NULL。
        let target_id = result
            .targets
            .first()
            .and_then(|t| t.artifact_id.clone())
            .or_else(|| {
                result
                    .targets
                    .first()
                    .map(|t| t.file_path.to_string_lossy().into_owned())
            })
            .unwrap_or_else(|| "(none)".to_string());
        // payload: 呼出し内容と解決結果のサマリ JSON（FB ContextAuditReader が raw で返す）。
        let payload = serde_json::json!({
            "command": input.command,
            "target_files": input
                .target_files
                .iter()
                .map(|p| p.to_string_lossy().into_owned())
                .collect::<Vec<_>>(),
            "resolved_targets": result
                .targets
                .iter()
                .filter_map(|t| t.artifact_id.clone())
                .collect::<Vec<_>>(),
            "upstream_count": result.upstream.len(),
            "unresolved": result
                .unresolved_targets
                .iter()
                .map(|p| p.to_string_lossy().into_owned())
                .collect::<Vec<_>>(),
        })
        .to_string();
        conn.execute(
            "INSERT INTO context_log (target_id, granularity, payload) VALUES (?1, ?2, ?3)",
            rusqlite::params![target_id, result.granularity.as_str(), payload],
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }
}
