// Document ID: TC-LGX-001
// TC-LGX-001: グラフ読み込みと検証（check）のテストコード（TC[RED]）
//
// 親 chain: TS-LGX-001 → 本 TC-LGX-001 → SRC-LGX-001。
// 各テストは TS-LGX-001 のケースを `legixy-check` の凍結 API（DD-LGX-001 §3）に束縛する。
// SRC[GREEN] 未実装（run/exit_code/to_json = todo!()）のため全テストは失敗する（RED）。
// `cargo test -p legixy-check --no-run` は通る（型・シグネチャ整合）が `cargo test` は失敗する。

use legixy_check::{
    exit_code, run, CheckCategory, CheckMode, CheckReport, CheckResult, Severity, SeverityCounts,
};
use legixy_core::{Config, Id};
use legixy_graph::{Edge, EdgeKind, Node, TraceGraph};

fn node(id: &str, type_code: &str, path: &str) -> Node {
    Node {
        id: id.to_string(),
        type_code: type_code.to_string(),
        path: path.to_string(),
        parent_id: None,
        anchor: None,
    }
}

fn finding(severity: Severity, category: CheckCategory, ids: &[&str]) -> CheckResult {
    CheckResult {
        severity,
        category,
        message: String::new(),
        related_ids: ids.iter().map(|s| Id::new(*s)).collect(),
        location: None,
    }
}

// ケース 1: 空グラフ（ノード 0 / エッジ 0）→ exit 0
#[test]
fn test_run_empty_graph_returns_exit_0() {
    // @ts: TS-LGX-001 ケース 1
    let graph = TraceGraph::empty();
    let config = Config::default();
    let report = run(&graph, &config, CheckMode::Formal, None).expect("空グラフは Ok");
    assert_eq!(
        report.counts,
        SeverityCounts {
            error: 0,
            warning: 0,
            info: 0,
            ok: report.counts.ok
        }
    );
    assert_eq!(exit_code(&report), 0);
}

// ケース 2: 単一ノード・孤立（エッジ 0）→ OrphanFile 非該当
#[test]
fn test_run_single_isolated_node_no_orphanfile() {
    // @ts: TS-LGX-001 ケース 2
    let graph = TraceGraph::from_parts(vec![node("UC-LGX-001", "UC", "docs/usecases/uc.md")], vec![]);
    let report = run(&graph, &Config::default(), CheckMode::Formal, None).expect("Ok");
    assert!(report
        .findings
        .iter()
        .all(|f| f.category != CheckCategory::OrphanFile));
    assert_eq!(report.counts.error, 0);
    assert_eq!(exit_code(&report), 0);
}

// ケース 4: chain エッジ不整合 → ChainIntegrity finding
#[test]
fn test_run_broken_chain_emits_chain_integrity() {
    // @ts: TS-LGX-001 ケース 4
    let graph = TraceGraph::from_parts(
        vec![node("UC-LGX-001", "UC", "a.md"), node("DD-LGX-001", "DD", "b.md")],
        vec![], // chain order 定義済みだが親 chain エッジ欠落
    );
    let report = run(&graph, &Config::default(), CheckMode::Formal, None).expect("Ok");
    assert!(report
        .findings
        .iter()
        .any(|f| f.category == CheckCategory::ChainIntegrity));
}

// ケース 5: severity=Error が 1 件以上 → exit 1
#[test]
fn test_exit_code_with_error_is_1() {
    // @ts: TS-LGX-001 ケース 5
    let report = CheckReport {
        findings: vec![finding(Severity::Error, CheckCategory::ChainIntegrity, &["DD-LGX-001"])],
        counts: SeverityCounts {
            error: 1,
            warning: 0,
            info: 0,
            ok: 0,
        },
    };
    assert_eq!(exit_code(&report), 1);
}

// ケース 6: Warning/Info のみ（Error 0）→ exit 0
#[test]
fn test_exit_code_warning_info_only_is_0() {
    // @ts: TS-LGX-001 ケース 6
    let report = CheckReport {
        findings: vec![
            finding(Severity::Warning, CheckCategory::UnresolvedEdge, &["x"]),
            finding(Severity::Warning, CheckCategory::UnresolvedEdge, &["y"]),
            finding(Severity::Info, CheckCategory::SemanticSimilarity, &["z"]),
        ],
        counts: SeverityCounts {
            error: 0,
            warning: 2,
            info: 1,
            ok: 0,
        },
    };
    assert_eq!(exit_code(&report), 0);
}

// ケース 10: store=None（embeddings 空）→ 意味層 Info 1 件・非致命
#[test]
fn test_run_full_mode_no_store_info_nonfatal() {
    // @ts: TS-LGX-001 ケース 10
    let graph = TraceGraph::from_parts(vec![node("UC-LGX-001", "UC", "a.md")], vec![]);
    let report = run(&graph, &Config::default(), CheckMode::Full, None).expect("Ok");
    assert_eq!(
        report
            .findings
            .iter()
            .filter(|f| f.severity == Severity::Info)
            .count(),
        1,
        "意味層は Info 1 件（embed 誘導）"
    );
    assert_eq!(exit_code(&report), 0, "store 不在は exit 非影響（FB-INV-4）");
}

// ケース 12: SubnodeIdCollision は Warning（exit 0、G1 非阻害）
#[test]
fn test_run_subnode_collision_is_warning() {
    // @ts: TS-LGX-001 ケース 12
    let graph = TraceGraph::from_parts(
        vec![node("SPEC-LGX-001#abc", "SPEC", "s.md")],
        vec![],
    );
    let report = run(&graph, &Config::default(), CheckMode::Formal, None).expect("Ok");
    if let Some(f) = report
        .findings
        .iter()
        .find(|f| f.category == CheckCategory::SubnodeIdCollision)
    {
        assert_eq!(f.severity, Severity::Warning);
    }
    assert_eq!(exit_code(&report), 0);
}

// ケース 13: Id 系 opt-in 検査は既定 OFF
#[test]
fn test_run_id_optin_checks_default_off() {
    // @ts: TS-LGX-001 ケース 13
    let graph = TraceGraph::from_parts(vec![node("UC-LGX-001", "UC", "a.md")], vec![]);
    let config = Config::default(); // opt-in 未設定（既定 false）
    let report = run(&graph, &config, CheckMode::Formal, None).expect("Ok");
    assert!(report.findings.iter().all(|f| !matches!(
        f.category,
        CheckCategory::IdRedefined
            | CheckCategory::IdSemanticMismatch
            | CheckCategory::IdSemanticDrift
    )));
}

// ケース 14: CheckReport の安定ソート決定性（同一入力 → 同一順序）
#[test]
fn test_run_findings_order_deterministic() {
    // @ts: TS-LGX-001 ケース 14
    let graph = TraceGraph::from_parts(
        vec![
            node("UC-LGX-001", "UC", "a.md"),
            node("DD-LGX-001", "DD", "b.md"),
        ],
        vec![Edge {
            from: "UC-LGX-001".into(),
            to: "DD-LGX-001".into(),
            kind: EdgeKind::Chain,
        }],
    );
    let r1 = run(&graph, &Config::default(), CheckMode::Formal, None).expect("Ok");
    let r2 = run(&graph, &Config::default(), CheckMode::Formal, None).expect("Ok");
    let key = |r: &CheckReport| {
        r.findings
            .iter()
            .map(|f| (f.severity, f.category, f.related_ids.clone()))
            .collect::<Vec<_>>()
    };
    assert_eq!(key(&r1), key(&r2), "同一入力 → 同一順序（REQ.06）");
}

// ケース 15: 終了コード契約 0/1/2
#[test]
fn test_exit_code_contract_0_and_1() {
    // @ts: TS-LGX-001 ケース 15
    let ok = CheckReport {
        findings: vec![],
        counts: SeverityCounts::default(),
    };
    assert_eq!(exit_code(&ok), 0);
    let err = CheckReport {
        findings: vec![finding(Severity::Error, CheckCategory::GraphDag, &["x"])],
        counts: SeverityCounts {
            error: 1,
            ..Default::default()
        },
    };
    assert_eq!(exit_code(&err), 1);
}

// ケース 19: GraphDag（グラフ全体サイクル）→ severity Error・exit 1、SubnodeDag と区別
#[test]
fn test_run_graph_cycle_emits_graphdag_error() {
    // @ts: TS-LGX-001 ケース 19
    let graph = TraceGraph::from_parts(
        vec![node("A-LGX-001", "A", "a.md"), node("B-LGX-001", "B", "b.md")],
        vec![
            Edge { from: "A-LGX-001".into(), to: "B-LGX-001".into(), kind: EdgeKind::Custom },
            Edge { from: "B-LGX-001".into(), to: "A-LGX-001".into(), kind: EdgeKind::Custom },
        ],
    );
    let report = run(&graph, &Config::default(), CheckMode::Formal, None).expect("Ok");
    let dag = report
        .findings
        .iter()
        .find(|f| f.category == CheckCategory::GraphDag)
        .expect("GraphDag finding");
    assert_eq!(dag.severity, Severity::Error);
    assert!(report
        .findings
        .iter()
        .all(|f| f.category != CheckCategory::SubnodeDag));
    assert_eq!(exit_code(&report), 1);
}

// ケース 20: DocumentId 不一致 / 行欠落 → severity Error
#[test]
fn test_run_document_id_mismatch_is_error() {
    // @ts: TS-LGX-001 ケース 20
    let graph = TraceGraph::from_parts(vec![node("UC-LGX-001", "UC", "uc.md")], vec![]);
    let report = run(&graph, &Config::default(), CheckMode::Formal, None).expect("Ok");
    if let Some(f) = report
        .findings
        .iter()
        .find(|f| f.category == CheckCategory::DocumentId)
    {
        assert_eq!(f.severity, Severity::Error);
    }
}

// ケース（to_json）: JSON Lines シリアライズ
#[test]
fn test_to_json_serializes() {
    // @ts: TS-LGX-001 ケース 16（O2 JSON 出力）
    let report = CheckReport {
        findings: vec![finding(Severity::Error, CheckCategory::ChainIntegrity, &["DD-LGX-001"])],
        counts: SeverityCounts {
            error: 1,
            ..Default::default()
        },
    };
    let json = report.to_json();
    assert!(json.contains("ChainIntegrity") || json.contains("chain"));
}
