// Document ID: TC-LGX-004
// TC-LGX-004: 粒度制御付きコンテキスト解決のテストコード（TC[RED]）
//
// 親 chain: TS-LGX-004 → 本 TC-LGX-004 → SRC-LGX-004。
// 各テストは TS-LGX-004 のケースを `legixy-ctx` の凍結 API（DD-LGX-004 §3）に束縛する。
// 純データ関数（Granularity / CompileInput::default / upstream_sort_rule / RENDER_SORT_STRATEGY）は
// 実体のため PASS しうるが、ロジック関数（build_outline / enforce_size_limit / walk / compile /
// render）は todo!() で panic → RED。`cargo test -p legixy-ctx --no-run` は通る。
//
// 委譲（本 Rust crate 対象外。コメントのみ）:
//  - ケース 11(b) / 16 の CLI 経由 exit 0・stderr Info 文言: legixy-cli 層（DD-LGX-004 §11 S2-23）。
//  - granularity 文字列→enum reject（clap/zod）: legixy-cli / TS-LGX-009 へ委譲。
//  - drift_score 数値妥当性: TS-LGX-007 / ScoreLookup へ委譲。

use std::path::{Path, PathBuf};

use legixy_ctx::{
    build_outline, CompileInput, ContextCompiler, ContextError, ContextResult, Granularity,
    SectionFormatter, TargetNodeMetadata, TraceConfig, UpstreamArtifact, UpstreamWalker,
    RESULT_SIZE_LIMIT_CHARS,
};
use legixy_graph::{Edge, EdgeKind, Node, NodeId, TraceGraph};

fn node(id: &str, type_code: &str, path: &str) -> Node {
    Node {
        id: id.to_string(),
        type_code: type_code.to_string(),
        path: path.to_string(),
        parent_id: None,
        anchor: None,
    }
}

fn chain_edge(from: &str, to: &str) -> Edge {
    Edge {
        from: from.to_string(),
        to: to.to_string(),
        kind: EdgeKind::Chain,
    }
}

fn seven_layer_graph() -> (TraceGraph, NodeId) {
    let ids = [
        "SPEC-LGX-003",
        "UC-LGX-004",
        "RBA-LGX-004",
        "SEQA-LGX-004",
        "DD-LGX-004",
        "TS-LGX-004",
        "TC-LGX-004",
        "SRC-LGX-004",
    ];
    let nodes: Vec<Node> = ids
        .iter()
        .map(|id| node(id, id.split('-').next().unwrap(), &format!("{id}.md")))
        .collect();
    let edges: Vec<Edge> = ids.windows(2).map(|w| chain_edge(w[0], w[1])).collect();
    (
        TraceGraph::from_parts(nodes, edges),
        "SRC-LGX-004".to_string(),
    )
}

fn empty_config() -> TraceConfig {
    TraceConfig::default()
}

fn empty_result(granularity: Granularity) -> ContextResult {
    ContextResult {
        targets: Vec::new(),
        layer_guidelines: Vec::new(),
        additional_guidelines: Vec::new(),
        upstream: Vec::new(),
        custom_documents: Vec::new(),
        target_metadata: vec![TargetNodeMetadata {
            artifact_id: "SRC-LGX-004".into(),
            outgoing_edges: Vec::new(),
            incoming_edges: Vec::new(),
            subnode_count: 0,
            unresolved_targets: Vec::new(),
        }],
        granularity,
        unresolved_targets: Vec::new(),
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// ケース 1: Granularity 既定 Document・2 値のみ・as_str 一致
#[test]
fn test_granularity_default_and_as_str() {
    // @ts: TS-LGX-004 ケース 1
    assert_eq!(Granularity::default(), Granularity::Document);
    assert_eq!(Granularity::Document.as_str(), "document");
    assert_eq!(Granularity::Subnode.as_str(), "subnode");
    assert_eq!(CompileInput::default().granularity, Granularity::Document);
}

// ケース 2: CompileInput::default の粒度制御フィールド（v0.3.0 後方互換）
#[test]
fn test_compile_input_default_fields() {
    // @ts: TS-LGX-004 ケース 2
    let d = CompileInput::default();
    assert!(!d.outline_only);
    assert_eq!(d.sections, None);
    assert_eq!(d.depth_limit, None);
    assert_eq!(d.granularity, Granularity::Document);
    assert_eq!(d.command, None);
    assert!(d.target_files.is_empty());
}

// ケース 3: upstream_sort_rule の granularity 分岐（A-1 裁定: 出現順）
#[test]
fn test_upstream_sort_rule_a1_appearance_order() {
    // @ts: TS-LGX-004 ケース 3
    assert_eq!(
        SectionFormatter::upstream_sort_rule(Granularity::Document),
        "artifact_id-asc"
    );
    assert_eq!(
        SectionFormatter::upstream_sort_rule(Granularity::Subnode),
        "parent_id-asc,anchor-appearance-order",
        "A-1 裁定: アンカー出現順（v3 バイト辞書順 anchor-bytes-asc ではない）"
    );
    assert_ne!(
        SectionFormatter::upstream_sort_rule(Granularity::Subnode),
        "parent_id-asc,anchor-bytes-asc",
        "回帰固定: バイト辞書順を返してはならない"
    );
    assert_eq!(SectionFormatter::RENDER_SORT_STRATEGY, "index-array");
}

// ─────────────────────────────────────────────────────────────────────────────
// ケース 4: build_outline h1〜h3 抽出・レベル別インデント
#[test]
fn test_build_outline_h1_h3_indent() {
    // @ts: TS-LGX-004 ケース 4
    let out = build_outline("# H1\n本文a\n## H2\n本文b\n### H3\n");
    assert_eq!(out, "- H1\n  - H2\n    - H3\n");
}

// ケース 5: build_outline h4+・スペース無し・空タイトル除外
#[test]
fn test_build_outline_excludes_h4_nospace_empty() {
    // @ts: TS-LGX-004 ケース 5
    let out = build_outline("#### H4\n#abc\n#\n#   \n## valid\n");
    assert_eq!(out, "  - valid\n");
}

// ケース 6: build_outline 見出し皆無時の空文字列（GAP-047 / S2-25）
#[test]
fn test_build_outline_no_heading_empty() {
    // @ts: TS-LGX-004 ケース 6
    let out = build_outline("本文のみ。見出しなし。\n#### only h4\n#nospace\n");
    assert_eq!(out, "", "見出し皆無は空文字列（プレースホルダ挿入なし）");
}

// ─────────────────────────────────────────────────────────────────────────────
// ケース 7: enforce_size_limit 境界 499,999 / 500,000 / 500,001
#[test]
fn test_enforce_size_limit_boundaries() {
    // @ts: TS-LGX-004 ケース 7
    assert!(
        SectionFormatter::enforce_size_limit(&"a".repeat(499_999)).is_ok(),
        "499,999 → Ok"
    );
    assert!(
        SectionFormatter::enforce_size_limit(&"a".repeat(RESULT_SIZE_LIMIT_CHARS)).is_ok(),
        "500,000 ちょうど → Ok"
    );
    match SectionFormatter::enforce_size_limit(&"a".repeat(RESULT_SIZE_LIMIT_CHARS + 1)) {
        Err(ContextError::ResultTooLarge { current, limit }) => {
            assert_eq!(current, 500_001);
            assert_eq!(limit, 500_000);
        }
        other => panic!("500,001 → ResultTooLarge: {other:?}"),
    }
}

// ケース 8: 文字カウント単位 = Unicode コードポイント
#[test]
fn test_enforce_size_limit_codepoint_unit() {
    // @ts: TS-LGX-004 ケース 8
    let s = "😀".repeat(RESULT_SIZE_LIMIT_CHARS + 1); // コードポイント 500,001、バイトは超過
    assert_eq!(s.chars().count(), 500_001);
    match SectionFormatter::enforce_size_limit(&s) {
        Err(ContextError::ResultTooLarge { current, .. }) => assert_eq!(current, 500_001),
        other => panic!("コードポイント数で判定: {other:?}"),
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// ケース 9: walk depth=Some(1) で直接親のみ
#[test]
fn test_walk_depth_1() {
    // @ts: TS-LGX-004 ケース 9
    let (graph, start) = seven_layer_graph();
    let walker = UpstreamWalker::new(&graph);
    let up = walker
        .walk_chain_parent_only_with_depth(&start, Some(1))
        .expect("Ok");
    assert!(up.iter().all(|a| a.chain_distance == 1));
}

// ケース 10: depth=Some(2) で祖父まで / depth=None で無制限
#[test]
fn test_walk_depth_2_and_none() {
    // @ts: TS-LGX-004 ケース 10
    let (graph, start) = seven_layer_graph();
    let walker = UpstreamWalker::new(&graph);
    let d2 = walker
        .walk_chain_parent_only_with_depth(&start, Some(2))
        .expect("Ok");
    assert!(d2.iter().all(|a| a.chain_distance <= 2), "depth 2 = 祖父まで");
    let dn = walker
        .walk_chain_parent_only_with_depth(&start, None)
        .expect("Ok");
    assert_eq!(dn.len(), 7, "None = 無制限（7 階層全て）");
}

// ケース 11: depth=Some(0) で空集合・正常終了（CLI exit 0・stderr Info は legixy-cli 委譲）
#[test]
fn test_walk_depth_0_empty() {
    // @ts: TS-LGX-004 ケース 11(a)（(b) CLI exit/Info は legixy-cli 委譲）
    let (graph, start) = seven_layer_graph();
    let walker = UpstreamWalker::new(&graph);
    let up = walker
        .walk_chain_parent_only_with_depth(&start, Some(0))
        .expect("Ok");
    assert!(up.is_empty(), "depth 0 = 空 Vec（エラーではない）");
}

// ─────────────────────────────────────────────────────────────────────────────
// ケース 12: subnode 粒度の 6 セクション順・整列・キャッシュマーカ
#[test]
fn test_subnode_six_sections_and_marker() {
    // @ts: TS-LGX-004 ケース 12
    let graph = TraceGraph::from_parts(
        vec![
            node("SRC-LGX-004", "SRC", "src.md"),
            node("DD-LGX-004", "DD", "dd.md"),
            node("DD-LGX-004#abc", "DD", "dd.md"),
        ],
        vec![
            chain_edge("DD-LGX-004", "SRC-LGX-004"),
            Edge {
                from: "DD-LGX-004".into(),
                to: "DD-LGX-004#abc".into(),
                kind: EdgeKind::ParentChild,
            },
        ],
    );
    let config = empty_config();
    let compiler = ContextCompiler::new(&graph, &config, None, Path::new("."));
    let input = CompileInput {
        target_files: vec![PathBuf::from("src.md")],
        granularity: Granularity::Subnode,
        ..Default::default()
    };
    let rendered = compiler
        .render(&compiler.compile(&input).expect("Ok"))
        .expect("render Ok");
    // 6 セクションがこの順に並ぶ。
    let positions: Vec<usize> = [
        "Layer Guidelines",
        "Additional Guidelines",
        legixy_ctx::CACHE_BREAKPOINT_MARKER,
        "Upstream Artifacts",
        "Target Node Metadata",
        "Custom Documents",
    ]
    .iter()
    .map(|m| rendered.find(m).unwrap_or_else(|| panic!("セクション欠落: {m}")))
    .collect();
    let mut sorted = positions.clone();
    sorted.sort_unstable();
    assert_eq!(positions, sorted, "6 セクションは規定順");
    assert_eq!(
        rendered.matches(legixy_ctx::CACHE_BREAKPOINT_MARKER).count(),
        1,
        "マーカは 1 箇所"
    );
}

// ケース 13: outline-only × subnode = anchor のみ / 見出し皆無で枠維持・空 body
#[test]
fn test_outline_subnode_anchor_only() {
    // @ts: TS-LGX-004 ケース 13
    let graph = TraceGraph::from_parts(
        vec![
            node("SRC-LGX-004", "SRC", "src.md"),
            node("DD-LGX-004", "DD", "dd.md"),
            node("DD-LGX-004#abc", "DD", "dd.md"),
        ],
        vec![
            chain_edge("DD-LGX-004", "SRC-LGX-004"),
            Edge {
                from: "DD-LGX-004".into(),
                to: "DD-LGX-004#abc".into(),
                kind: EdgeKind::ParentChild,
            },
        ],
    );
    let config = empty_config();
    let compiler = ContextCompiler::new(&graph, &config, None, Path::new("."));
    let input = CompileInput {
        target_files: vec![PathBuf::from("src.md")],
        granularity: Granularity::Subnode,
        outline_only: true,
        ..Default::default()
    };
    let result = compiler.compile(&input).expect("Ok");
    // subnode × outline_only: body は anchor 文字列のみ（または見出し皆無で空）。
    for art in result.upstream.iter().filter(|a| a.subnode_id.is_some()) {
        match &art.anchor {
            Some(anchor) => assert_eq!(&art.body, anchor, "outline×subnode = anchor のみ"),
            None => assert!(art.body.is_empty(), "anchor 無し subnode は空 body"),
        }
    }
}

// ケース 14: sections フィルタ — 存在 ID のみ通過 / 不在 ID 除外（混在）
#[test]
fn test_sections_existing_only() {
    // @ts: TS-LGX-004 ケース 14
    let graph = TraceGraph::from_parts(
        vec![
            node("SRC-LGX-004", "SRC", "src.md"),
            node("DD-LGX-004", "DD", "dd.md"),
            node("DD-LGX-004#abc", "DD", "dd.md"),
        ],
        vec![
            chain_edge("DD-LGX-004", "SRC-LGX-004"),
            Edge {
                from: "DD-LGX-004".into(),
                to: "DD-LGX-004#abc".into(),
                kind: EdgeKind::ParentChild,
            },
        ],
    );
    let config = empty_config();
    let compiler = ContextCompiler::new(&graph, &config, None, Path::new("."));
    let input = CompileInput {
        target_files: vec![PathBuf::from("src.md")],
        granularity: Granularity::Subnode,
        sections: Some(vec!["DD-LGX-004#abc".into(), "DD-LGX-004#zzz".into()]),
        ..Default::default()
    };
    let result = compiler.compile(&input).expect("不在 ID はエラーにしない");
    let ids: Vec<&str> = result
        .upstream
        .iter()
        .filter_map(|a| a.subnode_id.as_deref())
        .collect();
    assert!(ids.contains(&"DD-LGX-004#abc"), "存在 ID は通過");
    assert!(!ids.contains(&"DD-LGX-004#zzz"), "不在 ID は除外");
}

// ケース 15: sections — 全 ID 不在 → 空 upstream・exit 0
#[test]
fn test_sections_all_absent_empty() {
    // @ts: TS-LGX-004 ケース 15(a)
    let graph = TraceGraph::from_parts(
        vec![
            node("SRC-LGX-004", "SRC", "src.md"),
            node("DD-LGX-004", "DD", "dd.md"),
        ],
        vec![chain_edge("DD-LGX-004", "SRC-LGX-004")],
    );
    let config = empty_config();
    let compiler = ContextCompiler::new(&graph, &config, None, Path::new("."));
    let input = CompileInput {
        target_files: vec![PathBuf::from("src.md")],
        granularity: Granularity::Subnode,
        sections: Some(vec!["#nope1".into(), "#nope2".into()]),
        ..Default::default()
    };
    let result = compiler.compile(&input).expect("全不在でも Ok（exit 0）");
    assert!(result.upstream.is_empty());
}

// ケース 16: sections 親ドキュメント ID（# なし）→ 除外（stderr Info は legixy-cli 委譲）
#[test]
fn test_sections_parent_id_excluded() {
    // @ts: TS-LGX-004 ケース 16（CLI stderr Info は legixy-cli 委譲）
    let graph = TraceGraph::from_parts(
        vec![
            node("SRC-LGX-004", "SRC", "src.md"),
            node("DD-LGX-004", "DD", "dd.md"),
            node("DD-LGX-004#abc", "DD", "dd.md"),
        ],
        vec![
            chain_edge("DD-LGX-004", "SRC-LGX-004"),
            Edge {
                from: "DD-LGX-004".into(),
                to: "DD-LGX-004#abc".into(),
                kind: EdgeKind::ParentChild,
            },
        ],
    );
    let config = empty_config();
    let compiler = ContextCompiler::new(&graph, &config, None, Path::new("."));
    let input = CompileInput {
        target_files: vec![PathBuf::from("src.md")],
        granularity: Granularity::Subnode,
        sections: Some(vec!["DD-LGX-004".into()]), // # なし
        ..Default::default()
    };
    let result = compiler.compile(&input).expect("除外（エラーにしない）");
    assert!(result
        .upstream
        .iter()
        .all(|a| a.subnode_id.as_deref() != Some("DD-LGX-004")));
}

// ケース 17: sections × document 粒度 = sections 無視（REQ.18）
#[test]
fn test_sections_ignored_in_document_granularity() {
    // @ts: TS-LGX-004 ケース 17
    let (graph, _start) = seven_layer_graph();
    let config = empty_config();
    let compiler = ContextCompiler::new(&graph, &config, None, Path::new("."));
    let with_sections = CompileInput {
        target_files: vec![PathBuf::from("SRC-LGX-004.md")],
        granularity: Granularity::Document,
        sections: Some(vec!["DD-LGX-004#abc".into()]),
        ..Default::default()
    };
    let without = CompileInput {
        sections: None,
        ..with_sections.clone()
    };
    let a = compiler
        .render(&compiler.compile(&with_sections).expect("Ok"))
        .expect("render Ok");
    let b = compiler
        .render(&compiler.compile(&without).expect("Ok"))
        .expect("render Ok");
    assert_eq!(a, b, "document 粒度では sections は無視（同一バイト列）");
}

// ケース 18: フラグ組合せ優先順位（sections→outline、depth 直交）
#[test]
fn test_flag_combination_sections_then_outline() {
    // @ts: TS-LGX-004 ケース 18(c)（sections フィルタ先・outline 後）
    let graph = TraceGraph::from_parts(
        vec![
            node("SRC-LGX-004", "SRC", "src.md"),
            node("DD-LGX-004", "DD", "dd.md"),
            node("DD-LGX-004#abc", "DD", "dd.md"),
            node("DD-LGX-004#def", "DD", "dd.md"),
        ],
        vec![
            chain_edge("DD-LGX-004", "SRC-LGX-004"),
            Edge {
                from: "DD-LGX-004".into(),
                to: "DD-LGX-004#abc".into(),
                kind: EdgeKind::ParentChild,
            },
            Edge {
                from: "DD-LGX-004".into(),
                to: "DD-LGX-004#def".into(),
                kind: EdgeKind::ParentChild,
            },
        ],
    );
    let config = empty_config();
    let compiler = ContextCompiler::new(&graph, &config, None, Path::new("."));
    let input = CompileInput {
        target_files: vec![PathBuf::from("src.md")],
        granularity: Granularity::Subnode,
        sections: Some(vec!["DD-LGX-004#abc".into()]), // #abc に絞る
        outline_only: true,                            // 絞った後に outline 化（anchor のみ）
        depth_limit: Some(1),
        ..Default::default()
    };
    let result = compiler.compile(&input).expect("Ok");
    // sections フィルタが先: #def は登場しない。#abc のみ。
    let ids: Vec<&str> = result
        .upstream
        .iter()
        .filter_map(|a| a.subnode_id.as_deref())
        .collect();
    assert!(!ids.contains(&"DD-LGX-004#def"), "sections フィルタが先（#def 除外）");
}

// ケース 19: subnode + サブノード不在上流 → document fallback（代替 3a）
#[test]
fn test_subnode_fallback_to_document() {
    // @ts: TS-LGX-004 ケース 19
    let graph = TraceGraph::from_parts(
        vec![
            node("SRC-LGX-004", "SRC", "src.md"),
            node("DD-LGX-004", "DD", "dd.md"), // subnode あり（A）
            node("DD-LGX-004#abc", "DD", "dd.md"),
            node("UC-LGX-004", "UC", "uc.md"), // subnode なし（B）
        ],
        vec![
            chain_edge("UC-LGX-004", "DD-LGX-004"),
            chain_edge("DD-LGX-004", "SRC-LGX-004"),
            Edge {
                from: "DD-LGX-004".into(),
                to: "DD-LGX-004#abc".into(),
                kind: EdgeKind::ParentChild,
            },
        ],
    );
    let config = empty_config();
    let compiler = ContextCompiler::new(&graph, &config, None, Path::new("."));
    let input = CompileInput {
        target_files: vec![PathBuf::from("src.md")],
        granularity: Granularity::Subnode,
        ..Default::default()
    };
    let result = compiler.compile(&input).expect("Ok");
    // A は subnode 単位（subnode_id=Some）、B はノード単位 fallback（subnode_id=None）。
    let uc_arts: Vec<&UpstreamArtifact> = result
        .upstream
        .iter()
        .filter(|a| a.artifact_id == "UC-LGX-004")
        .collect();
    if !uc_arts.is_empty() {
        assert!(
            uc_arts.iter().all(|a| a.subnode_id.is_none()),
            "subnode 不在ノード B は document fallback（subnode_id=None）"
        );
    }
}

// ケース 20: ResultTooLarge エラーメッセージ固定（REQ.13、提案文あり）
#[test]
fn test_result_too_large_message_with_suggestion() {
    // @ts: TS-LGX-004 ケース 20
    let msg = ContextError::ResultTooLarge {
        current: 500_001,
        limit: 500_000,
    }
    .to_string();
    assert!(msg.contains("compile_context result exceeds"));
    assert!(msg.contains("Current size"));
    assert!(msg.contains("--granularity subnode"), "提案文必須");
    assert!(msg.contains("Narrow the target scope"), "提案文必須");
}

// ケース 21: 監査ログ書込失敗 → 本処理成功 exit 0 + stderr Warning（v3 差分）
#[test]
fn test_audit_failure_keeps_ok() {
    // @ts: TS-LGX-004 ケース 21
    use legixy_ctx::AuditLogger;
    let db = legixy_ctx::db::DbConn::new();
    let logger = AuditLogger::new(Some(&db));
    let input = CompileInput {
        target_files: vec![PathBuf::from("src.md")],
        ..Default::default()
    };
    let result = empty_result(Granularity::Subnode);
    assert!(
        logger.log(&input, &result).is_ok(),
        "書込失敗でも log は Ok(())（ベストエフォート、REQ.19）"
    );
}

// ケース 22: 起点不在・上流部分欠損 → 部分成功 exit 0（決定論的記録）
#[test]
fn test_partial_success_deterministic_record() {
    // @ts: TS-LGX-004 ケース 22(a)
    let graph = TraceGraph::from_parts(vec![node("SRC-LGX-004", "SRC", "known.md")], vec![]);
    let config = empty_config();
    let compiler = ContextCompiler::new(&graph, &config, None, Path::new("."));
    let unknown = PathBuf::from("unknown.md");
    let input = CompileInput {
        target_files: vec![PathBuf::from("known.md"), unknown.clone()],
        granularity: Granularity::Subnode,
        ..Default::default()
    };
    let result = compiler.compile(&input).expect("部分成功 exit 0");
    assert!(
        result.unresolved_targets.contains(&unknown),
        "未解決を unresolved_targets に決定論記録（パス辞書順、S2-24）"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// proptest（ケース 23 / 24）
// ─────────────────────────────────────────────────────────────────────────────
use proptest::prelude::*;

fn arb_upstream() -> impl Strategy<Value = Vec<UpstreamArtifact>> {
    proptest::collection::vec(
        ("[A-Z]{2,4}-LGX-00[1-9]", any::<bool>(), 1usize..5).prop_map(
            |(id, has_sub, dist)| UpstreamArtifact {
                artifact_id: id.clone(),
                type_code: id.split('-').next().unwrap().to_string(),
                file_path: PathBuf::from(format!("{id}.md")),
                chain_distance: dist,
                body: String::new(),
                subnode_id: if has_sub {
                    Some(format!("{id}#a"))
                } else {
                    None
                },
                anchor: if has_sub { Some("a".into()) } else { None },
                drift_score: None,
            },
        ),
        0..6,
    )
}

proptest! {
    // ケース 23: 同一入力 → 同一バイト列（CACHE-INV-1 / CTX-INV-1）。
    #[test]
    fn test_same_input_same_bytes(
        upstream in arb_upstream(),
        sub in any::<bool>(),
        outline in any::<bool>(),
        depth in proptest::option::of(0usize..=7),
    ) {
        // @ts: TS-LGX-004 ケース 23
        let g = if sub { Granularity::Subnode } else { Granularity::Document };
        let mut result = empty_result(g);
        result.upstream = upstream;
        let _ = (outline, depth); // 入力生成空間の網羅（render は result のみ依存）
        // render を複数回呼んでバイト一致を要求。render todo!() で panic → RED。
        let a = SectionFormatter::render(&result).expect("render Ok");
        let b = SectionFormatter::render(&result).expect("render Ok");
        prop_assert_eq!(a, b, "同一入力 → 同一バイト列");
    }

    // ケース 24: read-only 不変（粒度制御呼出しが graph を変更しない）。
    #[test]
    fn test_read_only_invariant(sub in any::<bool>()) {
        // @ts: TS-LGX-004 ケース 24
        let (graph, _start) = seven_layer_graph();
        let before = graph.node_count();
        let config = empty_config();
        let compiler = ContextCompiler::new(&graph, &config, None, Path::new("."));
        let g = if sub { Granularity::Subnode } else { Granularity::Document };
        let input = CompileInput {
            target_files: vec![PathBuf::from("SRC-LGX-004.md")],
            granularity: g,
            ..Default::default()
        };
        let _ = compiler.compile(&input); // todo!() panic → RED
        prop_assert_eq!(graph.node_count(), before, "compile は graph 不変（read-only）");
    }
}
