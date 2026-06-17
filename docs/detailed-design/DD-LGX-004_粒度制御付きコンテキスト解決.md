Document ID: DD-LGX-004

# DD-LGX-004: 粒度制御付きコンテキスト解決（context --granularity / --outline-only / --sections / --depth）の詳細設計

**親 SEQD**: SEQD-LGX-004
**親 RBD**: RBD-LGX-004 / **親 UC**: UC-LGX-004
**対象言語**: Rust（CLI 本体）
**境界 API 凍結**: yes（本ドキュメント確定後、crate 間公開 API は変更不可。追加は許容、削除・改名・型変更は次版 SPEC 改訂。HR7）

> 言語別規律は `guides/language-stacks/rust.md`。crate 境界・共有型は **ADR-LGX-020** で凍結済（本 DD は参照する）。型・関数シグネチャは v3 実装（`traceability-engine.v3.chg_to_lexigy/crates/lx-ctx/`）に整合させ引数互換を保つ。本 DD は粒度制御固有の型・関数を確定する。DD-LGX-002 と同一 `legixy-ctx` crate であり、両 DD の重複は DD-LGX-002 参照で解消する（`ContextCompiler` / `ContextError` / `ContextResult` 等の基盤は DD-LGX-002 で凍結）。

## 1. 対象範囲

- **主 crate**: `legixy-ctx`（粒度制御・サブノード展開・フィルタ・アウトライン変換・セクション整列・サイズ制限）
- **本 DD の担当範囲（DD-LGX-002 との分掌）**:
  - `Granularity` 列挙型（`document` / `subnode` 2 値）
  - `CompileInput` 粒度制御フィールド（`granularity` / `outline_only` / `sections` / `depth_limit`）
  - `build_outline` 関数（REQ.15 ATX 見出し抽出・階層インデント）
  - `UpstreamWalker::walk_chain_parent_only_with_depth`（REQ.17 depth 制限付き BFS）
  - sections フィルタ処理ロジック（`HashSet<&str>` + dedup、REQ.16）
  - `SectionFormatter::render`（REQ.10-14 の 6 セクション整列・サイズ上限・キャッシュブレーク点マーカ）
  - サブノード展開処理（`collect_upstream`、代替 3a fallback / 4-D 個別展開）
  - `AuditLogger`（REQ.07/19 ベストエフォート）
  - `RESULT_SIZE_LIMIT_CHARS` / `CACHE_BREAKPOINT_MARKER` 定数
- **依存 crate（共有型は ADR-LGX-020、再定義しない）**: `legixy-graph`（`TraceGraph` / `Node` / `EdgeKind` / `NodeId` / `SubnodeKind`）, `legixy-db`（`rusqlite::Connection` — context_log, layer_rules, layer_documents）, `legixy-core`（`TraceConfig`）
- **公開 API surface**: 本 DD §3（粒度制御固有。`ContextCompiler::compile` / `render` は DD-LGX-002 で凍結済、本 DD ではシグネチャを引用・参照のみ）
- **関連 SEQD**: SEQD-LGX-004

## 2. 型定義

### 2.1 主要データ型

```rust
// legixy-ctx（本 DD で確定する粒度制御固有の入力型）

/// SPEC-LGX-003.REQ.01/03: compile_context の入力。粒度制御フィールドを含む。
/// v3 底本: lx-ctx/src/compiler.rs CompileInput（行 39-64）
#[derive(Debug, Clone)]
pub struct CompileInput {
    /// 必須: 編集対象ファイルのパス（1 件以上）
    pub target_files: Vec<PathBuf>,
    /// 粒度制御（既定: Document。v3 Granularity::default() = Document）
    pub granularity: Granularity,
    /// context_log の payload.command に記録（返却内容に影響しない、REQ.01 S2-06）
    pub command: Option<String>,
    /// REQ.15: true 時、upstream body を ATX 見出し（h1〜h3）階層リストに置換
    pub outline_only: bool,
    /// REQ.16: Some(vec) 時、subnode 粒度で指定 ID のみを upstream に通す
    /// None = フィルタなし（全サブノード）。重複 ID は事前 dedup（CACHE-INV-1）
    pub sections: Option<Vec<String>>,
    /// REQ.17: Some(N) で上流 N 階層に制限、None で無制限（v0.2.0 互換）
    pub depth_limit: Option<usize>,
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

/// SPEC-LGX-003.REQ.03: 粒度種別（2 値のみ）
/// v3 底本: lx-ctx/src/compiler.rs Granularity（行 21-37）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub enum Granularity {
    /// v0.1.0 互換。ドキュメント全文を返却。
    #[default]
    Document,
    /// サブノード単位で返却（子サブノードを個別 artifact に展開、REQ.03/4-D）。
    Subnode,
}

impl Granularity {
    /// context_log の granularity カラム値 / CLI 引数文字列と一致させる
    pub fn as_str(&self) -> &'static str {
        match self {
            Granularity::Document => "document",
            Granularity::Subnode => "subnode",
        }
    }
}

// ─── 返却型（legixy-ctx / DD-LGX-002 で凍結済。本 DD は粒度制御フィールドを参照確認）
// v3 底本: lx-ctx/src/result.rs（行 11-67）

/// SPEC-LGX-003.REQ.10/11/14: ContextResult は 6 セクション相当のデータを保持する。
/// DD-LGX-002 で凍結済のため再定義しない。粒度制御関連フィールドを下記に抜粋。
///
/// pub struct ContextResult {
///     pub upstream: Vec<UpstreamArtifact>,      // REQ.11 サブノード整列対象
///     pub custom_documents: Vec<CustomDocument>,  // REQ.10 第 6 セクション
///     pub granularity: Granularity,               // 整列規則の分岐に使用
///     pub unresolved_targets: Vec<PathBuf>,        // REQ.20 未解決記録
///     // ... (他フィールドは DD-LGX-002 参照)
/// }
///
/// pub struct UpstreamArtifact {
///     pub artifact_id: NodeId,
///     pub type_code: String,
///     pub file_path: PathBuf,
///     pub chain_distance: usize,
///     pub body: String,             // subnode 粒度時はセクション本文 or anchor のみ
///     pub subnode_id: Option<NodeId>, // subnode 粒度時に Some
///     pub anchor: Option<String>,   // サブノード見出しテキスト
///     pub drift_score: Option<f32>,
/// }
// → 上記は legixy-ctx/src/result.rs で定義（ADR-LGX-020 §2.3）。本 DD で再定義しない。
```

### 2.2 列挙 / Sum 型

```rust
// Granularity は §2.1 で確定済み。

// SectionFormatter 内部の整列方針（公開定数として固定）
// v3 底本: lx-ctx/src/section_formatter.rs SectionFormatter::upstream_sort_rule（行 152-157）
impl SectionFormatter {
    /// --granularity に応じた Upstream Artifacts 整列規則名（テスト用）
    pub fn upstream_sort_rule(granularity: Granularity) -> &'static str {
        match granularity {
            Granularity::Document => "artifact_id-asc",
            // SPEC-LGX-003.REQ.11 準拠: 親ドキュメント ID 辞書順 → 同一ドキュメント内は
            // アンカー出現順（ドキュメント物理位置順）。v3 は anchor バイト辞書順だったが、
            // (1) 出現順の方が消費側（AI）の読解に資する、(2) 整列順は引数互換契約
            // （LGX-COMPAT-001 は CLI 引数/終了コード/MCP のみ凍結、出力整列順は対象外）、
            // (3) サブノード ID は heading_path ハッシュで順序不変（SPEC-LGX-002.REQ.05）、
            // により SPEC 準拠（出現順）を正準とする。S2-21 解決（DD-freeze 裁定 2026-06-13 A-1）。
            // CACHE-INV-1 は出現順でも決定論的に充足。
            Granularity::Subnode => "parent_id-asc,anchor-appearance-order",
        }
    }

    pub const RENDER_SORT_STRATEGY: &'static str = "index-array";
}
```

### 2.3 エラー型

```rust
/// legixy-ctx（粒度制御・サイズ上限エラーを含む）
/// DD-LGX-002 で凍結済。本 DD は粒度制御固有の variant を参照確認する。
/// v3 底本: lx-ctx/src/error.rs（行 8-31）
pub enum ContextError {
    /// SPEC-LGX-003.REQ.13 / CACHE-INV-3 / NFR-LGX-001.PERF.09
    /// 返却本文が 500,000 文字（Unicode コードポイント数）を超過した場合。
    /// 切り捨て・自動要約は行わない（P-02「判断は人間に委ねる」）。
    ResultTooLarge { current: usize, limit: usize },

    /// graph.toml パース失敗 → exit 1
    Graph(String),

    /// 設定ファイル不在/破損 → exit 1（SUPP S2-07 と整合）
    InvalidInput(String),

    /// engine.db 操作失敗（ベストエフォート経路は Err を抑制するが
    /// compile() 内部での Db エラーは伝搬する）
    Db(#[from] rusqlite::Error),

    Io(#[from] std::io::Error),
    Serde(#[from] serde_json::Error),
}
// → 実体は legixy-ctx/src/error.rs。本 DD は v3 整合の確認のみ。
```

- 終了コードは `ContextError` → exit 1、引数構文誤り（clap）→ exit 2（granularity 不正値は exit 1、SUPP S2-07）、正常終了 → exit 0（REQ.19: 本処理成功・audit 失敗も exit 0）。
- `ResultTooLarge` はエラーメッセージ文字列を SPEC-LGX-003.REQ.13 のとおりに固定（v3 `error.rs:12-15` の文字列と一致）。

## 3. 公開 API surface（凍結、HR7）

### 粒度制御固有関数（本 DD が確定）

| 関数 | シグネチャ | 不変条件 | 冪等性 | 同期/非同期 |
|---|---|---|---|---|
| `legixy_ctx::ContextCompiler::compile` | `fn compile(&self, input: &CompileInput) -> Result<ContextResult, ContextError>` | 同一入力 → 同一 ContextResult（CTX-INV-1/REQ.04）。read-only（context_log 以外の永続状態を変更しない） | yes（audit は non-idempotent だが本処理は幂等） | 同期 |
| `legixy_ctx::ContextCompiler::render` | `fn render(&self, result: &ContextResult) -> Result<String, ContextError>` | 6 セクション順・バイト単位決定論（CACHE-INV-1/2）。500,000 文字超過で `ResultTooLarge`（CACHE-INV-3） | yes | 同期 |
| `legixy_ctx::SectionFormatter::render` | `fn render(result: &ContextResult) -> Result<String, ContextError>` | REQ.10〜14 の 6 セクション整列・マーカ・サイズ上限を担保 | yes | 同期 |
| `legixy_ctx::SectionFormatter::enforce_size_limit` | `fn enforce_size_limit(rendered: &str) -> Result<(), ContextError>` | `rendered.chars().count() > 500_000` ⇒ `ResultTooLarge`（defence-in-depth） | yes | 同期 |
| `legixy_ctx::UpstreamWalker::walk_chain_parent_only_with_depth` | `fn walk_chain_parent_only_with_depth(&self, start: &NodeId, depth_limit: Option<usize>) -> Result<Vec<UpstreamArtifact>, ContextError>` | Chain/ParentChild エッジのみ逆方向 BFS。visited セットで循環遮断（CTX-INV-4）。`None` で無制限、`Some(N)` で N 階層（REQ.17） | yes | 同期 |
| `legixy_ctx::build_outline`（crate 内 pub(crate)） | `fn build_outline(content: &str) -> String` | h1〜h3 ATX 見出しのみ抽出。スペース必須（`# title` 形式のみ）。h4+ / スペース無し行 / 空タイトルを除外。インデントは `"  " × (level - 1)`（REQ.15） | yes | 同期 |

- `compile` の `input.sections` は `Some(vec)` 時に `subnode` 粒度でのみ有効（`document` 時は無視、REQ.16）。渡す前に trim 後空トークン除去・重複 dedup を完了させること（CACHE-INV-1 保全）。
- `depth_limit = Some(0)` の場合、`walk_chain_parent_only_with_depth` は空 Vec を返し exit 0（REQ.17 v3 差分）。CLI 経由では stderr Info 診断を出力（stdout・終了コード不変）。
- `sections` に親ドキュメント ID（`#` を含まない ID）が指定された場合、subnode ID と一致しないため単に除外（エラーにしない）。CLI 経由では stderr Info 診断（REQ.16 v3 差分）。
- v3 実装（`compile()` 内 `audit.log(input, &result)?`）は audit 失敗を `?` で伝搬するが、legixy では REQ.19 に従い `if let Err(e) = audit.log(...) { eprintln!(...) }` として Ok を維持する（v3 との差分）。

### 定数（凍結）

```rust
/// SPEC-LGX-003.REQ.13 / CACHE-INV-3
pub const RESULT_SIZE_LIMIT_CHARS: usize = 500_000;

/// REQ.12 / CACHE-INV-2: キャッシュブレーク点マーカ
pub const CACHE_BREAKPOINT_MARKER: &str = "<!-- cache-breakpoint: stable-end -->";
```

### 再エクスポート（lib.rs）

```rust
pub use compiler::{CompileInput, ContextCompiler, Granularity};
pub use error::ContextError;
pub use result::{
    ContextResult, CustomDocument, LayerDocument, ResolvedTarget, TargetNodeMetadata,
    UpstreamArtifact,
};
pub use section_formatter::SectionFormatter;
```

## 4. module / package 構成

```
legixy-ctx/
├── src/
│   ├── lib.rs              // Document ID: SRC-LGX-004（再エクスポート・定数）
│   ├── compiler.rs         // ContextCompiler::compile / render / collect_upstream
│   │                        // build_outline（pub(crate)、REQ.15）
│   │                        // CompileInput / Granularity
│   ├── upstream_walker.rs  // UpstreamWalker::walk_chain_parent_only_with_depth（REQ.17）
│   ├── section_formatter.rs // SectionFormatter::render（REQ.10-14、6 セクション）
│   ├── audit_logger.rs     // AuditLogger（REQ.07/19 ベストエフォート）
│   ├── layer_resolver.rs   // LayerResolver（Layer/Additional Guidelines 解決）
│   ├── custom_edge_resolver.rs // CustomEdgeResolver（REQ.10 第 6 セクション）
│   ├── file_resolver.rs    // FileResolver（target_files → NodeId 逆引き、REQ.20）
│   ├── content_cache.rs    // ContentCache（ファイル本文のインメモリキャッシュ）
│   ├── result.rs           // ContextResult / UpstreamArtifact / ... （返却型）
│   ├── error.rs            // ContextError（ResultTooLarge 等）
│   └── subnode/            // サブノード展開補助
│       ├── mod.rs           // pub use
│       ├── content_extractor.rs // ContentExtractor::extract_section（content_range 切出し）
│       ├── score_lookup.rs  // ScoreLookup::get_drift（drift_score 取得）
│       └── resolver.rs      // サブノード ID 逆引き補助
└── Cargo.toml
```

依存方向（DAG、ADR-LGX-020）: `legixy-ctx` → `legixy-graph` / `legixy-db` / `legixy-core`。循環なし。

## 5. ライフタイム / 所有権 / 借用 方針

- `ContextCompiler<'a>` は `&'a TraceGraph` / `&'a TraceConfig` / `Option<&'a rusqlite::Connection>` / `&'a Path` を **借用**（所有権を取らない。read-only 判定）。
  - v3 底本: `lx-ctx/src/compiler.rs ContextCompiler<'a>`（行 66-71）に整合。
- `CompileInput` は `Vec<PathBuf>` / `Option<Vec<String>>` を所有（CLI から生成し compile 呼び出し後は破棄）。
- `ContextResult` は所有を返す（呼び出し側が `render` → stdout 出力 → exit 判定に使う）。
- `UpstreamWalker<'a>` は `&'a TraceGraph` を借用。`collect_upstream` の戻り値 `Vec<UpstreamArtifact>` は所有。
- `ContentCache` は `Arc<str>` でキャッシュ（ファイル内容の共有参照。所有権を複数箇所で持つ）。
- sections フィルタの `HashSet<&str>` は `input.sections` から導出した一時借用（`compile` のスコープ内で完結）。
- `Arc` / `Mutex` 不要（単一スレッド逐次。§7）。

## 6. エラー伝播戦略

- **`compile()` の伝搬方針**: graph/config 読込失敗・ファイル解決失敗は `Err(ContextError)` として exit 1。audit 書込失敗は `if let Err(e) = audit.log(...) { eprintln!("[legixy-ctx] audit log write failed (best-effort): {e}") }` として `Ok` を維持（REQ.19。v3 との差分: v3 は `?` で伝搬）。
- **`render()` の伝搬方針**: `SectionFormatter::render` が逐次集計で `check_early_cut` → `ResultTooLarge` を返す。最終 `enforce_size_limit` で defence-in-depth（CACHE-INV-3、v3 底本: `section_formatter.rs:123`）。
- **部分成功**: 未解決 target_files は `ResolvedTarget { artifact_id: None }` として保持し、`compile()` は空上流で `Ok(ContextResult)` を返す（REQ.20）。欠損ノードは `enrich_upstream` 内で空 body で継続（`let Some(node) = ... else { return Ok(art) }`）。
- **サブノード不在 fallback**: サブノードエッジが空の上流ノードは `document` 粒度と同様に全文返却（代替 3a、`subnodes_of(&parent_id).is_empty()` 分岐、v3 底本: `compiler.rs:212-231`）。
- panic 禁止: `unwrap` / `expect` 禁止（rust.md §4）。`unwrap_or_default` / `unwrap_or_else` を使用。
- ユーザ通知: ContextResult 本文 → stdout、ログ → stderr（OBS.02）。

## 7. 並行性 / 非同期境界

- `context` は **同期・単一スレッド・read-only**（REQ.09 の concurrent safety は SQLite WAL + `busy_timeout` で実現）。
- `ContentCache` は `Arc<str>` キャッシュだが、`compile()` の呼び出しスコープ内で完結（スレッド間共有なし）。
- `engine.db` への `context_log` 書込のみが副作用。WAL モード（`journal_mode=WAL`）+ `busy_timeout=5000ms`（SUPP S2-04、NFR REL.07）で複数同時呼び出しを排他制御。
- async なし（MCP 層（ts-mcp）から CLI へのサブプロセス委譲で MCP の async 境界を吸収する）。

## 8. テスト分類

| 分類 | 内容 | 対応 TP |
|---|---|---|
| Unit | `build_outline`（h1〜h3 抽出・インデント・h4+除外・スペース無し除外・空タイトル除外・見出し皆無時の空文字列）。`SectionFormatter::enforce_size_limit`（499,999 / 500,000 / 500,001 文字境界）。sections dedup・trim・空トークン除外。upstream_sort_rule 返値確認 | TP-LGX-003, TS-LGX-002（未作成） |
| Integration | `ContextCompiler::compile` + `render` の 2 段：document / subnode 粒度での 6 セクション順（REQ.10）・バイト決定論（REQ.14）。outline-only × subnode（anchor のみ）。sections フィルタ（存在 ID / 不在 ID 混在 / 全不在 / 空文字列）。depth_limit（1 / 2 / 0 / None）。サブノード不在 fallback。監査記録失敗時 exit 0 + stderr Warning | TP-LGX-003, TS-LGX-002（未作成） |
| フラグ組合せ | REQ.18 マトリクス（outline × document / sections × document / outline × sections-subnode / depth × 各組合せ）| TP-LGX-003 |
| Property-based | 同一入力 → 同一バイト列（REQ.14 CACHE-INV-1）。proptest で target_files / granularity / sections を生成 | TP-LGX-003 |
| Bench | compile + render の応答時間（NFR-LGX-001.PERF.03: サブノード 100 件で < 300ms / < 200ms、criterion） | NFR-LGX-001 |

## 9. ADR への参照

- **ADR-LGX-020**: crate 分割・共有型凍結（本 DD の crate 境界）
- ADR-LGX-004: 可用性 > 監査完全性（REQ.19 の設計判断根拠 — 本処理優先・audit はベストエフォート）
- ADR-LGX-014: SPEC 準拠原則
- ADR-LGX-016: env（バイナリ解決・モデルディレクトリ）
- ADR-LGX-019: REQ.10 Custom Documents 6 番目（TRIAGE §4 #1 で spec-change 承認済）

## 10. 関連 NFR

- NFR-LGX-001.PERF.03: compile_context 応答（サブノード 100 件、Step1 < 300ms / Step2 < 200ms）
- NFR-LGX-001.PERF.09: 返却本文 500,000 文字上限（CACHE-INV-3）
- NFR-LGX-001.REL.03: 冪等性（同一入力 → 同一結果）
- NFR-LGX-001.REL.05: BFS 決定性（IndexMap 挿入順の隣接リスト依存）
- NFR-LGX-001.REL.07: busy_timeout 5000ms（SQLite WAL 排他）
- NFR-LGX-001.SEC.02: 並行呼出し安全性
- NFR-LGX-001.OBS.02: 出力先（本文 = stdout / ログ = stderr）

## 11. v3 整合と SUPP [要決定] の採択

本 DD での採択を記録する。

| SUPP 項目 | 採択 |
|---|---|
| S2-01 返却テキストのフォーマット | v3 `section_formatter.rs` の形式を底本に採用（各セクション `# {Title}\n\n`、エントリ間 `\n---\n`、末尾 `\n\n`、LF 固定、upstream エントリヘッダ `artifact_id:` / `type:` / `file_path:` / `chain_distance:` / `subnode_id:` / `anchor:` / `drift_score:` + 空行 + body） |
| S2-02 REQ.13 エラーメッセージ | v3 `error.rs:12-15` の文字列定数に合致（`ResultTooLarge` の Display 実装） |
| S2-20 Custom Documents 6 番目 | ADR-LGX-019（TRIAGE §4 #1）で承認済み。`SectionFormatter::render` の 6 番目セクションとして実装 |
| S2-21 subnode 整列キー | **SPEC-LGX-003.REQ.11 準拠（アンカー出現順）を正準とする**（DD-freeze 裁定 2026-06-13 A-1 で確定）。v3 は anchor バイト辞書順だったが、出現順の方が消費側（AI）の読解に資し、整列順は引数互換契約（LGX-COMPAT-001）の対象外、サブノード ID は heading_path ハッシュで順序不変（SPEC-LGX-002.REQ.05）のため SPEC を採用。CACHE-INV-1 は出現順でも決定論的に充足。v1.0 の「v3 バイト辞書順採用」を撤回 |
| S2-22 engine.db open 経路 | DB 存在する場合のみ open して記録、不在なら記録なし exit 0（FB-INV-4 整合）。CLI 経路は v3 同様 db=None を基本とし、`--with-db` 等オプションの有無は CLI 実装（DD-LGX-002 / `legixy-cli`）に委ねる |
| S2-23 stderr 診断文言 | 本 DD で確定: `[legixy-ctx] Warning: audit log write failed (best-effort): {e}` (REQ.19)。REQ.16 親 ID Info: `Info: --sections received a document-level ID '{id}', not a subnode ID. Use '#'-containing subnode IDs instead.`。REQ.17 depth 0 Info: `Info: --depth 0 results in empty upstream (no ancestors returned).`。REQ.20 未解決起点 Info: `Info: the following target paths were not found in the graph and were skipped: {paths}` |
| S2-24 REQ.20 欠損記録フォーマット | Target Node Metadata セクション末尾に `unresolved_targets:` キー（パス辞書順、CACHE-INV-1 保全のため決定論的）を追加。空の場合は出力しない |
| S2-25 REQ.15 見出し皆無時の空 body | artifact ヘッダ行群（`artifact_id:` / `type:` / ... `chain_distance:`）+ 空行 + 空文字列（末尾改行のみ）を正準とする（v3 `render_upstream_entry` の body="" ケースと同形） |
| S2-26 Target Node Metadata 内容 | `artifact_id:` / `outgoing_edges:` (件数) / `incoming_edges:` (件数) / `subnode_count:` + REQ.20 の `unresolved_targets:` (辞書順パスリスト、空時は出力しない) |

## 12. 凍結履歴

| Date | Version | Note |
|---|---|---|
| 2026-06-13 | 1.0 | 初版凍結。`legixy-ctx` の粒度制御固有型（`Granularity` / `CompileInput` 粒度フィールド）・関数（`build_outline` / `walk_chain_parent_only_with_depth` / `SectionFormatter::render` / `enforce_size_limit`）・定数（`RESULT_SIZE_LIMIT_CHARS` / `CACHE_BREAKPOINT_MARKER`）を確定。v3 lx-ctx 整合（SUPP S2-21 は v3 anchor バイト辞書順採用・人間確認推奨）。SUPP S2-23〜26 の [要決定] を本 DD で確定。crate 境界は ADR-LGX-020。HR7 凍結 |
| 2026-06-13 | 1.1 | DD-freeze 裁定 A-1: S2-21 subnode 整列キーを v3 バイト辞書順 → **SPEC-LGX-003.REQ.11 準拠（アンカー出現順）** へ訂正（`upstream_sort_rule` の Subnode 分岐 `anchor-bytes-asc` → `anchor-appearance-order`）。整列順は LGX-COMPAT-001 凍結対象外・サブノード ID は順序不変のため SPEC 準拠を採用。CACHE-INV-1 充足は不変。境界 API（型・関数シグネチャ）不変 |
