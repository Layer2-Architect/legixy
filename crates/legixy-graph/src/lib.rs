// legixy-graph: 有向グラフ・ノード・エッジ・サブノード（ADR-LGX-020）
// TC[RED] scaffold。データ型は実体を持つ（テストが入力を構築できるよう）。
// 走査・パース等のロジックは SRC[GREEN] で実装する。

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

// UC-LGX-003 サブノード自動抽出（DD-LGX-003 / SRC-LGX-003）。既存型は変更せず module を追加するのみ。
pub mod subnode;

/// グラフのノード識別子（ADR-LGX-021 §2.1、v3 lx-graph::model 準拠の型エイリアス）。
pub type NodeId = String;

/// エッジ種別（SPEC-LGX-002.REQ.04）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EdgeKind {
    Chain,
    Custom,
    ParentChild,
}

/// ノード。
/// `anchor` は明示サブノード（`#s:` ）が graph.toml で指定する見出しアンカー（例 `## 状態遷移`）、
/// または自動生成サブノードの生見出しテキスト。ドキュメントノードは None（LGX-EXT-001 §4.1、
/// 加算的フィールド・ADR-LGX-020 §HR7 想定済み。serde default で旧データ後方互換）。
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Node {
    pub id: NodeId,
    pub type_code: String,
    pub path: String,
    pub parent_id: Option<NodeId>,
    #[serde(default)]
    pub anchor: Option<String>,
}

/// エッジ。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Edge {
    pub from: NodeId,
    pub to: NodeId,
    pub kind: EdgeKind,
}

/// 有向グラフ（ノード・エッジを保持。挿入順 IndexMap で決定論）。
#[derive(Debug, Clone, Default)]
pub struct TraceGraph {
    nodes: IndexMap<NodeId, Node>,
    edges: Vec<Edge>,
}

impl TraceGraph {
    /// 空グラフ。
    pub fn empty() -> Self {
        TraceGraph {
            nodes: IndexMap::new(),
            edges: Vec::new(),
        }
    }

    /// ノード列・エッジ列から構築（テスト・パーサ共用の入口）。
    pub fn from_parts(nodes: Vec<Node>, edges: Vec<Edge>) -> Self {
        let mut map = IndexMap::new();
        for n in nodes {
            map.insert(n.id.clone(), n);
        }
        TraceGraph { nodes: map, edges }
    }

    pub fn nodes(&self) -> impl Iterator<Item = &Node> {
        self.nodes.values()
    }

    pub fn edges(&self) -> &[Edge] {
        &self.edges
    }

    pub fn node(&self, id: &str) -> Option<&Node> {
        self.nodes.get(id)
    }

    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// ドキュメントノード（id に `#` を含まない = サブノードでないノード）を列挙する
    /// （ADR-LGX-023、refresh-subnodes が親ドキュメント走査に使用。加算的・非破壊）。
    pub fn document_nodes(&self) -> impl Iterator<Item = &Node> {
        self.nodes.values().filter(|n| !n.id.contains('#'))
    }
}

/// グラフ読み込み・パース失敗（実行時失敗 = exit 1。DD-LGX-001 §2.3 `CheckError::GraphLoad`）。
#[derive(Debug, thiserror::Error)]
pub enum GraphError {
    #[error("graph parse error: {0}")]
    Parse(String),
    #[error("graph io error: {0}")]
    Io(String),
}

// --- graph.toml ローダ（DD-LGX-001 §2.3/§3。SRC[GREEN] = CLI 統合層） -------------
// `docs/traceability/graph.toml` の `[[nodes]]`（id/type/path/parent_id）と
// `[[edges]]`（from/to/kind = "chain"|"custom"|"parent_child"）を TraceGraph へ読み込む。
// パース失敗・不正 kind は GraphError::Parse、IO 失敗は GraphError::Io（exit 1、REQ.04）。

#[derive(Deserialize)]
struct GraphFile {
    #[serde(default)]
    nodes: Vec<NodeRow>,
    #[serde(default)]
    edges: Vec<EdgeRow>,
}

#[derive(Deserialize)]
struct NodeRow {
    id: String,
    #[serde(rename = "type")]
    type_code: String,
    path: String,
    #[serde(default)]
    parent_id: Option<String>,
}

#[derive(Deserialize)]
struct EdgeRow {
    from: String,
    to: String,
    kind: String,
}

/// graph.toml 文字列を TraceGraph へパースする（純関数、決定論）。
pub fn load_graph_from_str(text: &str) -> Result<TraceGraph, GraphError> {
    let gf: GraphFile = toml::from_str(text).map_err(|e| GraphError::Parse(e.to_string()))?;
    let nodes: Vec<Node> = gf
        .nodes
        .into_iter()
        .map(|n| Node {
            id: n.id,
            type_code: n.type_code,
            path: n.path,
            parent_id: n.parent_id,
            anchor: None,
        })
        .collect();
    let mut edges: Vec<Edge> = Vec::with_capacity(gf.edges.len());
    for e in gf.edges {
        let kind = match e.kind.as_str() {
            "chain" => EdgeKind::Chain,
            "custom" => EdgeKind::Custom,
            "parent_child" => EdgeKind::ParentChild,
            other => {
                return Err(GraphError::Parse(format!(
                    "unknown edge kind '{other}' (from {} to {})",
                    e.from, e.to
                )))
            }
        };
        edges.push(Edge {
            from: e.from,
            to: e.to,
            kind,
        });
    }
    Ok(TraceGraph::from_parts(nodes, edges))
}

/// graph.toml ファイルを読み込み TraceGraph を構築する（DD-LGX-001 §3、CLI 統合層が使用）。
pub fn load_graph(path: &std::path::Path) -> Result<TraceGraph, GraphError> {
    let text = std::fs::read_to_string(path).map_err(|e| GraphError::Io(e.to_string()))?;
    load_graph_from_str(&text)
}

#[cfg(test)]
mod loader_tests {
    use super::*;

    #[test]
    fn loads_nodes_and_edges_with_kind_mapping() {
        let toml = r#"
            [[nodes]]
            id = "UC-LGX-001"
            type = "UC"
            path = "docs/use-cases/UC-LGX-001.md"

            [[nodes]]
            id = "RBA-LGX-001"
            type = "RBA"
            path = "docs/robustness-abstract/RBA-LGX-001.md"

            [[edges]]
            from = "UC-LGX-001"
            to = "RBA-LGX-001"
            kind = "chain"
        "#;
        let g = load_graph_from_str(toml).expect("parse ok");
        assert_eq!(g.node_count(), 2);
        assert_eq!(g.edges().len(), 1);
        assert_eq!(g.edges()[0].kind, EdgeKind::Chain);
        assert_eq!(g.node("UC-LGX-001").unwrap().type_code, "UC");
    }

    #[test]
    fn parent_id_optional_and_empty_graph_ok() {
        let g = load_graph_from_str("").expect("empty ok");
        assert_eq!(g.node_count(), 0);
        let toml = r#"
            [[nodes]]
            id = "DD-LGX-002#abc"
            type = "DD"
            path = "docs/detailed-design/DD-LGX-002.md"
            parent_id = "DD-LGX-002"
        "#;
        let g = load_graph_from_str(toml).expect("ok");
        assert_eq!(
            g.node("DD-LGX-002#abc").unwrap().parent_id.as_deref(),
            Some("DD-LGX-002")
        );
    }

    #[test]
    fn unknown_edge_kind_is_parse_error() {
        let toml = r#"
            [[edges]]
            from = "A"
            to = "B"
            kind = "bogus"
        "#;
        let err = load_graph_from_str(toml).unwrap_err();
        assert!(matches!(err, GraphError::Parse(_)));
    }

    #[test]
    fn malformed_toml_is_parse_error() {
        let err = load_graph_from_str("this is not toml = = =").unwrap_err();
        assert!(matches!(err, GraphError::Parse(_)));
    }
}
