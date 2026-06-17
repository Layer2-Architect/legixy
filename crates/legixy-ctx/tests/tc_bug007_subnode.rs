// BUG-007 回帰テスト: context のサブノード粒度（SPEC-LGX-003、LGX-EXT-001 目的1=トークン削減）。
// `--granularity subnode` が各サブノードの content_range スライス（区画本文）を返すこと、
// すなわち document 全文と異なり**トークン削減**されることを検証する。
// pre-fix では各サブノード body に親全文を入れていた（削減なし）→ RED。
//
// 親: SPEC-LGX-003.REQ.11 / DD-LGX-002。

use std::path::PathBuf;

use legixy_ctx::{CompileInput, ContextCompiler, Granularity, TraceConfig};
use legixy_graph::subnode::extract_subnodes_with_levels;
use legixy_graph::{Edge, EdgeKind, Node, TraceGraph};

const PARENT: &str = "UC-LGX-001";
const DOC: &str = "# タイトル\n\n## セクションA\n\nアルファ本文 alpha-unique-AAA。\n\n## セクションB\n\nベータ本文 beta-unique-BBB。\n";

fn node(id: &str, type_code: &str, path: &str) -> Node {
    Node {
        id: id.to_string(),
        type_code: type_code.to_string(),
        path: path.to_string(),
        parent_id: None,
        anchor: None,
    }
}

/// 上流 UC（サブノードあり）→ 下流 DD のチェーン + UC の auto サブノード（ParentChild）。
/// subnode 粒度は**上流**成果物のサブノードに適用されるため、起点は下流 DD にする。
fn graph_with_subnodes() -> (TraceGraph, Vec<String>) {
    let subs = extract_subnodes_with_levels(PARENT, DOC, &[2, 3]);
    assert_eq!(subs.len(), 2, "h2 を 2 件抽出");
    let mut nodes = vec![node(PARENT, "UC", "uc.md"), node("DD-LGX-001", "DD", "dd.md")];
    let mut edges = vec![Edge {
        from: PARENT.to_string(), // 上流 UC → 下流 DD（chain）
        to: "DD-LGX-001".to_string(),
        kind: EdgeKind::Chain,
    }];
    let mut sub_ids = Vec::new();
    for s in &subs {
        nodes.push(node(&s.id, "UC", "uc.md"));
        edges.push(Edge {
            from: PARENT.to_string(),
            to: s.id.clone(),
            kind: EdgeKind::ParentChild,
        });
        sub_ids.push(s.id.clone());
    }
    (TraceGraph::from_parts(nodes, edges), sub_ids)
}

#[test]
fn subnode_granularity_slices_sections_not_full_document() {
    let tmp = tempfile::tempdir().unwrap();
    std::fs::write(tmp.path().join("uc.md"), DOC).unwrap();
    std::fs::write(tmp.path().join("dd.md"), "# DD\n").unwrap();
    let (graph, _sub_ids) = graph_with_subnodes();
    let config = TraceConfig::default();
    let compiler = ContextCompiler::new(&graph, &config, None, tmp.path());

    // 起点に親を与え、subnode 粒度で compile。
    let input = CompileInput {
        target_files: vec![PathBuf::from("dd.md")],
        granularity: Granularity::Subnode,
        ..Default::default()
    };
    let result = compiler.compile(&input).expect("compile ok");

    // サブノード粒度では各 upstream が「区画本文のみ」を持つ（全文でない＝削減）。
    let subnode_arts: Vec<_> = result
        .upstream
        .iter()
        .filter(|a| a.subnode_id.is_some())
        .collect();
    assert_eq!(subnode_arts.len(), 2, "2 サブノードが upstream に: {result:?}");

    for a in &subnode_arts {
        // 各区画は片方のユニーク本文のみを含み、両方は含まない（＝全文でない）。
        let has_a = a.body.contains("alpha-unique-AAA");
        let has_b = a.body.contains("beta-unique-BBB");
        assert!(
            has_a ^ has_b,
            "区画本文は片方のみ（全文なら両方含む＝削減なし）: body={:?}",
            a.body
        );
    }
}

#[test]
fn subnode_sections_filter_selects_one() {
    let tmp = tempfile::tempdir().unwrap();
    std::fs::write(tmp.path().join("uc.md"), DOC).unwrap();
    std::fs::write(tmp.path().join("dd.md"), "# DD\n").unwrap();
    let (graph, sub_ids) = graph_with_subnodes();
    let config = TraceConfig::default();
    let compiler = ContextCompiler::new(&graph, &config, None, tmp.path());

    // 最初のサブノード ID のみを --sections で指定 → 1 件のみ通る。
    let input = CompileInput {
        target_files: vec![PathBuf::from("dd.md")],
        granularity: Granularity::Subnode,
        sections: Some(vec![sub_ids[0].clone()]),
        ..Default::default()
    };
    let result = compiler.compile(&input).expect("compile ok");
    let subnode_arts: Vec<_> = result
        .upstream
        .iter()
        .filter(|a| a.subnode_id.is_some())
        .collect();
    assert_eq!(subnode_arts.len(), 1, "sections フィルタで 1 件: {result:?}");
    assert_eq!(
        subnode_arts[0].subnode_id.as_deref(),
        Some(sub_ids[0].as_str())
    );
}
