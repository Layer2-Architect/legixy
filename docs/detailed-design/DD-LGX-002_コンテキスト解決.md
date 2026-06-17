Document ID: DD-LGX-002

# DD-LGX-002: コンテキスト解決（compile_context）の詳細設計

**親 SEQD**: SEQD-LGX-002
**親 RBD**: RBD-LGX-002 / **親 UC**: UC-LGX-002
**対象言語**: Rust（CLI 本体）+ TypeScript（MCP ts-mcp 転送層）
**境界 API 凍結**: yes（本ドキュメント確定後、crate 間公開 API は変更不可。追加は許容、削除・改名・型変更は次版 SPEC 改訂。HR7）

> 言語別規律は `guides/language-stacks/rust.md`（Rust）/ `guides/language-stacks/typescript.md`（TS）。crate 境界・共有型は **ADR-LGX-020** で凍結済（本 DD は参照する）。型は v3 実装（traceability-engine.v3.chg_to_lexigy `crates/lx-ctx/`）に整合させ引数互換を保つ。

## 1. 対象範囲

- **主 crate**: `legixy-ctx`（compile_context・コンテキスト解決・粒度制御・キャッシュ整列）
- **依存 crate（共有型は ADR-LGX-020、再定義しない）**:
  - `legixy-graph`（`TraceGraph` / `Node` / `Edge` / `NodeId` / `EdgeKind` / `SubnodeKind`）
  - `legixy-db`（`rusqlite::Connection` 共有—オプション、WAL モード、busy_timeout 5000ms）
  - `legixy-core`（`Id` / `TraceConfig` / 共通エラー基底）
  - `legixy-cli`（サブコマンドディスパッチ経由で `ContextCompiler` を呼ぶ。`legixy-cli` の `commands/context.rs` が `compile` → `render` を実行し stdout へ出力）
- **MCP 転送層**: `ts-mcp`（TypeScript）の `src/tools/compile-context.ts` が MCP ツール `compile_context` を実装し、引数を snake_case→kebab-case 変換して Rust CLI へ忠実転送する（MCP-INV-2。ADR-LGX-020 §2.2）
- **公開 API surface**: 本 DD §3（`legixy-ctx` の crate 公開関数・型）
- **関連 SEQD**: SEQD-LGX-002

## 2. 型定義

### 2.1 主要データ型

```rust
// legixy-core（共有、ADR-LGX-020）— 再定義しない
pub struct Id(String); // {type}-{area}-{seq} または {id}#{subnode_hash}
pub type NodeId = String; // legixy-graph 所有（v3 lx-graph::model::NodeId = String）。参照のみ・再定義しない（ADR-LGX-021 §2.1）。グラフキー文脈=NodeId、ID 書式検証文脈=Id

// legixy-ctx: コンパイラ入力
pub struct CompileInput {
    pub target_files: Vec<PathBuf>,
    pub granularity: Granularity, // 既定 Granularity::Document
    pub command: Option<String>,  // context_log payload のみ（返却内容に影響しない）
    pub outline_only: bool,       // SPEC-LGX-003.REQ.15
    pub sections: Option<Vec<String>>, // SPEC-LGX-003.REQ.16。None = フィルタなし
    pub depth_limit: Option<usize>,    // SPEC-LGX-003.REQ.17。None = 無制限
}

impl Default for CompileInput {
    fn default() -> Self {
        Self {
            target_files: Vec::new(),
            granularity: Granularity::Document,
            command: None,
            outline_only: false,
            sections: None,
            depth_limit: None,
        }
    }
}

// legixy-ctx: コンテキスト解決メインクラス（v3 ContextCompiler 対応）
pub struct ContextCompiler<'a> {
    graph: &'a TraceGraph,
    config: &'a TraceConfig,
    db: Option<&'a rusqlite::Connection>,
    project_root: &'a Path,
}

// legixy-ctx: compile() の返却構造体（6 セクション対応、SPEC-LGX-003.REQ.10 v0.8.0）
pub struct ContextResult {
    pub targets: Vec<ResolvedTarget>,
    pub layer_guidelines: Vec<LayerDocument>,
    pub additional_guidelines: Vec<LayerDocument>,
    pub upstream: Vec<UpstreamArtifact>,
    pub custom_documents: Vec<CustomDocument>,
    pub target_metadata: Vec<TargetNodeMetadata>,
    pub granularity: Granularity,
    pub unresolved_targets: Vec<PathBuf>, // REQ.20: 未解決起点の記録
}

pub struct ResolvedTarget {
    pub file_path: PathBuf,
    pub artifact_id: Option<NodeId>, // 未解決時 None（REQ.20-1）
    pub type_code: Option<String>,
}

pub struct UpstreamArtifact {
    pub artifact_id: NodeId,
    pub type_code: String,
    pub file_path: PathBuf,
    pub chain_distance: usize,
    pub body: String,
    pub subnode_id: Option<NodeId>, // Subnode 粒度時に Some
    pub anchor: Option<String>,
    pub drift_score: Option<f32>,
}

pub struct LayerDocument {
    pub layer_name: String,
    pub node_id: NodeId,
    pub file_path: PathBuf,
    pub body: String,
    pub specificity: u32,
    pub priority: u32,
}

pub struct CustomDocument {
    pub from_id: NodeId,
    pub to_id: NodeId,
    pub file_path: PathBuf,
    pub body: String,
    pub reason: Option<String>,
}

pub struct TargetNodeMetadata {
    pub artifact_id: NodeId,
    pub outgoing_edges: Vec<(NodeId, EdgeKind)>,
    pub incoming_edges: Vec<(NodeId, EdgeKind)>,
    pub subnode_count: usize,
    // REQ.20: 未解決起点の記録（Target Node Metadata セクション内）
    pub unresolved_targets: Vec<PathBuf>,
}
```

> `TargetNodeMetadata.unresolved_targets` フィールドは v3 `TargetNodeMetadata` には存在しない（v3 は `result.rs:62-66` が `outgoing_edges/incoming_edges/subnode_count` のみ）。REQ.20 の決定論的記録要件（SUPP §2 S2-24 [要決定]）を充足するため legixy 新規追加。レンダリング: `# Target Node Metadata` セクション末尾に `unresolved_targets:` キーで決定論的列挙（PathBuf 辞書順昇順）。空の場合はフィールド自体を省略しバイト決定論（REQ.14）を保全する。

### 2.2 列挙 / Sum 型

```rust
// legixy-ctx
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub enum Granularity {
    /// v0.1.0 互換。ドキュメント全文を返却（REQ.01、既定値）。
    #[default]
    Document,
    /// サブノード単位で返却（SPEC-LGX-003.REQ.03 / UC-LGX-004）。
    Subnode,
}

impl Granularity {
    pub fn as_str(&self) -> &'static str {
        match self {
            Granularity::Document => "document",
            Granularity::Subnode => "subnode",
        }
    }
}

// legixy-ctx: 終了状態（REQ.19、凍結終了コード規約 LGX-COMPAT-001 §3）
pub enum ContextExitStatus {
    /// exit 0: 本処理成功（空 upstream・部分成功・監査ログ失敗を含む）
    Success,
    /// exit 1: 受理済み引数の意味的不正（granularity 不正値等）・実行時失敗・大規模返却エラー（REQ.13）
    Failure,
}
```

> **ResultTooLarge の終了コード（人間裁定 2026-06-13 で確定）= exit 1 / stderr**。v3 実装（`lx-cli/src/commands/context.rs` の `compiler.render(&result)?` が `ContextError::ResultTooLarge` を伝播 → stderr 出力 + exit 1）・SPEC-LGX-003.REQ.13「エラーを返却する」・DD-LGX-004（`ContextError → exit 1`）・LGX-COMPAT-001 §3（終了コード凍結）すべてに整合させる。v1.0 で検討した exit 0/stdout 案（「本処理完結後の size 検出ゆえ非失敗」）は v3 互換境界からの逸脱となるため撤回（DD-freeze 裁定 B-1、§11 v1.1）。

### 2.3 エラー型

```rust
// legixy-ctx（thiserror）
#[derive(Debug, thiserror::Error)]
pub enum ContextError {
    /// SPEC-LGX-003.REQ.13 / CACHE-INV-3 / NFR-LGX-001.PERF.09
    /// 返却本文が RESULT_SIZE_LIMIT_CHARS (500,000) を超過した場合。切り捨て・要約禁止。
    /// エラーメッセージは v3 実測（lx-ctx/src/error.rs:11-14）に整合。
    #[error(
        "compile_context result exceeds {limit} characters.\nCurrent size: {current} characters.\nSuggested action:\n  - Try --granularity subnode for finer-grained retrieval.\n  - Narrow the target scope."
    )]
    ResultTooLarge { current: usize, limit: usize },

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("db error: {0}")]
    Db(#[from] rusqlite::Error),

    #[error("graph error: {0}")]
    Graph(String),

    /// granularity 不正値など受理済み引数の意味的不正（exit 1）
    #[error("invalid input: {0}")]
    InvalidInput(String),

    #[error("serialization error: {0}")]
    Serde(#[from] serde_json::Error),
}
```

終了コード規約（LGX-COMPAT-001 §3）:
- `ContextError::InvalidInput` → exit 1
- `ContextError::Io` / `ContextError::Graph`（設定・グラフ読込失敗） → exit 1
- `ContextError::Db`（致命的 DB 障害） → exit 1
- `ContextError::ResultTooLarge` → stderr にエラー文言（REQ.13 規定書式）を出力し exit 1（v3 互換・DD-freeze 裁定 2026-06-13 B-1）
- 監査ログ書込失敗（`AuditLogger` 内部） → Ok(()) を維持し stderr Warning のみ（REQ.19 / ADR-LGX-004）
- 引数構文誤り（clap 層） → exit 2

部分成功（REQ.20-2）: 上流連鎖途中のファイル不在は `Err` に昇格させず `UpstreamArtifact { body: String::new() }` + 欠損記録として継続し exit 0。

## 3. 公開 API surface（凍結、HR7）

| 関数 | シグネチャ | 不変条件 | 冪等性 | 同期/非同期 |
|---|---|---|---|---|
| `legixy_ctx::ContextCompiler::new` | `fn new(graph: &'a TraceGraph, config: &'a TraceConfig, db: Option<&'a Connection>, project_root: &'a Path) -> Self` | graph/config/project_root は呼出スコープ内でライフタイム保持 | — | 同期 |
| `legixy_ctx::ContextCompiler::compile` | `fn compile(&self, input: &CompileInput) -> Result<ContextResult, ContextError>` | 同一入力 → 同一 ContextResult（REQ.04/14、CACHE-INV-1）。read-only（graph/db の検索のみ）。監査ログ失敗は Ok を維持（REQ.19/ADR-LGX-004）。起点未解決は partial success・exit 0（REQ.20）。ResultTooLarge は `Err(ContextError::ResultTooLarge)` を返し呼び出し側 CLI が stderr に出力し exit 1（v3 互換、DD-freeze 裁定 B-1） | yes（監査ログ副作用を除く） | 同期 |
| `legixy_ctx::ContextCompiler::render` | `fn render(&self, result: &ContextResult) -> Result<String, ContextError>` | 6 セクション・決定論的整列（REQ.10/11/12/14/CACHE-INV-1/2）。LF 固定。バイト単位決定論 | yes | 同期 |
| `legixy_ctx::SectionFormatter::render` | `fn render(result: &ContextResult) -> Result<String, ContextError>` | 上と同一仕様。`ContextCompiler::render` の内部委譲先（分離可能性のために公開） | yes | 同期 |
| `legixy_ctx::SectionFormatter::enforce_size_limit` | `fn enforce_size_limit(rendered: &str) -> Result<(), ContextError>` | `.chars().count() > 500,000` で `ResultTooLarge` | yes | 同期 |
| `legixy_ctx::UpstreamWalker::walk_chain_parent_only_with_depth` | `fn walk_chain_parent_only_with_depth(&self, start: &NodeId, depth_limit: Option<usize>) -> Result<Vec<UpstreamArtifact>, ContextError>` | Chain/ParentChild のみ逆 BFS。depth_limit=None で無制限（REQ.02/08/17）。visited セットで循環遮断（CTX-INV-4）。start がグラフ未登録なら空 Vec（REQ.20-1） | yes | 同期 |
| `legixy_ctx::AuditLogger::log` | `fn log(&self, input: &CompileInput, result: &ContextResult) -> Result<(), ContextError>` | db=None で no-op。失敗は stderr Warning のみで常に Ok(()) を返す（REQ.19/ADR-LGX-004）。WAL+busy_timeout 5000ms（REQ.09/S2-04）。DB 存在時のみ書込（S2-22 の解） | ベストエフォート | 同期 |

定数（凍結）:
```rust
pub const RESULT_SIZE_LIMIT_CHARS: usize = 500_000;
pub const CACHE_BREAKPOINT_MARKER: &str = "<!-- cache-breakpoint: stable-end -->";
pub const CONTEXT_LOG_BUSY_TIMEOUT_MS: u64 = 5_000; // S2-04
```

- `sections` フィルタの dedup・trim・空トークン無視は `compile` 呼び出し前に `legixy-cli` 層で正規化（REQ.16 縮退入力規則、CACHE-INV-1）。
- `outline_only` と `sections` の優先順位: sections フィルタが先、outline 変換が後（REQ.18 マトリクス、v3 `compiler.rs:186-265` の適用順と整合）。
- `depth=0` を CLI 経由で指定した場合、walker は空集合を返し exit 0 + stderr Info（REQ.17/v3 差分。SUPP S2-23 の文言確定は DD 後の人間査読）。
- **engine.db open 経路（S2-22 の解）**: `legixy-cli` の `commands/context.rs` は、`.legixy/engine.db` ファイルが存在する場合にのみ DB を開いて `ContextCompiler::new(db: Some(&conn))` を渡す。不在時は `db: None` で `AuditLogger::log` が no-op（FB-INV-4 / STATE-INV-1 / REQ.07 ベストエフォート）。DB ファイルを新規作成しない。

## 4. module / package 構成

```
legixy-ctx/
├── src/
│   ├── lib.rs              // Document ID: SRC-LGX-002 — CompileInput/ContextCompiler/ContextResult/
│   │                       //   Granularity/ContextError/SectionFormatter/UpstreamWalker/
│   │                       //   AuditLogger の pub use 再エクスポート。定数 2 件。
│   ├── compiler.rs         // ContextCompiler（compile / render / collect_upstream /
│   │                       //   enrich_upstream / collect_metadata）、CompileInput、Granularity
│   ├── result.rs           // ContextResult, ResolvedTarget, UpstreamArtifact, LayerDocument,
│   │                       //   CustomDocument, TargetNodeMetadata
│   ├── section_formatter.rs// SectionFormatter（render / enforce_size_limit / sorted_indices /
│   │                       //   render_*_entry / append_and_count / check_early_cut）
│   │                       //   REQ.10/11/12/13/14 / CACHE-INV-1/2/3 主担当
│   ├── upstream_walker.rs  // UpstreamWalker（walk_chain_parent_only_with_depth）
│   │                       //   REQ.02/08/17、Chain+ParentChild 逆 BFS、depth 制御
│   ├── audit_logger.rs     // AuditLogger（log / granularity_column）
│   │                       //   REQ.07/09/19、context_log ベストエフォート書込
│   ├── layer_resolver.rs   // LayerResolver（resolve / resolve_additional）
│   │                       //   engine.db layer_rules/layer_documents、First-Match-Wins
│   ├── file_resolver.rs    // FileResolver（resolve）
│   │                       //   target_files → (file_path, artifact_id) マッピング
│   ├── content_cache.rs    // ContentCache（get_or_load）
│   │                       //   ファイル本文の LRU/ARC キャッシュ（容量は DEFAULT_CACHE_CAPACITY）
│   ├── custom_edge_resolver.rs // CustomEdgeResolver（resolve）
│   │                       //   カスタムエッジ由来文書の解決（CTX-INV-3 準拠）
│   ├── error.rs            // ContextError（thiserror）
│   └── subnode/            // subnode 関連サブモジュール
│       ├── mod.rs          // pub use 再エクスポート
│       ├── content_extractor.rs // ContentExtractor（extract_section）
│       │                   //   content_range による本文切り出し
│       └── score_lookup.rs // ScoreLookup（get_drift）
│                           //   engine.db drift_scores から f32 lookup
└── Cargo.toml
```

依存方向（DAG、ADR-LGX-020）: `legixy-ctx` → `legixy-graph` / `legixy-db`（Optional）/ `legixy-core`。`legixy-cli` → `legixy-ctx`。循環なし。

**ts-mcp 構成**（TypeScript、MCP 転送層）:
```
ts-mcp/
└── src/
    └── tools/
        └── compile-context.ts  // Document ID: SRC-LGX-002-TS
                                //   MCP ツール compile_context の登録・引数変換・転送
```

- zod スキーマ: `target_files: z.array(z.string()).min(1)` / `command?: z.string()` / `granularity?: z.enum(["document","subnode"])` / `outline_only?: z.boolean()` / `sections?: z.string().min(1)` / `depth?: z.number().int().min(1)`
- snake_case → kebab-case 変換（`outline_only` → `--outline-only`、`depth` → `--depth`）
- `_meta["anthropic/maxResultSizeChars"] = 500_000`（CACHE-INV-4 / SPEC-LGX-009.REQ.13）付与
- `legixy/warnings` 転送: `_meta["legixy/warnings"]` に stderr の Warning 行を集積（SPEC-LGX-003.REQ.19 / ADR-LGX-004 / SPEC-LGX-009.REQ.03。v3 では未実装 = legixy 新規）

## 5. ライフタイム / 所有権 / 借用 方針

- `ContextCompiler<'a>` は `graph: &'a TraceGraph`、`config: &'a TraceConfig`、`db: Option<&'a Connection>`、`project_root: &'a Path` を **借用**（所有権を取らない）。呼び出し側 `legixy-cli` が所有し、read-only 判定。
- `compile()` は `&CompileInput` を借用し、`ContextResult` を所有して返す（render に渡すか、CLI 層が出力 + 解放）。
- `UpstreamWalker<'a>` は `graph: &'a TraceGraph` を借用。走査結果 `Vec<UpstreamArtifact>` は所有で返す（body 文字列は `String`）。
- `ContentCache` は `Arc<str>` でキャッシュ（ファイル本文の共有参照。スレッド安全は §7 参照）。
- `Id` / `NodeId` は `legixy-core`/`legixy-graph` 所有。`ContextCompiler` 内では `&NodeId` / `NodeId::clone()` で扱う。
- `'static` バウンド不要（全てコールスタック内完結）。

## 6. エラー伝播戦略

- **compile 境界**: `ContextCompiler::compile` は graph/config/file 読込失敗のみ `Err(ContextError)` を返す（exit 1）。検証 finding とは別概念。
- **ResultTooLarge**: `Err(ContextError::ResultTooLarge)` を `render()` が返す。`legixy-cli` が stderr にエラー文言（REQ.13 規定書式）を出力し exit 1（v3 互換 — `render(&result)?` 伝播で exit 1。DD-freeze 裁定 2026-06-13 B-1）。監査ログへの記録は ResultTooLarge 時でもベストエフォートで試行する。
- **部分成功継続（REQ.20-2）**: 上流ファイル不在は `ContextError` に昇格させず、`UpstreamArtifact { body: String::new() }` + 欠損フラグ記録として `ContextResult` に含め exit 0。欠損の決定論的記録位置: `TargetNodeMetadata.unresolved_targets`（PathBuf 辞書順昇順）。
- **監査ログ失敗（REQ.19/ADR-LGX-004）**: `AuditLogger::log` は `Err` を stderr に `eprintln!("[legixy-ctx] audit log write failed: {err}")` で出力し常に `Ok(())` を返す。compile() は Ok を維持。MCP 経由では stderr を `_meta["legixy/warnings"]` に転送（§1 ts-mcp 参照）。
- **DB 不在（FB-INV-4）**: `db=None` で `LayerResolver`・`AuditLogger` は no-op（空 Vec / Ok(())）。graph.toml のみで上流走査を返す。
- **panic 禁止**: `unwrap()`/`expect()` 禁止（rust.md §4）。グラフ参照失敗・ファイル読込失敗は全て `Result` 経由。
- **ユーザ通知**: render 結果 = stdout、ログ・診断 = stderr（REQ.07/19/20、OBS.02）。

## 7. 並行性 / 非同期境界

- `compile` / `render` は **同期・単一スレッド**。async なし。`legixy-cli` の tokio ランタイムからは `spawn_blocking` 相当で呼ぶ（CLI 全体は同期実行のため現行は直接呼出し）。
- **並行呼出し安全（REQ.09）**: 複数の MCP セッションが同時に `compile_context` を呼んでも、各 Rust プロセス（CLI サブプロセス）は独立。`engine.db` への `context_log` 書込み競合は SQLite WAL + busy_timeout 5000ms で排他制御（S2-04）。
- `ContentCache` は単一 `compile()` 呼出し内でのみ生存するスタック変数（スレッド共有なし）。`Arc<str>` は同一呼出し内の再利用用途のみ。
- `UpstreamWalker` の BFS は単一スレッド逐次（visited `HashSet` は `Arc`/`Mutex` 不要）。
- 検索性能: NFR-LGX-001.PERF.03（<300ms Windows / <200ms Linux、サブノード 100 件）は逐次で充足見込み（v3 実績）。必要時は benches/context_bench.rs（criterion）で計測。

## 8. テスト分類

| 分類 | 内容 | 対応 TP |
|---|---|---|
| Unit | `UpstreamWalker`: Chain/ParentChild のみ走査・Custom スキップ・depth 制御・循環遮断・未登録起点=空 Vec | TP-LGX-012, TP-LGX-014 |
| Unit | `SectionFormatter`: 6 セクション順序・CACHE_BREAKPOINT_MARKER 位置・バイト決定論・ResultTooLarge 超過検出 | TP-LGX-012 |
| Unit | `SectionFormatter::render` サブノード整列（アンカー出現順—SPEC-LGX-003.REQ.11 準拠、DD-freeze 裁定 2026-06-13 A-1。詳細は DD-LGX-004 §11/§12）| TP-LGX-012, TP-LGX-014 |
| Unit | `build_outline`: h1〜h3 抽出・インデント・見出し皆無で空（枠維持、REQ.15/GAP-LGX-047） | TP-LGX-012 |
| Unit | `sections` フィルタ: dedup・trim・空トークン無視・全無効=空 upstream（REQ.16/GAP-LGX-045） | TP-LGX-014 |
| Unit | `AuditLogger`: db=None で no-op・書込失敗は Ok() 維持 | TP-LGX-012 |
| Integration | `ContextCompiler::compile` 全体: 基本フロー・部分成功継続・監査ログ書込失敗・起点未解決・大規模返却エラー | TP-LGX-012, TP-LGX-014 |
| Integration | 終了コード: exit 0（正常・部分成功）/ exit 1（graph 読込失敗・ResultTooLarge〔v3 互換・stderr〕）/ exit 2（clap 構文誤り） | TP-LGX-012, TP-LGX-009 |
| Integration | フラグ組合せマトリクス（REQ.18）: outline+document / sections+document / outline+sections(subnode) / depth+all | TP-LGX-014 |
| Property-based | バイト単位決定論（REQ.14）: 同一入力で ContextResult を 10 回 render し全バイト列一致（proptest） | TP-LGX-012 |
| Property-based | セクション整列の決定論: 任意順の入力に対して sorted_indices が同一 index 配列を返す | TP-LGX-012 |
| Bench | サブノード 100 件の compile 応答時間（NFR-LGX-001.PERF.03、criterion） | NFR-LGX-001 |
| TS Unit | MCP `compile_context` の引数変換: snake_case→kebab-case・`depth: 0` は zod.min(1) で reject | TP-LGX-009 |
| TS Integration | `_meta["anthropic/maxResultSizeChars"]` = 500,000 付与確認 | TP-LGX-009 |
| TS Integration | `_meta["legixy/warnings"]` 転送: stderr Warning が _meta に到達する（legixy 新規） | TP-LGX-009 |

## 9. ADR への参照

- **ADR-LGX-020**: crate 分割・共有型凍結（本 DD の crate 境界）
- **ADR-LGX-004**: 監査ログ書込失敗時は可用性優先（MCP-INV-4 ベストエフォート化）— `AuditLogger` の設計根拠
- ADR-LGX-003: embedding 決定論モデル（drift_score 再現性、subnode 粒度の整列に影響）
- ADR-LGX-009: contextual retrieval 決定論（BFS 走査順序の安定性前提）
- ADR-LGX-014: SPEC 準拠原則
- ADR-LGX-016: env var 規約（プロジェクトルート・バイナリ解決）
- ADR-LGX-019: Custom Documents を 6 番目セクションとした SPEC 改訂の記録

## 10. 関連 NFR

- NFR-LGX-001.PERF.03: compile_context 応答時間（<300ms Windows / <200ms Linux、サブノード 100 件）
- NFR-LGX-001.PERF.09: 返却本文 500,000 文字上限（REQ.13 / CACHE-INV-3）
- NFR-LGX-001.REL.03: 冪等性（同一入力→同一結果）
- NFR-LGX-001.REL.05: BFS 決定性（IndexMap 挿入順依存、SPEC-LGX-002.REQ.08 前提）
- NFR-LGX-001.REL.07: SQLite busy_timeout 5000ms（並行書込排他）
- NFR-LGX-001.SEC.02: SQLite WAL モード（並行呼出し安全）
- NFR-LGX-001.OBS.02: 出力先（返却本文=stdout / 診断・警告=stderr）

## 11. 凍結履歴

| Date | Version | Note |
|---|---|---|
| 2026-06-13 | 1.0 | 初版凍結。`legixy-ctx` の CompileInput/Granularity/ContextCompiler/ContextResult/UpstreamArtifact/LayerDocument/CustomDocument/TargetNodeMetadata/ContextError 型と compile/render/walk_chain_parent_only_with_depth/log 公開 API を確定（v3 lx-ctx 整合）。6 セクション構成（SPEC-LGX-003.REQ.10 v0.8.0）反映。`legixy/warnings` _meta 転送（SPEC-LGX-003.REQ.19/ADR-LGX-004）の TS 側実装を凍結。crate 境界は ADR-LGX-020。HR7 凍結 |
| 2026-06-13 | 1.1 | cross-DD 整合（ADR-LGX-021 §2.1）: `NodeId` を `= Id` から `= String`（legixy-graph 所有・v3 lx-graph::model 準拠）へ訂正。公開 API・フィールド構成は不変（`NodeId` 記号のまま、解決先型のみ訂正）。HR7 境界契約は不変 |
| 2026-06-13 | 1.2 | DD-freeze 裁定 B-1（人間裁定 2026-06-13、TS フェーズで顕在化）: `ResultTooLarge` の終了コードを exit 0/stdout → **exit 1/stderr** へ訂正。v3 実装（`render(&result)?` 伝播 → exit 1）・SPEC-LGX-003.REQ.13・DD-LGX-004・LGX-COMPAT-001 §3 と整合。`ContextExitStatus` enum から `ResultTooLarge` バリアントを削除し `Failure`（exit 1）へ統合。型シグネチャ（`ContextError::ResultTooLarge` variant）は不変、終了コード分類のみ訂正 |
