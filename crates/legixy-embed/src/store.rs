// (module of SRC-LGX-007; anchor: orchestrator.rs)
// EmbeddingStore / EmbeddingRow / SnapshotMeta（DD-LGX-007 §2.1・§3、DD-LGX-012 §2.1）。
//
// EmbeddingStore は legixy-embed 所有（DD-007 §2.1、ADR-LGX-021 §2.3）。rusqlite::Connection で
// engine.db の embeddings / embedding_snapshots テーブルを本 crate 内に実装する。テストは
// in-memory（Connection::open_in_memory）。`stub` / `empty` 構築子は所与の行で in-memory DB を
// 初期化する test-double 入口（SRC[GREEN] では open_engine_db の Connection を受け取る配線へ）。

use std::path::Path;
use std::time::Duration;

use rusqlite::{params, Connection, OptionalExtension};

use legixy_db::DbError;
use legixy_graph::Node;

use crate::embedder::EmbedResult;

/// REL.07 既定 busy_timeout（暫定 5000ms、NFR-LGX-001.REL.07）。
/// 並行ロック待機の上限。超過時 SQLite は SQLITE_BUSY を返し、本層は Err へ昇格する（無限リトライ無し）。
pub const DEFAULT_BUSY_TIMEOUT_MS: u32 = 5000;

/// engine.db の embeddings テーブル 1 行（DD-LGX-007 §2.1、v3 EmbeddingRow と同一構造）。
#[derive(Debug, Clone, PartialEq)]
pub struct EmbeddingRow {
    pub node_id: String,
    pub embedding: Vec<f32>,
    pub dim: usize,
    pub model_version: String,
    pub content_hash: String,
    pub context: Option<String>,
    pub context_hash: Option<String>,
    pub created_at: String,
}

/// embedding_snapshots テーブルのメタ情報（DD-LGX-007 §2.1 / DD-LGX-012 §2.1。list 用）。
#[derive(Debug, Clone, PartialEq)]
pub struct SnapshotMeta {
    pub snapshot_id: String,
    pub label: Option<String>,
    pub node_count: usize,
    pub taken_at: String,
}

/// snapshot 行の表現（embedding_snapshots の 1 行、DD-LGX-012 §11 スキーマ相当）。
#[derive(Debug, Clone, PartialEq)]
pub struct SnapshotRow {
    pub snapshot_id: String,
    pub label: Option<String>,
    pub node_id: String,
    pub model_version: String,
    pub content_hash: String,
    pub taken_at: String,
}

/// engine.db read/write ラッパ（DD-LGX-007 §2.1、legixy-embed 所有）。
pub struct EmbeddingStore {
    conn: Connection,
}

fn map_err(e: rusqlite::Error) -> DbError {
    DbError::Query(e.to_string())
}

/// 接続 PRAGMA を適用する（REL.07 busy_timeout 上限 + PERF.07 WAL/synchronous=NORMAL）。
///
/// - REL.07: `busy_timeout` で並行ロック待機の上限を設定。超過時 SQLite は SQLITE_BUSY を返し、
///   呼出側はそれを Err として受ける（本層はリトライループを持たない＝無限リトライ禁止を構造的に満たす）。
/// - PERF.07: on-disk では WAL モード + synchronous=NORMAL。in-memory では WAL は no-op として無害。
///   接続層 PRAGMA の正準的所有は legixy-db（ADR-LGX-015）。本実装は接続層実装までの暫定配線。
fn configure_connection(conn: &Connection, busy_timeout_ms: u32) -> Result<(), DbError> {
    conn.busy_timeout(Duration::from_millis(busy_timeout_ms as u64))
        .map_err(map_err)?;
    // WAL / synchronous は on-disk のみ意味を持つ。in-memory では rusqlite が "memory" を返すため
    // 失敗扱いにせず無視する（PERF.07 は on-disk engine.db を対象とする）。
    let _ = conn.pragma_update(None, "journal_mode", "WAL");
    let _ = conn.pragma_update(None, "synchronous", "NORMAL");
    Ok(())
}

/// f32 スライスを little-endian u8 列に直列化。
fn f32_slice_to_bytes(v: &[f32]) -> Vec<u8> {
    let mut buf = Vec::with_capacity(v.len() * 4);
    for x in v {
        buf.extend_from_slice(&x.to_le_bytes());
    }
    buf
}

/// little-endian u8 列を f32 ベクトルに復元（端数 4 バイト未満は切り捨て）。
fn bytes_to_f32_vec(bytes: &[u8]) -> Vec<f32> {
    bytes
        .chunks_exact(4)
        .map(|c| f32::from_le_bytes([c[0], c[1], c[2], c[3]]))
        .collect()
}

impl EmbeddingStore {
    /// Connection を所有して構築（DD-007 §3。schema 既存前提）。
    /// REL.07/PERF.07 の接続 PRAGMA（busy_timeout 既定 5000ms + WAL）を適用する。
    pub fn new(conn: Connection) -> Self {
        let _ = configure_connection(&conn, DEFAULT_BUSY_TIMEOUT_MS);
        EmbeddingStore { conn }
    }

    /// on-disk の engine.db を開き、PRAGMA 適用（REL.07/PERF.07）+ schema 初期化して構築する。
    /// `busy_timeout_ms` で並行ロック待機の上限を指定（テストでは短い値で SQLITE_BUSY を誘発可能）。
    pub fn open_on_disk(path: &Path, busy_timeout_ms: u32) -> Result<Self, DbError> {
        let conn = Connection::open(path).map_err(map_err)?;
        configure_connection(&conn, busy_timeout_ms)?;
        Self::init_schema(&conn);
        Ok(EmbeddingStore { conn })
    }

    fn init_schema(conn: &Connection) {
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS embeddings (\
                 node_id TEXT PRIMARY KEY, \
                 embedding BLOB NOT NULL, \
                 embedding_dim INTEGER NOT NULL, \
                 model_version TEXT NOT NULL, \
                 content_hash TEXT NOT NULL, \
                 context TEXT, \
                 context_hash TEXT, \
                 parent_id TEXT, \
                 anchor TEXT, \
                 is_subnode INTEGER NOT NULL DEFAULT 0, \
                 created_at TEXT NOT NULL DEFAULT (datetime('now'))\
             );\
             CREATE TABLE IF NOT EXISTS embedding_snapshots (\
                 snapshot_id TEXT NOT NULL, \
                 label TEXT, \
                 node_id TEXT NOT NULL, \
                 embedding BLOB, \
                 embedding_dim INTEGER, \
                 model_version TEXT, \
                 content_hash TEXT, \
                 taken_at TEXT NOT NULL DEFAULT (datetime('now')), \
                 PRIMARY KEY (snapshot_id, node_id)\
             );",
        )
        .expect("in-memory schema init");
    }

    /// テスト用 test-double 構築子（embeddings 行・snapshot 行を所与で与え in-memory DB を作る）。
    pub fn stub(rows: Vec<EmbeddingRow>, snapshot_rows: Vec<SnapshotRow>) -> Self {
        let conn = Connection::open_in_memory().expect("in-memory db");
        Self::init_schema(&conn);
        for r in &rows {
            let blob = f32_slice_to_bytes(&r.embedding);
            conn.execute(
                "INSERT INTO embeddings \
                 (node_id, embedding, embedding_dim, model_version, content_hash, context, context_hash, created_at) \
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                params![
                    r.node_id,
                    blob,
                    r.dim as i64,
                    r.model_version,
                    r.content_hash,
                    r.context,
                    r.context_hash,
                    r.created_at,
                ],
            )
            .expect("seed embeddings");
        }
        for s in &snapshot_rows {
            conn.execute(
                "INSERT INTO embedding_snapshots \
                 (snapshot_id, label, node_id, model_version, content_hash, taken_at) \
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![
                    s.snapshot_id,
                    s.label,
                    s.node_id,
                    s.model_version,
                    s.content_hash,
                    s.taken_at,
                ],
            )
            .expect("seed snapshots");
        }
        EmbeddingStore { conn }
    }

    /// 空ストア（embeddings 0 行・snapshot 0 行）。DB 不在 ≡ 空ストアの test-double。
    pub fn empty() -> Self {
        let conn = Connection::open_in_memory().expect("in-memory db");
        Self::init_schema(&conn);
        EmbeddingStore { conn }
    }

    /// content_hash + model_version 双方一致で true（DD-LGX-007 §3、SCORE-INV-1 ∧ SCORE-INV-2）。
    pub fn is_up_to_date(
        &self,
        node_id: &str,
        content_hash: &str,
        model_version: &str,
    ) -> Result<bool, DbError> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT 1 FROM embeddings \
                 WHERE node_id = ?1 AND content_hash = ?2 AND model_version = ?3 LIMIT 1",
            )
            .map_err(map_err)?;
        let exists = stmt
            .query_row(params![node_id, content_hash, model_version], |_| Ok(()))
            .optional()
            .map_err(map_err)?
            .is_some();
        Ok(exists)
    }

    /// ノード単位 1 Tx の INSERT OR REPLACE upsert（DD-LGX-007 §3、REQ.08）。
    pub fn upsert_with_subnode_meta(
        &self,
        node: &Node,
        result: &EmbedResult,
    ) -> Result<(), DbError> {
        let blob = f32_slice_to_bytes(&result.embedding);
        self.conn
            .execute(
                "INSERT INTO embeddings \
                 (node_id, embedding, embedding_dim, model_version, content_hash, context, context_hash, parent_id, anchor, is_subnode) \
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, NULL, 0) \
                 ON CONFLICT(node_id) DO UPDATE SET \
                     embedding = excluded.embedding, \
                     embedding_dim = excluded.embedding_dim, \
                     model_version = excluded.model_version, \
                     content_hash = excluded.content_hash, \
                     context = excluded.context, \
                     context_hash = excluded.context_hash, \
                     parent_id = excluded.parent_id, \
                     created_at = datetime('now')",
                params![
                    node.id,
                    blob,
                    result.dim as i64,
                    result.model_version,
                    result.content_hash,
                    result.context,
                    result.context_hash,
                    node.parent_id,
                ],
            )
            .map_err(map_err)?;
        Ok(())
    }

    /// サブノード embedding を upsert する（is_subnode=1 + parent_id + anchor、ADR-LGX-023）。
    /// embed_all の include_subnodes 経路が使用。INSERT OR REPLACE（node_id PK）。
    pub fn upsert_subnode(
        &self,
        subnode_id: &str,
        parent_id: &str,
        anchor: &str,
        result: &EmbedResult,
    ) -> Result<(), DbError> {
        let blob = f32_slice_to_bytes(&result.embedding);
        self.conn
            .execute(
                "INSERT INTO embeddings \
                 (node_id, embedding, embedding_dim, model_version, content_hash, context, context_hash, parent_id, anchor, is_subnode) \
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, 1) \
                 ON CONFLICT(node_id) DO UPDATE SET \
                     embedding = excluded.embedding, \
                     embedding_dim = excluded.embedding_dim, \
                     model_version = excluded.model_version, \
                     content_hash = excluded.content_hash, \
                     context = excluded.context, \
                     context_hash = excluded.context_hash, \
                     parent_id = excluded.parent_id, \
                     anchor = excluded.anchor, \
                     is_subnode = 1, \
                     created_at = datetime('now')",
                params![
                    subnode_id,
                    blob,
                    result.dim as i64,
                    result.model_version,
                    result.content_hash,
                    result.context,
                    result.context_hash,
                    parent_id,
                    anchor,
                ],
            )
            .map_err(map_err)?;
        Ok(())
    }

    /// ORDER BY node_id ASC で全行ロード（DD-LGX-007 §3、SCORE-INV-1 決定性担保）。
    pub fn load_all(&self) -> Result<Vec<EmbeddingRow>, DbError> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT node_id, embedding, embedding_dim, model_version, content_hash, \
                        context, context_hash, created_at \
                 FROM embeddings ORDER BY node_id ASC",
            )
            .map_err(map_err)?;
        let rows = stmt
            .query_map([], |r| {
                let blob: Vec<u8> = r.get(1)?;
                let dim_i64: i64 = r.get(2)?;
                Ok(EmbeddingRow {
                    node_id: r.get(0)?,
                    embedding: bytes_to_f32_vec(&blob),
                    dim: dim_i64 as usize,
                    model_version: r.get(3)?,
                    content_hash: r.get(4)?,
                    context: r.get(5)?,
                    context_hash: r.get(6)?,
                    created_at: r.get(7)?,
                })
            })
            .map_err(map_err)?;
        let mut result = Vec::new();
        for row in rows {
            result.push(row.map_err(map_err)?);
        }
        Ok(result)
    }

    /// 未登録は Ok(None)（DD-LGX-007 §3、HashMatchState::Missing 判定に使用）。
    pub fn load_embedding(&self, node_id: &str) -> Result<Option<EmbeddingRow>, DbError> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT node_id, embedding, embedding_dim, model_version, content_hash, \
                        context, context_hash, created_at \
                 FROM embeddings WHERE node_id = ?1",
            )
            .map_err(map_err)?;
        let row = stmt
            .query_row(params![node_id], |r| {
                let blob: Vec<u8> = r.get(1)?;
                let dim_i64: i64 = r.get(2)?;
                Ok(EmbeddingRow {
                    node_id: r.get(0)?,
                    embedding: bytes_to_f32_vec(&blob),
                    dim: dim_i64 as usize,
                    model_version: r.get(3)?,
                    content_hash: r.get(4)?,
                    context: r.get(5)?,
                    context_hash: r.get(6)?,
                    created_at: r.get(7)?,
                })
            })
            .optional()
            .map_err(map_err)?;
        Ok(row)
    }

    /// 指定 snapshot 内の 1 ノードの embedding 行を読む（drift --against snapshot 用、BUG-004）。
    /// 未登録は Ok(None)。context 系列カラムは snapshot に無いため None。
    pub fn load_snapshot_embedding(
        &self,
        snapshot_id: &str,
        node_id: &str,
    ) -> Result<Option<EmbeddingRow>, DbError> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT node_id, embedding, embedding_dim, model_version, content_hash \
                 FROM embedding_snapshots WHERE snapshot_id = ?1 AND node_id = ?2",
            )
            .map_err(map_err)?;
        let row = stmt
            .query_row(params![snapshot_id, node_id], |r| {
                let blob: Vec<u8> = r.get(1)?;
                let dim_i64: i64 = r.get(2)?;
                Ok(EmbeddingRow {
                    node_id: r.get(0)?,
                    embedding: bytes_to_f32_vec(&blob),
                    dim: dim_i64 as usize,
                    model_version: r.get(3)?,
                    content_hash: r.get(4)?,
                    context: None,
                    context_hash: None,
                    created_at: String::new(),
                })
            })
            .optional()
            .map_err(map_err)?;
        Ok(row)
    }

    /// 現 embeddings 全行を embedding_snapshots へコピー（DD-LGX-007 §3、1 Tx）。行数を返す。
    /// PRIMARY KEY (snapshot_id, node_id) 衝突時は Err（ロールバック）。
    pub fn create_snapshot(
        &self,
        snapshot_id: &str,
        label: Option<&str>,
    ) -> Result<usize, DbError> {
        let tx = self.conn.unchecked_transaction().map_err(map_err)?;
        let n = tx
            .execute(
                "INSERT INTO embedding_snapshots \
                 (snapshot_id, label, node_id, embedding, embedding_dim, model_version, content_hash) \
                 SELECT ?1, ?2, node_id, embedding, embedding_dim, model_version, content_hash \
                 FROM embeddings",
                params![snapshot_id, label],
            )
            .map_err(map_err)?;
        tx.commit().map_err(map_err)?;
        Ok(n)
    }

    /// 全スナップショットのメタ情報（snapshot_id 単位で集計、taken_at DESC + snapshot_id DESC）。
    pub fn list_snapshots(&self) -> Result<Vec<SnapshotMeta>, DbError> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT snapshot_id, label, COUNT(*), MAX(taken_at) \
                 FROM embedding_snapshots \
                 GROUP BY snapshot_id, label \
                 ORDER BY MAX(taken_at) DESC, snapshot_id DESC",
            )
            .map_err(map_err)?;
        let rows = stmt
            .query_map([], |r| {
                Ok(SnapshotMeta {
                    snapshot_id: r.get(0)?,
                    label: r.get(1)?,
                    node_count: r.get::<_, i64>(2)? as usize,
                    taken_at: r.get(3)?,
                })
            })
            .map_err(map_err)?;
        let mut out = Vec::new();
        for row in rows {
            out.push(row.map_err(map_err)?);
        }
        Ok(out)
    }

    /// snapshot_id を指定して全行削除。削除件数を返す（1 Tx）。
    pub fn delete_snapshot(&self, snapshot_id: &str) -> Result<usize, DbError> {
        let tx = self.conn.unchecked_transaction().map_err(map_err)?;
        let n = tx
            .execute(
                "DELETE FROM embedding_snapshots WHERE snapshot_id = ?1",
                params![snapshot_id],
            )
            .map_err(map_err)?;
        tx.commit().map_err(map_err)?;
        Ok(n)
    }

    /// label 経由で snapshot_id を解決（taken_at DESC + snapshot_id DESC で 1 件）。
    pub fn resolve_snapshot_id_by_label(&self, label: &str) -> Result<Option<String>, DbError> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT snapshot_id FROM embedding_snapshots \
                 WHERE label = ?1 \
                 ORDER BY taken_at DESC, snapshot_id DESC LIMIT 1",
            )
            .map_err(map_err)?;
        let id = stmt
            .query_row(params![label], |r| r.get::<_, String>(0))
            .optional()
            .map_err(map_err)?;
        Ok(id)
    }

    /// 永続化済みサブノード行（is_subnode=1）の (node_id, parent_id, anchor) を node_id ASC で返す
    /// （ADR-LGX-023、refresh-subnodes の照合元）。
    pub fn list_subnodes(&self) -> Result<Vec<SubnodeRef>, DbError> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT node_id, parent_id, anchor FROM embeddings \
                 WHERE is_subnode = 1 ORDER BY node_id ASC",
            )
            .map_err(map_err)?;
        let rows = stmt
            .query_map([], |r| {
                Ok(SubnodeRef {
                    node_id: r.get(0)?,
                    parent_id: r.get(1)?,
                    anchor: r.get(2)?,
                })
            })
            .map_err(map_err)?;
        let mut out = Vec::new();
        for row in rows {
            out.push(row.map_err(map_err)?);
        }
        Ok(out)
    }

    /// サブノード ID のリネーム（embeddings.node_id を old→new に更新）。更新行数を返す（ADR-LGX-023）。
    /// 1 Tx。new_id 既存（PRIMARY KEY 衝突）は Err（呼出側がロールバック扱い）。
    pub fn rename_subnode(&self, old_id: &str, new_id: &str) -> Result<usize, DbError> {
        let n = self
            .conn
            .execute(
                "UPDATE embeddings SET node_id = ?1 WHERE node_id = ?2",
                params![new_id, old_id],
            )
            .map_err(map_err)?;
        Ok(n)
    }
}

/// 永続化済みサブノードの参照（ADR-LGX-023、list_subnodes の戻り）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubnodeRef {
    pub node_id: String,
    pub parent_id: Option<String>,
    pub anchor: Option<String>,
}
