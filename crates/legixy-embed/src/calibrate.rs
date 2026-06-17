// Document ID: SRC-LGX-011
// calibrate / compute_all_pair_scores_calibrate / histogram / compute_recommended と関連型
// （DD-LGX-011 §2.1・§2.2・§2.3・§3・§3.2）。
//
// TC[RED] scaffold。Histogram バケツ境界・nearest-rank パーセンタイル・RecommendedThresholds・
// CalibrateReport は SRC[GREEN] で実装する。
//
// 【上流裁定の反映 / cross-DD 署名衝突】
//   - `histogram`: DD-007 §3 は `(impl Iterator<Item=f32>, usize) -> Vec<Bucket>`、DD-011 §3 は
//     `(&[f32], BucketCount) -> Histogram` を凍結。割当指示「histogram 定義域 [0,1]、calibrate 所有は
//     DD-011」に従い **DD-011 正典**（`&[f32]` + `BucketCount` → `Histogram`）を採用する。TS-007
//     ケース 18/19（histogram 値域・clamp の委譲確認）は本 DD-011 正典シグネチャに束縛する。
//   - `compute_all_pair_scores`: DD-007 正典（タプル列）は similarity.rs。本ファイルの calibrate 向け
//     `AllPairScores` 返却版は `compute_all_pair_scores_calibrate` として置く（DD-011 §3 の意図を保持）。
//     TS-011 ケース 12〜14 はこちらに束縛する。NOTES に申し送り。
//   - `Bucket`: DD-007 §2.1 と DD-011 §2.1 が同一構造（low/high/count）で定義。本 crate で 1 回だけ定義し
//     similarity 経由でも参照する（ADR-LGX-020 §2.3「所有 crate で 1 回だけ定義」の精神）。

use legixy_core::{Config, Id};

use crate::error::CalibrateError;
use crate::store::EmbeddingStore;

/// ヒストグラムバケット（DD-LGX-007 §2.1 / DD-LGX-011 §2.1。本 crate 単一定義）。
#[derive(Debug, Clone, PartialEq)]
pub struct Bucket {
    pub low: f32,
    pub high: f32,
    pub count: usize,
}

/// ヒストグラム統計（DD-LGX-011 §2.1。min/max/mean は clamp 前生値。空入力時 None）。
#[derive(Debug, Clone, PartialEq)]
pub struct HistogramStats {
    pub min: Option<f32>,
    pub max: Option<f32>,
    pub mean: Option<f32>,
}

/// ヒストグラム全体（DD-LGX-011 §2.1、histogram の戻り値）。
#[derive(Debug, Clone, PartialEq)]
pub struct Histogram {
    pub buckets: Vec<Bucket>,
    pub stats: HistogramStats,
}

/// スキップ集計（DD-LGX-011 §2.1。次元不一致・非有限スコア）。
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SkipSummary {
    pub dim_mismatch: usize,
    pub nonfinite: usize,
}

impl SkipSummary {
    pub fn total(&self) -> usize {
        self.dim_mismatch + self.nonfinite
    }
}

/// 全ペア類似度集合（DD-LGX-011 §2.1、compute_all_pair_scores_calibrate の戻り値）。
#[derive(Debug, Clone, PartialEq)]
pub struct AllPairScores {
    /// (node_a, node_b, score)。node_a < node_b（昇順ペア）。
    pub pairs: Vec<(Id, Id, f32)>,
    pub skip_count: SkipSummary,
}

/// パーセンタイル算出結果（DD-LGX-011 §2.1、compute_recommended の戻り値）。
#[derive(Debug, Clone, PartialEq)]
pub struct RecommendedThresholds {
    pub similarity_threshold: f32,
    pub drift_threshold: f32,
    pub link_candidate_threshold: f32,
    pub percentiles: Percentiles,
}

/// 参考パーセンタイル p10/p25/p50/p75/p90（DD-LGX-011 §2.1）。
#[derive(Debug, Clone, PartialEq)]
pub struct Percentiles {
    pub p10: f32,
    pub p25: f32,
    pub p50: f32,
    pub p75: f32,
    pub p90: f32,
}

/// 設定ファイルの現閾値（DD-LGX-011 §2.1）。
#[derive(Debug, Clone, PartialEq)]
pub struct CurrentThresholds {
    pub similarity_threshold: f32,
    pub drift_threshold: f32,
    pub link_candidate_threshold: f32,
}

/// calibrate コマンドの集約出力（DD-LGX-011 §2.1）。
#[derive(Debug, Clone, PartialEq)]
pub struct CalibrateReport {
    pub pairs: usize,
    pub stats: HistogramStats,
    pub distribution: Vec<Bucket>,
    pub thresholds: CurrentThresholds,
    /// --recommend 時のみ Some。
    pub recommended: Option<RecommendedThresholds>,
}

/// --buckets の値。1 以上を保証するラッパ（DD-LGX-011 §2.2。0 は InvalidBuckets）。
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct BucketCount(usize);

impl BucketCount {
    pub fn new(n: usize) -> Result<Self, CalibrateError> {
        if n == 0 {
            Err(CalibrateError::InvalidBuckets)
        } else {
            Ok(BucketCount(n))
        }
    }

    pub fn get(&self) -> usize {
        self.0
    }
}

/// 早期終了種別（DD-LGX-011 §2.2、空ストア）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EarlyExit {
    EmptyStore,
}

/// calibrate 向け全ペア類似度算出（DD-LGX-011 §3 の `compute_all_pair_scores` 意図）。
/// node_id 昇順ペア（a < b）。非有限・次元不一致は skip し SkipSummary に計上。read-only。
/// （DD-007 正典の同名関数 similarity::compute_all_pair_scores とは戻り型が異なるため別名。NOTES 参照）
pub fn compute_all_pair_scores_calibrate(
    store: &EmbeddingStore,
) -> Result<AllPairScores, CalibrateError> {
    let rows = store
        .load_all()
        .map_err(CalibrateError::Db)?;
    let mut pairs = Vec::new();
    let mut skip_count = SkipSummary::default();
    for i in 0..rows.len() {
        for j in (i + 1)..rows.len() {
            let a = &rows[i];
            let b = &rows[j];
            if a.dim != b.dim || a.dim == 0 {
                skip_count.dim_mismatch += 1;
                continue;
            }
            let score = crate::similarity::cosine_similarity(&a.embedding, &b.embedding);
            if !score.is_finite() {
                skip_count.nonfinite += 1;
                continue;
            }
            // load_all は node_id ASC のため i<j で a < b 昇順ペアが保証される。
            pairs.push((Id::new(a.node_id.clone()), Id::new(b.node_id.clone()), score));
        }
    }
    Ok(AllPairScores { pairs, skip_count })
}

/// 値域 [0.0, 1.0] 固定の等幅 N バケット（DD-LGX-011 §3 正典）。域外は clamp して算入。
/// 上限 1.0 は末尾バケット inclusive。min/max/mean は clamp 前生値。空入力時 stats 全 None。
pub fn histogram(scores: &[f32], buckets: BucketCount) -> Histogram {
    let n = buckets.get();
    let bucket_width = 1.0_f32 / n as f32;
    let mut counts = vec![0usize; n];
    for &score in scores {
        let clamped = score.clamp(0.0, 1.0);
        let idx = if clamped >= 1.0 {
            n - 1
        } else {
            ((clamped / bucket_width).floor() as usize).min(n - 1)
        };
        counts[idx] += 1;
    }
    let buckets_vec: Vec<Bucket> = (0..n)
        .map(|i| {
            let low = i as f32 * bucket_width;
            let high = low + bucket_width;
            Bucket {
                low,
                high,
                count: counts[i],
            }
        })
        .collect();

    // stats は clamp 前生値。空入力時は全 None。
    let stats = if scores.is_empty() {
        HistogramStats {
            min: None,
            max: None,
            mean: None,
        }
    } else {
        let mut mn = f32::INFINITY;
        let mut mx = f32::NEG_INFINITY;
        let mut sum = 0.0_f32;
        for &s in scores {
            if s < mn {
                mn = s;
            }
            if s > mx {
                mx = s;
            }
            sum += s;
        }
        HistogramStats {
            min: Some(mn),
            max: Some(mx),
            mean: Some(sum / scores.len() as f32),
        }
    };

    Histogram {
        buckets: buckets_vec,
        stats,
    }
}

/// nearest-rank パーセンタイル（DD-LGX-011 §3、`idx = round((n-1)*frac); sorted[min(idx,n-1)]`）。
/// scores が空のとき None。similarity_threshold=p25, drift_threshold=1.0-p90, link_candidate_threshold=p75。
pub fn compute_recommended(scores: &[f32]) -> Option<RecommendedThresholds> {
    if scores.is_empty() {
        return None;
    }
    let mut sorted: Vec<f32> = scores.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let n = sorted.len();
    let pick = |frac: f64| -> f32 {
        let idx = (((n as f64) - 1.0) * frac).round() as usize;
        sorted[idx.min(n - 1)]
    };
    let p10 = pick(0.10);
    let p25 = pick(0.25);
    let p50 = pick(0.50);
    let p75 = pick(0.75);
    let p90 = pick(0.90);
    Some(RecommendedThresholds {
        similarity_threshold: p25,
        drift_threshold: 1.0 - p90,
        link_candidate_threshold: p75,
        percentiles: Percentiles {
            p10,
            p25,
            p50,
            p75,
            p90,
        },
    })
}

/// calibrate 統括（DD-LGX-011 §3）。空ストア時は pairs=0・stats 全 None・distribution N バケット
/// （count 0）・recommended None。read-only（engine.db 不変）。
pub fn calibrate(
    store: &EmbeddingStore,
    _config: &Config,
    buckets: BucketCount,
    recommend: bool,
) -> Result<CalibrateReport, CalibrateError> {
    // 現閾値（Config に意味層閾値が無い scaffold では既定値を提示）。
    let thresholds = CurrentThresholds {
        similarity_threshold: 0.75,
        drift_threshold: 0.25,
        link_candidate_threshold: 0.70,
    };

    let aps = compute_all_pair_scores_calibrate(store)?;
    let scores: Vec<f32> = aps.pairs.iter().map(|(_, _, s)| *s).collect();

    let hist = histogram(&scores, buckets);

    let recommended = if recommend && !scores.is_empty() {
        compute_recommended(&scores)
    } else {
        None
    };

    Ok(CalibrateReport {
        pairs: scores.len(),
        stats: hist.stats.clone(),
        distribution: hist.buckets,
        thresholds,
        recommended,
    })
}
