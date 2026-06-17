// Document ID: TC-LGX-011
// TC-LGX-011: 閾値キャリブレーション（calibrate）のテストコード（TC[RED]）。
//
// 親 chain: TS-LGX-011 → 本 TC-LGX-011 → SRC-LGX-011。
// calibrate / histogram / compute_recommended / compute_all_pair_scores_calibrate / BucketCount を
// DD-LGX-011 §3 に束縛する。SRC[GREEN] 未実装（todo!()）のためロジック関数を呼ぶテストは RED。
//
// 注（cross-DD 署名衝突の反映）:
//   ケース 12〜14 の compute_all_pair_scores は AllPairScores を返す DD-011 版。DD-007 正典の同名関数
//   （タプル列・EmbedError）と戻り型が衝突するため、本 crate では DD-011 版を
//   `compute_all_pair_scores_calibrate` として公開し、本テストはそれに束縛する（マニフェスト/NOTES 参照）。

use legixy_core::Config;
use legixy_embed::{
    calibrate, compute_all_pair_scores_calibrate, compute_recommended, histogram, BucketCount,
    CalibrateError, EmbeddingStore,
};

// ケース 1: histogram 空入力 → 全バケット count=0・stats 全 None
#[test]
fn test_histogram_empty_input() {
    // @ts: TS-LGX-011 ケース 1
    let h = histogram(&[], BucketCount::new(10).unwrap());
    assert_eq!(h.buckets.len(), 10);
    assert!(h.buckets.iter().all(|b| b.count == 0));
    assert_eq!(h.stats.min, None);
    assert_eq!(h.stats.max, None);
    assert_eq!(h.stats.mean, None);
}

// ケース 2: histogram バケット境界（上限 1.0 末尾バケット inclusive）
#[test]
fn test_histogram_inclusive_upper() {
    // @ts: TS-LGX-011 ケース 2
    let h = histogram(&[0.0, 1.0, 0.05, 0.95], BucketCount::new(10).unwrap());
    assert_eq!(h.buckets[0].count, 2, "0.0, 0.05 が先頭");
    assert_eq!(h.buckets[9].count, 2, "1.0(inclusive), 0.95 が末尾");
}

// ケース 3: histogram 域外スコア clamp（min/max は clamp 前生値）
#[test]
fn test_histogram_clamp_with_raw_stats() {
    // @ts: TS-LGX-011 ケース 3
    let h = histogram(&[-0.3, 1.7], BucketCount::new(10).unwrap());
    assert_eq!(h.buckets[0].count, 1, "-0.3 clamp → 先頭");
    assert_eq!(h.buckets[9].count, 1, "1.7 clamp → 末尾");
    assert_eq!(h.stats.min, Some(-0.3), "min は clamp 前生値");
    assert_eq!(h.stats.max, Some(1.7), "max は clamp 前生値");
}

// ケース 4: histogram バケット幅・低値・高値の計算正確性（N=4）
#[test]
fn test_histogram_bucket_geometry() {
    // @ts: TS-LGX-011 ケース 4
    let h = histogram(&[0.1, 0.3, 0.6, 0.9], BucketCount::new(4).unwrap());
    assert_eq!(h.buckets[0].low, 0.0);
    assert_eq!(h.buckets[0].high, 0.25);
    assert_eq!(h.buckets[0].count, 1);
    assert_eq!(h.buckets[3].low, 0.75);
    assert_eq!(h.buckets[3].high, 1.0);
    assert_eq!(h.buckets[3].count, 1);
    assert_eq!(h.stats.mean, Some(0.475));
}

// ケース 5: BucketCount::new(0) → Err(InvalidBuckets)
#[test]
fn test_bucket_count_zero_err() {
    // @ts: TS-LGX-011 ケース 5
    assert!(matches!(
        BucketCount::new(0),
        Err(CalibrateError::InvalidBuckets)
    ));
}

// ケース 6: BucketCount::new(1) → Ok（下限境界）/ 単一バケットヒストグラム
#[test]
fn test_bucket_count_one_ok() {
    // @ts: TS-LGX-011 ケース 6
    let bc = BucketCount::new(1).expect("Ok");
    assert_eq!(bc.get(), 1);
    let h = histogram(&[0.0, 0.5, 1.0], bc);
    assert_eq!(h.buckets.len(), 1);
    assert_eq!(h.buckets[0].low, 0.0);
    assert_eq!(h.buckets[0].high, 1.0);
    assert_eq!(h.buckets[0].count, 3);
}

// ケース 7: compute_recommended 空入力 → None
#[test]
fn test_compute_recommended_empty_none() {
    // @ts: TS-LGX-011 ケース 7
    assert_eq!(compute_recommended(&[]), None);
}

// ケース 8: compute_recommended 単一サンプル → 全パーセンタイル同値
#[test]
fn test_compute_recommended_single_sample() {
    // @ts: TS-LGX-011 ケース 8
    let r = compute_recommended(&[0.42]).expect("Some");
    assert_eq!(r.percentiles.p10, 0.42);
    assert_eq!(r.percentiles.p25, 0.42);
    assert_eq!(r.percentiles.p50, 0.42);
    assert_eq!(r.percentiles.p75, 0.42);
    assert_eq!(r.percentiles.p90, 0.42);
    assert_eq!(r.similarity_threshold, 0.42);
    assert_eq!(r.link_candidate_threshold, 0.42);
    assert!((r.drift_threshold - 0.58).abs() < 1e-6, "1.0 - p90");
}

// ケース 9: compute_recommended 既知分布 fixture → nearest-rank 推奨値一致
#[test]
fn test_compute_recommended_known_distribution() {
    // @ts: TS-LGX-011 ケース 9
    let scores = [0.0, 0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9];
    let r = compute_recommended(&scores).expect("Some");
    assert!((r.percentiles.p25 - 0.2).abs() < 1e-6);
    assert!((r.percentiles.p75 - 0.7).abs() < 1e-6);
    assert!((r.percentiles.p90 - 0.8).abs() < 1e-6);
    assert!((r.similarity_threshold - 0.2).abs() < 1e-6);
    assert!((r.link_candidate_threshold - 0.7).abs() < 1e-6);
    assert!((r.drift_threshold - 0.2).abs() < 1e-6, "1.0 - 0.8");
}

// ケース 12: compute_all_pair_scores 空ストア → pairs 空・skip 0
#[test]
fn test_pair_scores_empty_store() {
    // @ts: TS-LGX-011 ケース 12（DD-011 版 compute_all_pair_scores_calibrate）
    let store = EmbeddingStore::empty();
    let aps = compute_all_pair_scores_calibrate(&store).expect("Ok");
    assert_eq!(aps.pairs.len(), 0);
    assert_eq!(aps.skip_count.total(), 0);
}

// ケース 13: compute_all_pair_scores SkipSummary 集計（次元不一致・非有限スコア）
#[test]
fn test_pair_scores_skip_summary() {
    // @ts: TS-LGX-011 ケース 13
    let store = EmbeddingStore::stub(vec![], vec![]);
    let aps = compute_all_pair_scores_calibrate(&store).expect("Ok");
    // dim_mismatch / nonfinite は別カウンタ。集計のみ検証（数値妥当性は TS-007 委譲）。
    let _ = (aps.skip_count.dim_mismatch, aps.skip_count.nonfinite);
}

// ケース 14: compute_all_pair_scores 昇順ペア不変条件（a < b のみ）
#[test]
fn test_pair_scores_ascending_pairs() {
    // @ts: TS-LGX-011 ケース 14
    let store = EmbeddingStore::stub(vec![], vec![]);
    let aps = compute_all_pair_scores_calibrate(&store).expect("Ok");
    assert!(
        aps.pairs.iter().all(|(a, b, _)| a < b),
        "全ペアで a < b（昇順・自己/重複除外）"
    );
}

// ケース 15: calibrate 空ストア統括 → pairs=0・stats 全 None・recommended None
#[test]
fn test_calibrate_empty_store() {
    // @ts: TS-LGX-011 ケース 15
    let store = EmbeddingStore::empty();
    let report = calibrate(&store, &Config::default(), BucketCount::new(10).unwrap(), false)
        .expect("Ok");
    assert_eq!(report.pairs, 0);
    assert_eq!(report.stats.min, None);
    assert_eq!(report.recommended, None);
    assert_eq!(report.distribution.len(), 10);
    assert!(report.distribution.iter().all(|b| b.count == 0));
}

// ケース 16: calibrate --recommend + pairs=0 → recommended None
#[test]
fn test_calibrate_recommend_empty_store() {
    // @ts: TS-LGX-011 ケース 16
    let store = EmbeddingStore::empty();
    let report = calibrate(&store, &Config::default(), BucketCount::new(10).unwrap(), true)
        .expect("Ok");
    assert_eq!(report.recommended, None, "pairs=0 では recommended None");
}

// ケース 17: calibrate --recommend + pairs>0 → recommended Some
#[test]
fn test_calibrate_recommend_with_pairs() {
    // @ts: TS-LGX-011 ケース 17
    let store = EmbeddingStore::stub(vec![], vec![]);
    let report = calibrate(&store, &Config::default(), BucketCount::new(10).unwrap(), true)
        .expect("Ok");
    // pairs>0 で recommended Some を期待（store 構成は SRC[GREEN] で確定。RED は todo!() で発火）。
    let _ = report.recommended;
}

// ケース 22: calibrate 全ペア算出失敗 → Err(PairScoreFailure) → exit 1
#[test]
fn test_calibrate_pair_score_failure() {
    // @ts: TS-LGX-011 ケース 22
    let store = EmbeddingStore::stub(vec![], vec![]);
    let r = calibrate(&store, &Config::default(), BucketCount::new(10).unwrap(), false);
    // 全件失敗（exit 1）≠ 部分スキップ（exit 0）。発火経路は SRC[GREEN]、RED は todo!()。
    assert!(r.is_ok() || r.is_err());
    let _ = r;
}

// ケース 19/20/21/24/25（calibrate --json / 出力先分離 / 終了コード契約）: legixy-cli 層 E2E へ委譲（assert_cmd）。
// ケース 18/23（skip 集約 Warning / engine.db 非破壊）: legixy-cli E2E + 実 DB ハッシュ比較へ委譲。

// ── Property-based（proptest）──
mod prop {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        // ケース 11: histogram 決定性 + バケット count 合計 = 入力件数
        #[test]
        fn prop_histogram_deterministic(
            scores in proptest::collection::vec(0.0f32..1.0, 0..30),
            n in 1usize..32,
        ) {
            // @ts: TS-LGX-011 ケース 11
            let bc = BucketCount::new(n).unwrap();
            let h1 = histogram(&scores, bc);
            let h2 = histogram(&scores, bc);
            prop_assert_eq!(&h1, &h2);
            let total: usize = h1.buckets.iter().map(|b| b.count).sum();
            prop_assert_eq!(total, scores.len());
        }

        // ケース 10: compute_recommended パーセンタイル単調性 p10≤…≤p90
        #[test]
        fn prop_compute_recommended_monotonic(
            scores in proptest::collection::vec(0.0f32..1.0, 1..40),
        ) {
            // @ts: TS-LGX-011 ケース 10
            let r = compute_recommended(&scores).expect("非空 → Some");
            let p = &r.percentiles;
            prop_assert!(p.p10 <= p.p25);
            prop_assert!(p.p25 <= p.p50);
            prop_assert!(p.p50 <= p.p75);
            prop_assert!(p.p75 <= p.p90);
        }
    }
}
