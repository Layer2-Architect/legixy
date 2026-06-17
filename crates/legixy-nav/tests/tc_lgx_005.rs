// Document ID: TC-LGX-005
// TC-LGX-005: 逆方向探索（investigate）のテストコード（TC[RED]）。
//
// 親 chain: TS-LGX-005 → 本 TC-LGX-005 → SRC-LGX-005。
// 各テストは TS-LGX-005 のケースを legixy-nav の凍結 API（DD-LGX-005 §3）に束縛する。
// SRC[GREEN] 未実装（investigate / investigate_with_depth / render_pruned / render_outcome /
// DriftPruner::prune / MultiTraverser::traverse_reverse_multi = todo!()）のため、それらを呼ぶ
// テストは panic で失敗する（RED）。`cargo test -p legixy-nav --no-run` は通る（型・シグネチャ整合）。
//
// 委譲（テスト化不要、コメントで委譲先を記す）:
// - ケース 17(b)(c) graph ロード失敗 exit 1 / clap 構文 exit 2 → legixy-cli 層 / LGX-COMPAT-001 §3
// - ケース 18 stdout/stderr 分離 → legixy-cli E2E（cli_investigate.rs、本 crate 境界外）
// - ケース 19 MCP 非公開 → ts-mcp/test/tools.test.ts（MCP カタログ検査、本 crate 境界外）
// - drift 数値妥当性 → SPEC-LGX-006 / TP-LGX-006（本 TS は閾値判定・整列決定性のみ）

use indexmap::IndexMap;
use proptest::prelude::*;

use legixy_nav::{
    investigate, investigate_with_depth, render_outcome, render_pruned, DriftPruner,
    InvestigateOutcome, MultiTraversalResult, PrunedTraversalResult, ReportFormat, SuspiciousNode,
    VisitedNode,
};
use legixy_graph::{Edge, EdgeKind, Node, TraceGraph};

// --- fixture helpers ---

fn node(id: &str, type_code: &str, path: &str) -> Node {
    Node {
        id: id.to_string(),
        type_code: type_code.to_string(),
        path: path.to_string(),
        parent_id: None,
        anchor: None,
    }
}

fn chain(from: &str, to: &str) -> Edge {
    Edge {
        from: from.to_string(),
        to: to.to_string(),
        kind: EdgeKind::Chain,
    }
}

/// 線形 chain `A → B → C`（from→to）。逆方向起点は C。
fn linear_abc() -> TraceGraph {
    TraceGraph::from_parts(
        vec![node("A", "UC", "a.md"), node("B", "DD", "b.md"), node("C", "SRC", "c.md")],
        vec![chain("A", "B"), chain("B", "C")],
    )
}

fn visited(id: &str, type_code: &str, path: &str, depth: usize) -> VisitedNode {
    VisitedNode {
        id: id.to_string(),
        type_code: type_code.to_string(),
        path: path.to_string(),
        depth,
    }
}

fn suspicious(id: &str, drift: f32) -> SuspiciousNode {
    SuspiciousNode {
        id: id.to_string(),
        drift_score: drift,
        type_code: "DD".to_string(),
        path: format!("{id}.md"),
    }
}

/// truncated 系テスト用の最小 PrunedTraversalResult。
fn pruned_fixture(
    visited_nodes: Vec<VisitedNode>,
    suspicious_nodes: Vec<SuspiciousNode>,
    drift_threshold: f32,
) -> PrunedTraversalResult {
    let mut depth_map = IndexMap::new();
    for v in &visited_nodes {
        depth_map.insert(v.id.clone(), v.depth);
    }
    let start_ids = visited_nodes.first().map(|v| vec![v.id.clone()]).unwrap_or_default();
    PrunedTraversalResult {
        traversal: MultiTraversalResult {
            visited: visited_nodes,
            edges_traversed: vec![],
            depth_map,
            start_ids,
        },
        suspicious_nodes,
        drift_threshold,
    }
}

// ケース 1: max_depth = 0 → 起点のみ（depth 0）返る、truncated=true
#[test]
fn test_max_depth_0_only_start() {
    // @ts: TS-LGX-005 ケース 1
    let graph = linear_abc();
    let outcome = investigate_with_depth(&graph, &["C".to_string()], None, 0.3, Some(0))
        .expect("max_depth=0 は Ok");
    assert_eq!(outcome.result.traversal.visited.len(), 1);
    assert_eq!(outcome.result.traversal.visited[0].id, "C");
    assert_eq!(outcome.result.traversal.visited[0].depth, 0);
    assert_eq!(outcome.result.traversal.depth_map.get("C"), Some(&0));
    assert!(outcome.result.traversal.edges_traversed.is_empty());
    assert!(outcome.truncated, "起点に未訪問の上流隣接があるため truncated");
    assert_eq!(outcome.excluded_count, 1, "C の逆方向隣接 B が 1 件");
}

// ケース 2: max_depth 未指定（None）と巨大値と investigate の結果一致
#[test]
fn test_none_and_huge_and_investigate_agree() {
    // @ts: TS-LGX-005 ケース 2
    let graph = linear_abc();
    let start = ["C".to_string()];
    let none = investigate_with_depth(&graph, &start, None, 0.3, None).expect("Ok");
    let huge = investigate_with_depth(&graph, &start, None, 0.3, Some(1_000_000)).expect("Ok");
    let plain = investigate(&graph, &start, None, 0.3).expect("Ok");

    assert_eq!(none.result.traversal.visited, huge.result.traversal.visited);
    assert_eq!(none.result.traversal.depth_map, huge.result.traversal.depth_map);
    assert_eq!(none.result.traversal.edges_traversed, huge.result.traversal.edges_traversed);
    assert!(!none.truncated);
    assert_eq!(none.excluded_count, 0);
    assert!(!huge.truncated);
    assert_eq!(huge.excluded_count, 0);
    // investigate(.., None) と investigate_with_depth(.., None) は同一 PrunedTraversalResult
    assert_eq!(plain, none.result);
}

// ケース 3: max_depth 打ち切り発生 → truncated=true / excluded_count>0（exit 不変）
#[test]
fn test_truncation_partial_set() {
    // @ts: TS-LGX-005 ケース 3
    // A→B→C→D の逆方向起点 D、max_depth=1 → visited は depth 0(D)/1(C) のみ
    let graph = TraceGraph::from_parts(
        vec![
            node("A", "UC", "a.md"),
            node("B", "DD", "b.md"),
            node("C", "TS", "c.md"),
            node("D", "SRC", "d.md"),
        ],
        vec![chain("A", "B"), chain("B", "C"), chain("C", "D")],
    );
    let outcome = investigate_with_depth(&graph, &["D".to_string()], None, 0.3, Some(1))
        .expect("Ok");
    let ids: Vec<&str> = outcome
        .result
        .traversal
        .visited
        .iter()
        .map(|v| v.id.as_str())
        .collect();
    assert_eq!(ids, vec!["D", "C"], "depth 0/1 のみ");
    assert!(outcome.truncated);
    assert_eq!(outcome.excluded_count, 1, "depth==1 境界 C の未訪問逆隣接 B が 1 件");
}

// ケース 4: 起点ノード不在 → 空 visited・exit 0（D 裁定、エラーではない）
#[test]
fn test_nonexistent_start_empty_ok() {
    // @ts: TS-LGX-005 ケース 4
    let graph = linear_abc();
    let result = investigate(&graph, &["NONEXISTENT-LGX-999".to_string()], None, 0.3)
        .expect("起点不在は Ok（Err ではない、D 裁定）");
    assert!(result.traversal.visited.is_empty());
    assert!(result.traversal.depth_map.is_empty());
    assert!(result.traversal.edges_traversed.is_empty());
    assert!(result.suspicious_nodes.is_empty());
}

// ケース 5: 空グラフ / 単一ノード孤立 → 起点のみ or 空
#[test]
fn test_empty_and_single_node() {
    // @ts: TS-LGX-005 ケース 5
    // (a) 空グラフ → 起点不在に収束 → 空
    let empty = TraceGraph::empty();
    let ra = investigate(&empty, &["X".to_string()], None, 0.3).expect("Ok");
    assert!(ra.traversal.visited.is_empty());

    // (b) 単一ノード孤立・起点 = その 1 ノード → visited 要素 1（depth 0）
    let single = TraceGraph::from_parts(vec![node("X", "UC", "x.md")], vec![]);
    let rb = investigate(&single, &["X".to_string()], None, 0.3).expect("Ok");
    assert_eq!(rb.traversal.visited.len(), 1);
    assert_eq!(rb.traversal.visited[0].id, "X");
    assert_eq!(rb.traversal.visited[0].depth, 0);
    assert!(rb.traversal.edges_traversed.is_empty());
    assert!(rb.suspicious_nodes.is_empty());
}

// ケース 6: 逆方向走査の方向性（to→from を辿る）
#[test]
fn test_reverse_direction_to_from() {
    // @ts: TS-LGX-005 ケース 6
    // A --chain--> B --chain--> C。起点 C。逆方向 → C(0), B(1), A(2)
    let graph = linear_abc();
    let result = investigate(&graph, &["C".to_string()], None, 0.3).expect("Ok");
    let ids: Vec<&str> = result.traversal.visited.iter().map(|v| v.id.as_str()).collect();
    assert!(ids.contains(&"C") && ids.contains(&"B") && ids.contains(&"A"));
    assert_eq!(result.traversal.depth_map.get("C"), Some(&0));
    assert_eq!(result.traversal.depth_map.get("B"), Some(&1));
    assert_eq!(result.traversal.depth_map.get("A"), Some(&2));
    // edges_traversed はグラフ向き (from,to) 表現で (B,C)/(A,B) を含む（DD §2.1 注）
    assert!(result
        .traversal
        .edges_traversed
        .contains(&("B".to_string(), "C".to_string())));
    assert!(result
        .traversal
        .edges_traversed
        .contains(&("A".to_string(), "B".to_string())));
}

// ケース 7: BFS 決定論性（property）— 同一入力 → 同一 visited 順・depth_map
proptest! {
    #[test]
    fn test_bfs_determinism(depth in 1usize..6) {
        // @ts: TS-LGX-005 ケース 7（property、生成器 = 長さ可変の線形逆チェーン）
        // ノード N0→N1→...→N{depth}（from→to）。逆方向起点 = 末尾。
        let mut nodes = Vec::new();
        let mut edges = Vec::new();
        for i in 0..=depth {
            nodes.push(node(&format!("N{i}"), "DD", &format!("n{i}.md")));
            if i > 0 {
                edges.push(chain(&format!("N{}", i - 1), &format!("N{i}")));
            }
        }
        let start = vec![format!("N{depth}")];
        let g1 = TraceGraph::from_parts(nodes.clone(), edges.clone());
        let g2 = TraceGraph::from_parts(nodes, edges);
        let r1 = investigate(&g1, &start, None, 0.3).expect("Ok");
        let r2 = investigate(&g2, &start, None, 0.3).expect("Ok");
        prop_assert_eq!(&r1.traversal.visited, &r2.traversal.visited);
        prop_assert_eq!(&r1.traversal.depth_map, &r2.traversal.depth_map);
        prop_assert_eq!(&r1.traversal.edges_traversed, &r2.traversal.edges_traversed);
    }
}

// ケース 8: DAG 破れ（サイクル）・self-loop でも有限停止
#[test]
fn test_cycle_and_selfloop_finite() {
    // @ts: TS-LGX-005 ケース 8
    // (a) サイクル A→B→C→A、起点 A
    let cyclic = TraceGraph::from_parts(
        vec![node("A", "UC", "a.md"), node("B", "DD", "b.md"), node("C", "SRC", "c.md")],
        vec![chain("A", "B"), chain("B", "C"), chain("C", "A")],
    );
    let ra = investigate(&cyclic, &["A".to_string()], None, 0.3).expect("有限停止 Ok");
    let count_a = ra.traversal.visited.iter().filter(|v| v.id == "A").count();
    assert_eq!(count_a, 1, "A は visited に 1 回のみ");

    // (b) self-loop X→X、起点 X
    let selfloop = TraceGraph::from_parts(
        vec![node("X", "UC", "x.md")],
        vec![chain("X", "X")],
    );
    let rb = investigate(&selfloop, &["X".to_string()], None, 0.3).expect("有限停止 Ok");
    let count_x = rb.traversal.visited.iter().filter(|v| v.id == "X").count();
    assert_eq!(count_x, 1, "self-loop の X は再出力されない");
}

// ケース 9: read-only 不変（graph を変更しない、複数回呼び出しで結果同一）
#[test]
fn test_read_only_invariance() {
    // @ts: TS-LGX-005 ケース 9（db ハッシュ不変は cli/Integration 委譲、本 crate はグラフ不変 + 結果同一）
    let graph = linear_abc();
    let before_nodes = graph.node_count();
    let r1 = investigate(&graph, &["C".to_string()], None, 0.3).expect("Ok");
    let r2 = investigate(&graph, &["C".to_string()], None, 0.3).expect("Ok");
    assert_eq!(graph.node_count(), before_nodes, "graph は不変（read-only 借用）");
    assert_eq!(r1, r2, "複数回呼び出しでも結果同一");
}

// ケース 10: db = None → suspicious_nodes 空・走査結果のみ（代替フロー 3a）
#[test]
fn test_db_none_suspicious_empty() {
    // @ts: TS-LGX-005 ケース 10
    let graph = linear_abc();
    let result = investigate(&graph, &["C".to_string()], None, 0.3).expect("Ok");
    assert!(!result.traversal.visited.is_empty(), "visited は通常通り");
    assert!(result.suspicious_nodes.is_empty(), "db=None で suspicious 空");
    assert_eq!(result.drift_threshold, 0.3);
}

// ケース 11: db = Some・DB 照会失敗 → suspicious_nodes 空・継続（NavError 昇格なし）
#[test]
fn test_db_query_failure_best_effort() {
    // @ts: TS-LGX-005 ケース 11
    // scores テーブル不在の in-memory db（rusqlite::Error を誘発）
    let conn = rusqlite::Connection::open_in_memory().expect("in-memory db");
    let graph = linear_abc();
    let result = investigate(&graph, &["C".to_string()], Some(&conn), 0.3)
        .expect("照会失敗でも Ok（NavError 昇格なし、ベストエフォート）");
    assert!(result.suspicious_nodes.is_empty(), "照会失敗で suspicious 空");
    assert!(!result.traversal.visited.is_empty(), "visited は通常通り");
}

// ケース 12: db = Some・drift 閾値判定 → suspicious_nodes 抽出（threshold 以上）
#[test]
fn test_drift_threshold_extraction() {
    // @ts: TS-LGX-005 ケース 12
    // visited 内 N1=0.5, N2=0.3, N3=0.1 を scores テーブルに投入、threshold=0.3
    let conn = rusqlite::Connection::open_in_memory().expect("in-memory db");
    conn.execute_batch(
        "CREATE TABLE scores (node_id TEXT, score_type TEXT, value REAL);
         INSERT INTO scores VALUES ('N1','drift',0.5);
         INSERT INTO scores VALUES ('N2','drift',0.3);
         INSERT INTO scores VALUES ('N3','drift',0.1);",
    )
    .expect("fixture scores");
    // N0→N1→N2→N3（from→to）、逆方向起点 N3 → visited に N3,N2,N1,N0
    let graph = TraceGraph::from_parts(
        vec![
            node("N0", "UC", "n0.md"),
            node("N1", "DD", "n1.md"),
            node("N2", "TS", "n2.md"),
            node("N3", "SRC", "n3.md"),
        ],
        vec![chain("N0", "N1"), chain("N1", "N2"), chain("N2", "N3")],
    );
    let result = investigate(&graph, &["N3".to_string()], Some(&conn), 0.3).expect("Ok");
    let ids: Vec<&str> = result.suspicious_nodes.iter().map(|s| s.id.as_str()).collect();
    assert!(ids.contains(&"N1"), "0.5 >= 0.3");
    assert!(ids.contains(&"N2"), "0.3 >= 0.3（境界含む）");
    assert!(!ids.contains(&"N3"), "drift 未付与 / なし");
    assert!(!ids.contains(&"N0"), "drift 未付与 / なし");
}

// ケース 13: suspicious_nodes の整列決定性（drift 降順・同値 id 昇順、property）
proptest! {
    #[test]
    fn test_suspicious_sort_determinism(
        scores in proptest::collection::vec((0u32..5, 0u32..3), 0..8)
    ) {
        // @ts: TS-LGX-005 ケース 13（property）
        // (id_index, drift_bucket) から SuspiciousNode を生成（drift 同値ペアを含む）。
        // db=None 経路は空を返すため、本ケースは DriftPruner の整列契約を直接検証する。
        // 整列対象を内部に構築し prune の整列出力に渡せないため、prune は db 経由のみ。
        // ここでは「同一入力 → 同一整列順」を investigate 全体の決定論として束縛する代理検証は
        // ケース 7 が担い、本ケースは整列規則（降順・id 昇順 stable）を期待値計算で確認する。
        let mut nodes: Vec<SuspiciousNode> = scores
            .iter()
            .enumerate()
            .map(|(i, (drift_b, _))| suspicious(&format!("S{:02}", i), *drift_b as f32 * 0.1))
            .collect();

        // 期待: drift_score 降順、同値内 id 昇順（stable）
        let mut expected = nodes.clone();
        expected.sort_by(|a, b| {
            b.drift_score
                .partial_cmp(&a.drift_score)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.id.cmp(&b.id))
        });

        // DriftPruner::prune の整列を委譲経路（db scores 投入）で検証する。
        let conn = rusqlite::Connection::open_in_memory().expect("in-memory db");
        conn.execute_batch("CREATE TABLE scores (node_id TEXT, score_type TEXT, value REAL);")
            .expect("schema");
        for n in &nodes {
            conn.execute(
                "INSERT INTO scores VALUES (?1,'drift',?2)",
                rusqlite::params![n.id, n.drift_score as f64],
            )
            .expect("insert");
        }
        // visited にすべての suspect を含めた traversal を組む（threshold=0 で全件抽出）
        let mut depth_map = IndexMap::new();
        let visited_nodes: Vec<VisitedNode> = nodes
            .iter()
            .enumerate()
            .map(|(i, n)| {
                depth_map.insert(n.id.clone(), i);
                visited(&n.id, &n.type_code, &n.path, i)
            })
            .collect();
        let traversal = MultiTraversalResult {
            visited: visited_nodes,
            edges_traversed: vec![],
            depth_map,
            start_ids: vec![],
        };
        let pruned = DriftPruner::prune(&traversal, Some(&conn), 0.0);
        let got_ids: Vec<String> = pruned.iter().map(|s| s.id.clone()).collect();
        let exp_ids: Vec<String> = expected.iter().map(|s| s.id.clone()).collect();
        prop_assert_eq!(got_ids, exp_ids);

        // sort が安定であることを nodes 自体でも確認（捏造防止の二重化）
        nodes.sort_by(|a, b| {
            b.drift_score
                .partial_cmp(&a.drift_score)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.id.cmp(&b.id))
        });
        prop_assert_eq!(nodes, expected);
    }
}

// ケース 14: render_pruned Text 書式（v3 互換）
#[test]
fn test_render_pruned_text() {
    // @ts: TS-LGX-005 ケース 14
    let result = pruned_fixture(
        vec![visited("N0", "UC", "n0.md", 0), visited("N1", "DD", "n1.md", 1)],
        vec![suspicious("S1", 0.5)],
        0.3,
    );
    let out = render_pruned(&result, ReportFormat::Text);
    assert!(out.contains("N0 (type=UC, depth=0, path=n0.md)"));
    assert!(out.contains("N1 (type=DD, depth=1, path=n1.md)"));
    assert!(out.contains("Suspicious nodes (drift_threshold=0.3):"));
    assert!(out.contains("S1 (drift=0.5"));
    assert!(out.contains("Summary: visited=2, suspicious=1"));
}

// ケース 15: render_pruned JsonLines 書式（--json 機能化）
#[test]
fn test_render_pruned_jsonlines() {
    // @ts: TS-LGX-005 ケース 15
    let result = pruned_fixture(
        vec![visited("N0", "UC", "n0.md", 0)],
        vec![suspicious("S1", 0.3)],
        0.3,
    );
    let out = render_pruned(&result, ReportFormat::JsonLines);
    // 各行が valid JSON。visited 行 / suspicious 行 / summary 行。
    for line in out.lines().filter(|l| !l.trim().is_empty()) {
        let parsed: Result<serde_json::Value, _> = serde_json::from_str(line);
        assert!(parsed.is_ok(), "各行は valid JSON: {line}");
    }
    assert!(out.contains("\"id\":\"N0\""));
    assert!(out.contains("\"depth\":0"));
    assert!(out.contains("\"suspicious\""));
    assert!(out.contains("\"summary\""));
    assert!(out.contains("\"visited\":1"));
    assert!(out.contains("\"drift_threshold\":0.3"));
}

// ケース 16: render_outcome の JsonLines summary truncated フラグ（打ち切り時のみ付加）
#[test]
fn test_render_outcome_truncated_flag() {
    // @ts: TS-LGX-005 ケース 16
    let base = pruned_fixture(vec![visited("N0", "UC", "n0.md", 0)], vec![], 0.3);

    // (a) truncated=false → render_pruned と完全一致
    let oa = InvestigateOutcome {
        result: base.clone(),
        truncated: false,
        excluded_count: 0,
    };
    let out_a = render_outcome(&oa, ReportFormat::JsonLines);
    let baseline = render_pruned(&base, ReportFormat::JsonLines);
    assert_eq!(out_a, baseline, "truncated=false 時は render_pruned と同一");
    assert!(!out_a.contains("truncated"), "truncated フィールドは付かない");

    // (b) truncated=true, excluded=4 → summary に "truncated":true,"excluded":4
    let ob = InvestigateOutcome {
        result: base,
        truncated: true,
        excluded_count: 4,
    };
    let out_b = render_outcome(&ob, ReportFormat::JsonLines);
    assert!(out_b.contains("\"truncated\":true"));
    assert!(out_b.contains("\"excluded\":4"));
}

// ケース 17(a): 終了コード契約 — 正常走査（起点不在含む）は exit 0 相当（Ok 返却）
#[test]
fn test_exit_contract_normal_is_ok() {
    // @ts: TS-LGX-005 ケース 17(a)
    // 本 crate 層では exit 0 = investigate が Ok を返すこと。exit 1/2 は cli 層委譲。
    let graph = linear_abc();
    assert!(investigate(&graph, &["C".to_string()], None, 0.3).is_ok());
    // 起点不在も Ok（exit 0、D 裁定）
    assert!(investigate(&graph, &["GHOST".to_string()], None, 0.3).is_ok());
    // 打ち切り発生も Ok（exit 0、REQ.04）
    assert!(investigate_with_depth(&graph, &["C".to_string()], None, 0.3, Some(0)).is_ok());

    // 17(b) GraphError → exit 1 / 17(c) clap 構文誤り → exit 2 は legixy-cli 層へ委譲。
}

// ケース 18: 出力先分離（stdout/stderr）→ legixy-cli E2E（cli_investigate.rs）へ委譲。
//   本 crate は render_pruned が文字列（stdout 内容）を返すことのみ保証（ケース 14/15）。
//   stderr Info/警告の実出力は cli/Integration 層が担う。

// ケース 19: MCP 非公開 → ts-mcp/test/tools.test.ts（MCP カタログ検査）へ委譲。本 crate 境界外。
