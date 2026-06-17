// (module of SRC-LGX-002; anchor: compiler.rs)
// legixy-ctx::result — 返却型（DD-LGX-002 §2.1）
//
// TC[RED] scaffold。データ型は実体（pub フィールド付き、テストが構築可能）。
// 6 セクション対応（SPEC-LGX-003.REQ.10 v0.8.0）+ unresolved_targets（REQ.20）。

use std::path::PathBuf;

use legixy_graph::{EdgeKind, NodeId};

/// compile() の返却（6 セクション + 未解決起点。DD-LGX-002 §2.1）。
#[derive(Debug, Clone, PartialEq)]
pub struct ContextResult {
    pub targets: Vec<ResolvedTarget>,
    pub layer_guidelines: Vec<LayerDocument>,
    pub additional_guidelines: Vec<LayerDocument>,
    pub upstream: Vec<UpstreamArtifact>,
    pub custom_documents: Vec<CustomDocument>,
    pub target_metadata: Vec<TargetNodeMetadata>,
    pub granularity: crate::compiler::Granularity,
    /// REQ.20: 未解決起点の記録（PathBuf 辞書順昇順で決定論記録）。
    pub unresolved_targets: Vec<PathBuf>,
}

/// target_files の解決結果（DD-LGX-002 §2.1）。未解決時 artifact_id=None（REQ.20-1）。
#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedTarget {
    pub file_path: PathBuf,
    pub artifact_id: Option<NodeId>,
    pub type_code: Option<String>,
}

/// 上流連鎖の 1 アーティファクト（DD-LGX-002 §2.1 / DD-LGX-004 §2.1）。
#[derive(Debug, Clone, PartialEq)]
pub struct UpstreamArtifact {
    pub artifact_id: NodeId,
    pub type_code: String,
    pub file_path: PathBuf,
    pub chain_distance: usize,
    /// subnode 粒度時はセクション本文 or anchor のみ。ファイル不在時は空（REQ.20-2）。
    pub body: String,
    /// subnode 粒度時に Some。
    pub subnode_id: Option<NodeId>,
    /// サブノード見出しテキスト。
    pub anchor: Option<String>,
    pub drift_score: Option<f32>,
}

/// Layer / Additional Guidelines セクションの 1 文書（DD-LGX-002 §2.1）。
#[derive(Debug, Clone, PartialEq)]
pub struct LayerDocument {
    pub layer_name: String,
    pub node_id: NodeId,
    pub file_path: PathBuf,
    pub body: String,
    pub specificity: u32,
    pub priority: u32,
}

/// Custom Documents セクション（6 番目、ADR-LGX-019。DD-LGX-002 §2.1）。
#[derive(Debug, Clone, PartialEq)]
pub struct CustomDocument {
    pub from_id: NodeId,
    pub to_id: NodeId,
    pub file_path: PathBuf,
    pub body: String,
    pub reason: Option<String>,
}

/// Target Node Metadata セクション（DD-LGX-002 §2.1）。
/// `unresolved_targets` は v3 に無い legixy 新規（REQ.20 決定論記録）。
#[derive(Debug, Clone, PartialEq)]
pub struct TargetNodeMetadata {
    pub artifact_id: NodeId,
    pub outgoing_edges: Vec<(NodeId, EdgeKind)>,
    pub incoming_edges: Vec<(NodeId, EdgeKind)>,
    pub subnode_count: usize,
    /// REQ.20: 未解決起点の記録（Target Node Metadata セクション内、PathBuf 辞書順昇順）。
    pub unresolved_targets: Vec<PathBuf>,
}
