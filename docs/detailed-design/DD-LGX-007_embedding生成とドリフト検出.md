Document ID: DD-LGX-007

# DD-LGX-007: embedding 生成とドリフト検出の詳細設計

**親 SEQD**: SEQD-LGX-007
**親 RBD**: RBD-LGX-007 / **親 UC**: UC-LGX-007
**対象言語**: Rust（CLI 本体）
**境界 API 凍結**: yes（本ドキュメント確定後、crate 間公開 API は変更不可。追加は許容、削除・改名・型変更は次版 SPEC 改訂。HR7）

> 言語別規律は `guides/language-stacks/rust.md`。crate 境界・共有型は **ADR-LGX-020** で凍結済（本 DD は参照する）。型は v3 実装（traceability-engine.v3.chg_to_lexigy `crates/lx-embed/`）に整合させ引数互換を保つ。drift の standalone コマンド（`legixy drift <artifact_id>`）は **UC-LGX-013** が規定するため本 DD は embed 生成（`legixy-embed` crate の生成・格納・bulk API エンジン）に専念する。

## 1. 対象範囲

- **主 crate**: `legixy-embed`（Embedder / EmbedOptions / embed_all / bulk similarity API / DriftCalculator / EmbeddingStore / Contextual Retrieval フォールバック骨格）
- **依存 crate（共有型は ADR-LGX-020、再定義しない）**:
  - `legixy-graph`（`TraceGraph` / `Node` / `SubnodeKind` / `EdgeKind` / `content_range`）
  - `legixy-db`（`DbError` / engine.db 接続・embeddings テーブル DDL・migration）
  - `legixy-core`（`Config` / `SemanticConfig` / `ContextualRetrievalConfig` / 共通エラー基底）
- **公開 API surface**: 本 DD §3（`legixy-embed` crate 公開関数・型）
- **関連 SEQD**: SEQD-LGX-007

## 2. 型定義

### 2.1 主要データ型

```rust
// legixy-core（共有、ADR-LGX-020）
// SemanticConfig, ContextualRetrievalConfig は legixy-core 所有。ここでは再定義しない。

// legixy-embed

/// ONNX モデル + tokenizer を保持し、1 ノード分の embedding 生成を担う。
/// Session::run が &mut self を要求するため `RefCell<Session>` で内部可変性を確保
/// （v3 embedder.rs:32-37 と同方式）。
pub struct Embedder {
    session: std::cell::RefCell<ort::session::Session>,
    tokenizer: tokenizers::Tokenizer,
    model_version: String,   // REQ.10 複合キー（§2.2 ModelVersion 参照）
    dim: usize,              // 初期値は model.onnx から動的確定（REQ.01）
}

/// embed_node の出力。1 ノード分の embedding 生成結果。
pub struct EmbedResult {
    pub embedding: Vec<f32>,
    pub dim: usize,
    pub model_version: String,
    pub content_hash: String,        // 4 段正規化後の SHA-256 hex 64 桁（REQ.03、§2.3-a）
    pub context: Option<String>,     // Contextual Retrieval 有効時のみ Some
    pub context_hash: Option<String>,
}

/// embed_all の振る舞い制御オプション。
pub struct EmbedOptions {
    pub force: bool,             // true = content_hash 一致でも強制再生成（REQ.02）
    pub include_subnodes: bool,  // デフォルト true（REQ.09、Phase 2）
    pub contextual: Option<ContextualConfig>,
    pub project_root: Option<std::path::PathBuf>,
    pub node_filter: NodeFilter, // --all / --node <ID>+ の選択（REQ.02）
}

impl Default for EmbedOptions {
    fn default() -> Self {
        Self {
            force: false,
            include_subnodes: true,
            contextual: None,
            project_root: None,
            node_filter: NodeFilter::All,
        }
    }
}

/// embed_all の実行結果サマリ（--json スキーマに対応）。
/// v3 `EmbedReport` は (node_id, message) タプル列・failed 欄なし。
/// SPEC 0.7.0 REQ.02 で確定した新形式（v3 差分: failed 追加、errors オブジェクト化）。
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct EmbedReport {
    pub generated: usize,
    pub skipped: usize,
    pub failed: usize,           // v3 差分: errors.len() の明示フィールド
    pub errors: Vec<EmbedErrorItem>,  // v3 差分: オブジェクト（タプルでない）
}

/// errors 配列の 1 要素（--json スキーマ）。
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EmbedErrorItem {
    pub node_id: String,
    pub message: String,
}

/// engine.db の embeddings テーブル 1 行分の表現（v3 EmbeddingRow と同一構造）。
#[derive(Debug, Clone, PartialEq)]
pub struct EmbeddingRow {
    pub node_id: String,
    pub embedding: Vec<f32>,
    pub dim: usize,
    pub model_version: String,
    pub content_hash: String,
    pub context: Option<String>,
    pub context_hash: Option<String>,
    pub created_at: String,
}

/// engine.db read/write ラッパ（v3 EmbeddingStore と同一責務）。
pub struct EmbeddingStore {
    conn: rusqlite::Connection,
}

/// Contextual Retrieval の詳細設定（REQ.06/06.1）。
pub struct ContextualConfig {
    pub opts: CrOptions,
}

/// CrOptions（フォールバック制御パラメータ、v3 contextual.rs:15-23 と同一前例）。
pub struct CrOptions {
    pub timeout_sec: u64,       // デフォルト 30（REQ.06.1）
    pub max_retries: u32,       // デフォルト 3（REQ.06.1）
    pub base_backoff_ms: u64,   // デフォルト 1000（REQ.06.1）
}

/// 集約 Warning の収集バッファ（ノード毎ではなく 1 件まとめ出力のため）。
struct AggregatedWarnings {
    dim_mismatch_count: usize,
    zero_norm_count: usize,
    truncated_count: usize,   // トークン上限切り捨て（REQ.01）
    empty_skip_count: usize,  // 空テキスト skip（REQ.02）
}

/// bulk API: エッジ類似度スコア 1 件（REQ.11）。
/// legixy-embed 所有の共有型。report（DD-LGX-010）が consumer として参照する（ADR-LGX-021 §2.3）。
pub struct EdgeScore {
    pub from: String,
    pub to: String,
    pub score: f32,
    pub edge_kind: legixy_graph::EdgeKind,
}

/// bulk API: リンク候補スコア 1 件（REQ.11）。
/// legixy-embed 所有の共有型。report（DD-LGX-010）が consumer として参照する（ADR-LGX-021 §2.3）。
pub struct CandidateScore {
    pub from: String,
    pub to: String,
    pub score: f32,
}

/// ドリフト検出結果 1 件（REQ.05 / REQ.11）。
/// v3 `DriftFinding.missing_file: bool` を 3 種の `DriftKind` enum に置き換える（v3 差分）。
pub struct DriftFinding {
    pub node_id: String,
    pub stored_hash: Option<String>,  // kind=Missing の場合は None
    pub current_hash: Option<String>, // kind=FileMissing の場合は None
    pub kind: DriftKind,
}

/// calibrate ヒストグラム 1 バケット（REQ.11）。
pub struct Bucket {
    pub low: f32,
    pub high: f32,
    pub count: usize,
}

/// embedding_snapshots テーブルのメタ情報（list 用、v3 SnapshotMeta と同一）。
pub struct SnapshotMeta {
    pub snapshot_id: String,
    pub label: Option<String>,
    pub node_count: usize,
    pub taken_at: String,
}
```

### 2.2 列挙 / Sum 型

```rust
// legixy-embed

/// --all / --node <ID>+ の排他選択（REQ.02）。
/// CLI で clap が parse 後、legixy-cli がここへ変換して embed_all に渡す。
pub enum NodeFilter {
    All,
    Ids(Vec<String>),  // --node で指定した ID リスト（未登録 ID は embed_all が Err を返す）
}

/// ハッシュ照合結果の 3 状態（SEQD-LGX-007 基本フロー alt 分岐）。
/// SCORE-INV-1 + SCORE-INV-2（content_hash + model_version 双方一致で skip）。
pub enum HashMatchState {
    Skip,      // content_hash + model_version が一致 → 再計算不要
    Regen,     // content_hash 不一致（stale）または model_version 不一致
    Missing,   // embeddings 行が存在しない（未生成）
}

/// DriftFinding の種別（REQ.05 3 状態 + ファイル不在を区別）。
/// v3 差分: v3 の missing_file bool を 3 種の enum に置き換え（SUPP-006 §2.5-e）。
pub enum DriftKind {
    ContentChanged, // stale: embedding 行あり、content_hash 不一致
    FileMissing,    // embedding 行あり、ファイルが読めない
    EmbeddingMissing, // 未生成: embedding 行なし（v3 差分: v3 は無言 skip）
}

/// model_version 複合キー（REQ.10、GAP-LGX-115）。
/// 前処理プロファイルの識別子。
pub enum PreprocessProfile {
    Plain,      // prefix なし（paraphrase-multilingual-MiniLM-L12-v2 等 BERT 系）
    E5Prefix,   // "query:" / "passage:" prefix 付与（intfloat/multilingual-e5 系）
}

/// ONNX モデル出力 shape 検証結果（REQ.01 GAP-LGX-103）。
pub enum ShapeValidation {
    Ok { hidden_dim: usize },
    Invalid { reason: String },
}
```

### 2.3 エラー型

```rust
// legixy-embed（v3 EmbedError に相当、SPEC 0.7.0 差分を反映）
#[derive(Debug, thiserror::Error)]
pub enum EmbedError {
    #[error("model load failed: {path:?}: {reason}")]
    ModelLoadFailed { path: std::path::PathBuf, reason: String },
    // 試行パスのリストは ModelLoadFailed の reason に含める（REQ.02 / GAP-LGX-106）

    #[error("model shape invalid: {reason}")]
    ModelShapeInvalid { reason: String },
    // REQ.01 GAP-LGX-103: 読込時 shape 検証失敗 → exit 1

    #[error("tokenizer error: {reason}")]
    TokenizerError { reason: String },

    #[error("onnx inference error: {reason}")]
    OnnxInferenceError { reason: String },

    #[error("db error: {0}")]
    Db(#[from] legixy_db::DbError),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("dimension mismatch: stored={stored_dim}, current={current_dim}")]
    DimensionMismatch { stored_dim: usize, current_dim: usize },
    // standalone drift の次元不一致は Error（REQ.04）

    #[error("node not found: {0}")]
    NodeNotFound(String),

    #[error("invalid content range for {node_id}: {reason}")]
    InvalidContentRange { node_id: String, reason: String },
    // REQ.09 GAP-LGX-118: content_range 防御検証失敗（部分失敗継続の Error 計上経路）
    // → embed_all では Err に昇格させず EmbedErrorItem に変換して継続

    #[error("contextual retrieval failed for {node_id}: {reason}")]
    ContextualRetrievalFailed { node_id: String, reason: String },
}
```

- **終了コード規約**（v3 embed.rs:26-29 / 99-101 と整合）:
  - 引数構文誤り（clap 検出）→ exit 2
  - モデル解決/shape 検証失敗（`EmbedError::ModelLoadFailed` / `ModelShapeInvalid`）→ exit 1
  - 部分失敗（`failed > 0`）→ exit 1
  - 全件成功（skipped 含む） → exit 0
- **`--node` と `--all` の排他違反**: clap の `conflicts_with` で実装（exit 2）。指定 ID が graph.toml 未登録の場合は意味的不正のため exit 1（REQ.02）。
- **DB 書込失敗**: `EmbedError::Db` として `embed_all` が `Err` を返す（全体 abort、ノード単位 Tx のコミット失敗を全体失敗に昇格）。ノード単位の Tx 内失敗は部分失敗継続経路（§6 参照）。

## 3. 公開 API surface（凍結、HR7）

| 関数 | シグネチャ | 不変条件 | 冪等性 | 同期/非同期 |
|---|---|---|---|---|
| `Embedder::new` | `fn new(model_dir: &Path, model_version: &str) -> Result<Self, EmbedError>` | model.onnx + tokenizer.json 存在 + shape 検証合格の場合のみ Ok（REQ.01 GAP-LGX-103）。model_version は呼出側が複合キー文字列として生成済み（§2.2 参照）| no | 同期 |
| `Embedder::embed_node` | `fn embed_node(&self, text: &str, parent_doc: Option<&str>, node_id: &str) -> Result<EmbedResult, EmbedError>` | 空テキスト（正規化後 0 文字）の場合は Err を返さず呼出側が skip 判定する（REQ.02 空テキスト skip は embed_all / orchestration 層で処理）。内部で 4 段正規化 + content_hash 計算（REQ.03）| yes（決定論は順序のみ、ビット再現は対象外 REQ.04） | 同期 |
| `embed_all` | `fn embed_all(graph: &TraceGraph, store: &EmbeddingStore, embedder: &Embedder, options: EmbedOptions) -> Result<EmbedReport, EmbedError>` | ノード単位 Tx（REQ.08）。部分失敗継続（REQ.09）。DB 書込失敗のみ Err に昇格。結果 EmbedReport.failed == errors.len() | no（副作用: engine.db 更新） | 同期 |
| `compute_model_version` | `fn compute_model_version(model_name: &str, onnx_path: &Path, profile: PreprocessProfile, dim: usize) -> Result<String, EmbedError>` | `{model_name}:{onnx_sha256_8hex}:{profile}:{dim}` 形式（要 §5 参照）の文字列を返す（REQ.10 GAP-LGX-115）。同一 ONNX ファイルなら同一 model_version | yes | 同期 |
| `normalize_content` | `fn normalize_content(raw: &str) -> String` | BOM 除去→CRLF/CR→LF→NFC→末尾正規化（1 末尾改行に正規化）の 4 段を適用（REQ.03 GAP-LGX-114）。環境非依存の content_hash を保証（SCORE-INV-1） | yes | 同期 |
| `content_hash_for` | `fn content_hash_for(content: &str) -> String` | `normalize_content` 適用後の UTF-8 バイト列に対する SHA-256 hex 64 桁（小文字）。v3 `sha256_hex` に正規化前処理を追加した版 | yes | 同期 |
| `EmbeddingStore::new` | `fn new(conn: rusqlite::Connection) -> Self` | conn は呼出側が open（legixy-db 経由）。Store は conn を所有 | — | 同期 |
| `EmbeddingStore::is_up_to_date` | `fn is_up_to_date(&self, node_id: &str, content_hash: &str, model_version: &str) -> Result<bool, DbError>` | content_hash + model_version 双方一致（SCORE-INV-1 + SCORE-INV-2）で true（v3 store.rs:37-53 と同一） | yes | 同期 |
| `EmbeddingStore::upsert_with_subnode_meta` | `fn upsert_with_subnode_meta(&self, node: &Node, result: &EmbedResult) -> Result<(), DbError>` | ノード単位 1 Tx（REQ.08）。INSERT OR REPLACE による冪等 upsert | no | 同期 |
| `EmbeddingStore::load_all` | `fn load_all(&self) -> Result<Vec<EmbeddingRow>, DbError>` | ORDER BY node_id ASC（SCORE-INV-1 決定性担保、v3 store.rs:165-190） | yes | 同期 |
| `EmbeddingStore::load_embedding` | `fn load_embedding(&self, node_id: &str) -> Result<Option<EmbeddingRow>, DbError>` | 未登録は Ok(None)（HashMatchState::Missing 判定に使用） | yes | 同期 |
| `EmbeddingStore::create_snapshot` | `fn create_snapshot(&self, snapshot_id: &str, label: Option<&str>) -> Result<usize, DbError>` | 現 embeddings 全行を embedding_snapshots へコピー（1 Tx）。行数を返す | no | 同期 |
| `compute_edge_scores` | `fn compute_edge_scores(graph: &TraceGraph, store: &EmbeddingStore) -> Result<Vec<EdgeScore>, EmbedError>` | 次元不一致・未登録ペアは skip + 集約 Warning（v3 差分: 無言 skip）。出力順は graph.edges() 挿入順（REQ.11） | yes | 同期 |
| `compute_link_candidates` | `fn compute_link_candidates(graph: &TraceGraph, store: &EmbeddingStore, threshold: f32) -> Result<Vec<CandidateScore>, EmbedError>` | 次元不一致は skip + 集約 Warning。O(N²)。出力順は (from, to) 昇順（REQ.11） | yes | 同期 |
| `compute_all_pair_scores` | `fn compute_all_pair_scores(store: &EmbeddingStore) -> Result<Vec<(String, String, f32)>, EmbedError>` | 次元不一致は skip + 集約 Warning。i < j の昇順（REQ.11） | yes | 同期 |
| `detect_drift` | `fn detect_drift(graph: &TraceGraph, store: &EmbeddingStore, project_root: &Path) -> Result<Vec<DriftFinding>, EmbedError>` | 未生成ノードを `DriftKind::EmbeddingMissing` として結果に含む（v3 差分: 無言 skip）。正規化 + content_range 切り出しを embed_all と同一経路で計算（SUPP-006 §2.3-e）。出力順は node_id ASC | yes | 同期 |
| `histogram` | `fn histogram(scores: impl Iterator<Item = f32>, buckets: usize) -> Vec<Bucket>` | **値域 [0.0, 1.0] 固定**の等幅 N バケット。域外スコアは [0,1] に clamp して算入（v3 `similarity.rs` L225 `score.clamp(0.0,1.0)` と一致。bucket_width=1.0/buckets）。末尾バケット上限 inclusive（v3 同様）。ストリーミング（REQ.11）。**calibrate 専用ユーティリティで正準定義は DD-LGX-011 §3（ADR-LGX-021 §2.3、本行は参照。旧 v1.0 の [-1,1] 記述は誤りで撤回）** | yes | 同期 |
| `cosine_similarity` | `fn cosine_similarity(a: &[f32], b: &[f32]) -> f32` | 値域 [-1.0, 1.0] に clamp（REQ.04 GAP-LGX-105）。ゼロノルム時は skip 経路（呼出側 AggregatedWarnings に計上、v3 差分: v3 は 0.0 返却）。呼出側でゼロノルム検査後に呼ぶ想定（内部では assert しない） | yes | 同期 |
| `read_current_content_for_node` | `fn read_current_content_for_node(node: &Node, graph: &TraceGraph, project_root: &Path) -> Result<String, EmbedError>` | embed_all と同一経路で content_range 切り出し（SUPP-006 §2.3-e、ISSUE-003 BUG-3 fix）。detect_drift / compute_node_drift_at が呼ぶ共有ヘルパ | yes | 同期 |

- `cosine_similarity` は値域 [-1,1] に clamp した後の値を返す。L2 正規化済みベクトルの内積であれば結果は本来 [-1,1] 内に収まるが、浮動小数点誤差で僅かに域外になる場合の防護（REQ.04 GAP-LGX-105）。
- 集約 Warning（次元不一致・ゼロノルム・トークン切り捨て・空テキスト skip）はすべて stderr に 1 件まとめ出力する。`--json` 指定時も stderr のみ（JSON スキーマに warning 欄なし、SUPP-006 §2.5-d）。

## 4. module / package 構成

```
legixy-embed/
├── src/
│   ├── lib.rs              // Document ID: SRC-LGX-007（公開 API 再エクスポート）
│   ├── embedder.rs         // Embedder / EmbedResult / embed_node（ONNX 推論・Mean Pooling・L2 正規化）
│   ├── content.rs          // normalize_content / content_hash_for / read_current_content_for_node
│   │                       //   （content_range 切り出し・4 段正規化・SHA-256。embed/drift 共通ヘルパ）
│   ├── orchestrator.rs     // embed_all / EmbedOptions / EmbedReport / EmbedErrorItem / NodeFilter
│   │                       //   （部分失敗継続・ノード単位 Tx 呼出し・空テキスト skip・集約 Warning）
│   ├── store.rs            // EmbeddingStore / EmbeddingRow / SnapshotMeta
│   │                       //   （is_up_to_date / upsert_with_subnode_meta / load_all / snapshot API）
│   ├── similarity.rs       // bulk API: compute_edge_scores / compute_link_candidates /
│   │                       //   compute_all_pair_scores / histogram / AggregatedWarnings
│   ├── drift.rs            // DriftCalculator / detect_drift / cosine_similarity / DriftFinding / DriftKind
│   ├── model_version.rs    // compute_model_version / PreprocessProfile / ShapeValidation
│   │                       //   （model_version 複合キー生成・ONNX shape 検証）
│   ├── contextual.rs       // ContextualConfig / CrOptions / LlmClient trait /
│   │                       //   synthesize_with_fallback / mask_api_key（REQ.06/06.1）
│   ├── preprocessor.rs     // EmbeddingPreprocessor（Phase 1: パススルー。DocType 分岐用インターフェイス）
│   └── error.rs            // EmbedError
└── Cargo.toml
    // ort（ONNX Runtime）, tokenizers, ndarray, rusqlite, sha2, thiserror,
    // unicode-normalization（normalize_content の NFC、REQ.03）, serde/serde_json
```

依存方向（DAG、ADR-LGX-020）:
```
legixy-embed → legixy-graph / legixy-db / legixy-core
legixy-check → legixy-embed（SemanticChecker が bulk API を消費、SPEC-LGX-004）
legixy-cli   → legixy-embed（embed サブコマンド）
```
循環なし。

## 5. ライフタイム / 所有権 / 借用 方針

- **`embed_all`**: `&TraceGraph` / `&EmbeddingStore` / `&Embedder` を借用（read-only または Store が &self で書込。所有権を取らない）。`EmbedOptions` は値渡し（`project_root` の PathBuf 所有権含む）。`EmbedReport` は所有を返す。
- **`EmbeddingStore`**: `rusqlite::Connection` を所有（move）。`&self` で upsert・load が可能なのは `unchecked_transaction()` が `&self` で動作するため（v3 同方式）。複数 Store のインスタンス生成は禁止（1 プロセス 1 Connection が SQLite の busy_timeout 設定の前提）。
- **`Embedder`**: `RefCell<Session>` により `&self` で `embed_node` を呼べる（Session::run は `&mut self` が必要なため）。`Arc` 不要（単一スレッド逐次、§7）。
- **`DriftFinding`**: 所有を返す（Vec<DriftFinding>）。`node_id` / `stored_hash` / `current_hash` は String で所有。呼出側が出力・集計に使う。
- **`EmbeddingRow`**: `Vec<f32>` の embedding を所有。bulk API（similarity.rs）では `load_all` が返す Vec<EmbeddingRow> をローカル変数に保持し、参照スライスを渡して演算する（`'static` バウンド不要）。
- **`normalize_content` / `content_hash_for`**: `&str` を受け取り `String` を返す純関数。`read_current_content_for_node` は `String` を返す（スライスではなく所有）。

## 6. エラー伝播戦略

- **モデルロード失敗**（`ModelLoadFailed` / `ModelShapeInvalid`）: `Embedder::new` が `Err` を返す。`embed_all` の呼出し前に `legixy-cli` 層が処理し exit 1（全体 abort）。
- **ノード単位部分失敗継続**（REQ.09 / REQ.08）:
  - ファイル読込失敗・`InvalidContentRange`・空テキスト skip・`embed_node` 失敗（`OnnxInferenceError` 等）→ `EmbedErrorItem` に変換して `report.errors` に push し後続ノードへ継続。`Err` に昇格しない。
  - 個別ノードの `upsert_with_subnode_meta` 失敗: ノード単位 Tx が rollback（そのノードのみ）。ただし `DbError` は `EmbedError::Db` を経由して `embed_all` が `Err` に昇格させる（DB 接続異常は全体 abort）。
- **空テキスト skip**（REQ.02 GAP-LGX-101）: `normalize_content` 後 0 文字のノードは `embed_node` を呼ばず skipped に計上、集約 Warning 1 件（stderr）。
- **トークン上限超過**（REQ.01 GAP-LGX-102）: tokenizer が truncation 設定に従い先頭 N トークンで切り捨て。切り捨て発生ノード数を `AggregatedWarnings.truncated_count` に計上し、ループ後に集約 Warning 1 件（stderr）。
- **content_range 防御検証**（REQ.09 GAP-LGX-118）: 逆転（start > end）・ファイル長超過・UTF-8 境界違反を検出した場合は `EmbedError::InvalidContentRange` を生成し、`embed_all` 内でこれを `EmbedErrorItem` に変換して部分失敗継続。`unwrap` / `expect` / UTF-8 全文フォールバック禁止（v3 差分、v3 は全文 fallback）。文字境界安全な切り出し: `str::is_char_boundary` で境界を確認し、違反の場合は `InvalidContentRange` を返す。
- **CR フォールバック**（REQ.06.1）: LLM API 呼出し失敗（タイムアウト・リトライ尽き）の場合 `synthesize_with_fallback` は `Ok(None)` を返す（CR 無効扱いで通常 embedding 継続）。フォールバック発生時は stderr に Warning を出力。observations テーブルへの記録は後続確認事項（SUPP-006 §2.2-f、category 語彙未確定）。
- **panic 禁止**: `unwrap` / `expect` は禁止。形式不正・境界違反はすべて `Result` で返す（rust.md §4）。

## 7. 並行性 / 非同期境界

- `embed_all` は**同期・単一スレッド・逐次**。ノード間の並列化は将来最適化（本 DD では逐次）。
- ONNX Runtime セッションは `RefCell<Session>` を使用するため、`Embedder` は `Send + Sync` を満たさない。CLI のシングルスレッド実行を前提（async なし）。
- LLM API（Contextual Retrieval）呼出しは `reqwest::blocking` 等の同期 HTTP クライアントを想定（`async fn` を導入する場合は `tokio::Runtime::block_on` でラップし public interface は同期を維持する）。
- `EmbeddingStore` の `Connection` は `!Send`（rusqlite 制約）。CLI の単一スレッド実行で充足。
- 集約 Warning は `AggregatedWarnings` をループ前にゼロ初期化し、ループ後に 1 回 eprintln する（スレッド安全性不要）。

## 8. テスト分類

| 分類 | 内容 | 対応 TP |
|---|---|---|
| Unit | `normalize_content`（BOM/CRLF/NFC/末尾 各バリアント）、`content_hash_for`（クロスプラットフォーム一致）、`cosine_similarity`（ゼロノルム skip / clamp / 正常値）、`HashMatchState` 判定ロジック、`DriftKind` 3 状態、`compute_model_version`（ONNX 差し替え検出）、`histogram`（均等幅・末尾 inclusive・clamp） | TP-LGX-006, TP-LGX-004 |
| Integration | `embed_all`（--force 再生成・空テキスト skip・content_range Error 計上後続継続・ノード単位 Tx rollback・--json スキーマ検証）、`detect_drift`（EmbeddingMissing 包含・stale/missing 区別メッセージ）、`compute_edge_scores` / `compute_link_candidates`（次元不一致 skip + 集約 Warning）、shape 検証 exit 1、モデル解決 exit 1 | TP-LGX-006, TP-LGX-004 |
| Property-based | `embed_all` 決定性（同一入力 → 同一 EmbedReport、スキップ件数含む順序不変）、`normalize_content` 冪等性（2 回適用 == 1 回適用）、`content_hash_for` 決定性 | TP-LGX-006 D1（proptest） |
| E2E / AT | `legixy embed --all`（全ノード生成・DB 格納確認）、`--node` 複数指定、`--force` 強制再生成、`--all` と `--node` 排他 exit 2、未登録 ID exit 1、モデル差し替え model_version 変化確認 | TP-LGX-006 |
| Bench | `embed_all` スループット（≥ 50 nodes/sec の NFR-LGX-001.PERF.08 は暫定、L12 モデルで実測後見直し。SUPP-006 §2.8-a）| NFR-LGX-001 |

## 9. ADR への参照

- **ADR-LGX-020**: crate 分割・共有型凍結（本 DD の crate 境界）
- ADR-LGX-003: embedding 決定論モデル（順序決定性のみ保証・ビット再現は対象外、REQ.04）
- ADR-LGX-009: Contextual Retrieval 非決定性との両立（content_hash のみ freshness に寄与・context キャッシュ、REQ.06）
- ADR-LGX-002: `--node` / `--force` の加算的拡張記録（REQ.02 GAP-LGX-120、LGX-COMPAT-001 §4 #4）
- ADR-LGX-015: DB パス（engine.db の配置規約）
- ADR-LGX-016: env（バイナリ解決・モデルディレクトリ解決順: `--models-dir` > `LGX_MODELS_DIR` > `TE_MODELS_DIR` > 設定ファイル、REQ.02 GAP-LGX-106）
- ADR-LGX-014: SPEC 準拠原則

## 10. 関連 NFR

- NFR-LGX-001.PERF.08: embed スループット（≥50 nodes/sec 暫定、L12 モデルで実測後見直し要）
- NFR-LGX-001.REL.02: 部分失敗時の継続（ノード単位 rollback + 後続継続）
- NFR-LGX-001.REL.06: トランザクション境界（ノード/サブノード単位 1 Tx）
- NFR-LGX-001.SEC.03/04: panic 禁止・入力検証（content_range 防御・ゼロベクトル非クラッシュ）
- NFR-LGX-001.SEC.05: API キーのログ非出力（`mask_api_key` で sk-ant- 等をマスク、REQ.07）
- NFR-LGX-001.OBS.02: 出力先（EmbedReport=stdout / 集約 Warning・ログ=stderr）
- NFR-LGX-001.COMPAT.07/08: BOM 除去・改行統一（`normalize_content` REQ.03 GAP-LGX-114）

## 11. 凍結履歴

| Date | Version | Note |
|---|---|---|
| 2026-06-13 | 1.0 | 初版凍結。`legixy-embed` の型定義（EmbedResult / EmbedOptions / EmbedReport / EmbeddingRow / EmbeddingStore / DriftFinding / DriftKind / EdgeScore / CandidateScore / Bucket / SnapshotMeta / HashMatchState / NodeFilter / EmbedError）と公開 API surface 17 関数を確定（v3 lx-embed 整合）。SCORE-INV-1/2 の双方一致判定・model_version 複合キー・ノード単位 Tx（GAP-108）・content_hash 4 段正規化（GAP-114）・DriftKind 3 種（GAP-110 v3 差分）・集約 Warning 群（GAP-101/102/104/105）・detect_drift の EmbeddingMissing 包含を確定。crate 境界は ADR-LGX-020。HR7 凍結。drift standalone（UC-013）は本 DD 対象外 |
| 2026-06-13 | 1.1 | cross-DD 所有確認（ADR-LGX-021 §2.3）: `EdgeScore` / `CandidateScore` および bulk similarity エンジン関数（`compute_edge_scores` / `compute_link_candidates` / `compute_all_pair_scores`）が本 DD（legixy-embed エンジン）所有・report（DD-LGX-010）が consumer である旨を明記。型・シグネチャ・公開 API は不変（記載追加のみ、HR7 凍結維持） |
