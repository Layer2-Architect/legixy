// (module of SRC-LGX-007; anchor: orchestrator.rs)
// detect_drift / DriftFinding / DriftKind（DD-LGX-007 §2.1・§2.2・§3）。
//
// UC-007 の embed エンジン側 drift 検出（content_hash 照合による stale/missing/file-missing 検出）。
// UC-013 の standalone drift 対比コマンド（drift::run）とは別サブシステム。

use std::collections::HashMap;
use std::path::Path;

use legixy_graph::TraceGraph;

use crate::content::content_hash_for;
use crate::error::EmbedError;
use crate::store::{EmbeddingRow, EmbeddingStore};

/// ドリフト検出結果 1 件（DD-LGX-007 §2.1、REQ.05 / REQ.11）。
#[derive(Debug, Clone, PartialEq)]
pub struct DriftFinding {
    pub node_id: String,
    pub stored_hash: Option<String>,
    pub current_hash: Option<String>,
    pub kind: DriftKind,
}

/// DriftFinding の種別（DD-LGX-007 §2.2、REQ.05 3 状態 + ファイル不在）。
/// v3 差分: v3 の missing_file bool を 3 種の enum に置き換え。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DriftKind {
    /// stale: embedding 行あり、content_hash 不一致。
    ContentChanged,
    /// embedding 行あり、ファイルが読めない。
    FileMissing,
    /// 未生成: embedding 行なし（v3 差分: v3 は無言 skip）。
    EmbeddingMissing,
}

/// 未生成ノードを EmbeddingMissing として結果に含む（DD-LGX-007 §3、v3 差分）。
/// 正規化 + content_range 切り出しを embed_all と同一経路で計算。出力順は node_id ASC。
pub fn detect_drift(
    graph: &TraceGraph,
    store: &EmbeddingStore,
    project_root: &Path,
) -> Result<Vec<DriftFinding>, EmbedError> {
    let rows = store.load_all()?;
    let stored: HashMap<&str, &EmbeddingRow> =
        rows.iter().map(|r| (r.node_id.as_str(), r)).collect();

    let mut findings = Vec::new();
    for node in graph.nodes() {
        match stored.get(node.id.as_str()) {
            None => {
                // 未生成: embedding 行なし → EmbeddingMissing（偽 fresh 黙殺を防止）。
                let abs_path = project_root.join(&node.path);
                let current_hash = std::fs::read_to_string(&abs_path)
                    .ok()
                    .map(|raw| content_hash_for(&raw));
                findings.push(DriftFinding {
                    node_id: node.id.clone(),
                    stored_hash: None,
                    current_hash,
                    kind: DriftKind::EmbeddingMissing,
                });
            }
            Some(stored_row) => {
                let abs_path = project_root.join(&node.path);
                match std::fs::read_to_string(&abs_path) {
                    Err(_) => {
                        // ファイル読込不能 → FileMissing（current=None）。
                        findings.push(DriftFinding {
                            node_id: node.id.clone(),
                            stored_hash: Some(stored_row.content_hash.clone()),
                            current_hash: None,
                            kind: DriftKind::FileMissing,
                        });
                    }
                    Ok(raw) => {
                        let current = content_hash_for(&raw);
                        if current != stored_row.content_hash {
                            // stale: hash 不一致 → ContentChanged（stored/current 双方 Some）。
                            findings.push(DriftFinding {
                                node_id: node.id.clone(),
                                stored_hash: Some(stored_row.content_hash.clone()),
                                current_hash: Some(current),
                                kind: DriftKind::ContentChanged,
                            });
                        }
                        // 一致 → drift なし（finding 非発行）。
                    }
                }
            }
        }
    }

    // 出力順は node_id ASC（決定性）。
    findings.sort_by(|a, b| a.node_id.cmp(&b.node_id));
    Ok(findings)
}
