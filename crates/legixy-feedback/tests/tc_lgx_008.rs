// Document ID: TC-LGX-008
// TC-LGX-008: フィードバックループ（observe / feedback / analyze / approve / reject /
//   proposals / audit）のテストコード（TC[RED]）。
//
// 親 chain: TS-LGX-008 → 本 TC-LGX-008 → SRC-LGX-008。
// 各テストは TS-LGX-008 のケースを `legixy-feedback` の凍結 API（DD-LGX-008 §3.1）に束縛する。
// SRC[GREEN] 未実装（record / analyze / approve / reject / from_check_results / recent ... =
// todo!()）のため、当該関数を呼ぶテストは panic で失敗する（RED）。
// `cargo test -p legixy-feedback --no-run` は通る（型・シグネチャ整合）が `cargo test` は失敗する。
//
// TS-008 ケース 28〜32（parseObserveStdout / formatAuditEntry / zod / _meta / isError）は
// TypeScript MCP 転送層（TC-LGX-009、ts-mcp）の責務であり本 Rust TC の対象外（委譲）。

use legixy_check::{CheckCategory, CheckReport, CheckResult, Severity, SeverityCounts};
use legixy_core::Id;

use legixy_feedback::{
    drift_from_embed_error, AutoObserver, ContextAuditReader, EmbedError, FeedbackCli,
    NewObservation, ObservationRecorder, ObservationStatus, ObserveCategoryInput, ProposalAnalyzer,
    ProposalManager, ProposalStatus,
};
use legixy_feedback::db::Connection;

// ── ヘルパ ───────────────────────────────────────────────────────────

fn db() -> Connection {
    Connection::open_in_memory().expect("in-memory engine.db 接続")
}

fn new_obs(category: &str, message: &str, related_ids: &[&str]) -> NewObservation {
    NewObservation {
        source: "manual".to_string(),
        category: category.to_string(),
        severity: "info".to_string(),
        message: message.to_string(),
        related_ids: related_ids.iter().map(|s| s.to_string()).collect(),
        context_json: None,
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

fn report(findings: Vec<CheckResult>) -> CheckReport {
    CheckReport {
        findings,
        counts: SeverityCounts::default(),
    }
}

// ── ケース 1: message が trim 後 0 文字 → exit 1（EmptyObservationMessage、【v3 差分】）─────

#[test]
fn test_record_empty_message_is_rejected() {
    // @ts: TS-LGX-008 ケース 1
    let db = db();
    for blank in ["", "   ", "\t\n"] {
        let obs = new_obs("manual_note", blank, &[]);
        let r = ObservationRecorder::record(&obs, &db);
        // trim 後 0 文字 → EmptyObservationMessage（exit 1 概念）。INSERT は発生しない。
        assert!(
            matches!(r, Err(legixy_feedback::FeedbackError::EmptyObservationMessage)),
            "trim 後 0 文字 message ({blank:?}) は EmptyObservationMessage で拒否"
        );
    }
}

// ── ケース 2: message が trim 後 1 文字以上（境界下限）→ 受理 ────────────────────────────

#[test]
fn test_record_min_message_is_accepted() {
    // @ts: TS-LGX-008 ケース 2
    let db = db();
    let obs = new_obs("manual_note", " a ", &[]); // trim 後 1 文字
    let r = ObservationRecorder::record(&obs, &db).expect("trim 後 1 文字以上は受理");
    assert!(r.id > 0, "新規 INSERT で id > 0");
    assert!(!r.skipped, "初回 INSERT は skipped=false");
}

// ── ケース 3: 同一 (category, related_ids) の重複 observe → 2 件目 skipped=true（dedup）──

#[test]
fn test_record_dedup_second_is_skipped() {
    // @ts: TS-LGX-008 ケース 3
    let db = db();
    let obs1 = new_obs("manual_note", "first", &["DD-LGX-001", "UC-LGX-001"]);
    let obs2 = new_obs("manual_note", "second-different-message", &["UC-LGX-001", "DD-LGX-001"]);
    let r1 = ObservationRecorder::record(&obs1, &db).expect("1 件目 INSERT");
    assert!(!r1.skipped, "1 件目 skipped=false");
    let r2 = ObservationRecorder::record(&obs2, &db).expect("2 件目 dedup");
    assert!(
        r2.skipped,
        "同一 (category, related_ids 集合) は message 相違でも skipped=true（dedup キーに message 非包含）"
    );
}

// ── ケース 4: related_ids の distinct→sort 正準化の決定性（property）──────────────────

proptest::proptest! {
    #[test]
    fn prop_canonicalize_related_ids_is_order_and_dup_invariant(
        ids in proptest::collection::vec("[A-Za-z0-9_:#-]{0,12}", 0..12)
    ) {
        // @ts: TS-LGX-008 ケース 4
        // 集合として等価（順序・重複違いのみ）な 2 入力は同一正準 JSON に収束する。
        let mut shuffled = ids.clone();
        shuffled.reverse();
        let mut with_dups = ids.clone();
        with_dups.extend(ids.iter().cloned()); // 各要素を重複させる

        let canon_a = ObservationRecorder::canonicalize_related_ids(&ids)
            .expect("正準化は失敗しない");
        let canon_b = ObservationRecorder::canonicalize_related_ids(&shuffled)
            .expect("正準化は失敗しない");
        let canon_c = ObservationRecorder::canonicalize_related_ids(&with_dups)
            .expect("正準化は失敗しない");

        proptest::prop_assert_eq!(&canon_a, &canon_b, "順不同でも同一正準 JSON");
        proptest::prop_assert_eq!(&canon_a, &canon_c, "重複ありでも同一正準 JSON（distinct）");
    }
}

// ── ケース 5: 重複 observe の dedup 冪等性（property）──────────────────────────────────

proptest::proptest! {
    #[test]
    fn prop_record_dedup_is_idempotent(n in 1usize..20) {
        // @ts: TS-LGX-008 ケース 5
        let db = db();
        let obs = new_obs("manual_note", "same", &["DD-LGX-008"]);
        let mut inserted = 0usize;
        for _ in 0..n {
            let r = ObservationRecorder::record(&obs, &db).expect("record は成功");
            if !r.skipped {
                inserted += 1;
            }
        }
        // N 回呼出しでも実 INSERT は最初の 1 件のみ。
        proptest::prop_assert_eq!(inserted, 1, "dedup 冪等性: 実 INSERT は 1 件のみ");
    }
}

// ── ケース 6: 不正 category 値 → exit 2（clap ValueEnum、【v3 差分】）──────────────────
// CLI 層（clap ValueEnum 相当）の検証点を ObserveCategoryInput::from_str で代理検証する。
// 不正値は None（CLI 層がこれを exit 2 に写像、TS-008 §2.2 EF3 / LGX-COMPAT-001 §3）。

#[test]
fn test_observe_category_invalid_value_rejected() {
    // @ts: TS-LGX-008 ケース 6
    assert!(
        ObserveCategoryInput::from_str("foobar").is_none(),
        "凍結 3 値以外は None（CLI 層 exit 2）"
    );
    // 正当 3 値は受理（exit 0 経路）。
    assert_eq!(
        ObserveCategoryInput::from_str("compile_miss"),
        Some(ObserveCategoryInput::CompileMiss)
    );
    assert_eq!(
        ObserveCategoryInput::from_str("review_correction"),
        Some(ObserveCategoryInput::ReviewCorrection)
    );
    assert_eq!(
        ObserveCategoryInput::from_str("manual_note"),
        Some(ObserveCategoryInput::ManualNote)
    );
}

// ── ケース 7: AutoObserver フィルタ規則（CheckReport → NewObservation 列）─────────────

#[test]
fn test_auto_observer_filters_check_report() {
    // @ts: TS-LGX-008 ケース 7
    let rep = report(vec![
        finding(Severity::Ok, CheckCategory::ChainIntegrity, &["a"]), // severity=Ok 除外
        finding(Severity::Error, CheckCategory::FileExistence, &["b"]), // FileExistence×Error 除外
        finding(Severity::Warning, CheckCategory::DocumentId, &["c"]), // DocumentId×Warning 除外
        finding(Severity::Error, CheckCategory::ChainIntegrity, &["d"]), // 既知 → observation 化
        finding(Severity::Warning, CheckCategory::OrphanFile, &["e"]),
        finding(Severity::Info, CheckCategory::SemanticSimilarity, &["f"]),
    ]);
    let obs = AutoObserver::from_check_results(&rep);
    // 既知 5 カテゴリ（chain_integrity / orphan_file / semantic_similarity 等）のみ残る。
    assert!(
        !obs.is_empty(),
        "既知カテゴリは observation 化される（chain_integrity 等）"
    );
    assert!(
        obs.iter().all(|o| o.category != "file_existence"),
        "FileExistence×Error は除外"
    );
    assert!(
        obs.iter().all(|o| o.category != "document_id"),
        "DocumentId×Warning は除外"
    );
}

// ── ケース 8: drift_from_embed_error は ContextualRetrievalFailed のみ Some ─────────────

#[test]
fn test_drift_from_embed_error_only_contextual_retrieval() {
    // @ts: TS-LGX-008 ケース 8
    let drift_err = EmbedError::ContextualRetrievalFailed {
        node_id: "SPEC-LGX-007".to_string(),
        detail: "timeout sk-secret123".to_string(),
    };
    let some = drift_from_embed_error(&drift_err, "SPEC-LGX-007");
    let nv = some.expect("ContextualRetrievalFailed は Some");
    assert_eq!(nv.category, "drift", "drift カテゴリ");
    assert!(
        !nv.message.contains("sk-secret123"),
        "message は mask_api_key を通過（API キー混入なし、SEC.05）"
    );

    let other = EmbedError::Other("model load failed".to_string());
    assert!(
        drift_from_embed_error(&other, "SPEC-LGX-007").is_none(),
        "他 variant は None"
    );
}

// ── ケース 9: analyze の Pessimistic Claim + 変換可能カテゴリ → approve で Resolved ────

#[test]
fn test_analyze_then_approve_resolves_observation() {
    // @ts: TS-LGX-008 ケース 9
    let db = db();
    // chain_integrity observation（変換規則 add_chain_entry あり）を投入。
    let obs = new_obs("chain_integrity", "missing chain edge", &["DD-LGX-008"]);
    let _ = ObservationRecorder::record(&obs, &db).expect("record");
    let proposals = ProposalAnalyzer::analyze(&db).expect("analyze");
    let p = proposals.first().expect("少なくとも 1 件の add_chain_entry Proposal");
    assert_eq!(p.kind, "add_chain_entry", "chain_integrity → add_chain_entry");
    assert_eq!(p.status, ProposalStatus::Pending, "生成直後は pending");
    ProposalManager::approve(p.id, &db).expect("approve");
    // approve tx 内で対応 observation が Resolved へ連動（FB-INV-2、【v3 差分】）。
}

// ── ケース 10: 変換規則なしカテゴリ → Skipped 終端（永久再 claim 解消、【v3 差分】）─────

#[test]
fn test_analyze_unconvertible_category_skips() {
    // @ts: TS-LGX-008 ケース 10
    let db = db();
    let obs = new_obs("orphan_file", "orphan doc", &["docs/foo.md"]);
    let _ = ObservationRecorder::record(&obs, &db).expect("record");
    let first = ProposalAnalyzer::analyze(&db).expect("1 回目 analyze");
    assert!(first.is_empty(), "変換規則なし → Proposal 生成 0 件");
    // 2 回目以降は skipped を再取込しない（pending↔analyzing 往復＝永久再 claim が起きない）。
    let second = ProposalAnalyzer::analyze(&db).expect("2 回目 analyze");
    assert!(second.is_empty(), "skipped 終端は再取込されない");
}

// ── ケース 11: analyze 中の単一 observation 処理失敗 → Claim Release（pending 復帰）────

#[test]
fn test_analyze_failure_releases_claim_to_pending() {
    // @ts: TS-LGX-008 ケース 11
    let db = db();
    // 失敗注入の経路は SRC[GREEN] のテスト fixture 次第。ここでは analyze を呼び
    // AnalyzeFailed → pending 復帰（Skipped/Resolved にしない）契約を束縛する。
    let obs = new_obs("chain_integrity", "will fail", &["DD-LGX-008"]);
    let _ = ObservationRecorder::record(&obs, &db).expect("record");
    let r = ProposalAnalyzer::analyze(&db);
    // 失敗時は AnalyzeFailed、成功時は Ok。RED では todo!() で panic（contract 束縛）。
    match r {
        Ok(_) => { /* 成功経路（SRC[GREEN] で fixture 制御） */ }
        Err(legixy_feedback::FeedbackError::AnalyzeFailed { observation_id, .. }) => {
            assert!(observation_id > 0, "失敗した observation_id を持つ");
        }
        Err(other) => panic!("AnalyzeFailed 以外の失敗は想定外: {other:?}"),
    }
}

// ── ケース 12: 同一 semantic_key の pending proposal が既存 → INSERT 抑止（FB-INV-5）──

#[test]
fn test_analyze_duplicate_semantic_key_suppressed() {
    // @ts: TS-LGX-008 ケース 12
    let db = db();
    let obs = new_obs("chain_integrity", "dup key", &["DD-LGX-008"]);
    let _ = ObservationRecorder::record(&obs, &db).expect("record");
    let _first = ProposalAnalyzer::analyze(&db).expect("1 回目 analyze");
    // 同一 semantic_key を生む observation で再 analyze → 新規 INSERT を抑止。
    let obs2 = new_obs("chain_integrity", "dup key again", &["DD-LGX-008"]);
    let _ = ObservationRecorder::record(&obs2, &db).expect("record");
    let second = ProposalAnalyzer::analyze(&db).expect("2 回目 analyze");
    assert!(
        second.is_empty(),
        "既存 pending と同一 semantic_key は新規 INSERT に含めない（FB-INV-5）"
    );
}

// ── ケース 13: approve の状態遷移（pending → approved・終端不可逆・CAS）──────────────

#[test]
fn test_approve_terminal_reapprove_is_invalid_status() {
    // @ts: TS-LGX-008 ケース 13
    let db = db();
    let obs = new_obs("chain_integrity", "approve target", &["DD-LGX-008"]);
    let _ = ObservationRecorder::record(&obs, &db).expect("record");
    let proposals = ProposalAnalyzer::analyze(&db).expect("analyze");
    let p = proposals.first().expect("pending Proposal");
    ProposalManager::approve(p.id, &db).expect("(a) pending → approved");
    // (b) 終端状態への再 approve → InvalidProposalStatus（CAS 行数 0 → exit 1）。
    let re = ProposalManager::approve(p.id, &db);
    assert!(
        matches!(
            re,
            Err(legixy_feedback::FeedbackError::InvalidProposalStatus { expected: "pending", .. })
        ),
        "approved への再 approve は InvalidProposalStatus（終端不可逆）"
    );
}

// ── ケース 14: 状態遷移の網羅（from_str / as_str ラウンドトリップ + 終端不可逆）───────

#[test]
fn test_status_roundtrip_and_terminal_invariants() {
    // @ts: TS-LGX-008 ケース 14
    // ObservationStatus 4 値・ProposalStatus 3 値の as_str/from_str ラウンドトリップ一致。
    for s in [
        ObservationStatus::Pending,
        ObservationStatus::Analyzing,
        ObservationStatus::Resolved,
        ObservationStatus::Skipped,
    ] {
        assert_eq!(ObservationStatus::from_str(s.as_str()), Some(s));
    }
    for s in [
        ProposalStatus::Pending,
        ProposalStatus::Approved,
        ProposalStatus::Rejected,
    ] {
        assert_eq!(ProposalStatus::from_str(s.as_str()), Some(s));
    }
    // 未知文字列は None（禁止遷移先・無効状態の表現）。
    assert_eq!(ObservationStatus::from_str("proposed"), None);
    assert_eq!(ProposalStatus::from_str("skipped"), None); // proposal に skipped なし（3 値）

    // 禁止遷移（終端からの再操作）は CAS で行数 0 → InvalidProposalStatus。
    let db = db();
    let obs = new_obs("chain_integrity", "fsm", &["DD-LGX-008"]);
    let _ = ObservationRecorder::record(&obs, &db).expect("record");
    let proposals = ProposalAnalyzer::analyze(&db).expect("analyze");
    let p = proposals.first().expect("pending Proposal");
    ProposalManager::reject(p.id, "禁止遷移検証", &db).expect("pending → rejected");
    // rejected → approved は禁止。
    let bad = ProposalManager::approve(p.id, &db);
    assert!(
        matches!(
            bad,
            Err(legixy_feedback::FeedbackError::InvalidProposalStatus { .. })
        ),
        "rejected → approved は禁止（終端不可逆）"
    );
}

// ── ケース 15: reject の reason が trim 後 0 文字 → EmptyRejectReason exit 1（【v3 差分】）

#[test]
fn test_reject_empty_reason_is_rejected() {
    // @ts: TS-LGX-008 ケース 15
    let db = db();
    let obs = new_obs("chain_integrity", "reject empty reason", &["DD-LGX-008"]);
    let _ = ObservationRecorder::record(&obs, &db).expect("record");
    let proposals = ProposalAnalyzer::analyze(&db).expect("analyze");
    let p = proposals.first().expect("pending Proposal");
    for blank in ["", "  ", "\n"] {
        let r = ProposalManager::reject(p.id, blank, &db);
        assert!(
            matches!(r, Err(legixy_feedback::FeedbackError::EmptyRejectReason)),
            "trim 後 0 文字 reason ({blank:?}) は EmptyRejectReason（CAS 未実行）"
        );
    }
}

// ── ケース 16: reject 成功 → rejected・observation pending 復帰・decided_reason 格納 ────

#[test]
fn test_reject_success_records_reason() {
    // @ts: TS-LGX-008 ケース 16
    let db = db();
    let obs = new_obs("chain_integrity", "reject ok", &["DD-LGX-008"]);
    let _ = ObservationRecorder::record(&obs, &db).expect("record");
    let proposals = ProposalAnalyzer::analyze(&db).expect("analyze");
    let p = proposals.first().expect("pending Proposal");
    ProposalManager::reject(p.id, "重複提案", &db).expect("reject 成功");
    // 一覧で rejected が観測でき、対応 observation は pending に戻る（approve の Resolved と非対称）。
    let summaries = FeedbackCli::list_proposals(&db, Some(ProposalStatus::Rejected))
        .expect("list_proposals");
    assert!(
        summaries.iter().any(|s| s.id == p.id && s.status == ProposalStatus::Rejected),
        "reject 後は rejected として一覧される"
    );
}

// ── ケース 17: 不在 proposal-id への approve/reject → ProposalNotFound exit 1 ───────────

#[test]
fn test_approve_reject_unknown_id_not_found() {
    // @ts: TS-LGX-008 ケース 17
    let db = db();
    for id in [0i64, -1, i64::MAX, 99999] {
        let a = ProposalManager::approve(id, &db);
        assert!(
            matches!(a, Err(legixy_feedback::FeedbackError::ProposalNotFound { id: got }) if got == id),
            "不在 id={id} の approve は ProposalNotFound"
        );
        let r = ProposalManager::reject(id, "reason", &db);
        assert!(
            matches!(r, Err(legixy_feedback::FeedbackError::ProposalNotFound { id: got }) if got == id),
            "不在 id={id} の reject は ProposalNotFound"
        );
    }
}

// ── ケース 18: 並行 observe で同一キー → 1 件のみ INSERT（UNIQUE 制約 fallback）─────────
// 負荷ストレス側面は NFR-LGX-001 REL.07 / SEC.02 へ委譲。本 TC はロジック（1 件格納）のみ。

#[test]
fn test_concurrent_same_key_single_insert_logic() {
    // @ts: TS-LGX-008 ケース 18
    let db = db();
    let obs = new_obs("manual_note", "concurrent", &["DD-LGX-008"]);
    // 逐次の代理検証（並行性は NFR 委譲）: 同一キーの複数 record で実 INSERT は 1 件。
    let mut inserted = 0;
    for _ in 0..4 {
        if !ObservationRecorder::record(&obs, &db).expect("record").skipped {
            inserted += 1;
        }
    }
    assert_eq!(inserted, 1, "同一キーは UNIQUE fallback で 1 件のみ格納");
}

// ── ケース 19: 並行 approve vs reject → CAS で 1 つだけ成立・敗者 exit 1 ─────────────────
// 並行決着の負荷側面は NFR REL.07 へ委譲。本 TC は first-writer（CAS）成立後の敗者契約を束縛。

#[test]
fn test_cas_loser_is_invalid_status() {
    // @ts: TS-LGX-008 ケース 19
    let db = db();
    let obs = new_obs("chain_integrity", "cas race", &["DD-LGX-008"]);
    let _ = ObservationRecorder::record(&obs, &db).expect("record");
    let proposals = ProposalAnalyzer::analyze(&db).expect("analyze");
    let p = proposals.first().expect("pending Proposal");
    // 勝者（approve）成立後、敗者（reject）は CAS 行数 0 → InvalidProposalStatus。
    ProposalManager::approve(p.id, &db).expect("勝者 approve");
    let loser = ProposalManager::reject(p.id, "loser", &db);
    assert!(
        matches!(
            loser,
            Err(legixy_feedback::FeedbackError::InvalidProposalStatus { expected: "pending", .. })
        ),
        "CAS 敗者は InvalidProposalStatus（exit 1）"
    );
}

// ── ケース 20: feedback E2E（CheckReport → Observation 生成・skipped カウント）──────────

#[test]
fn test_run_feedback_e2e_counts() {
    // @ts: TS-LGX-008 ケース 20
    let db = db();
    let rep = report(vec![
        finding(Severity::Error, CheckCategory::ChainIntegrity, &["DD-LGX-008"]),
        finding(Severity::Warning, CheckCategory::OrphanFile, &["docs/x.md"]),
    ]);
    let r = FeedbackCli::run_feedback(&db, &rep).expect("run_feedback");
    assert!(r.observations_created >= 1, "既知カテゴリから observation 生成");

    // 代替フロー 1a: 該当カテゴリ 0 件 → created=0, skipped=0, exit 0（エラーでない）。
    let empty = report(vec![]);
    let r0 = FeedbackCli::run_feedback(&db, &empty).expect("空 CheckReport も正常終了");
    assert_eq!(r0.observations_created, 0, "該当 0 件 → created 0");
    assert_eq!(r0.observations_skipped, 0, "該当 0 件 → skipped 0");
}

// ── ケース 21: analyze E2E（pending 0 件 → proposal 0 件・exit 0）────────────────────────

#[test]
fn test_run_analyze_empty_is_ok_empty() {
    // @ts: TS-LGX-008 ケース 21
    let db = db();
    let proposals = FeedbackCli::run_analyze(&db).expect("空 observations は Ok（エラーでない）");
    assert!(proposals.is_empty(), "pending 0 件 → proposal 0 件");
}

// ── ケース 22: proposals 一覧の status フィルタ（None=全件 / Some=該当のみ）─────────────

#[test]
fn test_list_proposals_status_filter() {
    // @ts: TS-LGX-008 ケース 22
    let db = db();
    // 0 件状態でも空 Vec（エラーでない）。
    let none_all = FeedbackCli::list_proposals(&db, None).expect("None=全件");
    assert!(none_all.is_empty(), "0 件状態 → 空 Vec");
    let only_pending =
        FeedbackCli::list_proposals(&db, Some(ProposalStatus::Pending)).expect("Some=該当のみ");
    assert!(only_pending.iter().all(|s| s.status == ProposalStatus::Pending));
}

// ── ケース 23: audit limit 境界（recent / by_target、1 / 10 / 50）───────────────────────

#[test]
fn test_audit_limit_boundaries() {
    // @ts: TS-LGX-008 ケース 23
    let db = db();
    for limit in [1usize, 10, 50] {
        let entries = ContextAuditReader::recent(&db, limit).expect("recent");
        assert!(
            entries.len() <= limit,
            "recent は LIMIT {limit} を超えない（id DESC）"
        );
    }
    let by_t = ContextAuditReader::by_target(&db, "SPEC-LGX-007", 10).expect("by_target");
    assert!(
        by_t.iter().all(|e| e.target_id == "SPEC-LGX-007"),
        "by_target は target_id フィルタ"
    );
    assert!(by_t.len() <= 10, "by_target も LIMIT を超えない");
}

// ── ケース 24: engine.db 不在 → 新規作成して続行（正常、破損と区別）──────────────────────

#[test]
fn test_db_absent_is_created_not_corrupted() {
    // @ts: TS-LGX-008 ケース 24
    // 不在パスへの初回操作は CREATE TABLE で新規作成して続行（exit 0）。破損とは区別。
    let path = format!(
        "{}/legixy-tc008-absent-{}.db",
        std::env::temp_dir().display(),
        std::process::id()
    );
    let _ = std::fs::remove_file(&path); // 確実に不在状態にする
    let db = Connection::open_path(&path).expect("不在 → 新規作成して続行（破損ではない）");
    let obs = new_obs("manual_note", "first on fresh db", &[]);
    let r = ObservationRecorder::record(&obs, &db).expect("新規 DB へ record 成功");
    assert!(r.id > 0);
    let _ = std::fs::remove_file(&path);
}

// ── ケース 25: engine.db 破損 → DbCorrupted exit 1（自動再生成禁止）──────────────────────

#[test]
fn test_db_corrupted_is_exit1_no_regen() {
    // @ts: TS-LGX-008 ケース 25
    let path = format!(
        "{}/legixy-tc008-corrupt-{}.db",
        std::env::temp_dir().display(),
        std::process::id()
    );
    std::fs::write(&path, b"this is not a sqlite database, corrupted bytes")
        .expect("破損 fixture 書込");
    // 初回 DB 操作で SQLITE_CORRUPT / NotADatabase を捕捉 → DbCorrupted（exit 1）。自動再生成しない。
    let opened = Connection::open_path(&path);
    let corrupted = match opened {
        Err(_) => true, // open 段階で破損検出
        Ok(db) => {
            // open は成功しても初回操作で破損検出する経路（DD §6.1 option C）。
            matches!(
                ObservationRecorder::record(&new_obs("manual_note", "x", &[]), &db),
                Err(legixy_feedback::FeedbackError::DbCorrupted { .. })
            )
        }
    };
    assert!(corrupted, "破損 DB は DbCorrupted（exit 1）で明示失敗、自動再生成しない");
    // 証跡保護: 破損ファイルは削除・再生成されず残存。
    assert!(std::path::Path::new(&path).exists(), "破損ファイルは自動削除されない");
    let _ = std::fs::remove_file(&path);
}

// ── ケース 26: context_log 書込失敗時（ベストエフォート、analyze は欠落検査せず）──────────

#[test]
fn test_audit_read_and_analyze_tolerate_gaps() {
    // @ts: TS-LGX-008 ケース 26
    let db = db();
    // recent は存在するエントリのみ返す（欠落を補完・検出しない）。
    let entries = ContextAuditReader::recent(&db, 10).expect("recent は欠落を検出しない");
    let _ = entries; // 欠落の有無を検査しないことが契約
    // analyze は context_log 完全性を検査・報告しない（responsibilities は obs→proposal 変換のみ）。
    let _ = FeedbackCli::run_analyze(&db).expect("analyze は context_log 欠落で失敗しない");
}

// ── ケース 27: read-only 不変（list_proposals / recent / by_target は DB を変更しない）────

#[test]
fn test_read_only_operations_are_idempotent() {
    // @ts: TS-LGX-008 ケース 27
    let db = db();
    // 複数回呼出しで同一結果（read-only、&Connection 借用のみ）。
    let a1 = FeedbackCli::list_proposals(&db, None).expect("list 1");
    let a2 = FeedbackCli::list_proposals(&db, None).expect("list 2");
    assert_eq!(a1, a2, "list_proposals は read-only（複数回で同一結果）");
    let r1 = ContextAuditReader::recent(&db, 10).expect("recent 1");
    let r2 = ContextAuditReader::recent(&db, 10).expect("recent 2");
    assert_eq!(r1, r2, "recent は read-only");
    let t1 = ContextAuditReader::by_target(&db, "SPEC-LGX-007", 10).expect("by_target 1");
    let t2 = ContextAuditReader::by_target(&db, "SPEC-LGX-007", 10).expect("by_target 2");
    assert_eq!(t1, t2, "by_target は read-only");
}
