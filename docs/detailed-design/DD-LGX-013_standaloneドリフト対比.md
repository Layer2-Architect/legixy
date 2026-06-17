Document ID: DD-LGX-013

# DD-LGX-013: standalone ドリフト対比（drift）の詳細設計

**親 SEQD**: SEQD-LGX-013
**親 RBD**: RBD-LGX-013 / **親 UC**: UC-LGX-013
**対象言語**: Rust（CLI 本体）
**境界 API 凍結**: yes（本ドキュメント確定後、crate 間公開 API は変更不可。追加は許容、削除・改名・型変更は次版 SPEC 改訂。HR7）

> 言語別規律は `guides/language-stacks/rust.md`。crate 境界・共有型は **ADR-LGX-020** で凍結済（本 DD は参照する）。型は v3 実装（traceability-engine.v3.chg_to_lexigy `crates/lx-embed/`）に整合させ引数互換を保つ。

## 1. 対象範囲

- **主 crate**: `legixy-embed`（drift 対比ロジック・ベースライン解決・整合性検査・非有限スコア防御）
- **依存 crate（共有型は ADR-LGX-020、再定義しない）**:
  - `legixy-core`（`Id` / 共通エラー / `Severity` 基底 / `Config`）
  - `legixy-graph`（`TraceGraph` — 成果物登録確認・ファイルパス取得）
  - `legixy-db`（`EmbeddingStore` — embeddings 行・snapshots 行のロード）
  - `legixy-cli`（グローバルオプション `--models-dir` / `--json` / `--project-root` 解析後の渡し口）
- **公開 API surface**: 本 DD §3（`legixy-embed` の drift サブシステム公開関数）
- **関連 SEQD**: SEQD-LGX-013

本 DD は `legixy-embed` crate 内の drift サブシステムのみを対象とする。他の embed サブシステム（bulk similarity / calibrate / report / snapshot）は当該 UC の DD で扱う。

## 2. 型定義

### 2.1 主要データ型

```rust
// legixy-core（共有、ADR-LGX-020）
pub struct Id(String);  // {type}-{area}-{seq} or {id}#{subnode_hash}

// legixy-embed / drift サブシステム
/// 解決済み ONNX モデルの情報。モデル解決処理が生成し、埋め込み生成処理が参照する。
pub struct ResolvedModel {
    pub model_version: String,   // SPEC-LGX-006.REQ.10 複合キー（後述）
    pub dim: usize,              // 出力次元数（384 等、動的確定）
    pub source: ModelSource,     // 解決経路
    // 内部: onnxruntime セッション等は非公開フィールド
}

/// ONNX モデルの解決経路。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModelSource {
    Flag,          // --models-dir フラグ
    EnvLgx,       // 環境変数 LGX_MODELS_DIR
    EnvTe,        // 環境変数 TE_MODELS_DIR（旧名フォールバック）
    ConfigFile,   // .legixy.toml の semantic.model + project_root/models/
}

/// 成果物の解決結果。artifact_id が graph.toml に登録済で現行ファイルが存在する。
pub struct ArtifactRef {
    pub artifact_id: Id,
    pub file_path: std::path::PathBuf,
}

/// ONNX から生成した現行 embedding。
pub struct CurrentEmbedding {
    pub artifact_id: Id,
    pub vector: Vec<f32>,       // L2 正規化済（SPEC-LGX-006.REQ.04）
    pub dim: usize,
    pub model_version: String,  // 生成時の ResolvedModel.model_version と同値
}

/// DB から読み込んだベースライン embedding。
pub struct BaselineEmbedding {
    pub artifact_id: Id,
    pub vector: Vec<f32>,
    pub dim: usize,
    pub model_version: String,
    pub source: BaselineSource,
}

/// ベースラインの供給元。drift --json の baseline_source フィールドに対応。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BaselineSource {
    Embeddings,             // embeddings ストアの現行保存行（--against 省略時）
    Snapshot(String),       // snapshot の snapshot_id（--against snapshot:<token> 解決後）
}

/// drift 対比の最終出力。
pub struct DriftResult {
    pub artifact_id: Id,
    pub drift: Option<f32>,          // None = ベースライン不在（baseline_available: false）
    pub baseline_available: bool,
    pub baseline_source: Option<BaselineSource>,
}

/// --against 引数の解析済み表現。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AgainstSpec {
    Embeddings,                  // --against 省略（現行 embeddings ストア）
    SnapshotToken(String),       // snapshot:<token>（label 優先 → snapshot_id フォールバック）
    SnapshotLabelExplicit(String), // snapshot:label:<LABEL>（明示 label 形式、失敗は exit 1）
}
```

### 2.2 列挙 / Sum 型

```rust
// legixy-embed / drift サブシステム

/// model_version 複合キー（SPEC-LGX-006.REQ.10）。完全一致判定用。
/// 文字列表現: "{model_name}:{onnx_sha256_hex_16}:{preprocess_profile}:{dim}"
/// onnx_sha256_hex は先頭 16 hex 桁（64bit = 衝突リスク十分低い）。
/// 同名 ONNX 差し替え・前処理プロファイル変更を検出する（SCORE-INV-2）。
pub struct ModelVersion(String); // newtype、Display = 内部文字列

/// ベースライン指定の解析。AgainstSpec と重複しないよう、
/// AgainstSpec は legixy-cli の引数解析結果として渡される型、
/// こちら内部変換後の表現を明示。（実装では AgainstSpec をそのまま使う）

/// 整合性検査結果。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IntegrityCheckResult {
    Ok,
    DimMismatch { current_dim: usize, baseline_dim: usize },
    ModelVersionMismatch { current: String, baseline: String },
}

/// drift サブシステムの実行時エラー（exit 1 に対応）。
#[derive(Debug, thiserror::Error)]
pub enum DriftError {
    #[error("モデル解決失敗: {tried_paths:?}")]
    ModelNotFound { tried_paths: Vec<String> },

    #[error("モデル読み込みエラー: {0}")]
    ModelLoad(String),

    #[error("成果物 {artifact_id} は graph.toml に未登録です")]
    ArtifactNotFound { artifact_id: Id },

    #[error("成果物 {artifact_id} のファイルが見つかりません: {path}")]
    FileNotFound { artifact_id: Id, path: std::path::PathBuf },

    #[error("--against 値の形式が不正です（'snapshot:' プレフィクスが必要です）: {value}")]
    InvalidAgainstFormat { value: String },

    #[error("ラベル '{label}' に対応するスナップショットが見つかりません")]
    LabelNotFound { label: String },

    #[error("次元数不一致: 現行 {current_dim} / ベースライン {baseline_dim}")]
    DimMismatch { current_dim: usize, baseline_dim: usize },

    #[error("モデルバージョン不一致: 現行 {current} / ベースライン {baseline}")]
    ModelVersionMismatch { current: String, baseline: String },

    #[error("非有限スコアが発生しました（NaN/Inf）")]
    NonFiniteScore,

    #[error("DB エラー: {0}")]
    Db(#[from] legixy_db::DbError),

    #[error("グラフ読み込みエラー: {0}")]
    Graph(#[from] legixy_graph::GraphError),

    #[error("I/O エラー: {0}")]
    Io(#[from] std::io::Error),
}
```

### 2.3 エラー型

エラー型は §2.2 の `DriftError` に集約する。

- **終了コード規約**（SPEC-LGX-010.REQ.01 / NFR-LGX-001.OBS.05 / LGX-COMPAT-001 §3）:
  - `DriftError` のいずれか → exit 1
  - `AgainstSpec` の解析時の clap 構文エラー（型不正・必須引数欠落）→ exit 2（clap 既定）
  - ベースライン不在（`DriftResult.baseline_available = false`）→ exit 0（正常ライフサイクル）
- **部分的な `DriftError` 判断**（SPEC-LGX-010.REQ.03 の非対称性原則）:
  - `ArtifactNotFound` / `FileNotFound`: graph.toml が主張するファイルの壊れた状態 → exit 1
  - `LabelNotFound`（明示 label 形式 `snapshot:label:<L>`）→ exit 1（snapshot delete label:<L> と対称）
  - ベースライン行不在（曖昧形式 `snapshot:<token>` で解決したが行なし）→ exit 0（`DriftResult.baseline_available = false`）

## 3. 公開 API surface（凍結、HR7）

| 関数 | シグネチャ | 不変条件 | 冪等性 | 同期/非同期 |
|---|---|---|---|---|
| `legixy_embed::drift::run` | `fn run(graph: &TraceGraph, store: &EmbeddingStore, config: &Config, artifact_id: &Id, against: AgainstSpec) -> Result<DriftResult, DriftError>` | 同一入力（graph/store/config/artifact_id/against 全値）→ 同一 DriftResult。read-only（graph/store/fs を変更しない）。SPEC-LGX-010.REQ.06/REQ.07 | yes | 同期 |
| `legixy_embed::drift::resolve_model` | `fn resolve_model(config: &Config, models_dir_override: Option<&std::path::Path>) -> Result<ResolvedModel, DriftError>` | 解決順序: `models_dir_override` → `LGX_MODELS_DIR` env → `TE_MODELS_DIR` env（旧名。使用時 stderr Info）→ `config.semantic.model` + project_root/models/。失敗は全経路試行後に `DriftError::ModelNotFound{tried_paths}` | yes | 同期 |
| `legixy_embed::drift::parse_against` | `fn parse_against(raw: Option<&str>) -> Result<AgainstSpec, DriftError>` | `None` → `AgainstSpec::Embeddings`。`Some("snapshot:label:<L>")` → `AgainstSpec::SnapshotLabelExplicit`。`Some("snapshot:<t>")` → `AgainstSpec::SnapshotToken`。`Some(other)` → `DriftError::InvalidAgainstFormat` | yes | 同期 |
| `legixy_embed::drift::compute_drift` | `fn compute_drift(current: &CurrentEmbedding, baseline: &BaselineEmbedding) -> Result<f32, DriftError>` | `1.0 − cosine(current.vector, baseline.vector)`。値域 [0.0, 2.0]（SPEC-LGX-010.REQ.03）。非有限値 → `DriftError::NonFiniteScore` | yes | 同期 |
| `legixy_embed::drift::exit_code` | `fn exit_code(result: &Result<DriftResult, DriftError>) -> i32` | `Ok(r)` かつ `r.baseline_available = true` → 0、`Ok(r)` かつ `r.baseline_available = false` → 0、`Err(_)` → 1 | yes | 同期 |

- `run` は DB・graph・FS を変更しない（read-only。SPEC-LGX-010.REQ.07）。
- `engine.db` 不在時: `EmbeddingStore` の呼び出しが空ストア相当を返す（`legixy-db` 層が処理。本 API は `Err(DriftError::Db)` を受け取らず `DriftResult{baseline_available: false}` を返す）。DB ファイルを新規作成しない（REQ.07【v3 差分】）。
- `--json` の出力変換・stderr 診断出力は `legixy-cli` 層（本 DD のスコープ外）が担う。

## 4. module / package 構成

```
legixy-embed/
├── src/
│   ├── lib.rs           // Document ID: SRC-LGX-013（drift::* 再エクスポート）
│   ├── drift/
│   │   ├── mod.rs       // run / exit_code / 公開型の再エクスポート
│   │   ├── model.rs     // resolve_model / ResolvedModel / ModelSource / ModelVersion
│   │   ├── artifact.rs  // ArtifactRef（graph 境界との接続）
│   │   ├── embed.rs     // 現行 embedding 生成（ONNX 推論・mean pooling・L2 正規化）
│   │   ├── baseline.rs  // BaselineEmbedding / parse_against / AgainstSpec / ベースライン解決
│   │   │                //   （embeddings ストア / snapshot ラベル解決 / タイブレーク）
│   │   ├── integrity.rs // IntegrityCheckResult / 次元数照合 / model_version 照合
│   │   ├── compute.rs   // compute_drift / cosine_similarity / 非有限スコア防御
│   │   ├── result.rs    // DriftResult / BaselineSource
│   │   └── error.rs     // DriftError
│   ├── similarity.rs    // bulk similarity API（report/calibrate 用、他 UC の DD で扱う）
│   ├── store.rs         // EmbeddingStore の wrapper（他 UC 共用）
│   └── ... （他サブシステム）
└── Cargo.toml
```

**依存方向（DAG、ADR-LGX-020）**: `legixy-embed` → `legixy-graph` / `legixy-db` / `legixy-core`。`legixy-cli` → `legixy-embed`。循環なし。

`drift/model.rs` の ONNX 推論は `ort`（onnxruntime Rust バインディング）を使用する。`ort` は `legixy-embed` の依存として Cargo.toml に限定し、他 crate には漏出させない。

## 5. ライフタイム / 所有権 / 借用 方針

- `run` は `&TraceGraph` / `&EmbeddingStore` / `&Config` / `&Id` を **借用**（read-only 確認。複数呼び出しで共有可）。`DriftResult` は所有を返す。
- `ResolvedModel` は `run` 呼び出し前に `legixy-cli` 層で一度解決し、`&ResolvedModel` として `embed.rs` へ渡す（ONNX セッション再構築コストの回避）。セッション等の内部状態は `ResolvedModel` が所有し `Arc` 等で共有する必要はない（単一呼び出しの単一スレッドのため）。
- `CurrentEmbedding` / `BaselineEmbedding` は関数スコープ内で所有し、`compute_drift` へ `&CurrentEmbedding` / `&BaselineEmbedding` の参照を渡す。`'static` バウンド不要。
- `Vec<f32>` の embedding ベクトルは `BaselineEmbedding` が所有し、コピーしない（`integrity.rs` / `compute.rs` は `&[f32]` スライスで参照）。
- `Arc` / `Mutex` 不要（単一スレッド逐次。§7）。

## 6. エラー伝播戦略

- **内部**: 各サブモジュール（model/artifact/embed/baseline/integrity/compute）は `DriftError` を直接 `Err` で返す（`thiserror` の `From` 変換で `std::io::Error` / `legixy_db::DbError` / `legixy_graph::GraphError` を自動変換）。
- **公開境界**: `run` は `Result<DriftResult, DriftError>` を返す。ベースライン不在は `Ok(DriftResult{baseline_available: false})` として正常終了（exit 0）。
- **panic 禁止**: `unwrap` / `expect` を禁止（rust.md §4）。ONNX 推論内の不正スライス境界は `f32::is_finite()` 事前検査 + `DriftError::NonFiniteScore` で捕捉。
- **非有限スコア防御**（SPEC-LGX-010.REQ.09 / SUPP-010 C-6）: `compute.rs` で `cosine_similarity` の戻り値に `f32::is_finite()` を適用。`false` → `DriftError::NonFiniteScore` → exit 1。serde_json シリアライズ前に非有限値が残らないことを明示検査する（serde_json の挙動に依存しない）。
- **ユーザ通知**: エラーメッセージは stderr（`legixy-cli` 層が `eprintln!`）、drift 値・JSON は stdout（NFR-LGX-001.OBS.02）。日本語 primary（NFR-LGX-001.OBS.04）。

## 7. 並行性 / 非同期境界

- `drift` は **同期・単一スレッド・read-only**。async なし。ONNX 推論も同期（`ort` の同期 Session::run）。
- `legixy-cli` のサブコマンドディスパッチは `tokio::main` を採用しうるが、`legixy_embed::drift::run` 自体は sync fn として設計し `tokio::task::spawn_blocking` で呼び出し可能な形にしておく（将来の非同期対応余地）。本 DD では `spawn_blocking` を要求しない。
- 並行アクセス整合（外部更新中の drift）は対象外（UC-LGX-013 の事後条件が read-only を要求）。

## 8. テスト分類

| 分類 | 内容 | 対応 TP |
|---|---|---|
| Unit | `parse_against` の 3 形式と不正形式・`compute_drift` の値域 [0.0, 2.0]・非有限スコア `DriftError::NonFiniteScore`・IntegrityCheckResult 各ケース | TP-LGX-009（MCP サーバ関連 embedding テストを含む） |
| Unit（model_version 照合） | 次元一致 + model_version 不一致 → `DriftError::ModelVersionMismatch`・次元不一致 → `DriftError::DimMismatch`（SCORE-INV-2、SPEC-LGX-006.REQ.10） | TP-LGX-009 |
| Unit（ベースライン解決） | `AgainstSpec::Embeddings` / `SnapshotToken`（label 優先 → id フォールバック）/ `SnapshotLabelExplicit`（label 解決失敗 = exit 1）/ 曖昧形式で行不在 = exit 0 | TP-LGX-009 |
| Unit（モデル解決） | `resolve_model` の 4 経路（Flag / EnvLgx / EnvTe / ConfigFile）・旧名 `TE_MODELS_DIR` 使用時の stderr Info・全経路失敗 `DriftError::ModelNotFound{tried_paths}` | TP-LGX-009 |
| Integration | `run` の E2E（embed 済みノードへの drift 算出 → DriftResult 正常）・ベースライン不在 exit 0・現行ファイル欠落 exit 1・graph.toml 未登録 exit 1 | TP-LGX-009 |
| Integration（--against 3 形式） | 省略 / `snapshot:<id>` / `snapshot:label:<L>`（単一・複数=最新選択）/ `snapshot:label:<L>`（解決失敗 exit 1）のフルフロー | TP-LGX-009 |
| Integration（DB 不在） | engine.db 不在 → DB 新規作成せず → ベースライン不在相当 exit 0 | TP-LGX-009 |
| Property-based | `compute_drift` の決定性（同一 vector ペア → 同一 drift 値、proptest）・drift 値域 [0.0, 2.0] の任意ベクトル対（proptest） | TP-LGX-009 |
| Bench | 384 次元ベクトル対の `compute_drift` スループット（NFR PERF 要件の基礎計測、criterion） | NFR-LGX-001 |

## 9. ADR への参照

- **ADR-LGX-020**: crate 分割・共有型凍結（本 DD の crate 境界）
- **ADR-LGX-016**: env バイナリ解決・モデルディレクトリ（`LGX_MODELS_DIR` / `TE_MODELS_DIR` 解決の判断根拠）
- **ADR-LGX-019**: REQ.15 config 助言の射程・明示 label 形式の exit 1 化（TRIAGE §4 #19 の判断）
- ADR-LGX-003: embedding 決定論モデル（drift 冪等性 / SCORE-INV-1 再現性）
- ADR-LGX-007: 非有限スコアおよび model_version 照合ポリシー（GAP-LGX-185/186 の判断記録）
- ADR-LGX-014: SPEC 準拠原則
- ADR-LGX-015: DB パス（`.legixy/engine.db` の正準パス）

## 10. 関連 NFR

- NFR-LGX-001.OBS.02: 出力先（ドリフト値 = stdout / 診断ログ = stderr）
- NFR-LGX-001.OBS.04: エラーメッセージ日本語 primary
- NFR-LGX-001.OBS.05: 終了コード（0 = 正常 / 1 = 実行エラー / 2 = 引数構文誤り）
- NFR-LGX-001.PERF.07: WAL 必須（SQLite READ のアクセスに適用、legixy-db 層が保証）
- NFR-LGX-001.REL.06: embedding 生成の決定性（同一入力 → 同一ベクトル）

## 11. 凍結履歴

| Date | Version | Note |
|---|---|---|
| 2026-06-13 | 1.0 | 初版凍結。`legixy-embed` drift サブシステムの型定義・公開 API surface・module 構成・エラー伝播・所有権・並行性を確定。SPEC-LGX-010.REQ.03（drift）/ SPEC-LGX-006.REQ.10（model_version 完全一致照合）/ SUPP-LGX-010 §2.3 の v3 実測底本に整合。AgainstSpec 3 形式・DimMismatch/ModelVersionMismatch exit 1・NonFiniteScore exit 1・ベースライン不在 exit 0 の非対称性を型と関数で表現。model_version 複合キー（モデル名+ONNX SHA256 前 16 hex+前処理プロファイル+次元）を文字列型 `ModelVersion` で封止。タイブレーク（同一 taken_at: `ORDER BY taken_at DESC, snapshot_id DESC`）を baseline.rs で実装。HR7 凍結 |
