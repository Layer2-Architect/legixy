// Document ID: SRC-LGX-006
// legixy-nav 順方向探索（impact）公開 API（DD-LGX-006 §3、凍結 HR7）。
// impact: 順方向 BFS 走査。MultiTraverser::traverse_forward_multi へ委譲する薄いラッパー。
//
// 同一入力 → 同一 visited 順・depth_map（REQ.03 決定論性）。read-only（graph を変更しない）。
// start_ids 不在 ID は読み飛ばし（REQ.05）。空 start_ids は空結果。
// 起点不在 = 空結果・exit 0（GAP-234/ADR-019 確定、NavError に昇格しない）。
// 統括ロジックは todo!() として TC[RED] を失敗させ、SRC[GREEN] で実装する。

use legixy_graph::TraceGraph;

use crate::error::NavError;
use crate::result::MultiTraversalResult;

/// 順方向探索（DD-LGX-006 §3、凍結）。
/// `start_ids` に不在 ID があれば読み飛ばし（REQ.05）。空 `start_ids` は空結果。
/// engine.db 非依存（TP-016 R1）。同期・単一スレッド・read-only。
pub fn impact(
    graph: &TraceGraph,
    start_ids: &[String],
    max_depth: Option<usize>,
) -> Result<MultiTraversalResult, NavError> {
    Ok(crate::multi_traverser::MultiTraverser::traverse_forward_multi(
        graph, start_ids, max_depth,
    ))
}
