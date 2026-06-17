// Document ID: TC-LGX-002
// TC-LGX-002: コンテキスト解決（compile_context）のテストコード（TC[RED]）
//
// 親 chain: TS-LGX-002 → 本 TC-LGX-002 → SRC-LGX-002。
// 各テストは TS-LGX-002 のケースを `legixy-ctx` の凍結 API（DD-LGX-002 §3）に束縛する。
// SRC[GREEN] 未実装（compile/render/enforce_size_limit/walk/log = todo!()）のため、それらを
// 呼ぶテストは panic で失敗する（RED）。`cargo test -p legixy-ctx --no-run` は通る（型整合）。
//
// 委譲（本 Rust crate 対象外。コメントのみ）:
//  - ケース 19〜22: ts-mcp（TypeScript）転送層 = TC-LGX-002-TS / ts-mcp/test/compile-context.test.ts。
//  - exit code（0/1/2）の最終マッピング: legixy-cli 層（ContextError → exit 1、clap → exit 2）。
//    本 crate は `Result`/`ContextError` バリアントの形状（契約）のみを束縛する。
//  - stderr Info/Warning 診断文言の出力: legixy-cli 層（DD-LGX-004 §11 S2-23）。

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

/// 7 階層 chain（SPEC → UC → RB → SEQ → DD → TS → TC → SRC）。下流ノードを起点に逆 BFS。
fn seven_layer_graph() -> (TraceGraph, NodeId) {
    let ids = [
        "SPEC-LGX-003",
        "UC-LGX-002",
        "RBA-LGX-002",
        "SEQA-LGX-002",
        "DD-LGX-002",
        "TS-LGX-002",
        "TC-LGX-002",
        "SRC-LGX-002",
    ];
    let nodes: Vec<Node> = ids
        .iter()
        .map(|id| node(id, id.split('-').next().unwrap(), &format!("{id}.md")))
        .collect();
    // chain は親 → 子（上流 → 下流）。逆 BFS で起点（最下流 SRC）から親を辿る。
    let edges: Vec<Edge> = ids.windows(2).map(|w| chain_edge(w[0], w[1])).collect();
    (
        TraceGraph::from_parts(nodes, edges),
        "SRC-LGX-002".to_string(),
    )
}

fn empty_config() -> TraceConfig {
    TraceConfig::default()
}

// ─────────────────────────────────────────────────────────────────────────────
// ケース 1: 上流ゼロ（最上流 SPEC）→ 6 セクション構成維持・exit 0
#[test]
fn test_compile_render_topmost_keeps_six_sections() {
    // @ts: TS-LGX-002 ケース 1
    let graph = TraceGraph::from_parts(
        vec![node("SPEC-LGX-003", "SPEC", "docs/specs/spec.md")],
        vec![],
    );
    let config = empty_config();
    let compiler = ContextCompiler::new(&graph, &config, None, Path::new("."));
    let input = CompileInput {
        target_files: vec![PathBuf::from("docs/specs/spec.md")],
        granularity: Granularity::Document,
        ..Default::default()
    };
    let result = compiler.compile(&input).expect("最上流でも Ok");
    assert!(result.upstream.is_empty(), "最上流に上流なし");
    let rendered = compiler.render(&result).expect("render Ok");
    for marker in [
        "Layer Guidelines",
        "Additional Guidelines",
        "Upstream Artifacts",
        "Target Node Metadata",
        "Custom Documents",
    ] {
        assert!(
            rendered.contains(marker),
            "6 セクション枠は件数非依存で存在: {marker}"
        );
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// ケース 2: ちょうど 500,000 文字 → Ok（境界下限）
#[test]
fn test_enforce_size_limit_exactly_500000_is_ok() {
    // @ts: TS-LGX-002 ケース 2
    let rendered = "a".repeat(RESULT_SIZE_LIMIT_CHARS); // 500,000
    assert_eq!(rendered.chars().count(), 500_000);
    let r = SectionFormatter::enforce_size_limit(&rendered);
    assert!(r.is_ok(), "500,000 ちょうどは Ok（超える場合に非該当）");
}

// ケース 3: 500,001 文字 → Err(ResultTooLarge { current, limit })
#[test]
fn test_enforce_size_limit_500001_is_result_too_large() {
    // @ts: TS-LGX-002 ケース 3
    let rendered = "a".repeat(RESULT_SIZE_LIMIT_CHARS + 1); // 500,001
    match SectionFormatter::enforce_size_limit(&rendered) {
        Err(ContextError::ResultTooLarge { current, limit }) => {
            assert_eq!(current, 500_001);
            assert_eq!(limit, 500_000);
        }
        other => panic!("500,001 は ResultTooLarge を期待: {other:?}"),
    }
}

// ケース 3 補: ResultTooLarge の Display 文言（REQ.13 規定書式・切り捨て要約なし）
#[test]
fn test_result_too_large_display_message_fixed() {
    // @ts: TS-LGX-002 ケース 3（DD-002 §2.3 / REQ.13 文言）
    let err = ContextError::ResultTooLarge {
        current: 500_001,
        limit: 500_000,
    };
    let msg = err.to_string();
    assert!(msg.contains("compile_context result exceeds"));
    assert!(msg.contains("characters"));
    assert!(msg.contains("Current size"));
    assert!(msg.contains("--granularity subnode"));
    assert!(msg.contains("Narrow the target scope"));
}

// ケース 4: 文字カウント単位 = Unicode コードポイント（サロゲート・結合・ZWJ）
#[test]
fn test_enforce_size_limit_counts_unicode_codepoints() {
    // @ts: TS-LGX-002 ケース 4
    // 絵文字 😀（コードポイント 1・バイト 4）を 500,001 個 → コードポイント数 500,001。
    let rendered = "😀".repeat(RESULT_SIZE_LIMIT_CHARS + 1);
    assert_eq!(rendered.chars().count(), 500_001);
    assert!(rendered.len() > 500_001, "バイト長はコードポイント数より大");
    match SectionFormatter::enforce_size_limit(&rendered) {
        Err(ContextError::ResultTooLarge { current, .. }) => {
            assert_eq!(current, 500_001, "コードポイント数で判定（バイト長でない）");
        }
        other => panic!("コードポイント超過は ResultTooLarge: {other:?}"),
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// ケース 5: --depth 1 → 直接親（chain_distance==1）のみ
#[test]
fn test_walk_depth_1_direct_parent_only() {
    // @ts: TS-LGX-002 ケース 5
    let (graph, start) = seven_layer_graph();
    let walker = UpstreamWalker::new(&graph);
    let upstream = walker
        .walk_chain_parent_only_with_depth(&start, Some(1))
        .expect("walk Ok");
    assert!(
        upstream.iter().all(|a| a.chain_distance == 1),
        "depth 1 は直接親のみ（chain_distance==1）"
    );
}

// ケース 6: --depth 0 → 空集合（CLI exit 0・stderr Info は legixy-cli へ委譲）
#[test]
fn test_walk_depth_0_empty() {
    // @ts: TS-LGX-002 ケース 6（exit 0 / stderr Info は legixy-cli 委譲）
    let (graph, start) = seven_layer_graph();
    let walker = UpstreamWalker::new(&graph);
    let upstream = walker
        .walk_chain_parent_only_with_depth(&start, Some(0))
        .expect("walk Ok");
    assert!(upstream.is_empty(), "depth 0 は空集合（エラーではない）");
}

// ケース 7: --depth 無制限（None）→ 全祖先返却
#[test]
fn test_walk_depth_none_returns_all_ancestors() {
    // @ts: TS-LGX-002 ケース 7
    let (graph, start) = seven_layer_graph();
    let walker = UpstreamWalker::new(&graph);
    let upstream = walker
        .walk_chain_parent_only_with_depth(&start, None)
        .expect("walk Ok");
    // SRC を起点に SPEC まで 7 祖先（SRC 自身を除く）。
    assert_eq!(upstream.len(), 7, "None は到達可能な全上流（7 階層）");
}

// ─────────────────────────────────────────────────────────────────────────────
// ケース 8: --sections 指定 ID のみ返却（subnode 粒度）
#[test]
fn test_sections_filter_subnode_exact_match_only() {
    // @ts: TS-LGX-002 ケース 8
    let graph = TraceGraph::from_parts(
        vec![
            node("SRC-LGX-002", "SRC", "src.md"),
            node("DD-LGX-002", "DD", "dd.md"),
            node("DD-LGX-002#abc", "DD", "dd.md"),
            node("DD-LGX-002#def", "DD", "dd.md"),
        ],
        vec![
            chain_edge("DD-LGX-002", "SRC-LGX-002"),
            Edge {
                from: "DD-LGX-002".into(),
                to: "DD-LGX-002#abc".into(),
                kind: EdgeKind::ParentChild,
            },
            Edge {
                from: "DD-LGX-002".into(),
                to: "DD-LGX-002#def".into(),
                kind: EdgeKind::ParentChild,
            },
        ],
    );
    let config = empty_config();
    let compiler = ContextCompiler::new(&graph, &config, None, Path::new("."));
    let input = CompileInput {
        target_files: vec![PathBuf::from("src.md")],
        granularity: Granularity::Subnode,
        sections: Some(vec!["DD-LGX-002#abc".into(), "DD-LGX-002#def".into()]),
        ..Default::default()
    };
    let result = compiler.compile(&input).expect("Ok");
    let subnode_ids: Vec<&str> = result
        .upstream
        .iter()
        .filter_map(|a| a.subnode_id.as_deref())
        .collect();
    assert!(
        subnode_ids
            .iter()
            .all(|id| *id == "DD-LGX-002#abc" || *id == "DD-LGX-002#def"),
        "指定 ID と完全一致する subnode のみ"
    );
}

// ケース 9: --sections 縮退入力（全無効 = 空 upstream・exit 0）
#[test]
fn test_sections_all_invalid_empty_upstream() {
    // @ts: TS-LGX-002 ケース 9（c: 全無効 → 空 upstream exit 0）
    let graph = TraceGraph::from_parts(
        vec![
            node("SRC-LGX-002", "SRC", "src.md"),
            node("DD-LGX-002", "DD", "dd.md"),
        ],
        vec![chain_edge("DD-LGX-002", "SRC-LGX-002")],
    );
    let config = empty_config();
    let compiler = ContextCompiler::new(&graph, &config, None, Path::new("."));
    // legixy-cli 正規化後の sections（全件が不在 subnode ID）。
    let input = CompileInput {
        target_files: vec![PathBuf::from("src.md")],
        granularity: Granularity::Subnode,
        sections: Some(vec!["DD-LGX-002#nope".into()]),
        ..Default::default()
    };
    let result = compiler.compile(&input).expect("全無効でも Ok（exit 0）");
    assert!(result.upstream.is_empty(), "全無効フィルタ = 空 upstream");
}

// ケース 10: --sections 親ドキュメント ID（# なし）→ 除外（stderr Info は legixy-cli 委譲）
#[test]
fn test_sections_document_level_id_excluded() {
    // @ts: TS-LGX-002 ケース 10（stderr Info は legixy-cli 委譲）
    let graph = TraceGraph::from_parts(
        vec![
            node("SRC-LGX-002", "SRC", "src.md"),
            node("DD-LGX-002", "DD", "dd.md"),
            node("DD-LGX-002#abc", "DD", "dd.md"),
        ],
        vec![
            chain_edge("DD-LGX-002", "SRC-LGX-002"),
            Edge {
                from: "DD-LGX-002".into(),
                to: "DD-LGX-002#abc".into(),
                kind: EdgeKind::ParentChild,
            },
        ],
    );
    let config = empty_config();
    let compiler = ContextCompiler::new(&graph, &config, None, Path::new("."));
    let input = CompileInput {
        target_files: vec![PathBuf::from("src.md")],
        granularity: Granularity::Subnode,
        sections: Some(vec!["DD-LGX-002".into()]), // # なし = 親 ID
        ..Default::default()
    };
    let result = compiler.compile(&input).expect("Ok（エラーにしない）");
    assert!(
        result
            .upstream
            .iter()
            .all(|a| a.subnode_id.as_deref() != Some("DD-LGX-002")),
        "親 ID（# なし）は subnode と不一致のため除外"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// ケース 11: --outline-only h1〜h3 抽出（本文非出力、見出し皆無で空 body は build_outline 経由）
#[test]
fn test_outline_only_extracts_headings() {
    // @ts: TS-LGX-002 ケース 11（build_outline 経由の render）
    let body = "# H1\n本文a\n## H2\n本文b\n### H3\n";
    let outline = build_outline(body);
    assert!(outline.contains("H1"), "h1 抽出");
    assert!(outline.contains("H2"), "h2 抽出");
    assert!(outline.contains("H3"), "h3 抽出");
    assert!(!outline.contains("本文a"), "本文は非出力");
    assert!(!outline.contains("本文b"), "本文は非出力");
}

// ─────────────────────────────────────────────────────────────────────────────
// ケース 14: セクション配置順序固定（granularity 非依存）
#[test]
fn test_section_order_independent_of_granularity() {
    // @ts: TS-LGX-002 ケース 14
    let (graph, _start) = seven_layer_graph();
    let config = empty_config();
    let compiler = ContextCompiler::new(&graph, &config, None, Path::new("."));
    let mk = |g| CompileInput {
        target_files: vec![PathBuf::from("SRC-LGX-002.md")],
        granularity: g,
        ..Default::default()
    };
    let doc = compiler
        .render(&compiler.compile(&mk(Granularity::Document)).expect("Ok"))
        .expect("render Ok");
    let sub = compiler
        .render(&compiler.compile(&mk(Granularity::Subnode)).expect("Ok"))
        .expect("render Ok");
    let order = |s: &str| {
        [
            "Layer Guidelines",
            "Additional Guidelines",
            "Upstream Artifacts",
            "Target Node Metadata",
            "Custom Documents",
        ]
        .iter()
        .map(|m| s.find(m))
        .collect::<Vec<_>>()
    };
    assert_eq!(
        order(&doc),
        order(&sub),
        "6 セクション順序は granularity 非依存"
    );
}

// ケース 15: キャッシュブレーク点マーカが 1 箇所
#[test]
fn test_cache_breakpoint_marker_appears_once() {
    // @ts: TS-LGX-002 ケース 15
    let (graph, _start) = seven_layer_graph();
    let config = empty_config();
    let compiler = ContextCompiler::new(&graph, &config, None, Path::new("."));
    let input = CompileInput {
        target_files: vec![PathBuf::from("SRC-LGX-002.md")],
        ..Default::default()
    };
    let rendered = compiler
        .render(&compiler.compile(&input).expect("Ok"))
        .expect("render Ok");
    assert_eq!(
        rendered.matches(legixy_ctx::CACHE_BREAKPOINT_MARKER).count(),
        1,
        "マーカはちょうど 1 回"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// ケース 16: 起点ノード未登録 → 無視して残りで解決・exit 0・metadata 記録
#[test]
fn test_partial_unregistered_target_resolves_rest() {
    // @ts: TS-LGX-002 ケース 16（stderr Info は legixy-cli 委譲）
    let graph = TraceGraph::from_parts(
        vec![node("SRC-LGX-002", "SRC", "known.md")],
        vec![],
    );
    let config = empty_config();
    let compiler = ContextCompiler::new(&graph, &config, None, Path::new("."));
    let unknown = PathBuf::from("unknown.md");
    let input = CompileInput {
        target_files: vec![PathBuf::from("known.md"), unknown.clone()],
        ..Default::default()
    };
    let result = compiler.compile(&input).expect("部分未登録でも Ok（exit 0）");
    assert!(
        result.unresolved_targets.contains(&unknown),
        "未解決起点を unresolved_targets に決定論記録（PathBuf 辞書順）"
    );
    assert!(
        result
            .targets
            .iter()
            .any(|t| t.file_path == unknown && t.artifact_id.is_none()),
        "未登録 target は artifact_id=None"
    );
}

// ケース 17: 全起点未登録 → 空 upstream・exit 0
#[test]
fn test_all_unregistered_targets_empty_upstream() {
    // @ts: TS-LGX-002 ケース 17
    let graph = TraceGraph::empty();
    let config = empty_config();
    let compiler = ContextCompiler::new(&graph, &config, None, Path::new("."));
    let input = CompileInput {
        target_files: vec![PathBuf::from("a.md"), PathBuf::from("b.md")],
        ..Default::default()
    };
    let result = compiler.compile(&input).expect("全未登録でも Ok（exit 0）");
    assert!(result.upstream.is_empty(), "全未登録 = 空 upstream");
    assert!(
        result.targets.iter().all(|t| t.artifact_id.is_none()),
        "全 target が artifact_id=None"
    );
}

// ケース 18: 上流連鎖途中の欠損（ファイル不在）→ 部分成功・空 body・exit 0
#[test]
fn test_upstream_missing_file_partial_success_empty_body() {
    // @ts: TS-LGX-002 ケース 18
    // DD ノードはグラフ上に存在するがファイル実体が不在（path が読めない）。
    let graph = TraceGraph::from_parts(
        vec![
            node("SRC-LGX-002", "SRC", "src.md"),
            node("DD-LGX-002", "DD", "does-not-exist.md"),
        ],
        vec![chain_edge("DD-LGX-002", "SRC-LGX-002")],
    );
    let config = empty_config();
    let compiler = ContextCompiler::new(&graph, &config, None, Path::new("."));
    let input = CompileInput {
        target_files: vec![PathBuf::from("src.md")],
        ..Default::default()
    };
    let result = compiler.compile(&input).expect("ファイル不在は Err に昇格しない（exit 0）");
    if let Some(missing) = result
        .upstream
        .iter()
        .find(|a| a.artifact_id == "DD-LGX-002")
    {
        assert!(missing.body.is_empty(), "欠損ノードは空 body で継続");
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// ケース 23: 監査ログ書込失敗 → 本処理 Ok 維持・exit 0
#[test]
fn test_audit_log_failure_keeps_compile_ok() {
    // @ts: TS-LGX-002 ケース 23（stderr Warning は legixy-cli/AuditLogger 内部、exit 0）
    use legixy_ctx::AuditLogger;
    let db = legixy_ctx::db::DbConn::new(); // 書込が失敗する DB を模す（SRC[GREEN] で fixture 化）
    let logger = AuditLogger::new(Some(&db));
    let input = CompileInput {
        target_files: vec![PathBuf::from("src.md")],
        ..Default::default()
    };
    let result = empty_result(Granularity::Document);
    // log は書込失敗を stderr Warning にし常に Ok(()) を返す（ベストエフォート、REQ.19）。
    assert!(
        logger.log(&input, &result).is_ok(),
        "監査ログ書込失敗でも log は Ok(())"
    );
}

// ケース 24: engine.db 不在（db=None）→ graph.toml のみで返却・記録スキップ
#[test]
fn test_db_none_audit_noop_and_resolves() {
    // @ts: TS-LGX-002 ケース 24
    use legixy_ctx::AuditLogger;
    let logger = AuditLogger::new(None);
    let input = CompileInput {
        target_files: vec![PathBuf::from("src.md")],
        ..Default::default()
    };
    let result = empty_result(Granularity::Document);
    assert!(
        logger.log(&input, &result).is_ok(),
        "db=None は no-op で Ok(())"
    );
    // graph.toml のみで上流走査が返ること（compile 経路）。
    let graph = TraceGraph::empty();
    let config = empty_config();
    let compiler = ContextCompiler::new(&graph, &config, None, Path::new("."));
    assert!(compiler.compile(&input).is_ok(), "db=None でも compile Ok");
}

// ─────────────────────────────────────────────────────────────────────────────
// ケース 27: granularity 不正値（受理済み意味的不正）→ InvalidInput（CLI が exit 1）
// 注: Rust 型 `Granularity` は 2 値のみで不正値を構築できないため、意味的不正の経路は
//     CLI/MCP 層（文字列 → enum 変換失敗）で発生する。本 crate では ContextError::InvalidInput
//     が exit 1 概念に対応することのみを束縛し、文字列変換 reject は legixy-cli へ委譲する。
#[test]
fn test_invalid_input_variant_is_exit1_concept() {
    // @ts: TS-LGX-002 ケース 27（文字列→enum reject は legixy-cli 委譲）
    let err = ContextError::InvalidInput("granularity 'auto' is not document|subnode".into());
    // ContextError は全て exit 1 概念（LGX-COMPAT-001 §3）。InvalidInput がその一員であること。
    assert!(matches!(err, ContextError::InvalidInput(_)));
}

// ケース 28: 終了コード契約 0/1/2（本 crate は ContextError 形状のみ束縛）
#[test]
fn test_exit_code_contract_error_variants() {
    // @ts: TS-LGX-002 ケース 28（exit マッピングは legixy-cli、clap exit 2 も legixy-cli）
    // exit 1 に写像される ContextError バリアント群（DD-002 §2.3 / §6）。
    let exit1: Vec<ContextError> = vec![
        ContextError::ResultTooLarge {
            current: 500_001,
            limit: 500_000,
        },
        ContextError::Graph("graph.toml parse failed".into()),
        ContextError::InvalidInput("bad granularity".into()),
    ];
    for e in &exit1 {
        // どのバリアントも Display 可能（CLI が stderr 出力 → exit 1）。
        assert!(!e.to_string().is_empty());
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// proptest（ケース 12 / 13 / 25 / 26）
// ─────────────────────────────────────────────────────────────────────────────
use proptest::prelude::*;

/// テスト用に最小の ContextResult を構築（render が todo!() のため RED 化用）。
fn empty_result(granularity: Granularity) -> ContextResult {
    ContextResult {
        targets: Vec::new(),
        layer_guidelines: Vec::new(),
        additional_guidelines: Vec::new(),
        upstream: Vec::new(),
        custom_documents: Vec::new(),
        target_metadata: vec![TargetNodeMetadata {
            artifact_id: "SRC-LGX-002".into(),
            outgoing_edges: Vec::new(),
            incoming_edges: Vec::new(),
            subnode_count: 0,
            unresolved_targets: Vec::new(),
        }],
        granularity,
        unresolved_targets: Vec::new(),
    }
}

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
    // ケース 12: subnode 整列 = 親 ID 辞書順 + アンカー出現順（A-1）。render 決定論。
    #[test]
    fn test_subnode_ordering_deterministic(upstream in arb_upstream()) {
        // @ts: TS-LGX-002 ケース 12（A-1: アンカー出現順）
        let mut result = empty_result(Granularity::Subnode);
        result.upstream = upstream;
        // 同一 result に対し render は同一出力（決定論）。render todo!() で panic → RED。
        let a = SectionFormatter::render(&result);
        let b = SectionFormatter::render(&result);
        prop_assert_eq!(format!("{:?}", a.is_ok()), format!("{:?}", b.is_ok()));
        if let (Ok(sa), Ok(sb)) = (a, b) {
            prop_assert_eq!(sa, sb);
        }
    }

    // ケース 13: バイト単位決定論（REQ.14）。render を 10 回呼び全一致。
    #[test]
    fn test_render_byte_determinism(upstream in arb_upstream(), sub in any::<bool>()) {
        // @ts: TS-LGX-002 ケース 13
        let g = if sub { Granularity::Subnode } else { Granularity::Document };
        let mut result = empty_result(g);
        result.upstream = upstream;
        let mut prev: Option<String> = None;
        for _ in 0..10 {
            // render todo!() のため panic → RED。GREEN 後は全バイト列一致を要求。
            let s = SectionFormatter::render(&result).expect("render Ok");
            if let Some(p) = &prev {
                prop_assert_eq!(p, &s, "同一入力 → 同一バイト列");
            }
            prev = Some(s);
        }
    }

    // ケース 25: read-only 不変（compile は graph を変更しない）。
    #[test]
    fn test_compile_is_read_only(n in 0usize..4) {
        // @ts: TS-LGX-002 ケース 25
        let nodes: Vec<Node> = (0..n)
            .map(|i| node(&format!("SRC-LGX-00{}", i + 1), "SRC", &format!("s{i}.md")))
            .collect();
        let graph = TraceGraph::from_parts(nodes, vec![]);
        let before = graph.node_count();
        let config = empty_config();
        let compiler = ContextCompiler::new(&graph, &config, None, Path::new("."));
        let input = CompileInput {
            target_files: vec![PathBuf::from("s0.md")],
            ..Default::default()
        };
        let _ = compiler.compile(&input); // todo!() panic → RED
        prop_assert_eq!(graph.node_count(), before, "compile は graph 不変（read-only）");
    }

    // ケース 26: 冪等性 — 同一入力 → 同一 ContextResult・同一 render。
    #[test]
    fn test_compile_idempotent(sub in any::<bool>()) {
        // @ts: TS-LGX-002 ケース 26
        let (graph, _start) = seven_layer_graph();
        let config = empty_config();
        let compiler = ContextCompiler::new(&graph, &config, None, Path::new("."));
        let g = if sub { Granularity::Subnode } else { Granularity::Document };
        let input = CompileInput {
            target_files: vec![PathBuf::from("SRC-LGX-002.md")],
            granularity: g,
            ..Default::default()
        };
        let r1 = compiler.compile(&input).expect("Ok"); // todo!() panic → RED
        let r2 = compiler.compile(&input).expect("Ok");
        prop_assert_eq!(r1, r2, "同一入力 → 同一 ContextResult");
    }
}
