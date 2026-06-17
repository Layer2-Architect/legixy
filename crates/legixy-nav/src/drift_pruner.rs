// (module of SRC-LGX-005; anchor: investigate.rs)
// legixy-nav drift 枝刈り（DD-LGX-005 §4 / §6。v3 lx-nav/src/drift_pruner.rs 整合）。
// scores テーブル（score_type='drift'）照会 + drift_threshold 以上抽出 + 降順整列。
// db = None または SQL 照会失敗時は空 suspicious_nodes でベストエフォート継続（NFR REL.02）。
// SQL 失敗は NavError に昇格させず stderr 警告 `[nav] drift pruning skipped: ...`（DD-005 §6）。

use std::cmp::Ordering;
use std::collections::HashMap;

use rusqlite::Connection;

use crate::result::{MultiTraversalResult, SuspiciousNode, VisitedNode};

/// drift 枝刈り器（DD-LGX-005 §4）。
pub struct DriftPruner;

impl DriftPruner {
    /// visited ノードのうち drift_score >= drift_threshold を抽出（DD-LGX-005 §2.1 / §4 / §6）。
    /// db = None / 照会失敗時は空 Vec を返す（ベストエフォート、stderr 警告）。
    /// 抽出結果は drift_score 降順・同値 id 昇順（stable sort）。
    pub fn prune(
        traversal: &MultiTraversalResult,
        db: Option<&Connection>,
        drift_threshold: f32,
    ) -> Vec<SuspiciousNode> {
        let conn = match db {
            Some(c) => c,
            None => return Vec::new(),
        };

        if traversal.visited.is_empty() {
            return Vec::new();
        }

        match query_drift(conn, &traversal.visited, drift_threshold) {
            Ok(mut rows) => {
                rows.sort_by(|a, b| {
                    b.drift_score
                        .partial_cmp(&a.drift_score)
                        .unwrap_or(Ordering::Equal)
                        .then_with(|| a.id.cmp(&b.id))
                });
                rows
            }
            Err(e) => {
                eprintln!("[nav] drift pruning skipped: {}", e);
                Vec::new()
            }
        }
    }
}

fn query_drift(
    conn: &Connection,
    visited: &[VisitedNode],
    threshold: f32,
) -> Result<Vec<SuspiciousNode>, rusqlite::Error> {
    let placeholders: Vec<String> = (1..=visited.len()).map(|i| format!("?{}", i)).collect();
    let sql = format!(
        "SELECT node_id, MAX(value) FROM scores \
         WHERE score_type = 'drift' AND node_id IN ({}) \
         GROUP BY node_id",
        placeholders.join(", ")
    );

    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map(
        rusqlite::params_from_iter(visited.iter().map(|v| v.id.as_str())),
        |row| {
            let node_id: String = row.get(0)?;
            let value: f64 = row.get(1)?;
            Ok((node_id, value as f32))
        },
    )?;

    let visited_map: HashMap<&str, &VisitedNode> =
        visited.iter().map(|v| (v.id.as_str(), v)).collect();

    let mut collected = Vec::new();
    for row in rows {
        let (node_id, value) = row?;
        if value >= threshold {
            let (type_code, path) = visited_map
                .get(node_id.as_str())
                .map(|v| (v.type_code.clone(), v.path.clone()))
                .unwrap_or_default();
            collected.push(SuspiciousNode {
                id: node_id,
                drift_score: value,
                type_code,
                path,
            });
        }
    }

    Ok(collected)
}
