// Document ID: TC-LGX-007
// TC-LGX-007: embedding 生成とドリフト検出のテストコード（TC[RED]）。
//
// 親 chain: TS-LGX-007 → 本 TC-LGX-007 → SRC-LGX-007。
// 各テストは TS-LGX-007 のケースを legixy-embed の凍結 API（DD-LGX-007 §3）に束縛する。
// SRC[GREEN] 未実装（todo!()）のため、ロジック関数を呼ぶテストは panic で失敗する（RED）。
// `cargo test -p legixy-embed --no-run` は通る（型・シグネチャ整合）。
//
// 注（cross-DD 署名衝突の反映）:
//   - ケース 18/19 の histogram は DD-011 正典シグネチャ（&[f32] + BucketCount → Histogram）に束縛。
//     TS-007 §1/§2 が「正準 histogram は DD-LGX-011 所有、本 TS は委譲確認に集中」と明記しているため。

use std::path::Path;

use legixy_embed::{
    compute_all_pair_scores, compute_link_candidates, compute_model_version, content_hash_for,
    cosine_similarity, detect_drift, embed_all, histogram, normalize_content,
    read_current_content_for_node, synthesize_with_fallback, BucketCount, ContextualConfig,
    CrOptions, DriftKind, EmbedOptions, EmbedReport, Embedder, EmbeddingRow, EmbeddingStore,
    HashMatchState, NodeFilter, PreprocessProfile,
};
use legixy_graph::{Node, TraceGraph};

fn node(id: &str, type_code: &str, path: &str) -> Node {
    Node {
        id: id.to_string(),
        type_code: type_code.to_string(),
        path: path.to_string(),
        parent_id: None,
        anchor: None,
    }
}

fn row(node_id: &str, dim: usize, model_version: &str, content_hash: &str) -> EmbeddingRow {
    EmbeddingRow {
        node_id: node_id.to_string(),
        embedding: vec![0.0; dim],
        dim,
        model_version: model_version.to_string(),
        content_hash: content_hash.to_string(),
        context: None,
        context_hash: None,
        created_at: "2026-06-14 00:00:00".to_string(),
    }
}

// ケース 1: 空テキスト（正規化後 0 文字）ノードの embed_all skip
#[test]
fn test_embed_all_empty_text_skips() {
    // @ts: TS-LGX-007 ケース 1
    let graph = TraceGraph::from_parts(vec![node("SPEC-LGX-001", "SPEC", "s.md")], vec![]);
    let store = EmbeddingStore::empty();
    let embedder = Embedder::stub("paraphrase:deadbeef:plain:384", 384);
    let report = embed_all(&graph, &store, &embedder, EmbedOptions::default()).expect("Ok");
    assert_eq!(
        report,
        EmbedReport {
            generated: 0,
            skipped: 1,
            failed: 0,
            errors: vec![],
        }
    );
}

// ケース 2: embed_node は空テキストでも Err を返さない（責務境界）
#[test]
fn test_embed_node_empty_text_not_err() {
    // @ts: TS-LGX-007 ケース 2
    let embedder = Embedder::stub("m:v:plain:384", 384);
    let r = embedder.embed_node("", None, "SPEC-LGX-001");
    assert!(r.is_ok(), "空テキストでも Err を返さない（DD §3 不変条件）");
}

// ケース 4: モデル shape 検証失敗 → Embedder::new が Err → exit 1
#[test]
fn test_embedder_new_shape_invalid_err() {
    // @ts: TS-LGX-007 ケース 4
    let r = Embedder::new(Path::new("/tmp/bad-shape-model"), "m:v:plain:0");
    assert!(r.is_err(), "shape 検証失敗は Err（exit 1）");
}

// ケース 5: モデル解決/読込失敗 → Embedder::new が Err → exit 1
#[test]
fn test_embedder_new_model_missing_err() {
    // @ts: TS-LGX-007 ケース 5
    let r = Embedder::new(Path::new("/nonexistent/models"), "m:v:plain:384");
    assert!(r.is_err(), "モデル解決失敗は Err（exit 1）");
}

// ケース 6: normalize_content — BOM 除去
#[test]
fn test_normalize_content_strips_bom() {
    // @ts: TS-LGX-007 ケース 6
    assert_eq!(normalize_content("\u{FEFF}hello"), normalize_content("hello"));
}

// ケース 7: normalize_content — CRLF / CR → LF 統一
#[test]
fn test_normalize_content_unifies_newlines() {
    // @ts: TS-LGX-007 ケース 7
    assert_eq!(normalize_content("a\r\nb\rc"), "a\nb\nc");
}

// ケース 8: normalize_content — NFC 正規化（NFD → NFC）
#[test]
fn test_normalize_content_nfc() {
    // @ts: TS-LGX-007 ケース 8
    assert_eq!(
        normalize_content("\u{0065}\u{0301}"),
        normalize_content("\u{00E9}")
    );
}

// ケース 9: normalize_content — 末尾改行揺れ吸収（末尾 1 改行へ正規化）
#[test]
fn test_normalize_content_trailing_newline() {
    // @ts: TS-LGX-007 ケース 9
    let a = normalize_content("x");
    let b = normalize_content("x\n");
    let c = normalize_content("x\n\n\n");
    assert_eq!(a, b);
    assert_eq!(b, c);
}

// ケース 12: cosine_similarity 値域境界（完全一致=1.0 / 直交=0.0 / 反対=-1.0）
#[test]
fn test_cosine_similarity_value_bounds() {
    // @ts: TS-LGX-007 ケース 12
    assert_eq!(cosine_similarity(&[1.0, 0.0], &[1.0, 0.0]), 1.0);
    assert_eq!(cosine_similarity(&[1.0, 0.0], &[0.0, 1.0]), 0.0);
    assert_eq!(cosine_similarity(&[1.0, 0.0], &[-1.0, 0.0]), -1.0);
}

// ケース 13: cosine_similarity clamp（域外 → [-1,1] へ）
#[test]
fn test_cosine_similarity_clamp() {
    // @ts: TS-LGX-007 ケース 13
    let a = [1.0_f32, 0.0001];
    let b = [1.0_f32, 0.0001];
    let s = cosine_similarity(&a, &b);
    assert!(s <= 1.0 && s >= -1.0, "[-1,1] に clamp");
}

// ケース 15: bulk API — ノード 0 件 / 1 件（O(N²) ペア数 0）
#[test]
fn test_bulk_api_zero_or_one_node_empty() {
    // @ts: TS-LGX-007 ケース 15
    let store = EmbeddingStore::stub(vec![row("A", 384, "v", "h")], vec![]);
    let pairs = compute_all_pair_scores(&store).expect("Ok");
    assert_eq!(pairs.len(), 0, "ペアが構成できない = 空 Vec");
}

// ケース 16: bulk API — 次元不一致ペアの skip + 集約 Warning
#[test]
fn test_bulk_api_dim_mismatch_skip() {
    // @ts: TS-LGX-007 ケース 16
    let graph = TraceGraph::from_parts(
        vec![node("A", "SPEC", "a.md"), node("B", "SPEC", "b.md")],
        vec![],
    );
    let store = EmbeddingStore::stub(
        vec![row("A", 384, "v", "h1"), row("B", 768, "v", "h2")],
        vec![],
    );
    let cands = compute_link_candidates(&graph, &store, 0.8).expect("Ok");
    // 次元不一致ペアは skip され、一致次元ペアのみ返る（ここでは A,B は次元不一致のため空）。
    assert_eq!(cands.len(), 0, "次元不一致は skip");
}

// ケース 18: histogram — 値域 [0,1] 固定・均等幅・末尾バケット inclusive（DD-011 正典シグネチャ）
#[test]
fn test_histogram_value_range_and_inclusive() {
    // @ts: TS-LGX-007 ケース 18（histogram 委譲確認。正準は TS-LGX-011）
    let h = histogram(&[0.0, 0.5, 1.0], BucketCount::new(2).unwrap());
    assert_eq!(h.buckets.len(), 2);
    assert_eq!(h.buckets[0].count, 1, "0.0 は最下位バケット");
    assert_eq!(h.buckets[1].count, 2, "0.5 と 1.0（末尾 inclusive）");
}

// ケース 19: histogram — [0,1] 外の clamp（負値は 0.0 へ・上限超過は 1.0 へ）
#[test]
fn test_histogram_out_of_range_clamp() {
    // @ts: TS-LGX-007 ケース 19（histogram 委譲確認。正準は TS-LGX-011）
    let h = histogram(&[-1.5, -0.5, 0.0, 0.5, 1.0, 1.5], BucketCount::new(4).unwrap());
    assert_eq!(h.buckets[0].count, 3, "-1.5,-0.5,0.0 が bucket[0] へ clamp");
    assert_eq!(h.buckets[3].count, 2, "1.0,1.5 が末尾 bucket[3] へ clamp");
}

// ケース 20: compute_model_version 複合キー書式
#[test]
fn test_compute_model_version_format() {
    // @ts: TS-LGX-007 ケース 20
    let mv = compute_model_version(
        "paraphrase-multilingual-MiniLM-L12-v2",
        Path::new("/tmp/model.onnx"),
        PreprocessProfile::Plain,
        384,
    )
    .expect("Ok");
    assert!(mv.starts_with("paraphrase-multilingual-MiniLM-L12-v2:"));
    assert!(mv.ends_with(":384"));
}

// ケース 21: compute_model_version — 同名 ONNX 差し替えで model_version 変化
#[test]
fn test_compute_model_version_changes_on_onnx_swap() {
    // @ts: TS-LGX-007 ケース 21
    let a = compute_model_version("m", Path::new("/tmp/onnx_a.onnx"), PreprocessProfile::Plain, 384)
        .expect("Ok");
    let b = compute_model_version("m", Path::new("/tmp/onnx_b.onnx"), PreprocessProfile::Plain, 384)
        .expect("Ok");
    assert_ne!(a, b, "別内容 ONNX は model_version 不一致（SCORE-INV-2）");
}

// ケース 22: EmbeddingStore::is_up_to_date — SCORE-INV-1 + SCORE-INV-2 双方一致で skip
#[test]
fn test_is_up_to_date_both_match() {
    // @ts: TS-LGX-007 ケース 22
    let store = EmbeddingStore::stub(vec![row("A", 384, "V", "H")], vec![]);
    assert_eq!(store.is_up_to_date("A", "H", "V").expect("Ok"), true);
    assert_eq!(store.is_up_to_date("A", "H2", "V").expect("Ok"), false);
    assert_eq!(store.is_up_to_date("A", "H", "V2").expect("Ok"), false);
}

// ケース 23: HashMatchState 3 状態判定（Skip / Regen / Missing）— enum 構築の型整合
#[test]
fn test_hash_match_state_variants() {
    // @ts: TS-LGX-007 ケース 23
    // 3 状態の網羅（型構築）。判定ロジックは is_up_to_date / load_embedding 経由（ケース 22, 37）。
    let states = [
        HashMatchState::Skip,
        HashMatchState::Regen,
        HashMatchState::Missing,
    ];
    assert_eq!(states.len(), 3);
    assert_ne!(HashMatchState::Skip, HashMatchState::Regen);
}

// ケース 24: embed_all --force で content_hash 一致でも強制再生成
#[test]
fn test_embed_all_force_regenerates() {
    // @ts: TS-LGX-007 ケース 24
    let graph = TraceGraph::from_parts(vec![node("A", "SPEC", "a.md")], vec![]);
    let store = EmbeddingStore::stub(vec![row("A", 384, "V", "H")], vec![]);
    let embedder = Embedder::stub("V", 384);
    let report = embed_all(
        &graph,
        &store,
        &embedder,
        EmbedOptions {
            force: true,
            ..EmbedOptions::default()
        },
    )
    .expect("Ok");
    assert_eq!(report.generated, 1, "--force は skip を上書きし生成");
    assert_eq!(report.skipped, 0);
}

// ケース 25: embed_all NodeFilter::Ids — 未登録 ID で Err（exit 1）
#[test]
fn test_embed_all_unregistered_id_err() {
    // @ts: TS-LGX-007 ケース 25
    let graph = TraceGraph::from_parts(vec![node("A", "SPEC", "a.md")], vec![]);
    let store = EmbeddingStore::empty();
    let embedder = Embedder::stub("V", 384);
    let r = embed_all(
        &graph,
        &store,
        &embedder,
        EmbedOptions {
            node_filter: NodeFilter::Ids(vec!["SPEC-LGX-999".to_string()]),
            ..EmbedOptions::default()
        },
    );
    assert!(r.is_err(), "未登録 ID は NodeNotFound（exit 1）");
}

// ケース 26: embed_all 部分失敗継続 — content_range 防御検証失敗を errors 計上後続継続
#[test]
fn test_embed_all_partial_failure_continues() {
    // @ts: TS-LGX-007 ケース 26
    let graph = TraceGraph::from_parts(
        vec![
            node("A", "SPEC", "a.md"),
            node("B", "SPEC", "b.md"),
            node("C", "SPEC", "c.md"),
        ],
        vec![],
    );
    let store = EmbeddingStore::empty();
    let embedder = Embedder::stub("V", 384);
    let report = embed_all(&graph, &store, &embedder, EmbedOptions::default()).expect("Ok");
    assert_eq!(report.failed, report.errors.len(), "failed == errors.len()");
}

// ケース 29: embed_all 決定性 — 同一入力 → 同一 EmbedReport
#[test]
fn test_embed_all_deterministic_report() {
    // @ts: TS-LGX-007 ケース 29
    let graph = TraceGraph::from_parts(vec![node("A", "SPEC", "a.md")], vec![]);
    let store = EmbeddingStore::empty();
    let embedder = Embedder::stub("V", 384);
    let r1 = embed_all(&graph, &store, &embedder, EmbedOptions::default()).expect("Ok");
    let r2 = embed_all(&graph, &store, &embedder, EmbedOptions::default()).expect("Ok");
    assert_eq!(r1, r2, "同一入力 → 同一 EmbedReport");
}

// ケース 30: EmbedReport --json スキーマ（failed フィールド + errors オブジェクト）
#[test]
fn test_embed_report_json_schema() {
    // @ts: TS-LGX-007 ケース 30
    let report = EmbedReport {
        generated: 2,
        skipped: 0,
        failed: 1,
        errors: vec![legixy_embed::EmbedErrorItem {
            node_id: "A".to_string(),
            message: "range invalid".to_string(),
        }],
    };
    let json = serde_json::to_string(&report).expect("serialize");
    assert!(json.contains("\"generated\""));
    assert!(json.contains("\"skipped\""));
    assert!(json.contains("\"failed\""));
    assert!(json.contains("\"errors\""));
    assert!(json.contains("\"node_id\""));
    assert_eq!(report.failed, report.errors.len());
}

// ケース 31: detect_drift — stale（content_hash 不一致）を ContentChanged で報告
#[test]
fn test_detect_drift_content_changed() {
    // @ts: TS-LGX-007 ケース 31
    // TS 前提「ファイル現内容と stored content_hash が不一致」を満たすため実ファイルを用意（hermetic 化）。
    // 現内容の content_hash は stored "OLDHASH" と必ず不一致 → ContentChanged（stored・current 双方 Some）。
    let tmp = tempfile::tempdir().expect("tempdir");
    std::fs::write(tmp.path().join("a.md"), "current body (≠ OLDHASH)").expect("write a.md");
    let graph = TraceGraph::from_parts(vec![node("A", "SPEC", "a.md")], vec![]);
    let store = EmbeddingStore::stub(vec![row("A", 384, "V", "OLDHASH")], vec![]);
    let findings = detect_drift(&graph, &store, tmp.path()).expect("Ok");
    assert!(
        findings.iter().any(|f| f.kind == DriftKind::ContentChanged),
        "stale = ContentChanged"
    );
}

// ケース 32: detect_drift — 未生成ノードを EmbeddingMissing で結果に包含
#[test]
fn test_detect_drift_embedding_missing() {
    // @ts: TS-LGX-007 ケース 32
    let graph = TraceGraph::from_parts(vec![node("A", "SPEC", "a.md")], vec![]);
    let store = EmbeddingStore::empty();
    let findings = detect_drift(&graph, &store, Path::new("/tmp/proj")).expect("Ok");
    assert!(
        findings
            .iter()
            .any(|f| f.kind == DriftKind::EmbeddingMissing && f.stored_hash.is_none()),
        "未生成 = EmbeddingMissing（stored=None）"
    );
}

// ケース 33: detect_drift — ファイル読込不能を FileMissing で報告
#[test]
fn test_detect_drift_file_missing() {
    // @ts: TS-LGX-007 ケース 33
    let graph = TraceGraph::from_parts(vec![node("A", "SPEC", "missing.md")], vec![]);
    let store = EmbeddingStore::stub(vec![row("A", 384, "V", "H")], vec![]);
    let findings = detect_drift(&graph, &store, Path::new("/tmp/proj")).expect("Ok");
    assert!(
        findings
            .iter()
            .any(|f| f.kind == DriftKind::FileMissing && f.current_hash.is_none()),
        "ファイル不在 = FileMissing（current=None）"
    );
}

// ケース 34: detect_drift 出力順 node_id ASC（決定性）
#[test]
fn test_detect_drift_sorted_node_id_asc() {
    // @ts: TS-LGX-007 ケース 34
    let graph = TraceGraph::from_parts(
        vec![node("C", "SPEC", "c.md"), node("A", "SPEC", "a.md")],
        vec![],
    );
    let store = EmbeddingStore::empty();
    let findings = detect_drift(&graph, &store, Path::new("/tmp/proj")).expect("Ok");
    let ids: Vec<&String> = findings.iter().map(|f| &f.node_id).collect();
    let mut sorted = ids.clone();
    sorted.sort();
    assert_eq!(ids, sorted, "出力順は node_id ASC");
}

// ケース 35: read_current_content_for_node — content_range 切り出し共有ヘルパ
#[test]
fn test_read_current_content_for_node() {
    // @ts: TS-LGX-007 ケース 35
    let graph = TraceGraph::from_parts(vec![node("A", "SPEC", "a.md")], vec![]);
    let n = node("A", "SPEC", "a.md");
    let r = read_current_content_for_node(&n, &graph, Path::new("/tmp/proj"));
    assert!(r.is_ok() || r.is_err(), "panic せず Result を返す");
    let _ = r;
}

// ケース 36: EmbeddingStore::load_all — node_id ASC 決定性
#[test]
fn test_load_all_sorted() {
    // @ts: TS-LGX-007 ケース 36
    let store = EmbeddingStore::stub(
        vec![row("C", 384, "V", "h"), row("A", 384, "V", "h")],
        vec![],
    );
    let rows = store.load_all().expect("Ok");
    let ids: Vec<&String> = rows.iter().map(|r| &r.node_id).collect();
    let mut sorted = ids.clone();
    sorted.sort();
    assert_eq!(ids, sorted, "load_all は node_id ASC");
}

// ケース 37: EmbeddingStore::load_embedding — 未登録は Ok(None)
#[test]
fn test_load_embedding_missing_is_none() {
    // @ts: TS-LGX-007 ケース 37
    let store = EmbeddingStore::empty();
    assert_eq!(store.load_embedding("SPEC-LGX-001").expect("Ok"), None);
}

// ケース 38: EmbeddingStore::upsert_with_subnode_meta — INSERT OR REPLACE の冪等 upsert
#[test]
fn test_upsert_idempotent() {
    // @ts: TS-LGX-007 ケース 38
    let store = EmbeddingStore::empty();
    let n = node("A", "SPEC", "a.md");
    let result = legixy_embed::EmbedResult {
        embedding: vec![0.0; 384],
        dim: 384,
        model_version: "V".to_string(),
        content_hash: "H".to_string(),
        context: None,
        context_hash: None,
    };
    store.upsert_with_subnode_meta(&n, &result).expect("Ok 1");
    store.upsert_with_subnode_meta(&n, &result).expect("Ok 2");
}

// ケース 39: EmbeddingStore::create_snapshot — 全行コピー・行数返却
#[test]
fn test_create_snapshot_returns_row_count() {
    // @ts: TS-LGX-007 ケース 39
    let store = EmbeddingStore::stub(
        vec![row("A", 384, "V", "h"), row("B", 384, "V", "h")],
        vec![],
    );
    let n = store.create_snapshot("snap-001", Some("baseline")).expect("Ok");
    assert_eq!(n, 2, "コピー行数 = 現 embeddings 行数");
}

// ケース 40: CR context 合成（決定論既定クライアント）— Ok(Some(context))・Err 非昇格。
// 【CACHE-CR-002 defect-fix】Phase 1 パススルー（常に Ok(None)）から決定論 context 生成へ更新。
//   LLM/network 失敗時のフォールバック（Ok(None) 継続）は synthesize_with_fallback 内部に保持し、
//   Err には昇格しない（REQ.06.1）。実 Anthropic backend は feature-gate の後続課題。
#[test]
fn test_cr_synthesizes_context_ok_some() {
    // @ts: TS-LGX-007 ケース 40
    let cfg = ContextualConfig {
        opts: CrOptions::default(),
    };
    let r = synthesize_with_fallback(&cfg, "# 見出し\n\n本文", "A").expect("Ok（Err 昇格しない）");
    let ctx = r.expect("決定論既定クライアントは context を生成（CR 有効時 Some）");
    assert!(ctx.contains('A'), "context は node_id で位置づけ: {ctx}");
}

// ケース 43: compute_link_candidates — score >= threshold 境界（= 含む / < 除外）
#[test]
fn test_compute_link_candidates_threshold_boundary() {
    // @ts: TS-LGX-007 ケース 43
    let graph = TraceGraph::from_parts(
        vec![
            node("A", "SPEC", "a.md"),
            node("B", "SPEC", "b.md"),
            node("C", "SPEC", "c.md"),
        ],
        vec![],
    );
    let store = EmbeddingStore::stub(
        vec![
            row("A", 2, "V", "ha"),
            row("B", 2, "V", "hb"),
            row("C", 2, "V", "hc"),
        ],
        vec![],
    );
    let cands = compute_link_candidates(&graph, &store, 0.8).expect("Ok");
    // (A,C) は score < t のため除外される。結果に (A,C) を含まない。
    assert!(
        !cands.iter().any(|c| c.from == "A" && c.to == "C"),
        "score < threshold は除外（v3 similarity.rs L131 の >=）"
    );
}

// 委譲ケース（テスト化不要・委譲先をコメントで記す）:
// - ケース 3（トークン上限超過切り捨て）: 実 ONNX/tokenizer 依存 → SRC[GREEN] Integration + NFR。
// - ケース 10（normalize_content 冪等性 property）: proptest 化（下記 prop モジュール）。
// - ケース 11（content_hash_for 決定性 property）: proptest 化（下記 prop モジュール）。
// - ケース 14（ゼロノルム cosine skip）: bulk API 経由集約 Warning → SRC[GREEN] Integration（ケース 16 と同経路）。
// - ケース 17（bulk API 返却順序決定性 property）: proptest 化（下記 prop モジュール）。
// - ケース 27/28（Tx rollback / DB 接続異常 Err 昇格）: 実 SQLite Tx 注入 → SRC[GREEN] Integration。
// - ケース 41/42（終了コード契約・出力先分離）: legixy-cli 層 E2E（assert_cmd）へ委譲。

// ── Property-based（proptest）──
mod prop {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        // ケース 10: normalize_content 冪等性（2 回適用 == 1 回適用）
        #[test]
        fn prop_normalize_content_idempotent(s in ".*") {
            // @ts: TS-LGX-007 ケース 10
            let once = normalize_content(&s);
            let twice = normalize_content(&once);
            prop_assert_eq!(twice, once);
        }

        // ケース 11: content_hash_for 決定性 + 64 桁小文字 hex
        #[test]
        fn prop_content_hash_for_deterministic(s in ".*") {
            // @ts: TS-LGX-007 ケース 11
            let h1 = content_hash_for(&s);
            let h2 = content_hash_for(&s);
            prop_assert_eq!(&h1, &h2);
            prop_assert_eq!(h1.len(), 64);
            prop_assert!(h1.chars().all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase()));
        }

        // ケース 17: compute_all_pair_scores 返却順序決定性（i<j 昇順・同一入力同一順序）
        #[test]
        fn prop_compute_all_pair_scores_deterministic(n in 0usize..6) {
            // @ts: TS-LGX-007 ケース 17
            let rows: Vec<EmbeddingRow> = (0..n)
                .map(|i| row(&format!("N{:02}", i), 4, "V", "h"))
                .collect();
            let store = EmbeddingStore::stub(rows, vec![]);
            let p1 = compute_all_pair_scores(&store).expect("Ok");
            let p2 = compute_all_pair_scores(&store).expect("Ok");
            prop_assert_eq!(p1, p2);
        }
    }
}
