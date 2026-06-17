// (module of SRC-LGX-012; anchor: mod.rs)
// SnapshotCreateResult / SnapshotDeleteResult / LabelResolveResult（DD-LGX-012 §2.1・§2.2）。
//
// TC[RED] scaffold。SnapshotMeta は store.rs（DD-LGX-007 / DD-LGX-012 共有）に定義済み。

/// create 結果の集約型（DD-LGX-012 §2.1。node_count=0 のとき空ストア非永続）。
#[derive(Debug, Clone, PartialEq)]
pub struct SnapshotCreateResult {
    pub snapshot_id: String,
    pub label: Option<String>,
    pub node_count: usize,
}

/// delete 結果（DD-LGX-012 §2.1。deleted_rows=0 は Ok/exit0）。
#[derive(Debug, Clone, PartialEq)]
pub struct SnapshotDeleteResult {
    pub snapshot_id: String,
    pub deleted_rows: usize,
}

/// label 解決の結果（DD-LGX-012 §2.2）。
#[derive(Debug, Clone, PartialEq)]
pub enum LabelResolveResult {
    /// 解決済み snapshot_id。
    Resolved(String),
    /// 該当 label 0 件（delete 6c = exit 1 / drift 曖昧形式 = exit 0 で分岐）。
    NotFound,
}
