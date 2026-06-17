Document ID: DD-LGX-011

# DD-LGX-011: 閾値キャリブレーション（calibrate）の詳細設計

**親 SEQD**: SEQD-LGX-011
**親 RBD**: RBD-LGX-011 / **親 UC**: UC-LGX-011
**対象言語**: Rust（CLI 本体）
**境界 API 凍結**: yes（本ドキュメント確定後、crate 間公開 API は変更不可。追加は許容、削除・改名・型変更は次版 SPEC 改訂。HR7）

> 言語別規律は `guides/language-stacks/rust.md`。crate 境界・共有型は **ADR-LGX-020** で凍結済（本 DD は参照する）。型・挙動は v3 実装（traceability-engine.v3.chg_to_lexigy `crates/lx-cli/src/commands/calibrate.rs` / `crates/lx-embed/src/similarity.rs`）に整合させ引数互換を保つ。

## 1. 対象範囲

- **主 crate**: `legixy-embed`（`compute_all_pair_scores` / `histogram` / `compute_recommended` — calibrate の算出コアを担う crate）
- **依存 crate（共有型は ADR-LGX-020、再定義しない）**: `legixy-db`（`EmbeddingStore` / `EmbeddingRow` — 全件ロード）, `legixy-core`（`Id` / 共通エラー / `Config`）, `legixy-cli`（引数パース・サブコマンドディスパッチ・終了コード）
- **公開 API surface**: 本 DD §3（`legixy-embed` の calibrate 関連公開関数）
- **関連 SEQD**: SEQD-LGX-011

本 DD は DD-LGX-010（report）と同一の主 crate（`legixy-embed`）を対象とする。§4 の module 構成・§9 の ADR 参照のうち DD-LGX-010 と共通する部分は「DD-LGX-010 参照」と明示し重複定義を回避する（ADR-LGX-020 §2.3「所有 crate で 1 回だけ定義」の精神に倣う）。

## 2. 型定義

### 2.1 主要データ型

```rust
// legixy-core（共有、ADR-LGX-020）
pub struct Id(String);                   // {type}-{area}-{seq} or {id}#{subnode_hash}

// legixy-embed（calibrate 専用型）
/// 全ペア類似度集合（compute_all_pair_scores の戻り値）
pub struct AllPairScores {
    pub pairs: Vec<(Id, Id, f32)>,       // (node_a, node_b, score)。node_a < node_b（昇順ペア）
    pub skip_count: SkipSummary,         // 次元不一致・非有限スコアのスキップ集計
}

/// ヒストグラムバケット
pub struct Bucket {
    pub low: f32,
    pub high: f32,
    pub count: usize,
}

/// ヒストグラム全体（histogram の戻り値）
pub struct Histogram {
    pub buckets: Vec<Bucket>,            // 長さ = buckets 引数（N バケット）
    pub stats: HistogramStats,           // min/max/mean（clamp 前の生値）
}

/// ヒストグラム統計（min/max/mean、clamp 前生値。空入力時は None）
pub struct HistogramStats {
    pub min: Option<f32>,
    pub max: Option<f32>,
    pub mean: Option<f32>,
}

/// パーセンタイル計算結果（compute_recommended の戻り値）
pub struct RecommendedThresholds {
    pub similarity_threshold: f32,       // p25
    pub drift_threshold: f32,            // 1.0 - p90
    pub link_candidate_threshold: f32,   // p75
    pub percentiles: Percentiles,        // 参考情報 p10/p25/p50/p75/p90
}

pub struct Percentiles {
    pub p10: f32,
    pub p25: f32,
    pub p50: f32,
    pub p75: f32,
    pub p90: f32,
}

/// calibrate コマンドの集約出力（legixy-cli 層で JSON/text フォーマット）
pub struct CalibrateReport {
    pub pairs: usize,
    pub stats: HistogramStats,
    pub distribution: Vec<Bucket>,
    pub thresholds: CurrentThresholds,
    pub recommended: Option<RecommendedThresholds>, // --recommend 時のみ Some
}

/// 設定ファイルの現閾値（legixy-core の Config から取り出す）
pub struct CurrentThresholds {
    pub similarity_threshold: f32,
    pub drift_threshold: f32,
    pub link_candidate_threshold: f32,
}

/// スキップ集計（次元不一致・非有限スコア）
pub struct SkipSummary {
    pub dim_mismatch: usize,
    pub nonfinite: usize,
}

impl SkipSummary {
    pub fn total(&self) -> usize { self.dim_mismatch + self.nonfinite }
}
```

### 2.2 列挙 / Sum 型

```rust
// legixy-embed（calibrate 引数）
/// --buckets の値。1 以上を保証するラッパ（0 は CalibrateError::InvalidBuckets に変換）
#[derive(Copy, Clone)]
pub struct BucketCount(usize);           // 内部値は 1 以上を不変条件とする

impl BucketCount {
    pub fn new(n: usize) -> Result<Self, CalibrateError> {
        if n == 0 {
            Err(CalibrateError::InvalidBuckets)
        } else {
            Ok(BucketCount(n))
        }
    }
    pub fn get(&self) -> usize { self.0 }
}

/// 早期終了種別（空ストア）
pub enum EarlyExit {
    EmptyStore,  // embeddings テーブルが空 → INFO（stderr）+ exit 0
}
```

### 2.3 エラー型

```rust
// legixy-embed
pub enum CalibrateError {
    /// --buckets 0 指定（exit 1。v3: anyhow::bail! → legixy は型付きエラー）
    InvalidBuckets,
    /// 全ペア算出の実行時失敗（exit 1）
    PairScoreFailure(anyhow::Error),
    /// engine.db アクセス失敗（exit 1）
    Db(legixy_db::DbError),
    /// 設定ファイル読み込み失敗（exit 1）
    Config(legixy_core::ConfigError),
}
```

- **終了コード マッピング**（LGX-COMPAT-001 §3、SPEC-LGX-010.REQ.01）:
  - `CalibrateError::InvalidBuckets` → exit 1（引数の**値**の意味的不正。clap 層は `--buckets 0` を `usize` として受け付けるため exit 2 ではない。v3 実測: `calibrate.rs` L64-67 の `anyhow::bail!`）
  - `CalibrateError::PairScoreFailure` / `Db` / `Config` → exit 1
  - clap 構文エラー（未知フラグ・型不正）→ exit 2（clap 既定、SPEC-LGX-010.REQ.01）
  - `EarlyExit::EmptyStore` → exit 0（正常系として扱う）
- panic 禁止（rust.md §4）。`unwrap` / `expect` は crate 公開境界付近では使用しない。

## 3. 公開 API surface（凍結、HR7）

| 関数 | シグネチャ | 不変条件 | 冪等性 | 同期/非同期 |
|---|---|---|---|---|
| `legixy_embed::compute_all_pair_scores` | `fn compute_all_pair_scores(store: &EmbeddingStore) -> Result<AllPairScores, CalibrateError>` | node_id 昇順ペア（a < b）のみ出力。非有限スコア・次元不一致ペアは skip し `SkipSummary` に計上。同一入力 → 同一出力（SCORE-INV-1）。read-only | yes | 同期 |
| `legixy_embed::histogram` | `fn histogram(scores: &[f32], buckets: BucketCount) -> Histogram` | 値域 [0.0, 1.0] 固定の等幅 N バケット。域外スコアは clamp して算入。上限 1.0 は末尾バケット inclusive。min/max/mean は clamp 前生値。空入力時は全バケット count=0・stats.min/max/mean=None | yes | 同期 |
| `legixy_embed::compute_recommended` | `fn compute_recommended(scores: &[f32]) -> Option<RecommendedThresholds>` | scores が空のとき `None`。非空のとき nearest-rank パーセンタイル（`idx = round((n-1)*frac); sorted[min(idx, n-1)]`）で p10/25/50/75/90 を算出。`similarity_threshold=p25`, `drift_threshold=1.0-p90`, `link_candidate_threshold=p75`（SPEC-LGX-010.REQ.05。v3: `calibrate.rs` L216-242） | yes | 同期 |
| `legixy_embed::calibrate` | `fn calibrate(store: &EmbeddingStore, config: &Config, buckets: BucketCount, recommend: bool) -> Result<CalibrateReport, CalibrateError>` | 上記 3 関数を統括。空ストア時は `Ok(CalibrateReport{ pairs:0, stats: HistogramStats{None,None,None}, distribution: vec![Bucket{count:0}; buckets.get()], recommended: None, ... })` で返す（exit 0 は呼び出し側 legixy-cli が判定）。`recommend=true` かつ pairs=0 の場合 `recommended=None` + 呼び出し側が stderr INFO を出力（§6 参照）。read-only（engine.db 不変） | yes | 同期 |

- `EmbeddingStore` は `legixy-db` 所有型（ADR-LGX-020 参照）。本 DD は再定義しない。
- `Config` は `legixy-core` 所有型（`config.semantic.similarity_threshold` / `drift_threshold` / `link_candidate_threshold` を参照）。
- 引数の意味は LGX-COMPAT-001 §4 #7（`calibrate [--buckets N] [--recommend] [--json]`）に凍結済。

### 3.1 legixy-cli 層の責務（公開 API surface 外だが凍結）

`legixy-cli` の `commands/calibrate.rs` が担う処理（`legixy-embed::calibrate` の consumer として）:

1. clap から `--buckets`（`usize`、既定 10）/ `--recommend`（flag）/ `--json`（flag）を受け取る
2. `BucketCount::new(n)` で検証（`InvalidBuckets` → `eprintln!` + exit 1）
3. `calibrate(store, config, buckets, recommend)` を呼ぶ
4. `CalibrateReport.pairs == 0` の場合: text モード → `eprintln!("INFO: ベクトルストアが空です。embed --all を実行してください")` + exit 0
5. `recommend=true` かつ `CalibrateReport.recommended.is_none()` かつ `pairs > 0` はあり得ない（`compute_recommended` が空スライスのみ `None` を返す。pairs=0 は step 4 が先行）
6. `recommend=true` かつ `pairs == 0` の情報通知: `eprintln!("INFO: ペア数 0 のため推奨値は算出されません")` を stderr に出力（SPEC-LGX-010.REQ.05【v3 差分】。`--json` の stdout は汚さない）
7. `SkipSummary.total() > 0` の場合: `eprintln!("WARNING: {} ペアをスキップしました（次元不一致 {} / 非有限スコア {}）", total, dim_mismatch, nonfinite)` を stderr に出力（SPEC-LGX-010.REQ.05【v3 差分】）
8. `--json` モード: stdout に `serde_json::to_string_pretty` で `CalibrateJsonOutput`（§3.2）を出力
9. text モード: v3 実測（`calibrate.rs` L149-211）準拠の ASCII ヒストグラム + 統計 + 現閾値 + `--recommend` 時推奨値を stdout に出力
10. exit 0

### 3.2 JSON 出力スキーマ（legixy-cli 層、凍結）

```rust
/// serde::Serialize / Deserialize（--json モード stdout 出力用）
#[derive(Serialize)]
pub struct CalibrateJsonOutput {
    pub pairs: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min: Option<f32>,               // pairs=0 時 null（JSON の null は None→serde）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mean: Option<f32>,
    pub distribution: Vec<BucketJson>,  // 空ストア時は空配列（pairs=0 でも N バケット構造を返す）
    pub thresholds: ThresholdsJson,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recommended_thresholds: Option<RecommendedJson>,
}

#[derive(Serialize)]
pub struct BucketJson {
    pub low: f32,
    pub high: f32,
    pub count: usize,
}

#[derive(Serialize)]
pub struct ThresholdsJson {
    pub similarity_threshold: f32,
    pub drift_threshold: f32,
    pub link_candidate_threshold: f32,
}

#[derive(Serialize)]
pub struct RecommendedJson {
    pub similarity_threshold: f32,      // p25
    pub drift_threshold: f32,           // 1.0 - p90
    pub link_candidate_threshold: f32,  // p75
    pub p10: f32,
    pub p25: f32,
    pub p50: f32,
    pub p75: f32,
    pub p90: f32,
    // note フィールドは省略（SUPP-LGX-010 §2 K-4: legixy はサブノード embedding 既定で Phase 1 文言が陳腐化のため削除）
}
```

`--json` 出力に非有限値（NaN/Inf）を一切含まない（SPEC-LGX-010.REQ.09）。`serde_json` に依存せず `f32::is_finite()` で事前検査し、非有限値は非到達状態とする（`histogram` / `compute_all_pair_scores` が skip 済みのため正常系では非有限値は混入しない）。

## 4. module / package 構成

```
legixy-embed/
├── src/
│   ├── lib.rs            // calibrate / compute_all_pair_scores / histogram / compute_recommended を再エクスポート
│   ├── calibrate.rs      // Document ID: SRC-LGX-011（calibrate 統括関数・CalibrateReport 生成）
│   ├── similarity.rs     // compute_all_pair_scores / histogram / SkipSummary（DD-LGX-010 と共有 module）
│   │                     //   └── 本 DD で追加: compute_recommended（パーセンタイル算出）
│   ├── store.rs          // EmbeddingStore（legixy-db 参照。DD-LGX-010 参照）
│   ├── types.rs          // AllPairScores / Histogram / Bucket / HistogramStats / BucketCount /
│   │                     //   RecommendedThresholds / Percentiles / CurrentThresholds / SkipSummary /
│   │                     //   CalibrateReport / EarlyExit（DD-LGX-010 と共有 module）
│   ├── error.rs          // CalibrateError（EmbedError との階層整理は DD-LGX-010 参照）
│   └── （他 DD が管理する module: drift.rs / embed.rs / orchestrator.rs 等）
└── Cargo.toml
```

`legixy-cli/`
```
legixy-cli/
└── src/
    └── commands/
        └── calibrate.rs  // §3.1 の consumer ロジック（clap 引数受取・BucketCount 検証・出力フォーマット・exit）
```

依存方向（DAG、ADR-LGX-020）: `legixy-cli` → `legixy-embed` → `legixy-db` / `legixy-core`。循環なし。

### 4.1 DD-LGX-010 との共有 module

`legixy-embed::similarity` module の `compute_all_pair_scores` / `histogram` / `SkipSummary` は DD-LGX-010（report）との共有対象。DD-LGX-010 が先に凍結される場合、本 DD の `compute_recommended` のみを `similarity.rs` へ追加する（加算的拡張、HR7 準拠）。DD-LGX-010 より先に本 DD が凍結される場合も同様に `similarity.rs` に定義し、DD-LGX-010 はそれを参照する。

## 5. ライフタイム / 所有権 / 借用 方針

- `calibrate` は `&EmbeddingStore` / `&Config` を**借用**（read-only、所有権を取らない）。
- `AllPairScores` は所有して返す（O(N²) ペア数 × `f32` 3要素のタプル。N=1,000 で N²=1,000,000 ペア、約 12 MB — NFR PERF.07 の許容範囲内とするが、大規模 N では将来ストリーミング最適化を検討。現バージョンは全件 Vec で充分）。
- `Histogram` / `CalibrateReport` は所有して返す（呼び出し側が出力・JSON 変換に使用）。
- `BucketCount` は `Copy`（軽量な値型）。
- `Arc` / `Mutex` 不要（単一スレッド逐次。§7）。
- `Id` は `legixy-core` 所有。ペア格納時は `Id` を clone（`AllPairScores.pairs: Vec<(Id, Id, f32)>`）。`'static` バウンド不要。

## 6. エラー伝播戦略

- **内部 / 公開エラー変換**: `legixy-db::DbError` → `CalibrateError::Db`、`legixy-core::ConfigError` → `CalibrateError::Config`（`?` 演算子 + `From` impl）。`anyhow::Error` は `PairScoreFailure` でラップ。
- **`InvalidBuckets`** はユーザ入力の意味的不正。`legixy-cli` 層で `eprintln!("エラー: --buckets は 1 以上を指定してください（v3: --buckets は 1 以上を指定してください）")` + exit 1 に変換（v3: `calibrate.rs` L64-67）。
- **早期終了（EmptyStore）**: `calibrate` は `Ok(CalibrateReport{ pairs: 0, ... })` を返す。空ストアを `Err` として伝播しない（exit 0 は正常系）。`legixy-cli` 層が `pairs == 0` を判定して INFO を stderr 出力後 exit 0。
- **skip（非有限スコア・次元不一致）**: `compute_all_pair_scores` 内部で skip し `SkipSummary` に計上。`Err` 伝播しない（calibrate/report は部分成功を正常系とする。SPEC-LGX-010.REQ.05/REQ.09）。`legixy-cli` 層が `SkipSummary.total() > 0` を見て集約 Warning を stderr 出力。
- panic 禁止。`compute_all_pair_scores` 内の `f32` 算術で NaN/Inf が生じても `f32::is_finite()` で検出して skip。コサイン類似度の分母ゼロは `legixy-embed::drift` 側の正規化保証（SPEC-LGX-006.REQ.04）を前提とするが、consumer 側でも防御検査する。
- ユーザ通知: 計測結果・JSON = stdout、INFO/WARNING/ERROR = stderr（SPEC-LGX-010.REQ.01, NFR-LGX-001.OBS.02）。

## 7. 並行性 / 非同期境界

- `calibrate` / `compute_all_pair_scores` / `histogram` / `compute_recommended` は **同期・単一スレッド・read-only**。async なし。
- O(N²) の全ペア算出は単一スレッド逐次（N ≤ 数千で実用的）。将来の rayon 並列化は NFR PERF.07 計測後に判断（本 DD では逐次）。
- `EmbeddingStore` の並行アクセスは対象外（read-only、外部更新整合はレイヤ外）。

## 8. テスト分類

| 分類 | 内容 | 対応 TP |
|---|---|---|
| Unit | `histogram`: 境界（1.0 末尾 inclusive、域外 clamp、空入力で全バケット 0 件）/ バケット幅・低値・高値の計算正確性（v3: `similarity.rs` L339-380 踏襲）。`compute_recommended`: 既知分布 fixture に対する nearest-rank 式の推奨値一致（SPEC-LGX-010.REQ.05 検証方法）/ pairs=0 で None / p10〜p90 値の計算。`compute_all_pair_scores`: SkipSummary 集計（次元不一致・非有限スコアのカウント）、空ストアで pairs=Vec::new() | TP-LGX-021 |
| Integration | `calibrate` 統括: 空ストア → `pairs=0` + exit 0 / `--buckets 0` → exit 1 / `--recommend` + pairs=0 → stderr INFO / SkipSummary.total()>0 → 集約 Warning / `--json` 出力に非有限値なし / `--json` + `--recommend` で `recommended_thresholds` 含む出力 / engine.db 不変（実行前後ハッシュ比較）。`calibrate` + `--json` の空ストア応答: `{"pairs":0,"distribution":[],"thresholds":{...}}` | TP-LGX-021 |
| Property-based | `histogram` の決定性（同一スコア列 → 同一 Histogram。SCORE-INV-1、proptest）/ バケット合計 = 入力スコア件数 / min ≤ mean ≤ max（空でない場合）/ `compute_recommended` の単調性（p10 ≤ p25 ≤ p50 ≤ p75 ≤ p90） | TP-LGX-021 |
| Bench | N=500 ノード（ペア数 124,750）での `compute_all_pair_scores` + `histogram` 応答時間（NFR PERF.07 ≤ 実測値。criterion） | NFR-LGX-001 |

## 9. ADR への参照

- **ADR-LGX-020**: crate 分割・共有型凍結（本 DD の crate 境界）
- ADR-LGX-003: embedding 決定論モデル（SCORE-INV-1 — 同一 embeddings → 同一ヒストグラム保証の基盤）
- ADR-LGX-013: check/audit/calibrate の加算的拡張方針（calibrate が bulk API の consumer として check と責務分離を維持する根拠）
- ADR-LGX-019: 仕様変更の記録規律（SPEC-LGX-010 v0.3.0 の `snapshot:label:` 挙動変更が本 DD に影響しないことの確認）

## 10. 関連 NFR

- NFR-LGX-001.OBS.02: 出力先（計測結果・JSON = stdout / INFO・WARNING・ERROR = stderr）
- NFR-LGX-001.OBS.05: 終了コード（0=正常・空ストア、1=`--buckets 0`・実行エラー、2=clap 構文エラー）
- NFR-LGX-001.PERF.07: WAL モード必須（engine.db 読取時の同時アクセス整合）
- NFR-LGX-001.OBS.04: エラーメッセージ日本語 primary（v3 文言を日本語に統一。SUPP-LGX-010 §2 C-8 方針に従う）

## 11. 設計判断と v3 差分

### 11.1 `note` フィールドの削除

v3 `calibrate.rs` L16-54 の `recommended_thresholds.note` フィールド（「Phase 1 ノード単位 embedding ベース。Phase 2 サブノード化後に再算出推奨」）は削除する（SUPP-LGX-010 §2 K-4 オプション a）。legixy はサブノード embedding 既定（SPEC-LGX-006.REQ.09）であり文言が陳腐化しているため。SPEC 非記載の任意フィールドであり削除は互換安全。

### 11.2 ヒストグラムの `distribution` フィールド

空ストア時（pairs=0）の `--json` 出力で `distribution` は空配列（`[]`）を返す（v3 実測: SUPP-LGX-010 §2 K-3、`pairs=0 + distribution=[]`）。`BucketCount` 個の `{low, high, count: 0}` バケット構造を返す案もあるが、v3 実測準拠で空配列を選択する。

### 11.3 パーセンタイル算出式の凍結

nearest-rank 変種: `idx = round((n-1) * frac); sorted[min(idx, n-1)]`（SUPP-LGX-010 §2 K-2、v3: `calibrate.rs` L216-242）。本 DD で凍結する。算出式の変更は SPEC-LGX-010.REQ.05 の「既知分布 fixture に対する推奨値一致テスト」が破綻させ、意識的判断 + 変更履歴を強制する。

### 11.4 SUPP-010 [要決定] の扱い

本 DD の範囲内で確定できる [要決定] 項目:

| 項目 | 確定内容 |
|---|---|
| C-7 集約 Warning の文言 | `"WARNING: {N} ペアをスキップしました（次元不一致 {X} / 非有限スコア {Y}）"` を stderr。`--json` stdout には出力しない（SPEC-LGX-010.REQ.05 の趣旨 + REQ.09 の非有限スコア扱い） |
| C-8 診断メッセージの言語 | 日本語 primary（NFR-LGX-001.OBS.04）。v3 の英語文言は日本語に置換 |
| K-4 `note` フィールド | 削除（§11.1） |

本 DD では確定しない [要決定]:

| 項目 | 理由 |
|---|---|
| C-4 DB 旧ディレクトリ `.trace-engine/` フォールバック | SPEC-LGX-008（マイグレーション）の所管。本 UC の範囲外 |
| D-2 / D-4 / D-6 | drift コマンド（UC-013）の所管 |
| S-4 / S-5 / S-8 | snapshot コマンド（UC-012）の所管 |

## 12. 凍結履歴

| Date | Version | Note |
|---|---|---|
| 2026-06-13 | 1.0 | 初版凍結。`legixy-embed` の calibrate 関連公開 API（`compute_all_pair_scores` / `histogram` / `compute_recommended` / `calibrate`）と型（`AllPairScores` / `Histogram` / `HistogramStats` / `Bucket` / `BucketCount` / `RecommendedThresholds` / `Percentiles` / `CurrentThresholds` / `SkipSummary` / `CalibrateReport` / `CalibrateJsonOutput`）を確定。v3 `calibrate.rs` / `similarity.rs` 整合。SPEC-LGX-010.REQ.05 のパーセンタイル算出式（nearest-rank 変種）を凍結。`note` フィールド削除【v3 差分】。集約 Warning・INFO 文言を日本語 primary で確定【v3 差分】。crate 境界は ADR-LGX-020。HR7 凍結 |
