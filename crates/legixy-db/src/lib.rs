// legixy-db: engine.db スキーマ・接続・永続層（ADR-LGX-020 / ADR-LGX-015）
// TC[RED] scaffold。意味層検証で check が `Option<&EmbeddingStore>` を借用する。
// embeddings/observations/proposals/snapshots の実スキーマは SRC[GREEN] で実装する。

/// embeddings テーブルアクセスラッパ（DD-LGX-007 所有、ADR-LGX-021 §2.3）。
/// 本 scaffold では check が意味層で借用する不透明ハンドルとしてのみ用いる。
#[derive(Debug, Default)]
pub struct EmbeddingStore {
    _private: (),
}

impl EmbeddingStore {
    pub fn new() -> Self {
        EmbeddingStore { _private: () }
    }
}

/// DB 接続・クエリ失敗（DD-LGX-001 §2.3 `CheckError::Db`）。
#[derive(Debug, thiserror::Error)]
pub enum DbError {
    #[error("db open error: {0}")]
    Open(String),
    #[error("db query error: {0}")]
    Query(String),
}
