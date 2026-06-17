// Document ID: SRC-LGX-012
// snapshot::create / list / delete / resolve_label / generate_snapshot_id（DD-LGX-012 §3）。
//
// TC[RED] scaffold。単一トランザクション・taken_at DESC + snapshot_id DESC タイブレーク・
// 空ストア非永続・deleted_rows=0=Ok は SRC[GREEN] で実装する。

pub mod types;

pub use types::{LabelResolveResult, SnapshotCreateResult, SnapshotDeleteResult};

use std::time::{SystemTime, UNIX_EPOCH};

use crate::error::SnapshotError;
use crate::store::{EmbeddingStore, SnapshotMeta};

/// `snap-{epoch_ms 13 桁 16 進}-{8 桁 16 進乱数}` 形式（DD-LGX-012 §3）。一意性保証なし。
pub fn generate_snapshot_id() -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    let epoch_ms = now.as_millis() as u64;
    // 13 桁 16 進（epoch_ms は 2026 年で約 11 桁、ゼロパディングで 13 桁固定）。
    let ms_hex = format!("{:013x}", epoch_ms & 0xFFFF_FFFF_FFFFF); // 52bit = 13 hex
    // nanos 下位を疑似乱数として 8 桁 16 進化（rand 依存回避、一意性保証は不要）。
    let rand32 = (now.subsec_nanos()) ^ (epoch_ms as u32).rotate_left(13);
    format!("snap-{}-{:08x}", ms_hex, rand32)
}

/// 単一トランザクション。node_count=0 のとき DB 永続なし（DD-LGX-012 §3、SPEC REQ.02 2a）。
/// embeddings テーブルは変更しない（read-only）。
pub fn create(
    store: &EmbeddingStore,
    snapshot_id: &str,
    label: Option<&str>,
) -> Result<SnapshotCreateResult, SnapshotError> {
    let node_count = store
        .create_snapshot(snapshot_id, label)
        .map_err(SnapshotError::TransactionFailed)?;
    Ok(SnapshotCreateResult {
        snapshot_id: snapshot_id.to_string(),
        label: label.map(|s| s.to_string()),
        node_count,
    })
}

/// taken_at 降順 + snapshot_id DESC タイブレークの安定整列（DD-LGX-012 §3、§11 SQL）。read-only。
pub fn list(store: &EmbeddingStore) -> Result<Vec<SnapshotMeta>, SnapshotError> {
    Ok(store.list_snapshots()?)
}

/// 単一トランザクション。deleted_rows=0 は SnapshotError ではなく Ok（exit 0 パス、DD-LGX-012 §3、
/// SPEC REQ.02 6b）。embeddings テーブルは変更しない（read-only）。
pub fn delete(
    store: &EmbeddingStore,
    snapshot_id: &str,
) -> Result<SnapshotDeleteResult, SnapshotError> {
    let deleted_rows = store.delete_snapshot(snapshot_id)?;
    Ok(SnapshotDeleteResult {
        snapshot_id: snapshot_id.to_string(),
        deleted_rows,
    })
}

/// taken_at DESC + snapshot_id DESC で 1 件に決定論的解決（DD-LGX-012 §3、SPEC REQ.02 6a）。
/// DB エラーのみ Err（label 不在は Ok(NotFound)）。
pub fn resolve_label(
    store: &EmbeddingStore,
    label: &str,
) -> Result<LabelResolveResult, SnapshotError> {
    match store.resolve_snapshot_id_by_label(label)? {
        Some(id) => Ok(LabelResolveResult::Resolved(id)),
        None => Ok(LabelResolveResult::NotFound),
    }
}
