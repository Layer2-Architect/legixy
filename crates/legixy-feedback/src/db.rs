// Document ID: SRC-LGX-008
// (module of SRC-LGX-008; anchor: manager.rs)（db アクセス部）
// SHARED-NEED 解消: Connection は本来 legixy-db 所有（rusqlite::Connection、ADR-LGX-021 §2.4 /
//   DD-LGX-008 §2.1）。legixy-db scaffold に接続済み rusqlite::Connection 型が未配置のため、本
//   crate で rusqlite::Connection を直接ラップする（legixy-db 編集禁止・統合時に移設）。
//   WAL + busy_timeout=5000 + foreign_keys=ON の接続生成と observations/proposals/context_log/
//   custom_edges スキーマ作成（DD-008 所有）を本 crate に内蔵する。

use rusqlite::Connection as SqliteConnection;

use crate::error::FeedbackError;

/// 接続済み engine.db ハンドル（DD-LGX-008 §2.1。rusqlite::Connection をラップ）。
///
/// WAL + busy_timeout=5000 + foreign_keys=ON を接続時に設定し、自 crate 所有テーブル
/// （observations / proposals / context_log / custom_edges）のスキーマを作成する。
#[derive(Debug)]
pub struct Connection {
    inner: SqliteConnection,
}

impl Connection {
    /// in-memory engine.db を開く（テスト用 / DD §6.1）。スキーマと PRAGMA を設定する。
    pub fn open_in_memory() -> Result<Self, DbStubError> {
        let inner = SqliteConnection::open_in_memory().map_err(map_open_err)?;
        Self::init(inner)
    }

    /// 既存ファイルパスを開く（不在は新規作成、破損は DbCorrupted で検出 — DD §6.1 option C）。
    /// open 段階で破損検出された場合は DbStubError を返し、open 成功後の初回操作で破損検出
    /// された場合は各操作が FeedbackError::DbCorrupted を返す。
    pub fn open_path(path: &str) -> Result<Self, DbStubError> {
        let inner = SqliteConnection::open(path).map_err(map_open_err)?;
        Self::init(inner)
    }

    /// PRAGMA 設定 + スキーマ作成。破損 DB はここで NotADatabase / SQLITE_CORRUPT を返し得る。
    fn init(inner: SqliteConnection) -> Result<Self, DbStubError> {
        // WAL は in-memory では適用されないが、ファイル DB では並行安全性に必要（NFR REL.07）。
        // in-memory に対する journal_mode=WAL の pragma_update は失敗し得るため許容する。
        let _ = inner.pragma_update(None, "journal_mode", "WAL");
        let _ = inner.pragma_update(None, "synchronous", "NORMAL");
        inner
            .pragma_update(None, "busy_timeout", 5000)
            .map_err(map_open_err)?;
        inner
            .pragma_update(None, "foreign_keys", "ON")
            .map_err(map_open_err)?;
        inner.execute_batch(SCHEMA).map_err(map_open_err)?;
        Ok(Connection { inner })
    }

    /// 内部 rusqlite::Connection への借用（自 crate モジュール内のみ）。
    pub(crate) fn sql(&self) -> &SqliteConnection {
        &self.inner
    }
}

/// rusqlite::Error を破損判定して FeedbackError に変換する（DD §6.1 option C）。
/// SQLITE_CORRUPT / NotADatabase は DbCorrupted（自動再生成禁止）、それ以外は Db。
pub(crate) fn map_sql_err(err: rusqlite::Error) -> FeedbackError {
    if is_corruption(&err) {
        FeedbackError::DbCorrupted {
            detail: err.to_string(),
        }
    } else {
        FeedbackError::Db(DbStubError::Op(err.to_string()))
    }
}

/// open / init 段階のエラーは DbStubError として返す（呼出し側が open_path で扱う）。
fn map_open_err(err: rusqlite::Error) -> DbStubError {
    DbStubError::Op(err.to_string())
}

fn is_corruption(err: &rusqlite::Error) -> bool {
    use rusqlite::ErrorCode;
    matches!(
        err,
        rusqlite::Error::SqliteFailure(ffi, _)
            if matches!(ffi.code, ErrorCode::DatabaseCorrupt | ErrorCode::NotADatabase)
    )
}

/// DB 操作失敗（本来 rusqlite::Error / legixy_db::DbError）。
/// SHARED-NEED: 統合時に legixy-db のエラー型へ置換。
#[derive(Debug, thiserror::Error)]
pub enum DbStubError {
    #[error("db error: {0}")]
    Op(String),
}

/// 自 crate 所有テーブルのスキーマ（DD-LGX-008 所有。v3 lx-db schema.rs を底本に該当 4 表を抽出）。
const SCHEMA: &str = r#"
    CREATE TABLE IF NOT EXISTS observations (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        source TEXT NOT NULL,
        category TEXT NOT NULL,
        severity TEXT NOT NULL,
        message TEXT NOT NULL,
        related_ids TEXT NOT NULL DEFAULT '[]',
        context_json TEXT,
        status TEXT NOT NULL DEFAULT 'pending',
        created_at TEXT NOT NULL DEFAULT (datetime('now'))
    );

    CREATE UNIQUE INDEX IF NOT EXISTS idx_obs_dedup
        ON observations (category, related_ids)
        WHERE status IN ('pending', 'analyzing');

    CREATE TABLE IF NOT EXISTS proposals (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        observation_id INTEGER,
        kind TEXT NOT NULL DEFAULT '',
        semantic_key TEXT NOT NULL DEFAULT '',
        title TEXT NOT NULL DEFAULT '',
        description TEXT NOT NULL DEFAULT '',
        action_json TEXT NOT NULL DEFAULT '',
        status TEXT NOT NULL DEFAULT 'pending',
        decided_at TEXT,
        decided_reason TEXT,
        created_at TEXT NOT NULL DEFAULT (datetime('now'))
    );

    CREATE INDEX IF NOT EXISTS idx_proposals_semantic_key
        ON proposals (semantic_key, status);

    CREATE TABLE IF NOT EXISTS custom_edges (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        from_id TEXT NOT NULL,
        to_id TEXT NOT NULL,
        reason TEXT NULL,
        created_at TEXT DEFAULT (datetime('now')),
        UNIQUE(from_id, to_id)
    );

    CREATE TABLE IF NOT EXISTS context_log (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        target_id TEXT NOT NULL,
        granularity TEXT NULL,
        payload TEXT NOT NULL,
        created_at TEXT DEFAULT (datetime('now'))
    );
"#;
