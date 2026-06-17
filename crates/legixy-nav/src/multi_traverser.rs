// (module of SRC-LGX-006; anchor: impact.rs)
// legixy-nav 多起点 BFS ラッパー（DD-LGX-006 §3 / §4。v3 lx-nav/src/multi_traverser.rs 整合）。
// traverse_forward_multi（順方向、impact 委譲先）/ traverse_reverse_multi（逆方向、investigate 委譲先）。
// start_ids 入力順 × 各起点 BFS 順マージ。不在起点は読み飛ばし。多起点では min(depth) 記録。
// BFS 本体は本 crate 内 single_traverse（v3 lx-graph::traversal 整合、共有 crate 非改変のためローカル化）。
// 走査ロジックを SRC[GREEN] で実装。

use std::collections::{HashSet, VecDeque};

use legixy_graph::{EdgeKind, TraceGraph};

use crate::result::{MultiTraversalResult, VisitedNode};

/// 多起点 BFS ラッパー（DD-LGX-006 §3 / §4）。
pub struct MultiTraverser;

impl MultiTraverser {
    /// 順方向多起点走査（DD-LGX-006 §3。`impact` が委譲する内部ラッパー）。
    /// start_ids 入力順 × BFS 順マージ。不在起点は読み飛ばし。
    pub fn traverse_forward_multi(
        graph: &TraceGraph,
        start_ids: &[String],
        max_depth: Option<usize>,
    ) -> MultiTraversalResult {
        multi_traverse(graph, start_ids, max_depth, Direction::Forward)
    }

    /// 逆方向多起点走査（DD-LGX-005 §4。`investigate` が委譲する内部ラッパー）。
    /// to→from を辿る上流方向 BFS（SPEC-LGX-005.REQ.02）。
    pub fn traverse_reverse_multi(
        graph: &TraceGraph,
        start_ids: &[String],
        max_depth: Option<usize>,
    ) -> MultiTraversalResult {
        let mut result = multi_traverse(graph, start_ids, max_depth, Direction::Reverse);
        augment_with_linked_subnodes(graph, &mut result);
        result
    }
}

/// 逆方向走査（investigate）に「細粒度トレース」のサブノードを補う（R-2）。
/// 既訪問ノード v が chain/custom エッジで subnode（id に `#`）へ結線する場合、その subnode を
/// 上流の細粒度終点として visited に加える（ctx の `--granularity subnode` 解像度と整合）。
/// fixture 例: `SRC-SN-001 → DD-SN-001#s:state-machine` を investigate SRC-SN-001 が解像する。
/// chain/custom 限定（ParentChild は materialize エッジのため対象外）。subnode 結線が無いグラフでは no-op。
fn augment_with_linked_subnodes(graph: &TraceGraph, result: &mut MultiTraversalResult) {
    let visited_ids: HashSet<String> = result.visited.iter().map(|v| v.id.clone()).collect();
    // (subnode_id, depth, edge) を決定論順（visited 順 × edges 挿入順）で収集。
    let mut additions: Vec<(String, usize)> = Vec::new();
    let mut added: HashSet<String> = HashSet::new();
    for v in &result.visited {
        for edge in graph.edges() {
            if !matches!(edge.kind, EdgeKind::Chain | EdgeKind::Custom) {
                continue;
            }
            if edge.from != v.id || !edge.to.contains('#') {
                continue;
            }
            if visited_ids.contains(&edge.to) || !added.insert(edge.to.clone()) {
                continue;
            }
            additions.push((edge.to.clone(), v.depth + 1));
        }
    }
    for (id, depth) in additions {
        if let Some(node) = graph.node(&id) {
            result.visited.push(VisitedNode {
                id: id.clone(),
                type_code: node.type_code.clone(),
                path: node.path.clone(),
                depth,
            });
            result.depth_map.insert(id, depth);
        }
    }
}

#[derive(Copy, Clone)]
enum Direction {
    Forward,
    Reverse,
}

/// 単起点 BFS の生結果（v3 lx-graph::traversal::TraversalResult 整合、ローカル）。
struct SingleResult {
    visited: Vec<String>,
    edges_traversed: Vec<(String, String)>,
    depth_map: Vec<(String, usize)>,
}

/// 単起点 BFS（v3 lx-graph::traversal::traverse 整合）。
/// 隣接エッジ走査順 = graph.edges() の挿入順（v3 adjacency_fwd/rev のビルド順と一致）。
fn single_traverse(
    graph: &TraceGraph,
    start: &str,
    max_depth: Option<usize>,
    dir: Direction,
) -> SingleResult {
    let mut result = SingleResult {
        visited: Vec::new(),
        edges_traversed: Vec::new(),
        depth_map: Vec::new(),
    };
    if graph.node(start).is_none() {
        return result;
    }

    let mut visited: HashSet<String> = HashSet::new();
    let mut queue: VecDeque<(String, usize)> = VecDeque::new();
    queue.push_back((start.to_string(), 0));
    visited.insert(start.to_string());
    result.visited.push(start.to_string());
    result.depth_map.push((start.to_string(), 0));

    while let Some((current, depth)) = queue.pop_front() {
        if let Some(limit) = max_depth {
            if depth >= limit {
                continue;
            }
        }

        for edge in graph.edges() {
            // 隣接判定: 順方向は from==current で to へ、逆方向は to==current で from へ。
            let next = match dir {
                Direction::Forward => {
                    if edge.from != current {
                        continue;
                    }
                    &edge.to
                }
                Direction::Reverse => {
                    if edge.to != current {
                        continue;
                    }
                    &edge.from
                }
            };
            // pair はどちら向きでもグラフ向き (from,to) を記録（v3 整合）。
            let pair = (edge.from.clone(), edge.to.clone());
            if visited.insert(next.clone()) {
                result.visited.push(next.clone());
                result.depth_map.push((next.clone(), depth + 1));
                result.edges_traversed.push(pair);
                queue.push_back((next.clone(), depth + 1));
            }
        }
    }

    result
}

fn multi_traverse(
    graph: &TraceGraph,
    start_ids: &[String],
    max_depth: Option<usize>,
    dir: Direction,
) -> MultiTraversalResult {
    let mut result = MultiTraversalResult {
        visited: Vec::new(),
        edges_traversed: Vec::new(),
        depth_map: Default::default(),
        start_ids: start_ids.to_vec(),
    };
    let mut seen: HashSet<String> = HashSet::new();

    for start in start_ids {
        if graph.node(start).is_none() {
            continue;
        }
        let sub = single_traverse(graph, start, max_depth, dir);

        for (id, sub_depth) in &sub.depth_map {
            let sub_depth = *sub_depth;
            if seen.insert(id.clone()) {
                let node = graph
                    .node(id)
                    .expect("BFS visited IDs are guaranteed to exist in graph");
                result.visited.push(VisitedNode {
                    id: id.clone(),
                    type_code: node.type_code.clone(),
                    path: node.path.clone(),
                    depth: sub_depth,
                });
                result.depth_map.insert(id.clone(), sub_depth);
            } else if let Some(&existing) = result.depth_map.get(id) {
                if sub_depth < existing {
                    result.depth_map.insert(id.clone(), sub_depth);
                }
            }
        }

        for pair in sub.edges_traversed {
            result.edges_traversed.push(pair);
        }
    }

    result
}
