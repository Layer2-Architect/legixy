// Document ID: SRC-LGX-005
// legixy-nav 逆方向探索（investigate）公開 API（DD-LGX-005 §3、凍結 HR7）。
// investigate（後方互換、max_depth=None 相当）/ investigate_with_depth（max_depth 機能化、
// SUPP-005 §2.4 [要決定]A 採用、GAP-LGX-085 truncated/excluded_count）。
//
// 逆方向 BFS（MultiTraverser::traverse_reverse_multi）→ DriftPruner::prune → PrunedTraversalResult。
// 起点不在 = 空 visited・exit 0（D 裁定、DD-005 §3 不変条件 / §2.3 / REQ.05）。Err ではない。
// read-only（graph/db を変更しない）。同期・単一スレッド。
// 走査・枝刈り統括ロジックは todo!() として TC[RED] を失敗させ、SRC[GREEN] で実装する。

use std::collections::HashSet;

use legixy_graph::TraceGraph;

use crate::drift_pruner::DriftPruner;
use crate::error::NavError;
use crate::multi_traverser::MultiTraverser;
use crate::result::{InvestigateOutcome, PrunedTraversalResult};

/// 逆方向探索（DD-LGX-005 §3、凍結）。max_depth = None 相当（無制限）。
/// 同一入力 → 同一 PrunedTraversalResult（visited 順・suspicious 順含む、REQ.03）。read-only。
/// 起点不在は Ok（空結果、exit 0）。`investigate_with_depth(.., None)` と同一結果。
pub fn investigate(
    graph: &TraceGraph,
    start_ids: &[String],
    db: Option<&rusqlite::Connection>,
    drift_threshold: f32,
) -> Result<PrunedTraversalResult, NavError> {
    let traversal = MultiTraverser::traverse_reverse_multi(graph, start_ids, None);
    let suspicious_nodes = DriftPruner::prune(&traversal, db, drift_threshold);
    Ok(PrunedTraversalResult {
        traversal,
        suspicious_nodes,
        drift_threshold,
    })
}

/// max_depth 機能化版（DD-LGX-005 §3、凍結。REQ.04 GAP-LGX-085）。
/// max_depth=None のとき investigate と同一結果（truncated=false、excluded_count=0）。
/// max_depth 指定時は打ち切り発生を InvestigateOutcome.truncated で表現。exit 不変（打ち切りでも 0）。
pub fn investigate_with_depth(
    graph: &TraceGraph,
    start_ids: &[String],
    db: Option<&rusqlite::Connection>,
    drift_threshold: f32,
    max_depth: Option<usize>,
) -> Result<InvestigateOutcome, NavError> {
    let traversal = MultiTraverser::traverse_reverse_multi(graph, start_ids, max_depth);
    let suspicious_nodes = DriftPruner::prune(&traversal, db, drift_threshold);

    // 打ち切り可観測性（GAP-LGX-085）。max_depth 指定かつ境界深度ノードに未訪問逆隣接があるとき truncated。
    // excluded_count = 境界深度ノード（depth_map[id] == max_depth）の未訪問逆隣接数の合計。
    let excluded_count = match max_depth {
        Some(limit) => reverse_excluded_count(graph, &traversal, limit),
        None => 0,
    };
    let truncated = excluded_count > 0;

    let result = PrunedTraversalResult {
        traversal,
        suspicious_nodes,
        drift_threshold,
    };
    Ok(InvestigateOutcome {
        result,
        truncated,
        excluded_count,
    })
}

/// 逆方向走査の打ち切り除外数（DD-LGX-005 §6 / GAP-LGX-085）。
/// 境界深度ノード（depth == max_depth）の、visited に含まれない逆隣接（to==node の edge.from）数を合計。
fn reverse_excluded_count(
    graph: &TraceGraph,
    traversal: &crate::result::MultiTraversalResult,
    max_depth: usize,
) -> usize {
    let visited: HashSet<&str> = traversal.visited.iter().map(|v| v.id.as_str()).collect();
    let mut excluded = 0usize;
    for v in &traversal.visited {
        if v.depth != max_depth {
            continue;
        }
        for edge in graph.edges() {
            if edge.to == v.id && !visited.contains(edge.from.as_str()) {
                excluded += 1;
            }
        }
    }
    excluded
}
