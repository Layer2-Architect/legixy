// CACHE-CR-002（R-6）回帰テスト: contextual_retrieval 有効時に embeddings.context を生成する。
// pre-fix では CLI が ContextualConfig を渡さず／synthesize がパススルー／embed_node が context を
// 無視するため、enabled=true でも context が NULL のままだった → RED。
//
// 本テストは決定論既定クライアント（LLM/network 不要、Phase 1）の経路を Embedder::stub で検証する。
// 親: SPEC-LGX-006.REQ.06（contextual retrieval）/ LGX-EXT-001 §5.8（context を embedding 前に付加）/
//     DD-LGX-007 §2.1（EmbedResult.context/context_hash）/ ADR-LGX-009（content_hash のみ freshness 寄与）。

use legixy_embed::{
    embed_all, ContextualConfig, CrOptions, EmbedOptions, Embedder, EmbeddingStore, NodeFilter,
};
use legixy_graph::{Node, TraceGraph};

fn doc_node(id: &str, path: &str) -> Node {
    Node {
        id: id.to_string(),
        type_code: "UC".to_string(),
        path: path.to_string(),
        parent_id: None,
        anchor: None,
    }
}

const DOC: &str = "# タイトルX\n\n## セクションA\n\n本文 alpha-unique。\n";

fn opts(contextual: Option<ContextualConfig>, root: &std::path::Path) -> EmbedOptions {
    EmbedOptions {
        force: true, // freshness skip を避けて確実に生成
        include_subnodes: false,
        contextual,
        project_root: Some(root.to_path_buf()),
        node_filter: NodeFilter::All,
    }
}

#[test]
fn contextual_enabled_populates_context_and_hash() {
    let tmp = tempfile::tempdir().unwrap();
    std::fs::write(tmp.path().join("uc.md"), DOC).unwrap();
    let graph = TraceGraph::from_parts(vec![doc_node("UC-LGX-001", "uc.md")], vec![]);
    let store = EmbeddingStore::empty();
    let embedder = Embedder::stub("test-model", 8);

    let cfg = ContextualConfig {
        opts: CrOptions::default(),
    };
    let report = embed_all(&graph, &store, &embedder, opts(Some(cfg), tmp.path())).expect("embed");
    assert!(report.generated >= 1, "1 件以上生成: {report:?}");

    let rows = store.load_all().unwrap();
    let row = rows
        .iter()
        .find(|r| r.node_id == "UC-LGX-001")
        .expect("UC 行");
    assert!(
        row.context.is_some(),
        "contextual 有効 → context 非 NULL: {row:?}"
    );
    assert!(
        row.context_hash.is_some(),
        "context_hash 非 NULL: {row:?}"
    );
    // 決定論既定クライアントは node_id を context に含める（位置づけ）。
    assert!(
        row.context.as_deref().unwrap().contains("UC-LGX-001"),
        "context に node_id: {:?}",
        row.context
    );
}

#[test]
fn enabling_cr_backfills_context_without_force() {
    // 既存 embedding（CR 無効・context NULL）に対し CR を有効化して再 embed（--force なし）すると、
    // content 不変でも context が 1 度だけ backfill される（CACHE-CR-002 / ADR-LGX-009 キャッシュ整合）。
    let tmp = tempfile::tempdir().unwrap();
    std::fs::write(tmp.path().join("uc.md"), DOC).unwrap();
    let graph = TraceGraph::from_parts(vec![doc_node("UC-LGX-001", "uc.md")], vec![]);
    let store = EmbeddingStore::empty();
    let embedder = Embedder::stub("test-model", 8);

    // 1st: CR 無効 + force なし → context NULL。
    let o1 = EmbedOptions {
        force: false,
        include_subnodes: false,
        contextual: None,
        project_root: Some(tmp.path().to_path_buf()),
        node_filter: NodeFilter::All,
    };
    embed_all(&graph, &store, &embedder, o1).expect("embed1");
    assert!(store.load_all().unwrap()[0].context.is_none(), "初回は context NULL");

    // 2nd: CR 有効 + force なし → content 不変だが context backfill。
    let o2 = EmbedOptions {
        force: false,
        include_subnodes: false,
        contextual: Some(ContextualConfig {
            opts: CrOptions::default(),
        }),
        project_root: Some(tmp.path().to_path_buf()),
        node_filter: NodeFilter::All,
    };
    let r2 = embed_all(&graph, &store, &embedder, o2).expect("embed2");
    assert_eq!(r2.generated, 1, "backfill で 1 件再生成（skip しない）: {r2:?}");
    let row = &store.load_all().unwrap()[0];
    assert!(row.context.is_some(), "CR 有効化で context backfill: {row:?}");

    // 3rd: CR 有効のまま再 embed → 既に context あり → skip（再合成しない、キャッシュ）。
    let o3 = EmbedOptions {
        force: false,
        include_subnodes: false,
        contextual: Some(ContextualConfig {
            opts: CrOptions::default(),
        }),
        project_root: Some(tmp.path().to_path_buf()),
        node_filter: NodeFilter::All,
    };
    let r3 = embed_all(&graph, &store, &embedder, o3).expect("embed3");
    assert_eq!(r3.skipped, 1, "context 既存 → skip（キャッシュ、ADR-LGX-009）: {r3:?}");
}

#[test]
fn embed_all_does_not_double_embed_materialized_auto_subnodes() {
    // BUG-007 E2E 配線(load→parse_graph)の副作用回帰: parse_graph が materialize した auto サブノードを
    // embed_all の main loop が全文で二重 embed し、embed_subnodes の区画スライス(is_subnode=1)を
    // 上書きしてしまう問題。main loop は auto サブノードを除外し、embed_subnodes のみが所有すべき。
    let tmp = tempfile::tempdir().unwrap();
    std::fs::write(
        tmp.path().join("uc.md"),
        "# タイトル\n\n## セクションA\n\nアルファ。\n",
    )
    .unwrap();
    // parse_graph で UC ドキュメント + auto サブノードを materialize（CLI load() と同経路）。
    let gp = tmp.path().join("graph.toml");
    std::fs::write(
        &gp,
        "[[nodes]]\nid = \"UC-LGX-001\"\ntype = \"UC\"\npath = \"uc.md\"\n",
    )
    .unwrap();
    let graph = legixy_graph::subnode::parse_graph(&gp, tmp.path()).expect("parse_graph");
    // materialize 確認: doc + auto サブノード1 = 2 ノード。
    assert_eq!(graph.node_count(), 2, "auto サブノード materialize 済");

    let store = EmbeddingStore::empty();
    let embedder = Embedder::stub("test-model", 8);
    let report = embed_all(
        &graph,
        &store,
        &embedder,
        EmbedOptions {
            force: true,
            include_subnodes: true,
            contextual: None,
            project_root: Some(tmp.path().to_path_buf()),
            node_filter: NodeFilter::All,
        },
    )
    .expect("embed");

    // doc(1) + サブノード区画(1) = 2。auto サブノードの main loop 二重処理(=3)が起きない。
    assert_eq!(report.generated, 2, "二重 embed なし: {report:?}");
    // サブノード行は is_subnode=1 で 1 件（main loop の全文上書き=is_subnode=0 が起きていない）。
    let subs = store.list_subnodes().unwrap();
    assert_eq!(subs.len(), 1, "サブノード行は is_subnode=1 で保持: {subs:?}");
    // サブノードの content_hash は区画スライス基準（doc 全文と異なる）。
    let rows = store.load_all().unwrap();
    let doc = rows.iter().find(|r| r.node_id == "UC-LGX-001").unwrap();
    let sub = rows.iter().find(|r| r.node_id.contains('#')).unwrap();
    assert_ne!(
        doc.content_hash, sub.content_hash,
        "サブノードは全文でなく区画スライスを embed"
    );
}

#[test]
fn contextual_disabled_leaves_context_null() {
    // 既定（無効）では context は NULL（回帰防止 / CR 任意性）。
    let tmp = tempfile::tempdir().unwrap();
    std::fs::write(tmp.path().join("uc.md"), DOC).unwrap();
    let graph = TraceGraph::from_parts(vec![doc_node("UC-LGX-001", "uc.md")], vec![]);
    let store = EmbeddingStore::empty();
    let embedder = Embedder::stub("test-model", 8);

    embed_all(&graph, &store, &embedder, opts(None, tmp.path())).expect("embed");
    let rows = store.load_all().unwrap();
    let row = rows.iter().find(|r| r.node_id == "UC-LGX-001").unwrap();
    assert!(row.context.is_none(), "CR 無効 → context NULL: {row:?}");
    assert!(row.context_hash.is_none(), "context_hash NULL: {row:?}");
}
