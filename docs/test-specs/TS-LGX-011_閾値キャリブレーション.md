Document ID: TS-LGX-011

# TS-LGX-011: 閾値キャリブレーション（calibrate）のテスト仕様

> TS は **上流 TP の翻訳**。ゼロから観点を考えない。各 TP 観点を DD-LGX-011 で確定した型・関数シグネチャに即した具体的な入力・期待出力・前提条件へ展開する。

**親 DD**: DD-LGX-011
**継承 TP**: TP-LGX-010（TP[SPEC] embedding 運用・監査、71 観点）, TP-LGX-021（TP[UC] UC-011 閾値キャリブレーション、24 観点）

## 1. 対象範囲

このテスト仕様がカバーする DD-LGX-011 の関数 / 型:

- DD-LGX-011 §3 `legixy_embed::compute_all_pair_scores(store: &EmbeddingStore) -> Result<AllPairScores, CalibrateError>`（本 TS は SkipSummary 集計・空ストア・昇順ペア不変条件・決定論のみ。bulk pair スコアの数値妥当性は委譲）
- DD-LGX-011 §3 `legixy_embed::histogram(scores: &[f32], buckets: BucketCount) -> Histogram`
- DD-LGX-011 §3 `legixy_embed::compute_recommended(scores: &[f32]) -> Option<RecommendedThresholds>`
- DD-LGX-011 §3 `legixy_embed::calibrate(store: &EmbeddingStore, config: &Config, buckets: BucketCount, recommend: bool) -> Result<CalibrateReport, CalibrateError>`
- DD-LGX-011 §2.2 `BucketCount::new(n: usize) -> Result<Self, CalibrateError>` / `BucketCount::get(&self) -> usize`
- DD-LGX-011 §2.1 型: `AllPairScores` / `Bucket` / `Histogram` / `HistogramStats`{min/max/mean: Option<f32>} / `RecommendedThresholds` / `Percentiles` / `CalibrateReport` / `CurrentThresholds` / `SkipSummary`(+`total()`)
- DD-LGX-011 §2.2 `BucketCount` / `EarlyExit`{EmptyStore}、§2.3 `CalibrateError`{InvalidBuckets, PairScoreFailure, Db, Config}
- DD-LGX-011 §3.1 legixy-cli consumer（INFO/WARNING の stderr 出力・exit 判定）、§3.2 `CalibrateJsonOutput` / `BucketJson` / `ThresholdsJson` / `RecommendedJson`

委譲（本 TS 対象外）:
- **bulk pair スコアの数値妥当性**（コサイン類似度計算自体・[-1,1] 値域・clamp の実体・次元一致ペアの正しいスコア値）→ **TS-LGX-007**（`compute_all_pair_scores` の類似度算出ロジック所有。SPEC-LGX-006 REQ.04/11）。本 TS は同関数の skip 集計・出力契約面のみ。
- **意味層スコアの閾値妥当性・report 側統計**（report links / summary）→ **TS-LGX-010**（report コマンド）
- **性能予算 PERF.07**（N=500 ペア数 124,750 の応答時間、criterion）→ **bench / NFR-LGX-001**
- **engine.db 並行アクセス整合・WAL**（SEC.02 / REL.07）→ **NFR-LGX-001**
- snapshot / drift コマンド固有（baseline 不変性・TOCTOU・モデル解決）→ TS-LGX-012/013。

本 TS は「calibrate が SPEC-LGX-010 REQ.05/REQ.09/REQ.01 の規定を DD-LGX-011 の型で正しく具体化しているか」（Histogram バケツ境界・nearest-rank パーセンタイル・RecommendedThresholds 算出・`--recommend`・CalibrateReport・空/単一サンプル境界・決定論・skip 集約 Warning）を検証する。

## 2. ケース一覧

### ケース 1: histogram 空入力（scores 0 件）→ 全バケット count=0・stats 全 None

- **観点出典**: TP-LGX-010 §2.1 B6（値域固定・空入力統計）, TP-LGX-021 §2.6 R2
- **分類**: Unit
- **前提**: `scores = &[]`、`buckets = BucketCount::new(10).unwrap()`
- **入力**: `histogram(&[], buckets)`
- **期待**: `Histogram{ buckets: Vec<Bucket>（長さ 10、全 Bucket.count == 0）, stats: HistogramStats{ min: None, max: None, mean: None } }`。各 Bucket の `low`/`high` は [0.0,1.0] 等幅（low=i/10, high=(i+1)/10）
- **境界条件**: 空入力 = 全バケット 0 件 + 統計 None（panic せず正常値を返す。DD §3 不変条件「空入力時は全バケット count=0・stats.min/max/mean=None」）

### ケース 2: histogram バケット境界（上限 1.0 末尾バケット inclusive）

- **観点出典**: TP-LGX-010 §2.1 B6（上限 1.0 inclusive）, TP-LGX-021 §2.6 R2
- **分類**: Unit
- **前提**: `buckets = BucketCount::new(10).unwrap()`。`scores = &[0.0, 1.0, 0.05, 0.95]`
- **入力**: `histogram(&[0.0, 1.0, 0.05, 0.95], buckets)`
- **期待**: `0.0` と `0.05` は先頭バケット（low=0.0, high=0.1）に算入、`1.0` と `0.95` は末尾バケット（low=0.9, high=1.0）に算入。末尾バケットは上限 1.0 を inclusive（`1.0` が末尾に入り、N+1 番目には溢れない）。`buckets[0].count == 2`、`buckets[9].count == 2`
- **境界条件**: 下限 0.0（先頭 inclusive）/ 上限 1.0（末尾 inclusive）。バケット幅 = 1.0/N

### ケース 3: histogram 域外スコア clamp（< 0.0 / > 1.0 を端バケットへ）

- **観点出典**: TP-LGX-010 §2.1 B6（域外 clamp）, TP-LGX-021 §2.6 R2
- **分類**: Unit
- **前提**: `buckets = BucketCount::new(10).unwrap()`。`scores = &[-0.3, 1.7]`（コサイン値域 [-1,1] 由来の域外。算入時 clamp）
- **入力**: `histogram(&[-0.3, 1.7], buckets)`
- **期待**: `-0.3` は clamp して先頭バケット（0.0–0.1）へ、`1.7` は clamp して末尾バケット（0.9–1.0）へ算入。`buckets[0].count == 1`、`buckets[9].count == 1`。ただし `stats.min == Some(-0.3)`、`stats.max == Some(1.7)`（min/max/mean は **clamp 前生値**、DD §3 不変条件）
- **境界条件**: バケット算入は clamp 後、統計は clamp 前生値（二系統の値域扱いの差分）

### ケース 4: histogram バケット幅・低値・高値の計算正確性（N 可変）

- **観点出典**: TP-LGX-010 §2.1 B6, §2.10 D5（パーセンタイル方式と独立のバケット定義）, TP-LGX-021 §2.6 R2
- **分類**: Unit
- **前提**: `buckets = BucketCount::new(4).unwrap()`。`scores = &[0.1, 0.3, 0.6, 0.9]`
- **入力**: `histogram(&[0.1,0.3,0.6,0.9], BucketCount::new(4).unwrap())`
- **期待**: 等幅 4 バケット。`buckets[0]={low:0.0,high:0.25,count:1}`(0.1), `buckets[1]={low:0.25,high:0.5,count:1}`(0.3), `buckets[2]={low:0.5,high:0.75,count:1}`(0.6), `buckets[3]={low:0.75,high:1.0,count:1}`(0.9)。`stats.mean == Some(0.475)`
- **境界条件**: バケット幅 = 1.0/N、low_i = i/N、high_i = (i+1)/N（v3 `similarity.rs` L339-380 踏襲）

### ケース 5: BucketCount::new(0) → Err(CalibrateError::InvalidBuckets)

- **観点出典**: TP-LGX-010 §2.1 B4（`--buckets 0` → exit 1）, TP-LGX-021 §2.2 AF4
- **分類**: Unit
- **前提**: なし
- **入力**: `BucketCount::new(0)`
- **期待**: `Err(CalibrateError::InvalidBuckets)`。`legixy-cli` 層は `eprintln!("エラー: --buckets は 1 以上を指定してください…")`（stderr）+ exit 1 に変換（DD §3.1 step 2 / §6）
- **境界条件**: 0 = 値の意味的不正（clap は `usize` として 0 を受理するため exit 2 ではない。DD §2.3 終了コードマッピング、TP-010 §2.2 E1）

### ケース 6: BucketCount::new(1) → Ok（下限境界）/ 単一バケットヒストグラム

- **観点出典**: TP-LGX-010 §2.1 B5（N の下限・既定）, TP-LGX-021 §2.6 R1
- **分類**: Unit
- **前提**: `scores = &[0.0, 0.5, 1.0]`
- **入力**: `BucketCount::new(1)` → `histogram(&[0.0,0.5,1.0], bucket1)`
- **期待**: `BucketCount::new(1) == Ok(_)`（`get()==1`）。`histogram` は単一バケット `{low:0.0, high:1.0, count:3}`（全スコアが唯一のバケットに算入、上限 1.0 inclusive）。`stats == {min:Some(0.0), max:Some(1.0), mean:Some(0.5)}`
- **境界条件**: N=1 は有効最小値（0 のみ Err）。N の上限・型不正（負値/非数）は clap 層 exit 2（→ TP-010 §2.1 B5、本 TS 対象外の構文層）

### ケース 7: compute_recommended 空入力 → None

- **観点出典**: TP-LGX-010 §2.1 B7（ペア数 0 → recommended 非出力）, TP-LGX-021 §2.2 AF2
- **分類**: Unit
- **前提**: `scores = &[]`
- **入力**: `compute_recommended(&[])`
- **期待**: `None`（DD §3 不変条件「scores が空のとき None」）
- **境界条件**: 空スライスのみ None（DD §3.1 step 5: pairs=0 は先行する early-exit が処理）

### ケース 8: compute_recommended 単一サンプル → 全パーセンタイル同値

- **観点出典**: TP-LGX-010 §2.1 B7（ノード 1 件相当・単一スコア境界）, §2.10 D5
- **分類**: Unit
- **前提**: `scores = &[0.42]`（n=1）
- **入力**: `compute_recommended(&[0.42])`
- **期待**: `Some(RecommendedThresholds{ similarity_threshold: 0.42 (p25), drift_threshold: 1.0-0.42=0.58 (1.0-p90), link_candidate_threshold: 0.42 (p75), percentiles: Percentiles{ p10:0.42, p25:0.42, p50:0.42, p75:0.42, p90:0.42 } })`。nearest-rank: `idx = round((1-1)*frac) = 0`、`sorted[min(0,0)] = 0.42` で全 percentile 同値
- **境界条件**: n=1 は非空の最小境界（None ではなく全 percentile = 唯一値。`idx` 式が n=1 で常に 0）

### ケース 9: compute_recommended 既知分布 fixture → nearest-rank 推奨値一致

- **観点出典**: TP-LGX-010 §2.10 D5（パーセンタイル方式凍結）, TP-LGX-021 §2.6 R2, SPEC-LGX-010 REQ.05 検証方法（DD §8 Unit）
- **分類**: Unit
- **前提**: `scores = &[0.0,0.1,0.2,0.3,0.4,0.5,0.6,0.7,0.8,0.9]`（昇順 n=10、計算検証用 fixture）
- **入力**: `compute_recommended(&scores)`
- **期待**: nearest-rank（`idx = round((n-1)*frac); sorted[min(idx, n-1)]`、n=10）で:
  - p10: idx=round(9*0.10)=1 → 0.1
  - p25: idx=round(9*0.25)=2 → 0.2
  - p50: idx=round(9*0.50)=5 → 0.5（round(4.5) は最近接偶数か 0.5 切上げか実装に依存 — DD 凍結式 `round` の rounding mode を底本にする。v3 `calibrate.rs` L216-242 準拠）
  - p75: idx=round(9*0.75)=7 → 0.7
  - p90: idx=round(9*0.90)=8 → 0.8
  - `similarity_threshold == p25 == 0.2`、`drift_threshold == 1.0-p90 == 1.0-0.8 == 0.2`、`link_candidate_threshold == p75 == 0.7`
- **境界条件**: 算出式凍結（DD §11.3）。入力 scores は内部で昇順ソートされる前提（compute_recommended が sort）

### ケース 10: compute_recommended パーセンタイル単調性（property）

- **観点出典**: TP-LGX-010 §2.10 D5, TP-LGX-021 §2.6 R3, DD §8 Property（p10 ≤ p25 ≤ p50 ≤ p75 ≤ p90）
- **分類**: Property-based（proptest）
- **生成器**: 任意の `Vec<f32>`（長さ 1 以上、各要素 [0.0,1.0] にフィルタ。非有限値は除外）
- **不変条件**: `compute_recommended(&v) == Some(r)` のとき `r.percentiles.p10 <= r.percentiles.p25 <= r.percentiles.p50 <= r.percentiles.p75 <= r.percentiles.p90`。導出値 `similarity_threshold == p25`、`link_candidate_threshold == p75`、`drift_threshold == 1.0 - p90`（drift は 1.0-p90 のため percentiles と逆順だが定義どおり）
- **反例ハンドリング**: shrink して最小の単調性違反例を記録

### ケース 11: histogram 決定性（同一スコア列 → 同一 Histogram、property）

- **観点出典**: TP-LGX-010 §2.10 D1（読取系決定性）, TP-LGX-021 §2.6 R3（SCORE-INV-1）, DD §8 Property
- **分類**: Property-based（proptest）
- **生成器**: 任意の `Vec<f32>`（[0.0,1.0] にフィルタ）と `buckets: 1..=64`
- **不変条件**: 同一 `(scores, buckets)` に対し `histogram` は常に同一 `Histogram`（buckets の low/high/count・stats が完全一致）。さらに `Σ buckets[i].count == scores.len()`（全スコアが正確に 1 バケットへ算入。clamp 込み）。非空のとき `stats.min <= stats.mean <= stats.max`
- **反例ハンドリング**: shrink して最小の不一致 / count 合計ずれ例を記録

### ケース 12: compute_all_pair_scores 空ストア → pairs 空・skip 0

- **観点出典**: TP-LGX-010 §2.1 B7（空ストア）, §2.3 S2（DB 不在 ≡ 空ストア）, TP-LGX-021 §2.2 AF3
- **分類**: Unit
- **前提**: `EmbeddingStore` が embeddings 0 行（空ストア）
- **入力**: `compute_all_pair_scores(&empty_store)`
- **期待**: `Ok(AllPairScores{ pairs: Vec::new(), skip_count: SkipSummary{ dim_mismatch: 0, nonfinite: 0 } })`。`skip_count.total() == 0`
- **境界条件**: 空ストア = ペア 0（N²=0）+ skip 0。`Err` ではなく `Ok` の空集合（DD §6 早期終了は calibrate 統括側が判定）

### ケース 13: compute_all_pair_scores SkipSummary 集計（次元不一致・非有限スコア）

- **観点出典**: TP-LGX-010 §2.2 E10（スキップ集約 Warning）, §2.1 B9（NaN/Inf 扱い）, TP-LGX-021 §2.3 EF1/EF2（次元不一致 / 非有限の skip）
- **分類**: Unit
- **前提**: ストアに (a) 次元不一致ペアが X 組、(b) スコアが非有限（NaN/±Inf）となるペアが Y 組生じる embeddings
- **入力**: `compute_all_pair_scores(&store)`
- **期待**: `Ok(AllPairScores)`。`skip_count.dim_mismatch == X`、`skip_count.nonfinite == Y`、`skip_count.total() == X+Y`。スキップされたペアは `pairs` に含まれない。`Err` 伝播しない（部分成功を正常系とする、DD §6）
- **境界条件**: 次元不一致 / 非有限は別カウンタ（dim_mismatch vs nonfinite を区別）。スコアの**数値妥当性**自体は → TS-LGX-007 委譲（本ケースは集計のみ）

### ケース 14: compute_all_pair_scores 昇順ペア不変条件（a < b のみ・read-only）

- **観点出典**: TP-LGX-010 §2.10 D1（決定性）, §2.5 P2（read-only）, TP-LGX-021 §2.5 DF3（engine.db 非破壊）, DD §3 不変条件（node_a < node_b）
- **分類**: Property/Integration
- **前提**: N ≥ 2 のノードを持つストア（昇順検証用に既知 ID 集合）
- **入力**: `compute_all_pair_scores(&store)` 実行前後の engine.db ハッシュ
- **期待**: `pairs` の各 `(Id a, Id b, f32)` で `a < b`（昇順ペアのみ、`(b,a)` や `(a,a)` は不出力）。ペア総数 = C(N',2)（N' = skip 後の有効ノード数）。実行前後で engine.db が不変（read-only、借用のみ）
- **境界条件**: 自己ペア除外（a≠b）+ 重複ペア除外（a<b で一意化）。SCORE-INV-1 の決定論前提

### ケース 15: calibrate 空ストア統括 → pairs=0・stats 全 None・recommended None

- **観点出典**: TP-LGX-010 §2.1 B7, §2.3 S2, TP-LGX-021 §2.2 AF3（2a 空ストア収束）
- **分類**: Integration
- **前提**: 空ストア。`config` に有効な現閾値。`buckets = BucketCount::new(10).unwrap()`、`recommend = false`
- **入力**: `calibrate(&empty_store, &config, buckets, false)`
- **期待**: `Ok(CalibrateReport{ pairs: 0, stats: HistogramStats{None,None,None}, distribution: vec![Bucket{count:0}; 10]（N=10 個・各 count=0）, thresholds: CurrentThresholds{ config 由来 }, recommended: None })`（DD §3 calibrate 不変条件）。exit 0 は legixy-cli が `pairs==0` を見て判定
- **境界条件**: 空ストアは `Err` ではなく正常系の空レポート（早期終了 EarlyExit::EmptyStore → exit 0、DD §2.2/§6）

### ケース 16: calibrate `--recommend` + pairs=0 → recommended None + cli は INFO（stderr）

- **観点出典**: TP-LGX-010 §2.1 B7（pairs=0 で `--recommend` INFO）, §2.8 L2（json INFO 併出）, TP-LGX-021 §2.2 AF2
- **分類**: Integration
- **前提**: 空ストア。`recommend = true`
- **入力**: `calibrate(&empty_store, &config, buckets, true)` → legixy-cli consumer
- **期待**: `CalibrateReport.recommended == None`（DD §3.1 step 5: `compute_recommended` は空スライスのみ None、pairs=0 は step 4 が先行処理）。legixy-cli は `eprintln!("INFO: ペア数 0 のため推奨値は算出されません")`（**stderr**）+ exit 0。`--json` の stdout は汚さない（DD §3.1 step 6）
- **境界条件**: `recommend=true ∧ pairs=0` でも exit 0（非致命）。INFO は stderr 限定

### ケース 17: calibrate `--recommend` + pairs>0 → recommended Some

- **観点出典**: TP-LGX-021 §2.2 AF2（--recommend 分岐）, §2.6 R5（recommended_thresholds キー）, TP-LGX-010 §2.10 D5
- **分類**: Integration
- **前提**: pairs ≥ 1 を生むストア。`recommend = true`
- **入力**: `calibrate(&store, &config, buckets, true)`
- **期待**: `CalibrateReport.recommended == Some(RecommendedThresholds{ similarity_threshold=p25, drift_threshold=1.0-p90, link_candidate_threshold=p75, percentiles })`。`pairs > 0`、`stats.min/max/mean` は全 Some
- **境界条件**: `recommend=false` のとき同入力で `recommended == None`（フラグによる出力分岐）

### ケース 18: calibrate skip 発生 → cli が集約 Warning 1 件（stderr）

- **観点出典**: TP-LGX-010 §2.2 E10/E12（集約 Warning 1 件）, TP-LGX-021 §2.3 EF1/EF2/EF4（部分スキップ継続 exit 0）
- **分類**: Integration
- **前提**: ストアに次元不一致 X 組・非有限 Y 組（`SkipSummary.total() = X+Y > 0`）。有効ペアも 1 組以上
- **入力**: `calibrate(&store, ..)` → legixy-cli consumer
- **期待**: `calibrate` は `Ok(CalibrateReport)`（部分成功 = exit 0）。legixy-cli は `eprintln!("WARNING: {} ペアをスキップしました（次元不一致 {} / 非有限スコア {}）", X+Y, X, Y)`（**stderr**、1 件集約。DD §3.1 step 7 / §11.4 C-7）。`--json` stdout には出さない
- **境界条件**: 部分スキップ（exit 0 継続）≠ 全件失敗（PairScoreFailure → exit 1、ケース 22）。Warning は集約 1 件・stderr 限定

### ケース 19: calibrate `--json` 出力に非有限値（NaN/±Inf）を含まない

- **観点出典**: TP-LGX-010 §2.1 B9（NaN/Inf 出力契約）, §2.9 F3（json スキーマ）, TP-LGX-021 §2.6 R5, SPEC-LGX-010 REQ.09（DD §3.2 / §8 Integration）
- **分類**: Integration
- **前提**: 非有限スコアを生む embeddings を含むストア（skip 経路を踏む）
- **入力**: `calibrate(&store, ..)` → `CalibrateJsonOutput` 構築 → stdout
- **期待**: stdout の JSON 内のすべての `f32` フィールド（min/max/mean/distribution[].low/high・thresholds・recommended_thresholds）が `f32::is_finite() == true`。NaN/Inf は事前 skip 済のため非到達（DD §3.2）。出力 JSON は `serde_json` でパース可能（不正トークンなし）
- **境界条件**: 非有限値は histogram/compute_all_pair_scores が skip 済 → 正常系で混入しない。`is_finite()` 事前検査で防御

### ケース 20: calibrate `--json` 空ストア応答スキーマ

- **観点出典**: TP-LGX-010 §2.9 F3（json 各キー）, TP-LGX-021 §2.2 AF3（空ストア json）, §2.6 R5, DD §11.2（distribution=[]）
- **分類**: Integration
- **前提**: 空ストア。`--json` 指定
- **入力**: `calibrate(&empty_store, ..)` → `CalibrateJsonOutput` → stdout
- **期待**: stdout JSON が `{"pairs":0, "distribution":[], "thresholds":{...}}`。`min`/`max`/`mean` は `skip_serializing_if=Option::is_none` によりキー自体が出ない（None→未出力、DD §3.2）。`recommended_thresholds` も未出力（None）。`distribution` は **空配列**（DD §11.2 v3 実測準拠。N バケット構造ではなく `[]`）
- **境界条件**: 空ストア json の distribution は `[]`（pairs=0 でも CalibrateReport.distribution は N バケットだが、CalibrateJsonOutput への変換で空配列化 — DD §11.2 凍結）

### ケース 21: calibrate `--json` + `--recommend`（pairs>0）→ recommended_thresholds キー含む

- **観点出典**: TP-LGX-021 §2.6 R5（--recommend 時の json スキーマ）, TP-LGX-010 §2.9 F3, DD §3.2 RecommendedJson
- **分類**: Integration
- **前提**: pairs ≥ 1 のストア。`--json` かつ `--recommend`
- **入力**: `calibrate(&store, .., recommend=true)` → `CalibrateJsonOutput` → stdout
- **期待**: stdout JSON に `"recommended_thresholds": { "similarity_threshold":p25, "drift_threshold":1.0-p90, "link_candidate_threshold":p75, "p10","p25","p50","p75","p90" }` を含む。`note` フィールドは**含まない**（DD §11.1 削除）。`min`/`max`/`mean` は Some のためキー出力あり
- **境界条件**: `--recommend` 非指定なら `recommended_thresholds` キーは未出力（`skip_serializing_if=Option::is_none`）

### ケース 22: calibrate 全ペア算出失敗 → Err(PairScoreFailure) → exit 1

- **観点出典**: TP-LGX-010 §2.2 E1（実行時失敗 exit 1）, TP-LGX-021 §2.3 EF4（全件失敗 vs 部分スキップの境界）, DD §2.3
- **分類**: Integration
- **前提**: `compute_all_pair_scores` が `anyhow::Error` を返す実行時失敗（部分 skip ではなく全体失敗）
- **入力**: `calibrate(&store, ..)`
- **期待**: `Err(CalibrateError::PairScoreFailure(_))`。legixy-cli が exit 1 へ変換。engine.db は不変（read-only、エラー時も中間状態破壊なし）
- **境界条件**: **全件失敗（exit 1）≠ 部分スキップ継続（exit 0、ケース 18）**。EF4 が問う観察可能な境界

### ケース 23: calibrate engine.db 非破壊（read-only 不変）

- **観点出典**: TP-LGX-010 §2.5 P2（読取系非破壊）, TP-LGX-021 §2.5 DF3 / §2.3 EF3（エラー時も不変）, DD §5/§7（borrow read-only）
- **分類**: Property/Integration
- **前提**: 任意の入力（空ストア・正常・部分 skip・全件失敗いずれも）
- **入力**: `calibrate(&store, &config, buckets, recommend)` 実行前後の engine.db ハッシュ
- **期待**: 実行前後で engine.db のバイト/ハッシュが不変（`&EmbeddingStore` 借用のみ、所有権を取らない）。成功・`Err` いずれの経路でも不変
- **境界条件**: calibrate は読取専用コマンド（STATE-INV-1 / SPEC-LGX-010 REQ.07）。書込み副作用なし

### ケース 24: 終了コード契約 0/1/2（LGX-COMPAT-001 §4 #7 凍結）

- **観点出典**: TP-LGX-010 §2.2 E1, §2.9 F2, TP-LGX-021 §2.6 R4, DD §2.3 終了コードマッピング
- **分類**: Contract
- **前提**: (a) 正常算出（pairs≥0）または空ストア、(b) `--buckets 0`（InvalidBuckets）/ PairScoreFailure / Db / Config、(c) clap 構文誤り（未知フラグ・`--buckets abc` 型不正）
- **入力**: それぞれ legixy-cli ディスパッチ
- **期待**: (a)→**exit 0**（空ストアも正常系）、(b)→**exit 1**（値の意味的不正・実行時失敗）、(c)→**exit 2**（clap 既定の構文層エラー）
- **境界条件**: `--buckets 0` は値の不正 = exit 1（clap は `usize` 0 を受理）。型不正（非数値）は構文層 = exit 2。両者の分離が契約の核（DD §2.3）

### ケース 25: 出力先分離（結果・JSON=stdout / INFO・WARNING・ERROR=stderr）

- **観点出典**: TP-LGX-010 §2.8 L1/L2, TP-LGX-021 §2.5 DF1/DF2（stdout/stderr 分離）, NFR-LGX-001 OBS.02（DD §10）
- **分類**: Contract/Integration
- **前提**: (a) text モード正常、(b) `--json` 正常、(c) 空ストア INFO、(d) skip 発生 WARNING、(e) `--buckets 0` ERROR
- **入力**: legixy-cli 実行で stdout / stderr を分離キャプチャ
- **期待**: ヒストグラム・統計・現閾値（text）/ CalibrateJsonOutput（json）は **stdout**。INFO（空ストア・pairs=0 推奨非算出）・WARNING（skip 集約）・ERROR（--buckets 0）は **stderr**。`--json` モードでも stdout は機械可読（INFO は stderr 併出で stdout を汚さない）
- **境界条件**: チャネル分離（OBS.02）。`--json` の stdout 純度保証（L2）

## 3. 観点カバレッジ表

| TP § | 観点 | カバーする TS ケース |
|---|---|---|
| TP-010 §2.1 B4 `--buckets 0` exit 1 | 境界値 | ケース 5, 24 |
| TP-010 §2.1 B5 buckets 既定/上限/負/型不正 | 境界値 | ケース 6（下限 1）, 24（型不正 exit 2 構文層）|
| TP-010 §2.1 B6 ヒストグラム値域/clamp/inclusive/生値統計 | 境界値 | ケース 1, 2, 3, 4 |
| TP-010 §2.1 B7 ペア数 0 → recommended 非出力 + `--recommend` INFO | 境界値 | ケース 7, 12, 15, 16 |
| TP-010 §2.1 B9 NaN/Inf 出力契約 | 境界値 | ケース 13（skip 集計）, 19（json 非有限なし）|
| TP-010 §2.1 B1/B2/B3/B8/B10 snapshot 系境界 | 境界値 | TS-LGX-012/013 へ委譲（snapshot/drift 固有）|
| TP-010 §2.2 E1 終了コード 3 分類 | エラー | ケース 5, 22, 24 |
| TP-010 §2.2 E10 スキップ集約 Warning 1 件 stderr | エラー | ケース 18 |
| TP-010 §2.2 E12 全件集計のスキップ集約収束 | エラー | ケース 13, 18 |
| TP-010 §2.2 E2〜E9/E11 drift/snapshot エラー | エラー | TS-LGX-012/013 へ委譲 |
| TP-010 §2.3 S2 DB 不在 ≡ 空ストア・非作成 | 状態 | ケース 12, 15 |
| TP-010 §2.3 S1/S3/S4/S5/S6 snapshot ライフサイクル | 状態 | TS-LGX-012 へ委譲（snapshot 固有）|
| TP-010 §2.4 C1〜C4 並行性 | 並行 | NFR-LGX-001 SEC.02/REL.07 へ委譲 + ケース 23（read-only）|
| TP-010 §2.5 P2 読取系非破壊 | 永続化 | ケース 14, 23 |
| TP-010 §2.5 P1/P3/P4/P5 snapshot 永続化 | 永続化 | TS-LGX-012 へ委譲 |
| TP-010 §2.6 V1/V2 引数体系・グローバルオプション | 互換 | ケース 24（calibrate 引数契約）/ LGX-COMPAT-001 §4 #7 |
| TP-010 §2.6 V3〜V7 snapshot_id/モデル解決 | 互換 | TS-LGX-012/013 へ委譲 |
| TP-010 §2.7 I1〜I5 入力検証（drift/snapshot 引数）| 入力 | ケース 5, 24（calibrate 引数）/ TS-LGX-012/013 委譲 |
| TP-010 §2.8 L1 結果=stdout / 診断=stderr | 観測 | ケース 25 |
| TP-010 §2.8 L2 json 時 INFO 併出（stdout 純度）| 観測 | ケース 16, 25 |
| TP-010 §2.8 L3〜L6 drift 診断/監査/機密 | 観測 | TS-LGX-013 委譲 / NFR-LGX-001 SEC.05 |
| TP-010 §2.9 F1 MCP 非公開 | 境界 API | LGX-COMPAT-001（MCP-INV-1）/ TS-LGX-009 委譲 |
| TP-010 §2.9 F2 CLI 引数契約一致 | 境界 API | ケース 24 |
| TP-010 §2.9 F3 json スキーマ各キー定義 | 境界 API | ケース 19, 20, 21 |
| TP-010 §2.10 D1 読取系決定性（同一入力→同一バイト）| 決定性 | ケース 11, 14 |
| TP-010 §2.10 D5 推奨閾値 = パーセンタイル方式凍結 | 領域 | ケース 9, 10, 17 |
| TP-010 §2.10 D6 bulk API consumer（類似度非再実装）| 領域 | ケース 13, 14（集計のみ）/ 数値妥当性 TS-LGX-007 委譲 |
| TP-010 §2.10 D2/D3/D4/D7/D8 snapshot/report/drift 領域 | 領域 | TS-LGX-010/012/013 へ委譲 |
| TP-021 §2.1 BF1 ステップ連鎖整合（ロード→算出→ヒスト→出力→exit0）| UC フロー | ケース 14, 4, 15, 25 |
| TP-021 §2.1 BF2 段階区分（compute_all_pair_scores → histogram）| UC フロー | ケース 12, 14, 4 |
| TP-021 §2.1 BF3 text/JSON 出力分岐 | UC フロー | ケース 20, 21, 25 |
| TP-021 §2.1 BF4 成功時事後条件（stdout 出力・db 不変）| UC フロー | ケース 23, 25 |
| TP-021 §2.2 AF1 代替フロー網羅性 | 代替フロー | ケース 5, 15, 16, 18 |
| TP-021 §2.2 AF2 `--recommend` 分岐 | 代替フロー | ケース 7, 16, 17, 21 |
| TP-021 §2.2 AF3 空ストア収束（exit 0）| 代替フロー | ケース 12, 15, 20 |
| TP-021 §2.2 AF4 `--buckets 0` exit 1 | 代替フロー | ケース 5, 24 |
| TP-021 §2.2 AF5 遷移条件の明示 | 代替フロー | ケース 5, 15, 22 |
| TP-021 §2.3 EF1 次元不一致スキップ + 集約 Warning | 例外フロー | ケース 13, 18 |
| TP-021 §2.3 EF2 非有限スコア skip + 集約 Warning | 例外フロー | ケース 13, 18, 19 |
| TP-021 §2.3 EF3 エラー時の engine.db 不変 | 例外フロー | ケース 22, 23 |
| TP-021 §2.3 EF4 全件失敗(exit1) vs 部分スキップ(exit0) 境界 | 例外フロー | ケース 18, 22 |
| TP-021 §2.4 AT1/AT2/AT3 アクター権限/責任境界 | アクター | ケース 23（read-only 全アクター共通）/ UC 事後条件（システム=計測・非対象）|
| TP-021 §2.5 DF1 入出力データ分離 | データフロー | ケース 25 |
| TP-021 §2.5 DF2 stdout/stderr 分離 | データフロー | ケース 16, 25 |
| TP-021 §2.5 DF3 engine.db 非破壊 | データフロー | ケース 14, 23 |
| TP-021 §2.6 R1 `--buckets` 境界値 | 領域 | ケース 5, 6, 24 |
| TP-021 §2.6 R2 ヒストグラム値域・clamp | 領域 | ケース 1, 2, 3, 4 |
| TP-021 §2.6 R3 出力決定性（SCORE-INV-1）| 領域 | ケース 11, 14 |
| TP-021 §2.6 R4 終了コード契約 | 領域 | ケース 24 |
| TP-021 §2.6 R5 `--json` スキーマ + `--recommend` キー | 領域 | ケース 20, 21 |

> 継承 TP 観点はすべて本テーブルで TS ケースまたは明示委譲先に mapping 済み（人間ゲート判断対象）。bulk pair スコアの数値妥当性は TS-LGX-007（SPEC-006 類似度算出所有）、snapshot/drift/report 固有観点は TS-LGX-010/012/013、性能 PERF.07 は NFR-LGX-001 / bench、並行アクセス整合 SEC.02/REL.07 は NFR へ委譲し、本 TS は calibrate 固有（Histogram バケツ境界・nearest-rank パーセンタイル・RecommendedThresholds 算出・`--recommend`・CalibrateReport・空/単一サンプル境界・決定論・skip 集約 Warning・出力先分離・終了コード）に集中する。

## 4. テスト技法選択

- 同値分割: histogram 入力（空 / 域内 / 域外 clamp 対象）、calibrate ストア状態（空 / 正常 / 部分 skip / 全件失敗）、`recommend` フラグ（true/false）、終了コード（0/1/2）。
- 境界値分析: BucketCount（0=Err / 1=下限 Ok）、histogram バケット境界（0.0 先頭 inclusive / 1.0 末尾 inclusive / 域外 clamp）、compute_recommended サンプル数（0=None / 1=全 percentile 同値 / n=既知 fixture）。
- Property-based: histogram 決定性 + バケット count 合計 = 入力件数（ケース 11）、compute_recommended パーセンタイル単調性 p10≤…≤p90（ケース 10）、昇順ペア不変条件（ケース 14）。
- 状態遷移: ストア状態（空 → pairs=0 早期収束 / 非空 → 算出 → 出力）と exit コード収束（0/1/2）。

## 5. テスト基盤

- 言語: Rust（CLI 本体、主 crate `legixy-embed` + consumer `legixy-cli`）
- フレームワーク: cargo test
- Property-based: proptest（histogram 決定性・パーセンタイル単調性・昇順ペア）
- モック: `EmbeddingStore` は in-memory / 一時 SQLite fixture（既知 embeddings 注入）。`compute_all_pair_scores` の実行時失敗（ケース 22）は注入可能なエラー経路で再現。stdout/stderr は分離キャプチャ。

## 6. 関連 TC

| TS ケース | 対応 TC | 場所 |
|---|---|---|
| ケース 1〜4 (histogram) | TC-LGX-011 系 | crates/legixy-embed/src/similarity.rs（#[cfg(test)]）|
| ケース 5, 6 (BucketCount) | TC-LGX-011 系 | crates/legixy-embed/src/types.rs |
| ケース 7〜10 (compute_recommended) | TC-LGX-011 系 | crates/legixy-embed/src/similarity.rs |
| ケース 11 (histogram property) | TC-LGX-011 系 | crates/legixy-embed/tests/calibrate_prop.rs（proptest）|
| ケース 12〜14 (compute_all_pair_scores) | TC-LGX-011 系（一部 TS-LGX-007 共有）| crates/legixy-embed/src/similarity.rs |
| ケース 15〜24 (calibrate 統括) | TC-LGX-011 系 | crates/legixy-cli/tests/calibrate_integration.rs |
| ケース 25 (出力先分離) | TC-LGX-011 系 | crates/legixy-cli/tests/calibrate_integration.rs |
