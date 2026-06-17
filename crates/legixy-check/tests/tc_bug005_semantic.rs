// BUG-005 回帰テスト: check 意味層（SemanticChecker、SPEC-LGX-004.REQ.02）。
// store を渡した Full 検証が SemanticSimilarity（低類似度エッジ Warning）/ LinkCandidate（Info）を
// 生成することを、合成ベクトルで決定論的に検証する。pre-fix（store 無視）では空 → RED。
//
// 親: SPEC-LGX-004.REQ.02 / DD-LGX-001。store は legixy-embed の stub（in-memory）。

use legixy_check::{run, CheckCategory, CheckMode, Severity};
use legixy_core::Config;
use legixy_embed::{EmbeddingRow, EmbeddingStore};
use legixy_graph::{Edge, EdgeKind, Node, TraceGraph};

fn node(id: &str, type_code: &str) -> Node {
    Node {
        id: id.to_string(),
        type_code: type_code.to_string(),
        path: format!("docs/{id}.md"),
        parent_id: None,
        anchor: None,
    }
}

fn row(node_id: &str, embedding: Vec<f32>) -> EmbeddingRow {
    let dim = embedding.len();
    EmbeddingRow {
        node_id: node_id.to_string(),
        embedding,
        dim,
        model_version: "test-model".to_string(),
        content_hash: "h".to_string(),
        context: None,
        context_hash: None,
        created_at: "2026-06-14T00:00:00Z".to_string(),
    }
}

#[test]
fn low_similarity_chain_edge_emits_semantic_similarity_warning() {
    // A→B（chain）。直交ベクトル（cosine=0 < 0.4）→ SemanticSimilarity Warning。
    let graph = TraceGraph::from_parts(
        vec![node("UC-LGX-001", "UC"), node("RBA-LGX-001", "RBA")],
        vec![Edge {
            from: "UC-LGX-001".to_string(),
            to: "RBA-LGX-001".to_string(),
            kind: EdgeKind::Chain,
        }],
    );
    let store = EmbeddingStore::stub(
        vec![
            row("UC-LGX-001", vec![1.0, 0.0]),
            row("RBA-LGX-001", vec![0.0, 1.0]),
        ],
        vec![],
    );
    let report = run(&graph, &Config::default(), CheckMode::Full, Some(&store)).expect("Ok");
    let warns: Vec<_> = report
        .findings
        .iter()
        .filter(|f| {
            f.category == CheckCategory::SemanticSimilarity && f.severity == Severity::Warning
        })
        .collect();
    assert_eq!(warns.len(), 1, "低類似度エッジ 1 件が Warning: {report:?}");
    assert!(
        warns[0].message.contains("低類似度エッジ"),
        "メッセージ: {}",
        warns[0].message
    );
    // 「embeddings 未生成」Info は出ない（store が渡っている）。
    assert!(
        !report
            .findings
            .iter()
            .any(|f| f.message.contains("embeddings 未生成")),
        "store 配線済 → 未生成 Info は出ない"
    );
}

#[test]
fn high_similarity_edge_no_warning() {
    // A→B（chain）。同一ベクトル（cosine=1.0 ≥ 0.4）→ Warning なし。
    let graph = TraceGraph::from_parts(
        vec![node("UC-LGX-001", "UC"), node("RBA-LGX-001", "RBA")],
        vec![Edge {
            from: "UC-LGX-001".to_string(),
            to: "RBA-LGX-001".to_string(),
            kind: EdgeKind::Chain,
        }],
    );
    let store = EmbeddingStore::stub(
        vec![
            row("UC-LGX-001", vec![1.0, 0.0]),
            row("RBA-LGX-001", vec![1.0, 0.0]),
        ],
        vec![],
    );
    let report = run(&graph, &Config::default(), CheckMode::Full, Some(&store)).expect("Ok");
    assert!(
        !report
            .findings
            .iter()
            .any(|f| f.category == CheckCategory::SemanticSimilarity
                && f.severity == Severity::Warning),
        "高類似度 → Warning なし: {report:?}"
    );
}

#[test]
fn unlinked_similar_pair_emits_link_candidate_info() {
    // エッジ無しの 2 ノードが高類似（cosine=1.0 ≥ 0.7）→ LinkCandidate Info。
    let graph = TraceGraph::from_parts(
        vec![node("UC-LGX-001", "UC"), node("UC-LGX-002", "UC")],
        vec![], // エッジ無し
    );
    let store = EmbeddingStore::stub(
        vec![
            row("UC-LGX-001", vec![1.0, 0.0]),
            row("UC-LGX-002", vec![1.0, 0.0]),
        ],
        vec![],
    );
    let report = run(&graph, &Config::default(), CheckMode::Full, Some(&store)).expect("Ok");
    let infos: Vec<_> = report
        .findings
        .iter()
        .filter(|f| {
            f.category == CheckCategory::SemanticSimilarity
                && f.severity == Severity::Info
                && f.message.contains("リンク候補")
        })
        .collect();
    assert_eq!(infos.len(), 1, "リンク候補 1 件が Info: {report:?}");
}
