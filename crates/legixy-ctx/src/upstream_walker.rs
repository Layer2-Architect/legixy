// (module of SRC-LGX-002; anchor: compiler.rs)
// legixy-ctx::upstream_walker — UpstreamWalker（DD-LGX-002 §3 / DD-LGX-004 §3）
//
// TC[RED] scaffold。new は借用保持の実体、walk_chain_parent_only_with_depth は todo!()。
// REQ.02/08/17。Chain/ParentChild エッジのみ逆方向 BFS、visited で循環遮断（CTX-INV-4）。
// depth_limit=None で無制限、Some(N) で N 階層、Some(0) で空 Vec（REQ.17 v3 差分）。
// start がグラフ未登録なら空 Vec（REQ.20-1）。

use std::collections::{HashSet, VecDeque};
use std::path::PathBuf;

use legixy_graph::{EdgeKind, NodeId, TraceGraph};

use crate::error::ContextError;
use crate::result::UpstreamArtifact;

/// 上流連鎖の逆 BFS ウォーカ（DD-LGX-002 §3）。graph を借用（read-only、§5）。
pub struct UpstreamWalker<'a> {
    graph: &'a TraceGraph,
}

impl<'a> UpstreamWalker<'a> {
    /// graph 借用で構築。
    pub fn new(graph: &'a TraceGraph) -> Self {
        UpstreamWalker { graph }
    }

    /// DD-LGX-002 §3 / DD-LGX-004 §3 凍結シグネチャ。
    /// Chain/ParentChild のみ逆 BFS。depth_limit=None で無制限、Some(0) で空 Vec。
    /// start 未登録なら空 Vec。visited で循環遮断（CTX-INV-4）。
    /// 決定論: edges() の挿入順を使う（NFR-LGX-001.REL.05、v3 upstream_walker.rs 底本）。
    pub fn walk_chain_parent_only_with_depth(
        &self,
        start: &NodeId,
        depth_limit: Option<usize>,
    ) -> Result<Vec<UpstreamArtifact>, ContextError> {
        let mut result = Vec::new();
        if self.graph.node(start).is_none() {
            return Ok(result);
        }

        let mut visited: HashSet<NodeId> = HashSet::new();
        visited.insert(start.clone());
        let mut queue: VecDeque<(NodeId, usize)> = VecDeque::new();
        queue.push_back((start.clone(), 0));

        while let Some((current, depth)) = queue.pop_front() {
            // depth_limit 到達時は当該ノードの上流（incoming）を探索しない。
            if let Some(limit) = depth_limit {
                if depth >= limit {
                    continue;
                }
            }
            // incoming エッジ = to == current のエッジ。挿入順で決定論的に走査。
            for edge in self.graph.edges() {
                if edge.to != current {
                    continue;
                }
                if !matches!(edge.kind, EdgeKind::Chain | EdgeKind::ParentChild) {
                    continue;
                }
                let next = &edge.from;
                if !visited.insert(next.clone()) {
                    continue;
                }
                if let Some(node) = self.graph.node(next) {
                    result.push(UpstreamArtifact {
                        artifact_id: node.id.clone(),
                        type_code: node.type_code.clone(),
                        file_path: PathBuf::from(&node.path),
                        chain_distance: depth + 1,
                        body: String::new(),
                        subnode_id: None,
                        anchor: None,
                        drift_score: None,
                    });
                }
                queue.push_back((next.clone(), depth + 1));
            }
        }

        Ok(result)
    }
}
