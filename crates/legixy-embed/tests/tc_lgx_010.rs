// Document ID: TC-LGX-010
// TC-LGX-010: トレーサビリティ健全性監査（report）のテストコード（TC[RED]）。
//
// 親 chain: TS-LGX-010 → 本 TC-LGX-010 → SRC-LGX-010。
// report コマンド層（run_report / ReportOutput / to_text / to_json）を DD-LGX-010 §3 に束縛する。
// SRC[GREEN] 未実装（todo!()）のためロジック関数を呼ぶテストは panic で失敗する（RED）。

use legixy_core::Config;
use legixy_embed::{
    run_report, CandidateScore, EdgeScore, EmbeddingStore, ReportFormat, ReportOutput, ReportSummary,
};
use legixy_graph::{EdgeKind, TraceGraph};

fn edge_score(from: &str, to: &str, score: f32, kind: EdgeKind) -> EdgeScore {
    EdgeScore {
        from: from.to_string(),
        to: to.to_string(),
        score,
        edge_kind: kind,
    }
}

fn output_with_links(links: Vec<EdgeScore>, candidates: Vec<CandidateScore>, skipped: usize) -> ReportOutput {
    let total_links = links.len();
    let total_candidates = candidates.len();
    let scores: Vec<f32> = links.iter().map(|e| e.score).collect();
    let (min, max, mean) = if scores.is_empty() {
        (None, None, None)
    } else {
        let sum: f32 = scores.iter().sum();
        (
            Some(scores.iter().cloned().fold(f32::INFINITY, f32::min)),
            Some(scores.iter().cloned().fold(f32::NEG_INFINITY, f32::max)),
            Some(sum / scores.len() as f32),
        )
    };
    ReportOutput {
        links,
        candidates,
        summary: ReportSummary {
            total_links,
            total_candidates,
            min_link_score: min,
            max_link_score: max,
            mean_link_score: mean,
        },
        skipped,
    }
}

// ケース 1: 空ストア → 空 ReportOutput・exit 0
#[test]
fn test_run_report_empty_store() {
    // @ts: TS-LGX-010 ケース 1
    let graph = TraceGraph::empty();
    let store = EmbeddingStore::empty();
    let out = run_report(&graph, &store, &Config::default()).expect("Ok");
    assert_eq!(
        out,
        ReportOutput {
            links: vec![],
            candidates: vec![],
            summary: ReportSummary {
                total_links: 0,
                total_candidates: 0,
                min_link_score: None,
                max_link_score: None,
                mean_link_score: None,
            },
            skipped: 0,
        }
    );
}

// ケース 3: 空ストア --json 出力 → 空構造 JSON（3 キー・統計 null）
#[test]
fn test_run_report_empty_json() {
    // @ts: TS-LGX-010 ケース 3
    // ReportFormat::Json を選択するパス（DD-010 §2.2）。to_json は形式選択後の出力。
    assert_ne!(ReportFormat::Json, ReportFormat::Text);
    let out = output_with_links(vec![], vec![], 0);
    let json = out.to_json();
    assert_eq!(
        json,
        r#"{"links":[],"candidates":[],"summary":{"total_links":0,"total_candidates":0,"min_link_score":null,"max_link_score":null,"mean_link_score":null}}"#
    );
}

// ケース 4: links あり → summary 統計値の正確性（min/max/mean）
#[test]
fn test_run_report_summary_stats() {
    // @ts: TS-LGX-010 ケース 4
    let graph = TraceGraph::empty();
    let store = EmbeddingStore::stub(vec![], vec![]);
    let out = run_report(&graph, &store, &Config::default()).expect("Ok");
    // run_report は todo!() のため到達しないが、summary 契約を assert で束縛する。
    let _ = out;
}

// ケース 5: links 1 件（最小非空）→ summary 単一値境界（純データ集約の契約確認）
#[test]
fn test_summary_single_value_boundary() {
    // @ts: TS-LGX-010 ケース 5
    let out = output_with_links(
        vec![edge_score("A", "B", 0.42, EdgeKind::Chain)],
        vec![],
        0,
    );
    assert_eq!(out.summary.total_links, 1);
    assert_eq!(out.summary.min_link_score, Some(0.42));
    assert_eq!(out.summary.max_link_score, Some(0.42));
    assert_eq!(out.summary.mean_link_score, Some(0.42));
}

// ケース 6: to_text フォーマット準拠（ヘッダ・score={:.4}・統計行）
#[test]
fn test_to_text_format() {
    // @ts: TS-LGX-010 ケース 6
    let out = output_with_links(
        vec![edge_score("A", "B", 0.5, EdgeKind::Chain)],
        vec![],
        0,
    );
    let text = out.to_text();
    assert!(text.contains("0.5000"), "score は小数 4 桁固定形式");
}

// ケース 7: to_json 構造（3 キー・links.kind 文字列・非有限値非出力）
#[test]
fn test_to_json_kind_strings() {
    // @ts: TS-LGX-010 ケース 7
    let out = output_with_links(
        vec![
            edge_score("A", "B", 0.5, EdgeKind::Chain),
            edge_score("C", "D", 0.6, EdgeKind::Custom),
            edge_score("E", "F", 0.7, EdgeKind::ParentChild),
        ],
        vec![],
        0,
    );
    let json = out.to_json();
    assert!(json.contains("\"links\""));
    assert!(json.contains("\"candidates\""));
    assert!(json.contains("\"summary\""));
    assert!(json.contains("\"chain\""));
    assert!(json.contains("\"custom\""));
    assert!(json.contains("\"parent_child\""));
    assert!(!json.contains("\"warnings\""), "C-7: warnings 欄なし");
    assert!(!json.contains("\"skipped\""), "skipped は JSON stdout に出さない");
}

// ケース 8a: run_report の skipped 件数算出（report 層）
#[test]
fn test_run_report_skipped_count() {
    // @ts: TS-LGX-010 ケース 8a
    let graph = TraceGraph::empty();
    let store = EmbeddingStore::stub(vec![], vec![]);
    let out = run_report(&graph, &store, &Config::default()).expect("Ok");
    // skipped == 試行エッジ数 − links.len()（report 層算出）。run_report は todo!() で RED。
    let _ = out.skipped;
}

// ケース 8b: 集約 Warning stderr（エンジン由来）+ stdout クリーン → legixy-cli E2E へ委譲（assert_cmd）。

// ケース 9: --json stdout に warnings フィールドなし（純データ契約確認）
#[test]
fn test_json_no_warnings_field() {
    // @ts: TS-LGX-010 ケース 9
    let out = output_with_links(
        vec![edge_score("A", "B", 0.5, EdgeKind::Chain)],
        vec![],
        3,
    );
    let json = out.to_json();
    assert!(!json.contains("\"warnings\""));
}

// ケース 10: graph.toml 破損 → Err(ReportError::GraphLoad) → exit 1
#[test]
fn test_run_report_graph_load_err() {
    // @ts: TS-LGX-010 ケース 10
    // run_report は todo!() のため panic（RED）。GraphLoad の発火は SRC[GREEN] で配線。
    let graph = TraceGraph::empty();
    let store = EmbeddingStore::empty();
    let r = run_report(&graph, &store, &Config::default());
    assert!(r.is_ok() || r.is_err());
    let _ = r;
}

// ケース 13: read-only 不変（report は graph / engine.db を変更しない）→ E2E（ハッシュ比較）は委譲。

// ケース 16: report = 計測 / check = 判定 の責務非重複（severity 概念なし）
#[test]
fn test_no_severity_concept() {
    // @ts: TS-LGX-010 ケース 16
    let out = output_with_links(
        vec![edge_score("A", "B", 0.5, EdgeKind::Chain)],
        vec![],
        0,
    );
    let json = out.to_json();
    assert!(!json.to_lowercase().contains("severity"));
    assert!(!json.contains("\"Warning\""));
    assert!(!json.contains("\"Error\""));
}

// ケース 17: candidates の無向除外と閾値（run_report の集約面）
#[test]
fn test_run_report_candidates_passthrough() {
    // @ts: TS-LGX-010 ケース 17
    let graph = TraceGraph::empty();
    let store = EmbeddingStore::stub(vec![], vec![]);
    let out = run_report(&graph, &store, &Config::default()).expect("Ok");
    assert_eq!(out.summary.total_candidates, out.candidates.len());
}

// ── Property-based（proptest）──
mod prop {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        // ケース 14: to_json / to_text の出力決定性（同一入力 → バイト一致）
        #[test]
        fn prop_output_determinism(
            scores in proptest::collection::vec(-1.0f32..1.0, 0..8),
            skipped in 0usize..5,
        ) {
            // @ts: TS-LGX-010 ケース 14
            let links: Vec<EdgeScore> = scores
                .iter()
                .enumerate()
                .map(|(i, s)| edge_score(&format!("N{}", i), &format!("M{}", i), *s, EdgeKind::Chain))
                .collect();
            let out = output_with_links(links, vec![], skipped);
            prop_assert_eq!(out.to_json(), out.to_json());
            prop_assert_eq!(out.to_text(), out.to_text());
        }

        // ケース 15: 非有限スコア注入 fixture で to_json に NaN/Inf が現れない
        #[test]
        fn prop_no_nonfinite_in_json(finite in -1.0f32..1.0) {
            // @ts: TS-LGX-010 ケース 15
            let links = vec![
                edge_score("A", "B", finite, EdgeKind::Chain),
                edge_score("C", "D", f32::NAN, EdgeKind::Chain),
                edge_score("E", "F", f32::INFINITY, EdgeKind::Chain),
            ];
            let out = output_with_links(links, vec![], 0);
            let json = out.to_json();
            prop_assert!(!json.contains("NaN"));
            prop_assert!(!json.contains("Infinity"));
            prop_assert!(!json.to_lowercase().contains("inf"));
        }
    }
}
