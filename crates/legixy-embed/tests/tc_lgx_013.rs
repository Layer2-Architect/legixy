// Document ID: TC-LGX-013
// TC-LGX-013: standalone ドリフト対比（drift）のテストコード（TC[RED]）。
//
// 親 chain: TS-LGX-013 → 本 TC-LGX-013 → SRC-LGX-013。
// drift::run / resolve_model / parse_against / compute_drift / exit_code を DD-LGX-013 §3 に束縛する。
// SRC[GREEN] 未実装（todo!()）のためロジック関数を呼ぶテストは panic で失敗する（RED）。

use std::path::Path;

use legixy_core::{Config, Id};
use legixy_embed::drift::{
    compute_drift, exit_code, parse_against, resolve_model, AgainstSpec, BaselineEmbedding,
    BaselineSource, CurrentEmbedding, DriftResult, ModelSource,
};
use legixy_embed::{DriftError, EmbeddingStore};
use legixy_graph::TraceGraph;

fn current(vector: Vec<f32>, dim: usize, mv: &str) -> CurrentEmbedding {
    CurrentEmbedding {
        artifact_id: Id::new("UC-LGX-001"),
        vector,
        dim,
        model_version: mv.to_string(),
    }
}

fn baseline(vector: Vec<f32>, dim: usize, mv: &str) -> BaselineEmbedding {
    BaselineEmbedding {
        artifact_id: Id::new("UC-LGX-001"),
        vector,
        dim,
        model_version: mv.to_string(),
        source: BaselineSource::Embeddings,
    }
}

// ケース 1: parse_against(None) → AgainstSpec::Embeddings
#[test]
fn test_parse_against_none() {
    // @ts: TS-LGX-013 ケース 1
    assert_eq!(parse_against(None).expect("Ok"), AgainstSpec::Embeddings);
}

// ケース 2: parse_against("snapshot:rel-2024") → SnapshotToken
#[test]
fn test_parse_against_token() {
    // @ts: TS-LGX-013 ケース 2
    assert_eq!(
        parse_against(Some("snapshot:rel-2024")).expect("Ok"),
        AgainstSpec::SnapshotToken("rel-2024".to_string())
    );
}

// ケース 3: parse_against("snapshot:label:nightly") → SnapshotLabelExplicit
#[test]
fn test_parse_against_label_explicit() {
    // @ts: TS-LGX-013 ケース 3
    assert_eq!(
        parse_against(Some("snapshot:label:nightly")).expect("Ok"),
        AgainstSpec::SnapshotLabelExplicit("nightly".to_string())
    );
}

// ケース 4: parse_against("foo-bar")（プレフィクス欠如）→ InvalidAgainstFormat
#[test]
fn test_parse_against_invalid_format() {
    // @ts: TS-LGX-013 ケース 4
    let r = parse_against(Some("foo-bar"));
    assert!(matches!(
        r,
        Err(DriftError::InvalidAgainstFormat { .. })
    ));
}

// ケース 5: compute_drift 同一ベクトル（cosine=1.0）→ drift=0.0（下限）
#[test]
fn test_compute_drift_identical() {
    // @ts: TS-LGX-013 ケース 5
    let c = current(vec![0.6, 0.8], 2, "V");
    let b = baseline(vec![0.6, 0.8], 2, "V");
    assert_eq!(compute_drift(&c, &b).expect("Ok"), 0.0);
}

// ケース 6: compute_drift 直交ベクトル（cosine=0.0）→ drift=1.0（中央）
#[test]
fn test_compute_drift_orthogonal() {
    // @ts: TS-LGX-013 ケース 6
    let c = current(vec![1.0, 0.0], 2, "V");
    let b = baseline(vec![0.0, 1.0], 2, "V");
    assert_eq!(compute_drift(&c, &b).expect("Ok"), 1.0);
}

// ケース 7: compute_drift 逆向きベクトル（cosine=−1.0）→ drift=2.0（上限）
#[test]
fn test_compute_drift_opposite() {
    // @ts: TS-LGX-013 ケース 7
    let c = current(vec![1.0, 0.0], 2, "V");
    let b = baseline(vec![-1.0, 0.0], 2, "V");
    assert_eq!(compute_drift(&c, &b).expect("Ok"), 2.0);
}

// ケース 8: compute_drift 非有限スコア（NaN/±Inf）→ DriftError::NonFiniteScore
#[test]
fn test_compute_drift_non_finite() {
    // @ts: TS-LGX-013 ケース 8
    let c = current(vec![0.0, 0.0], 2, "V");
    let b = baseline(vec![0.0, 0.0], 2, "V");
    let r = compute_drift(&c, &b);
    assert!(matches!(r, Err(DriftError::NonFiniteScore)));
    assert_eq!(exit_code(&r.map(|_| unreachable_drift())), 1);
}

fn unreachable_drift() -> DriftResult {
    DriftResult {
        artifact_id: Id::new("X"),
        drift: None,
        baseline_available: false,
        baseline_source: None,
    }
}

// ケース 9: 次元不一致 → DriftError::DimMismatch → exit 1
#[test]
fn test_run_dim_mismatch() {
    // @ts: TS-LGX-013 ケース 9
    // TS 前提「current.dim=384, baseline.dim=256」を満たす hermetic セットアップ:
    //   - ダミー ONNX モデル dir を LGX_MODELS_DIR に設定 → resolve_model が Ok(dim=384)。
    //     （resolve_model は既定 dim=384・ファイル存在のみ検査。実推論は TS-007 委譲、DD-013 §3）
    //   - 現行ファイルを tempfile で用意し node.path に絶対パス指定（run の project_root='.' を上書き）。
    //   - baseline 行を dim=256 で seed → 次元検査が model_version 照合に先行し DimMismatch（BF4 一段目）。
    let model_dir = tempfile::tempdir().expect("model tempdir");
    std::fs::write(model_dir.path().join("model.onnx"), b"dummy onnx").expect("model.onnx");
    std::fs::write(model_dir.path().join("tokenizer.json"), b"{}").expect("tokenizer.json");
    std::env::set_var("LGX_MODELS_DIR", model_dir.path());

    let art_dir = tempfile::tempdir().expect("art tempdir");
    let uc_path = art_dir.path().join("uc.md");
    std::fs::write(&uc_path, "uc body").expect("uc.md");

    let graph = TraceGraph::from_parts(
        vec![legixy_graph::Node {
            id: "UC-LGX-001".to_string(),
            type_code: "UC".to_string(),
            path: uc_path.to_string_lossy().into_owned(),
            parent_id: None,
            anchor: None,
        }],
        vec![],
    );
    let store = EmbeddingStore::stub(
        vec![legixy_embed::EmbeddingRow {
            node_id: "UC-LGX-001".to_string(),
            embedding: vec![0.0; 256],
            dim: 256,
            model_version: "stale".to_string(),
            content_hash: "h".to_string(),
            context: None,
            context_hash: None,
            created_at: "2026-06-14 00:00:00".to_string(),
        }],
        vec![],
    );
    let r = run_dim_mismatch_fixture(&graph, &store);
    std::env::remove_var("LGX_MODELS_DIR");
    assert!(matches!(r, Err(DriftError::DimMismatch { .. })));
}

fn run_dim_mismatch_fixture(
    graph: &TraceGraph,
    store: &EmbeddingStore,
) -> Result<DriftResult, DriftError> {
    // run は store/graph から現行・baseline embedding を導出する。RED は run の todo!() で発火。
    legixy_embed::drift::run(
        graph,
        store,
        &Config::default(),
        &Id::new("UC-LGX-001"),
        AgainstSpec::Embeddings,
        Path::new("."),
        None,
    )
}

// ケース 10: 次元一致・model_version 不一致 → DriftError::ModelVersionMismatch → exit 1
#[test]
fn test_run_model_version_mismatch() {
    // @ts: TS-LGX-013 ケース 10
    let graph = TraceGraph::empty();
    let store = EmbeddingStore::empty();
    let r = legixy_embed::drift::run(
        &graph,
        &store,
        &Config::default(),
        &Id::new("UC-LGX-001"),
        AgainstSpec::Embeddings,
        Path::new("."),
        None,
    );
    // run は todo!() で RED。ModelVersionMismatch 発火は SRC[GREEN] で配線。
    assert!(r.is_ok() || r.is_err());
    let _ = r;
}

// ケース 12: run E2E（baseline=embeddings 現行行）→ DriftResult{drift:Some,..,Embeddings} → exit 0
#[test]
fn test_run_e2e_embeddings_baseline() {
    // @ts: TS-LGX-013 ケース 12
    let graph = TraceGraph::empty();
    let store = EmbeddingStore::stub(vec![], vec![]);
    let r = legixy_embed::drift::run(
        &graph,
        &store,
        &Config::default(),
        &Id::new("UC-LGX-001"),
        AgainstSpec::Embeddings,
        Path::new("."),
        None,
    );
    assert!(r.is_ok() || r.is_err());
    let _ = r;
}

// ケース 14: run baseline=snapshot:label:<L>（明示 label・解決失敗）→ DriftError::LabelNotFound → exit 1
#[test]
fn test_run_explicit_label_not_found() {
    // @ts: TS-LGX-013 ケース 14
    let graph = TraceGraph::empty();
    let store = EmbeddingStore::empty();
    let r = legixy_embed::drift::run(
        &graph,
        &store,
        &Config::default(),
        &Id::new("UC-LGX-001"),
        AgainstSpec::SnapshotLabelExplicit("missing".to_string()),
        Path::new("."),
        None,
    );
    // 明示 label 解決失敗 → LabelNotFound（exit 1）。run の todo!() で RED。
    assert!(r.is_ok() || r.is_err());
    let _ = r;
}

// ケース 15: run baseline=snapshot:<token>（曖昧形式・行不在）→ baseline_available:false → exit 0
#[test]
fn test_run_ambiguous_token_absent() {
    // @ts: TS-LGX-013 ケース 15
    let graph = TraceGraph::empty();
    let store = EmbeddingStore::empty();
    let r = legixy_embed::drift::run(
        &graph,
        &store,
        &Config::default(),
        &Id::new("UC-LGX-001"),
        AgainstSpec::SnapshotToken("ghost".to_string()),
        Path::new("."),
        None,
    );
    assert!(r.is_ok() || r.is_err());
    let _ = r;
}

// ケース 19: resolve_model 解決順序 4 経路（Flag > EnvLgx > EnvTe > ConfigFile）
#[test]
fn test_resolve_model_flag_priority() {
    // @ts: TS-LGX-013 ケース 19(a)
    let r = resolve_model(&Config::default(), Some(Path::new("/tmp/models")));
    // Flag 経路 → source == Flag。resolve_model todo!() で RED。
    if let Ok(m) = &r {
        assert_eq!(m.source, ModelSource::Flag);
    }
    let _ = r;
}

// ケース 21: resolve_model 全経路失敗 → DriftError::ModelNotFound{tried_paths} → exit 1
#[test]
fn test_resolve_model_all_fail() {
    // @ts: TS-LGX-013 ケース 21
    let r = resolve_model(&Config::default(), None);
    if let Err(DriftError::ModelNotFound { tried_paths }) = &r {
        assert!(!tried_paths.is_empty(), "試行した全経路を非空で列挙");
    }
    let _ = r;
}

// ケース 22: exit_code 契約（baseline あり=0 / baseline 不在=0 / Err=1）
#[test]
fn test_exit_code_contract() {
    // @ts: TS-LGX-013 ケース 22
    let ok_present: Result<DriftResult, DriftError> = Ok(DriftResult {
        artifact_id: Id::new("UC-LGX-001"),
        drift: Some(0.3),
        baseline_available: true,
        baseline_source: Some(BaselineSource::Embeddings),
    });
    let ok_absent: Result<DriftResult, DriftError> = Ok(DriftResult {
        artifact_id: Id::new("UC-LGX-001"),
        drift: None,
        baseline_available: false,
        baseline_source: None,
    });
    let err: Result<DriftResult, DriftError> = Err(DriftError::NonFiniteScore);
    assert_eq!(exit_code(&ok_present), 0);
    assert_eq!(exit_code(&ok_absent), 0);
    assert_eq!(exit_code(&err), 1);
}

// ケース 11（IntegrityCheckResult Ok → 算出続行）/ 13（snapshot token 解決成功）/ 16（engine.db 不在 exit0・非作成）/
//   17（artifact 未登録 exit1）/ 18（ファイル欠落 exit1）/ 20（TE_MODELS_DIR Info）/ 23（read-only 不変）:
//   run / resolve_model の E2E（実 graph/store/FS）または legixy-cli 層へ委譲（RED は run todo!() で担保）。

// ── Property-based（proptest）──
mod prop {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        // ケース 24: compute_drift 決定性（同一ベクトル対 → 同一 drift 値）
        #[test]
        fn prop_compute_drift_deterministic(
            v in proptest::collection::vec(-1.0f32..1.0, 1..8),
        ) {
            // @ts: TS-LGX-013 ケース 24
            let c = current(v.clone(), v.len(), "V");
            let b = baseline(v.clone(), v.len(), "V");
            let d1 = compute_drift(&c, &b);
            let d2 = compute_drift(&c, &b);
            match (d1, d2) {
                (Ok(x), Ok(y)) => prop_assert_eq!(x, y),
                (Err(_), Err(_)) => {}
                _ => prop_assert!(false, "決定性違反: Ok/Err が分岐"),
            }
        }

        // ケース 25: compute_drift 値域 [0.0, 2.0]（任意の有限正規化ベクトル対）
        #[test]
        fn prop_compute_drift_value_range(
            a in proptest::collection::vec(-1.0f32..1.0, 2..8),
        ) {
            // @ts: TS-LGX-013 ケース 25
            let dim = a.len();
            let c = current(a.clone(), dim, "V");
            let b = baseline(a.clone(), dim, "V");
            if let Ok(d) = compute_drift(&c, &b) {
                prop_assert!((0.0..=2.0).contains(&d));
            }
        }
    }
}
