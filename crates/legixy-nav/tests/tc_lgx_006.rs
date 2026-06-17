// Document ID: TC-LGX-006
// TC-LGX-006: 順方向探索（impact）のテストコード（TC[RED]）。
//
// 親 chain: TS-LGX-006 → 本 TC-LGX-006 → SRC-LGX-006。
// 各テストは TS-LGX-006 のケースを legixy-nav の凍結 API（DD-LGX-006 §3）に束縛する。
// SRC[GREEN] 未実装（impact / traverse_forward_multi / detect_truncation / emit_truncation_info /
// render_multi = todo!()）のため、それらを呼ぶテストは panic で失敗する（RED）。
// `cargo test -p legixy-nav --no-run` は通る（型・シグネチャ整合）。
//
// 委譲（テスト化不要、コメントで委譲先を記す）:
// - 純 BFS 走査・共有型・書式の正典検証 → TS-LGX-005 / TC-LGX-005（DD-005 正典）
// - ケース 21(graph ロード失敗 exit 1 / clap 構文 exit 2) → legixy-cli 層 / LGX-COMPAT-001 §3
// - ケース 22 E2E（graph.toml fixture ロード / cli ディスパッチ） → legixy-cli E2E（impact_e2e.rs）
// - 性能予算 PERF.02 → NFR-LGX-001 / criterion bench

use proptest::prelude::*;

use legixy_nav::{
    detect_truncation, emit_truncation_info, impact, render_multi, MultiTraverser, ReportFormat,
    TruncationInfo,
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

fn edge(from: &str, to: &str, kind: EdgeKind) -> Edge {
    Edge {
        from: from.to_string(),
        to: to.to_string(),
        kind,
    }
}

fn chain(from: &str, to: &str) -> Edge {
    edge(from, to, EdgeKind::Chain)
}

/// 線形 chain `A → B → C → D`（深度 0..3）。
fn linear_abcd() -> TraceGraph {
    TraceGraph::from_parts(
        vec![
            node("A", "UC", "a.md"),
            node("B", "RBA", "b.md"),
            node("C", "DD", "c.md"),
            node("D", "SRC", "d.md"),
        ],
        vec![chain("A", "B"), chain("B", "C"), chain("C", "D")],
    )
}

fn ids_of(visited: &[legixy_nav::VisitedNode]) -> Vec<String> {
    visited.iter().map(|v| v.id.clone()).collect()
}

fn id_set(visited: &[legixy_nav::VisitedNode]) -> std::collections::BTreeSet<String> {
    visited.iter().map(|v| v.id.clone()).collect()
}

// ケース 1: max_depth = Some(0) → 起点のみ（深度 0）
#[test]
fn test_max_depth_0_only_start() {
    // @ts: TS-LGX-006 ケース 1
    let graph = TraceGraph::from_parts(
        vec![node("UC-LGX-001", "UC", "uc.md"), node("RBA-LGX-001", "RBA", "rba.md")],
        vec![chain("UC-LGX-001", "RBA-LGX-001")],
    );
    let result = impact(&graph, &["UC-LGX-001".to_string()], Some(0)).expect("Ok");
    assert_eq!(result.visited.len(), 1);
    assert_eq!(result.visited[0].id, "UC-LGX-001");
    assert_eq!(result.visited[0].type_code, "UC");
    assert_eq!(result.visited[0].depth, 0);
    assert_eq!(result.depth_map.get("UC-LGX-001"), Some(&0));
    assert!(!result.depth_map.contains_key("RBA-LGX-001"));
}

// ケース 2: max_depth = None（無制限）と巨大値が一致
#[test]
fn test_none_and_huge_agree() {
    // @ts: TS-LGX-006 ケース 2
    let graph = linear_abcd();
    let none = impact(&graph, &["A".to_string()], None).expect("Ok");
    let huge = impact(&graph, &["A".to_string()], Some(usize::MAX)).expect("Ok");
    assert_eq!(none.visited, huge.visited);
    assert_eq!(none.depth_map, huge.depth_map);
    assert_eq!(id_set(&none.visited).len(), 4, "到達可能全 4 ノード");
}

// ケース 3: max_depth 境界打ち切り（深度 = max_depth まで含む、超過は除外）
#[test]
fn test_max_depth_boundary() {
    // @ts: TS-LGX-006 ケース 3
    let graph = linear_abcd();
    let result = impact(&graph, &["A".to_string()], Some(2)).expect("Ok");
    let set = id_set(&result.visited);
    assert_eq!(
        set,
        ["A", "B", "C"].iter().map(|s| s.to_string()).collect()
    );
    assert!(!set.contains("D"), "depth 3 の D は除外");
    assert_eq!(result.depth_map.get("A"), Some(&0));
    assert_eq!(result.depth_map.get("B"), Some(&1));
    assert_eq!(result.depth_map.get("C"), Some(&2));
    assert!(!result.depth_map.contains_key("D"));
}

// ケース 4: 存在しない起点 → 空結果・exit 0（NavError に昇格しない）
#[test]
fn test_nonexistent_start_empty() {
    // @ts: TS-LGX-006 ケース 4
    let graph = linear_abcd();
    let result = impact(&graph, &["XYZ-LGX-999".to_string()], None)
        .expect("起点不在は Ok（Err ではない、GAP-234）");
    assert!(result.visited.is_empty());
    assert!(result.depth_map.is_empty());
}

// ケース 5: 空 start_ids → 空結果
#[test]
fn test_empty_start_ids() {
    // @ts: TS-LGX-006 ケース 5
    let graph = linear_abcd();
    let result = impact(&graph, &[], None).expect("Ok");
    assert!(result.visited.is_empty());
    assert!(result.start_ids.is_empty());
}

// ケース 6: 空グラフ / 単一ノード（孤立）→ 起点 1 件相当
#[test]
fn test_empty_graph_and_single_node() {
    // @ts: TS-LGX-006 ケース 6
    // (a) 空グラフ → 起点不在 → 空
    let empty = TraceGraph::empty();
    let ra = impact(&empty, &["A".to_string()], None).expect("Ok");
    assert!(ra.visited.is_empty());

    // (b) 単一ノード孤立 → visited 1 件（depth 0）
    let single = TraceGraph::from_parts(vec![node("A", "UC", "a.md")], vec![]);
    let rb = impact(&single, &["A".to_string()], None).expect("Ok");
    assert_eq!(rb.visited.len(), 1);
    assert_eq!(rb.visited[0].id, "A");
    assert_eq!(rb.visited[0].depth, 0);
}

// ケース 7: 単起点 impact の委譲（traverse_forward_multi → 共有 BFS）
#[test]
fn test_impact_delegates_to_traverse_forward_multi() {
    // @ts: TS-LGX-006 ケース 7
    // 分岐 A→B, A→C, B→D（IndexMap 挿入順 B,C,D）
    let graph = TraceGraph::from_parts(
        vec![
            node("A", "UC", "a.md"),
            node("B", "DD", "b.md"),
            node("C", "DD", "c.md"),
            node("D", "SRC", "d.md"),
        ],
        vec![chain("A", "B"), chain("A", "C"), chain("B", "D")],
    );
    let via_impact = impact(&graph, &["A".to_string()], None).expect("Ok");
    let via_multi = MultiTraverser::traverse_forward_multi(&graph, &["A".to_string()], None);
    assert_eq!(via_impact.visited, via_multi.visited, "impact は委譲する");
    assert_eq!(ids_of(&via_impact.visited), vec!["A", "B", "C", "D"], "BFS = 挿入順");
}

// ケース 8: 多起点マージ（入力順 × 各起点 BFS 順、既出は追加しない）
#[test]
fn test_multi_start_merge() {
    // @ts: TS-LGX-006 ケース 8
    // A→C, B→C, B→E。起点 ["A","B"]
    let graph = TraceGraph::from_parts(
        vec![
            node("A", "UC", "a.md"),
            node("B", "UC", "b.md"),
            node("C", "DD", "c.md"),
            node("E", "DD", "e.md"),
        ],
        vec![chain("A", "C"), chain("B", "C"), chain("B", "E")],
    );
    let result = impact(&graph, &["A".to_string(), "B".to_string()], None).expect("Ok");
    assert_eq!(ids_of(&result.visited), vec!["A", "C", "B", "E"], "A の BFS → B の BFS（C 既出）");
    assert_eq!(result.start_ids, vec!["A".to_string(), "B".to_string()]);
}

// ケース 9: 多起点での depth_map は min(depth) 記録
#[test]
fn test_multi_start_min_depth() {
    // @ts: TS-LGX-006 ケース 9
    // A→X→T（T は A から深度 2）, B→T（T は B から深度 1）。起点 ["A","B"]
    let graph = TraceGraph::from_parts(
        vec![
            node("A", "UC", "a.md"),
            node("B", "UC", "b.md"),
            node("X", "DD", "x.md"),
            node("T", "SRC", "t.md"),
        ],
        vec![chain("A", "X"), chain("X", "T"), chain("B", "T")],
    );
    let result = impact(&graph, &["A".to_string(), "B".to_string()], None).expect("Ok");
    assert_eq!(result.depth_map.get("T"), Some(&1), "min(A 経由 2, B 経由 1) = 1");
}

// ケース 10: 不在起点を含む多起点 → 不在分を読み飛ばし
#[test]
fn test_multi_start_skip_missing() {
    // @ts: TS-LGX-006 ケース 10
    let graph = TraceGraph::from_parts(
        vec![node("A", "UC", "a.md"), node("B", "DD", "b.md")],
        vec![chain("A", "B")],
    );
    let result = impact(&graph, &["A".to_string(), "GHOST".to_string()], None).expect("Ok");
    let set = id_set(&result.visited);
    assert!(set.contains("A") && set.contains("B"), "A 起点の到達集合");
    assert!(!set.contains("GHOST"), "GHOST は欠落");
    assert_eq!(result.start_ids, vec!["A".to_string(), "GHOST".to_string()], "入力列は保持");
}

// ケース 11: 循環グラフでも有限停止（visited 抑止）→ impact 経路
#[test]
fn test_cycle_finite_stop() {
    // @ts: TS-LGX-006 ケース 11
    let graph = TraceGraph::from_parts(
        vec![node("A", "UC", "a.md"), node("B", "DD", "b.md"), node("C", "SRC", "c.md")],
        vec![chain("A", "B"), chain("B", "C"), chain("C", "A")],
    );
    let result = impact(&graph, &["A".to_string()], None).expect("有限停止 Ok");
    let set = id_set(&result.visited);
    assert_eq!(set, ["A", "B", "C"].iter().map(|s| s.to_string()).collect());
    assert_eq!(result.visited.iter().filter(|v| v.id == "A").count(), 1, "起点 A 再出力なし");
}

// ケース 12: custom / parent_child エッジは from→to 順方向のみ辿る
#[test]
fn test_edge_kinds_forward_only() {
    // @ts: TS-LGX-006 ケース 12
    // custom A→B, parent_child A→A.1, custom Z→A（逆向き）。起点 A
    let graph = TraceGraph::from_parts(
        vec![
            node("A", "UC", "a.md"),
            node("B", "DD", "b.md"),
            node("A.1", "UC", "a1.md"),
            node("Z", "SPEC", "z.md"),
        ],
        vec![
            edge("A", "B", EdgeKind::Custom),
            edge("A", "A.1", EdgeKind::ParentChild),
            edge("Z", "A", EdgeKind::Custom),
        ],
    );
    let result = impact(&graph, &["A".to_string()], None).expect("Ok");
    let set = id_set(&result.visited);
    assert_eq!(
        set,
        ["A", "B", "A.1"].iter().map(|s| s.to_string()).collect(),
        "from=A の順方向出エッジのみ"
    );
    assert!(!set.contains("Z"), "Z→A は to=A で逆方向、辿らない");
}

// ケース 13: detect_truncation 打ち切り発生（境界ノードの未訪問隣接をカウント）
#[test]
fn test_detect_truncation_truncated() {
    // @ts: TS-LGX-006 ケース 13
    // A(0)→B(1)→C(2), C→D, C→E（D/E は深度 3 で除外）
    let graph = TraceGraph::from_parts(
        vec![
            node("A", "UC", "a.md"),
            node("B", "DD", "b.md"),
            node("C", "TS", "c.md"),
            node("D", "SRC", "d.md"),
            node("E", "SRC", "e.md"),
        ],
        vec![chain("A", "B"), chain("B", "C"), chain("C", "D"), chain("C", "E")],
    );
    let result = impact(&graph, &["A".to_string()], Some(2)).expect("Ok");
    let info = detect_truncation(&graph, &result, 2);
    assert_eq!(
        info,
        TruncationInfo {
            truncated: true,
            excluded_count: 2,
            max_depth: 2
        },
        "境界 C の未訪問隣接 D, E の 2 件"
    );
}

// ケース 14: detect_truncation 打ち切りなし → truncated = false
#[test]
fn test_detect_truncation_none() {
    // @ts: TS-LGX-006 ケース 14
    // A(0)→B(1)→C(2)、深度 3 以降なし。C は出次数 0
    let graph = TraceGraph::from_parts(
        vec![node("A", "UC", "a.md"), node("B", "DD", "b.md"), node("C", "SRC", "c.md")],
        vec![chain("A", "B"), chain("B", "C")],
    );
    let result = impact(&graph, &["A".to_string()], Some(2)).expect("Ok");
    let info = detect_truncation(&graph, &result, 2);
    assert_eq!(
        info,
        TruncationInfo {
            truncated: false,
            excluded_count: 0,
            max_depth: 2
        }
    );
}

// ケース 15: emit_truncation_info は truncated 時のみ stderr 出力（stdout・exit 不変）
#[test]
fn test_emit_truncation_info() {
    // @ts: TS-LGX-006 ケース 15
    // 戻り値なしの副作用関数。本 crate ではパニックせず呼べることを束縛（stderr 内容検証は
    // cli/Integration 委譲）。SRC[GREEN] 未実装のため todo!() で RED。
    let info_true = TruncationInfo {
        truncated: true,
        excluded_count: 2,
        max_depth: 2,
    };
    emit_truncation_info(&info_true);
    let info_false = TruncationInfo {
        truncated: false,
        excluded_count: 0,
        max_depth: 2,
    };
    emit_truncation_info(&info_false);
}

// ケース 16: impact E2E → render_multi Text 書式（v3 互換）
#[test]
fn test_render_multi_text() {
    // @ts: TS-LGX-006 ケース 16
    let graph = linear_abcd();
    let result = impact(&graph, &["A".to_string()], None).expect("Ok");
    let out = render_multi(&result, ReportFormat::Text);
    assert!(out.contains("A (type=UC, depth=0, path=a.md)"));
    assert!(out.contains("Summary: visited=4"));
}

// ケース 17: impact E2E → render_multi JsonLines 書式（--json 機能化）
#[test]
fn test_render_multi_jsonlines() {
    // @ts: TS-LGX-006 ケース 17
    let graph = linear_abcd();
    let result = impact(&graph, &["A".to_string()], None).expect("Ok");
    let out = render_multi(&result, ReportFormat::JsonLines);
    for line in out.lines().filter(|l| !l.trim().is_empty()) {
        let parsed: Result<serde_json::Value, _> = serde_json::from_str(line);
        assert!(parsed.is_ok(), "各行は valid JSON: {line}");
    }
    assert!(out.contains("\"id\":\"A\""));
    assert!(out.contains("\"depth\":0"));
    assert!(out.contains("\"summary\""));
    assert!(out.contains("\"visited\":4"));
}

// ケース 18: 起点不在の impact 結果に render_multi → 空表示・summary 0
#[test]
fn test_render_multi_empty() {
    // @ts: TS-LGX-006 ケース 18
    let graph = linear_abcd();
    let result = impact(&graph, &["GHOST".to_string()], None).expect("Ok");
    let text = render_multi(&result, ReportFormat::Text);
    assert!(text.contains("Summary: visited=0"));
    let json = render_multi(&result, ReportFormat::JsonLines);
    assert!(json.contains("\"visited\":0"));
}

// ケース 19: BFS visited 順・depth_map の決定論性（property）
proptest! {
    #[test]
    fn test_impact_determinism(
        branch in 1usize..5,
        depth in 1usize..5,
        max_depth_opt in proptest::option::of(0usize..6)
    ) {
        // @ts: TS-LGX-006 ケース 19（property、生成器 = ランダム幅×深さの DAG + 任意 max_depth）
        // 起点 R から幅 branch の木を深さ depth まで生成（決定論順序の検証用）。
        let mut nodes = vec![node("R", "UC", "r.md")];
        let mut edges = Vec::new();
        let mut frontier = vec!["R".to_string()];
        let mut counter = 0usize;
        for _ in 0..depth {
            let mut next = Vec::new();
            for parent in &frontier {
                for _ in 0..branch {
                    let id = format!("N{counter}");
                    counter += 1;
                    nodes.push(node(&id, "DD", &format!("{id}.md")));
                    edges.push(chain(parent, &id));
                    next.push(id);
                }
            }
            frontier = next;
        }
        let g1 = TraceGraph::from_parts(nodes.clone(), edges.clone());
        let g2 = TraceGraph::from_parts(nodes, edges);
        let r1 = impact(&g1, &["R".to_string()], max_depth_opt).expect("Ok");
        let r2 = impact(&g2, &["R".to_string()], max_depth_opt).expect("Ok");
        prop_assert_eq!(&r1.visited, &r2.visited);
        prop_assert_eq!(&r1.depth_map, &r2.depth_map);
    }
}

// ケース 20: impact の read-only 不変（グラフを変更しない）
#[test]
fn test_impact_read_only() {
    // @ts: TS-LGX-006 ケース 20
    let graph = linear_abcd();
    let before = graph.node_count();
    let r1 = impact(&graph, &["A".to_string()], None).expect("Ok");
    let r2 = impact(&graph, &["A".to_string()], None).expect("Ok");
    assert_eq!(graph.node_count(), before, "graph 不変（read-only 借用）");
    assert_eq!(r1.visited, r2.visited, "複数回呼び出しでも副作用なし");
}

// ケース 21: 終了コード契約 0（正常 / 起点不在 / 打ち切り）の確認（contract）
#[test]
fn test_exit_contract_ok() {
    // @ts: TS-LGX-006 ケース 21
    // 本 crate 層では exit 0 = impact が Ok を返すこと。exit 1(graph ロード失敗)/2(clap) は cli 委譲。
    let graph = linear_abcd();
    assert!(impact(&graph, &["A".to_string()], None).is_ok(), "(a) 到達ノードあり");
    assert!(impact(&graph, &["GHOST".to_string()], None).is_ok(), "(b) 起点不在=空結果");
    assert!(impact(&graph, &["A".to_string()], Some(1)).is_ok(), "(c) 打ち切り発生も Ok");
}

// ケース 22: impact E2E（graph.toml fixture）→ visited 正確・打ち切り stderr・--json
//   graph.toml ロード + cli ディスパッチ + stderr 観察を伴う E2E は legixy-cli 層
//   （crates/legixy-cli/tests/impact_e2e.rs）へ委譲（DD-006 §1 cli 層境界外、TS-006 §1 委譲）。
//   本 crate ではプログラム構築グラフでの impact→detect_truncation→render の連鎖を
//   ケース 13/16/17 が分担して検証する。

// ケース 23: edges_traversed の内容直接 assert（spanning-tree・初訪問エッジ・(from,to)・多起点蓄積）
#[test]
fn test_edges_traversed_content() {
    // @ts: TS-LGX-006 ケース 23
    // (a) A→B, B→C, A→C（挿入順 A→B, A→C が B→C より先。C は深度 1 の A→C で初訪問）
    let graph = TraceGraph::from_parts(
        vec![node("A", "UC", "a.md"), node("B", "DD", "b.md"), node("C", "SRC", "c.md")],
        vec![chain("A", "B"), chain("B", "C"), chain("A", "C")],
    );
    let ra = impact(&graph, &["A".to_string()], None).expect("Ok");
    assert_eq!(
        ra.edges_traversed,
        vec![("A".to_string(), "B".to_string()), ("A".to_string(), "C".to_string())],
        "初訪問エッジのみ・既訪問 C への B→C は含まない・グラフ向き (from,to)"
    );

    // (b) graph2 = A→B, B→C, A→C, D→E。多起点 ["A","D"] → 第 2 起点 D の初訪問エッジを蓄積
    let graph2 = TraceGraph::from_parts(
        vec![
            node("A", "UC", "a.md"),
            node("B", "DD", "b.md"),
            node("C", "SRC", "c.md"),
            node("D", "UC", "d.md"),
            node("E", "DD", "e.md"),
        ],
        vec![chain("A", "B"), chain("B", "C"), chain("A", "C"), chain("D", "E")],
    );
    let rb = impact(&graph2, &["A".to_string(), "D".to_string()], None).expect("Ok");
    assert_eq!(
        rb.edges_traversed,
        vec![
            ("A".to_string(), "B".to_string()),
            ("A".to_string(), "C".to_string()),
            ("D".to_string(), "E".to_string()),
        ],
        "第 1 起点 A の spanning-tree に続いて第 2 起点 D の初訪問エッジを蓄積"
    );
}
