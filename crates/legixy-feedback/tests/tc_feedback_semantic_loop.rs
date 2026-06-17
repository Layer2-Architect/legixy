// R-4/R-5 回帰テスト: 意味層 → フィードバックループの接続。
// check 意味層（SPEC-LGX-004.REQ.02）が発行する SemanticSimilarity 所見は、message で
// 3 種に分かれる:
//   - リンク候補（未接続）: Info, related_ids 2 件  → observation category "link_candidate"（変換可: add_link）
//   - content drift:        Warning, related_ids 1 件 → "drift"（変換可: update_doc）
//   - 低類似度エッジ:        Warning, related_ids 2 件 → "semantic_similarity"（変換規則なし → skipped）
//   - 「embeddings 未生成」notice: Info, related_ids 0 件 → "semantic_similarity"（skipped）
// pre-fix では SemanticSimilarity を一律 "semantic_similarity" に潰し、link_candidate/drift を
// Observation 化できず Proposal 変換できなかった（analyzer 側の変換規則は実装済）→ RED。
//
// 親: SPEC-LGX-007.REQ.02（feedback は check の結果や embedding から observation 生成）/
//     REQ.08（chain_integrity/link_candidate/drift は変換可、orphan_file/semantic_similarity は skipped）/
//     DD-LGX-008 §2.2 FeedbackCategory・§3.1 AutoObserver/ProposalAnalyzer。

use legixy_check::{CheckCategory, CheckReport, CheckResult, Severity, SeverityCounts};
use legixy_core::Id;
use legixy_feedback::analyzer::ProposalAnalyzer;
use legixy_feedback::db::Connection;
use legixy_feedback::observer::AutoObserver;
use legixy_feedback::ObservationRecorder;

fn db() -> Connection {
    Connection::open_in_memory().expect("in-memory engine.db 接続")
}

fn semantic(severity: Severity, message: &str, ids: &[&str]) -> CheckResult {
    CheckResult {
        severity,
        category: CheckCategory::SemanticSimilarity,
        message: message.to_string(),
        related_ids: ids.iter().map(|s| Id::new(*s)).collect(),
        location: None,
    }
}

fn report(findings: Vec<CheckResult>) -> CheckReport {
    CheckReport {
        findings,
        counts: SeverityCounts::default(),
    }
}

/// check_semantic の 4 所見形を AutoObserver が正しい category へ振り分けること。
#[test]
fn semantic_findings_route_to_distinct_observation_categories() {
    let rep = report(vec![
        // リンク候補（Info, 2 ids）。
        semantic(
            Severity::Info,
            "リンク候補（未接続）: UC-LGX-001 ↔ UC-LGX-002 = 0.9000（≥ 閾値 0.70）",
            &["UC-LGX-001", "UC-LGX-002"],
        ),
        // content drift（Warning, 1 id）。
        semantic(
            Severity::Warning,
            "content drift: SPEC-LGX-001 の現内容が保存済 embedding と不一致（再 embed 推奨）",
            &["SPEC-LGX-001"],
        ),
        // 低類似度エッジ（Warning, 2 ids）。
        semantic(
            Severity::Warning,
            "低類似度エッジ: UC-LGX-001 → RBA-LGX-001 = 0.1000（< 閾値 0.40）",
            &["UC-LGX-001", "RBA-LGX-001"],
        ),
        // embeddings 未生成 notice（Info, 0 ids）。link_candidate に誤誘導してはならない。
        semantic(
            Severity::Info,
            "embeddings 未生成: `legixy embed --all` を実行してください（意味層は省略）",
            &[],
        ),
    ]);

    let obs = AutoObserver::from_check_results(&rep);
    let cats: Vec<&str> = obs.iter().map(|o| o.category.as_str()).collect();

    assert!(
        cats.contains(&"link_candidate"),
        "リンク候補 → link_candidate: {cats:?}"
    );
    assert!(cats.contains(&"drift"), "content drift → drift: {cats:?}");
    // 低類似度エッジ + embeddings 未生成 = semantic_similarity 2 件。
    assert_eq!(
        cats.iter().filter(|c| **c == "semantic_similarity").count(),
        2,
        "低類似度エッジ + embeddings未生成 → semantic_similarity 2 件: {cats:?}"
    );
    // 0 ids の notice が link_candidate に化けていない（誤った add_link 提案の防止）。
    let link = obs.iter().find(|o| o.category == "link_candidate").unwrap();
    assert_eq!(link.related_ids.len(), 2, "link_candidate は 2 id を保持");
}

/// link_candidate / drift observation が analyze で add_link / update_doc proposal へ変換されること。
#[test]
fn link_candidate_and_drift_observations_convert_to_proposals() {
    let db = db();
    let rep = report(vec![
        semantic(
            Severity::Info,
            "リンク候補（未接続）: UC-LGX-001 ↔ UC-LGX-002 = 0.9000（≥ 閾値 0.70）",
            &["UC-LGX-001", "UC-LGX-002"],
        ),
        semantic(
            Severity::Warning,
            "content drift: SPEC-LGX-001 の現内容が保存済 embedding と不一致（再 embed 推奨）",
            &["SPEC-LGX-001"],
        ),
        semantic(
            Severity::Warning,
            "低類似度エッジ: UC-LGX-001 → RBA-LGX-001 = 0.1000（< 閾値 0.40）",
            &["UC-LGX-001", "RBA-LGX-001"],
        ),
    ]);
    for o in AutoObserver::from_check_results(&rep) {
        ObservationRecorder::record(&o, &db).expect("record observation");
    }

    let proposals = ProposalAnalyzer::analyze(&db).expect("analyze");
    let kinds: Vec<&str> = proposals.iter().map(|p| p.kind.as_str()).collect();
    assert!(kinds.contains(&"add_link"), "link_candidate → add_link: {kinds:?}");
    assert!(kinds.contains(&"update_doc"), "drift → update_doc: {kinds:?}");
    // 低類似度エッジ（semantic_similarity）は変換規則なし → proposal 化されない。
    assert_eq!(proposals.len(), 2, "変換可能 2 件のみ proposal 化: {kinds:?}");
}
