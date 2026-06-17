// (module of SRC-LGX-005; anchor: investigate.rs)
// legixy-nav 共有走査結果型。DD-LGX-005 §2.1 を正典とする（VisitedNode / MultiTraversalResult /
// SuspiciousNode / PrunedTraversalResult / InvestigateOutcome）。DD-LGX-006 が追加する impact 固有型
// （TraversalResult / TruncationInfo）も本モジュールに同居させる（DD-LGX-006 §4 result.rs）。
//
// NodeId は String（ADR-LGX-021 §2.1）。VisitedNode.id / SuspiciousNode.id / start_ids 要素 /
// edges_traversed タプル要素はすべて String。
// データ型は実体（pub フィールド付き、テストが構築可能）。ロジックは持たない。

use std::collections::HashMap;

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

use legixy_graph::NodeId;

/// visited ノードの要約情報（出力用）。DD-LGX-005 §2.1 / DD-LGX-006 §2.1（v3 lx-nav result.rs と 1:1）。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VisitedNode {
    /// ノード識別子（legixy-graph NodeId = String と同一、ADR-LGX-021）。
    pub id: String,
    /// ノードタイプコード（例: "UC", "DD", "SRC"）。
    pub type_code: String,
    /// ノードファイルパス（graph.toml の path 値）。
    pub path: String,
    /// 起点からの BFS 距離（起点 = 0）。
    pub depth: usize,
}

/// 多起点走査の集約結果（DD-LGX-005 §2.1 正典 / DD-LGX-006 §2.1）。
/// serde 対応（--json 出力、SPEC-LGX-005.REQ.09）。
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct MultiTraversalResult {
    /// 走査された全ノード（visited）の要約情報。
    /// start_ids 入力順 × 各起点の BFS 順による決定論的順序（SPEC-LGX-005.REQ.03）。
    pub visited: Vec<VisitedNode>,
    /// 走査で使用したエッジ (from, to) の spanning tree 表現（グラフ向き）。
    /// v3 互換: 初訪問を生んだエッジのみ記録、既訪問ノードへのエッジは含まない。
    pub edges_traversed: Vec<(String, String)>,
    /// 各ノードの起点からの最短 BFS 距離。多起点では min(depth) を記録。
    /// IndexMap で挿入順（発見順）を保持（REQ.03 の決定論性）。
    pub depth_map: IndexMap<String, usize>,
    /// 入力 start_ids（CLI からは常に単起点、ライブラリ API は多起点対応）。
    pub start_ids: Vec<String>,
}

/// drift 閾値以上のサスペクトノード（DD-LGX-005 §2.1）。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SuspiciousNode {
    pub id: String,
    /// scores テーブルの MAX(value) where score_type='drift'。
    pub drift_score: f32,
    pub type_code: String,
    pub path: String,
}

/// drift 枝刈り済み調査結果（DD-LGX-005 §2.1）。
#[derive(Debug, Clone, PartialEq)]
pub struct PrunedTraversalResult {
    pub traversal: MultiTraversalResult,
    /// drift_threshold 以上のノード。drift_score 降順、同値は id 昇順（stable sort）。
    /// db = None または DB 照会失敗時は空（REQ.09 代替 3a、NFR REL.02）。
    pub suspicious_nodes: Vec<SuspiciousNode>,
    /// 閾値値（Reporter Text / JSON 出力に含まれる）。
    pub drift_threshold: f32,
}

/// 打ち切り発生時の stderr Info 付き調査結果（DD-LGX-005 §2.1、REQ.04 GAP-LGX-085 対応）。
/// truncated = true かつ excluded_count > 0 のとき呼び出し側が stderr へ Info を出力する。
#[derive(Debug, Clone, PartialEq)]
pub struct InvestigateOutcome {
    pub result: PrunedTraversalResult,
    /// max_depth 打ち切りが発生したか。
    pub truncated: bool,
    /// 打ち切りで除外された到達可能ノード数（近似: 境界ノードの未訪問隣接数）。
    pub excluded_count: usize,
}

/// 単起点 BFS の生結果（DD-LGX-006 §2.1。legixy-graph 層 traversal が返す内部型）。
#[derive(Debug, Clone, PartialEq)]
pub struct TraversalResult {
    /// BFS 発見順（起点含む）。
    pub visited: Vec<NodeId>,
    /// spanning tree エッジ (from, to) グラフ向き。
    pub edges_traversed: Vec<(NodeId, NodeId)>,
    /// 起点からの BFS 距離（起点=0）。
    pub depth_map: HashMap<NodeId, usize>,
}

/// 打ち切り情報（DD-LGX-006 §2.1、REQ.04 打ち切り可観測性、GAP-LGX-085 対応、v3 差分）。
/// --max-depth 指定かつ深度超過ノードが 1 件以上ある場合のみ生成される。
#[derive(Debug, Clone, PartialEq)]
pub struct TruncationInfo {
    /// 打ち切り発生フラグ（true = 除外ノードあり）。
    pub truncated: bool,
    /// --max-depth 境界ノードから出た未訪問隣接ノード数（近似値、算定法は DD-006 §6 参照）。
    pub excluded_count: usize,
    /// 使用した max_depth 値（stderr Info メッセージ生成用）。
    pub max_depth: usize,
}
