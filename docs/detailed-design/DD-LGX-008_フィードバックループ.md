Document ID: DD-LGX-008

# DD-LGX-008: フィードバックループ（observation / proposal / approve・reject / audit）の詳細設計

**親 SEQD**: SEQD-LGX-008
**親 RBD**: RBD-LGX-008 / **親 UC**: UC-LGX-008
**対象言語**: Rust（CLI 本体 `legixy-feedback`）+ TypeScript（MCP 転送層 `ts-mcp`）
**境界 API 凍結**: yes（本ドキュメント確定後、crate 間公開 API は変更不可。追加は許容、削除・改名・型変更は次版 SPEC 改訂。HR7）

> 言語別規律は `guides/language-stacks/rust.md` / `guides/language-stacks/typescript.md`。crate 境界・共有型は **ADR-LGX-020** で凍結済（本 DD は参照する）。型は v3 実装（traceability-engine.v3 `crates/lx-feedback/`）に整合させ引数互換を保つ。SPEC-LGX-007 v0.6.0 が正準であり、v3 との意図的差分は §§3/4/7 で明示する。

## 1. 対象範囲

- **主 crate（Rust）**: `legixy-feedback`（AutoObserver / ObservationRecorder / ProposalAnalyzer / ProposalManager / ContextAuditReader / FeedbackCli ファサード）
- **関連 crate（Rust、共有型参照 ADR-LGX-020）**: `legixy-db`（SQLite 接続・スキーマ・`observations` / `proposals` / `context_log` / `custom_edges` テーブル）、`legixy-check`（`CheckReport` / `CheckResult` — `feedback` コマンドの入力源）、`legixy-core`（共通エラー）、`legixy-embed`（`EmbedError`・`mask_api_key`）
- **転送層（TypeScript）**: `ts-mcp/src/tools/observe.ts`、`ts-mcp/src/tools/get-compile-audit.ts`（MCP ツール `observe` / `get_compile_audit` の zod スキーマ定義・CLI 転送・stdout パース・`_meta` 付与）
- **公開 API surface（Rust）**: 本 DD §3（`legixy-feedback` の crate 公開関数）
- **関連 SEQD**: SEQD-LGX-008

## 2. 型定義

### 2.1 主要データ型（Rust）

```rust
// ─── legixy-db（共有、ADR-LGX-020 参照） ─────────────────────────
// engine.db 接続（WAL + busy_timeout=5000 + foreign_keys=ON）は legixy-db が担う。
// legixy-feedback は接続済み rusqlite::Connection を受取る（所有権またはリファレンス）。

// ─── legixy-feedback ─────────────────────────────────────────────

/// 新規記録前の Observation 入力値。
/// source / severity は FB 内部で決定するため呼出し側（CLI / AutoObserver）が設定する。
pub struct NewObservation {
    pub source: String,       // "manual" | "auto:{category}" | "drift:contextual_retrieval"
    pub category: String,     // SPEC-007 REQ.01 凍結 3 値 (CLI/MCP 入口) or feedback 生成カテゴリ
    pub severity: String,     // "error" | "warning" | "info"
    pub message: String,      // trim 後 1 文字以上（REQ.01 GAP-LGX-121）
    pub related_ids: Vec<String>, // 正準化は record() 内部で実施（distinct→昇順 sort→JSON）
    pub context_json: Option<String>, // --target-file / --missing-doc / --source-glob 由来
}

/// engine.db に永続化済みの Observation。
pub struct Observation {
    pub id: i64,
    pub source: String,
    pub category: String,
    pub severity: String,
    pub message: String,
    pub related_ids: Vec<String>, // DB の related_ids TEXT（正準化 JSON）を Vec に逆変換
    pub context_json: Option<String>,
    pub status: ObservationStatus,
    pub created_at: String,
}

/// engine.db に永続化済みの Proposal。
/// 列名は v3 実測（decided_at / decided_reason）を踏襲（SUPP-LGX-007 §2-2 [補完]）。
pub struct Proposal {
    pub id: i64,
    pub observation_id: i64,
    pub kind: String,         // "add_chain_entry" | "add_link" | "update_doc"
    pub semantic_key: String, // REQ.09 の 3 形式で生成
    pub title: String,        // "{kind}: {message}"
    pub description: String,  // message そのまま
    pub action_json: String,  // JSON 文字列
    pub status: ProposalStatus,
    pub decided_at: Option<String>,    // approve/reject 時の datetime('now')
    pub decided_reason: Option<String>, // reject 理由（approve 時は NULL）
    pub created_at: String,
}

/// RecordResult: ObservationRecorder::record の戻り値。
pub struct RecordResult {
    pub id: i64,
    pub skipped: bool, // true = 既存 pending/analyzing との dedup で INSERT スキップ
}

/// FeedbackReport: run_feedback の実行結果サマリ。
/// stdout 契約: `feedback: {observations_created} created, {observations_skipped} skipped`
pub struct FeedbackReport {
    pub observations_created: usize,
    pub observations_skipped: usize,
}

/// ProposalSummary: proposals 一覧表示用（FeedbackCli::list_proposals の戻り）。
pub struct ProposalSummary {
    pub id: i64,
    pub kind: String,
    pub semantic_key: String,
    pub title: String,
    pub status: ProposalStatus,
    pub created_at: String,
}

/// ContextLogEntry: context_log テーブルの 1 エントリ（読取専用、書込は legixy-ctx 担当）。
pub struct ContextLogEntry {
    pub id: i64,
    pub target_id: String,
    pub granularity: Option<String>,
    pub payload: String, // CTX が書き込んだ JSON 文字列（FB は raw で返す）
    pub created_at: String,
}
```

### 2.2 列挙 / Sum 型（Rust）

```rust
// ─── legixy-feedback ─────────────────────────────────────────────

/// Observation の永続化状態（SPEC-LGX-007 REQ.08 v0.6.0 の 4 値モデル）。
///
/// 遷移グラフ:
///   observe/feedback → Pending
///   analyze 取込    → Analyzing（Pessimistic Claim）
///   Proposal 生成成功 or semantic_key 重複 → Resolved（対応 proposal approve 後）
///   analyze 失敗（Claim Release）   → Pending（再分析対象に戻る）
///   変換規則なし（orphan_file / semantic_similarity / observe 3 値）→ Skipped（終端）
///
/// Resolved / Skipped は終端・不可逆（REQ.08）。
/// Skipped は観察カテゴリを Proposal に変換できない場合の終端であり、
/// 永久再 claim（pending↔analyzing 無限ループ）を防ぐ（SPEC-LGX-007 REQ.08 v0.6.0 追加）。
///
/// 【v3 差分】v3 の状態値は Pending/Analyzing/Proposed/Skipped であり、
///   analyze 完了時に proposed/skipped へ即遷移し approve が observation に波及しない。
///   legixy は SPEC REQ.08 を正準とし、approve により observation → Resolved へ連動させる。
///   ただし approve トランザクション内で observation_id から observation を UPDATE する。
pub enum ObservationStatus {
    Pending,
    Analyzing,
    Resolved,
    Skipped, // 終端（変換規則なし category）
}

impl ObservationStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Pending   => "pending",
            Self::Analyzing => "analyzing",
            Self::Resolved  => "resolved",
            Self::Skipped   => "skipped",
        }
    }
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "pending"   => Some(Self::Pending),
            "analyzing" => Some(Self::Analyzing),
            "resolved"  => Some(Self::Resolved),
            "skipped"   => Some(Self::Skipped),
            _ => None,
        }
    }
}

/// Proposal の永続化状態（SPEC-LGX-007 REQ.09 の 3 値モデル）。
///
/// 遷移グラフ（HR7 凍結）:
///   (無) → Pending → { Approved | Rejected }
/// Approved / Rejected は終端・不可逆（REQ.09）。
/// 終端状態への再 approve/reject は CAS 更新行数 0 → exit 1（REQ.09・FB-INV-2）。
///
/// 【v3 差分】v3 enum に Skipped が存在するが DB 行として作られる経路はない。
///   legixy は 3 値で実装し Skipped variant を持たない（SUPP-LGX-007 §2-2）。
pub enum ProposalStatus {
    Pending,
    Approved,
    Rejected,
}

impl ProposalStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Pending  => "pending",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
        }
    }
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "pending"  => Some(Self::Pending),
            "approved" => Some(Self::Approved),
            "rejected" => Some(Self::Rejected),
            _ => None,
        }
    }
}

/// ObserveCategoryInput: CLI / MCP 入口の category 3 値（REQ.01 凍結）。
/// CLI 層は clap ValueEnum 相当で強制。不正値は exit 2（【v3 差分】v3 は String 無検証）。
pub enum ObserveCategoryInput {
    CompileMiss,        // "compile_miss"
    ReviewCorrection,   // "review_correction"
    ManualNote,         // "manual_note"
}

/// FeedbackCategory: feedback コマンドが AutoObserver を通じて生成する category 値。
/// REQ.01 の凍結 3 値とは別の集合（観察事項: 混同禁止、SUPP-LGX-007 §2-5 注意）。
pub enum FeedbackCategory {
    ChainIntegrity,     // "chain_integrity"
    LinkCandidate,      // "link_candidate"
    Drift,              // "drift"
    OrphanFile,         // "orphan_file"
    SemanticSimilarity, // "semantic_similarity"
}
```

### 2.3 エラー型（Rust）

```rust
// legixy-feedback（thiserror 使用）
#[derive(Debug, thiserror::Error)]
pub enum FeedbackError {
    #[error("database error: {0}")]
    Db(#[from] rusqlite::Error),

    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    /// Proposal が存在しない（approve/reject 対象 id 不在 → exit 1）
    #[error("proposal {id} not found")]
    ProposalNotFound { id: i64 },

    /// Proposal が終端状態（approved/rejected）への再操作（CAS 失敗 → exit 1）
    #[error("proposal {id} expected status {expected:?}, found {actual:?}")]
    InvalidProposalStatus {
        id: i64,
        expected: &'static str,
        actual: String,
    },

    /// reject の --reason が trim 後 0 文字（GAP-LGX-124 → exit 1）
    /// 【v3 差分】v3 は is_empty() のみ。legixy は trim().is_empty() で拒否。
    #[error("reject reason must not be empty")]
    EmptyRejectReason,

    /// observe の message が trim 後 0 文字（GAP-LGX-121 → exit 1）
    /// 【v3 差分】v3 にこの検証は存在しない。legixy で新規実装。
    #[error("observation message must not be empty")]
    EmptyObservationMessage,

    /// analyze 中に単一 Observation の処理が失敗（Claim Release で pending に戻す）
    #[error("analyze failed for observation {observation_id}: {detail}")]
    AnalyzeFailed { observation_id: i64, detail: String },

    /// engine.db が破損（不在とは区別、REQ.09 GAP-LGX-126 → exit 1）
    /// 検出契機: ファイル存在 + PRAGMA integrity_check 失敗（option C: SQLITE_CORRUPT 捕捉）
    /// 不在（正常）= CREATE TABLE で新規作成。破損 = exit 1 で明示的失敗、自動再生成禁止。
    #[error("engine.db is corrupted; restore from backup or remove to reinitialize: {detail}")]
    DbCorrupted { detail: String },
}
```

終了コード規約（LGX-COMPAT-001 §3）:
- `FeedbackError::Db` / `DbCorrupted` / `ProposalNotFound` / `InvalidProposalStatus` / `EmptyRejectReason` / `EmptyObservationMessage` / `AnalyzeFailed` → exit 1
- category 不正値（CLI 層の clap ValueEnum 違反）→ exit 2
- CAS 敗者（approve/reject 競合で行数 0）→ `InvalidProposalStatus` → exit 1

### 2.4 TypeScript 型（ts-mcp）

```typescript
// ts-mcp/src/types.ts（既存型に追加 or 参照）

/** observe tool の CLI stdout パース結果。
 *  CLI stdout 契約: "observation: id=<N> skipped=<true|false>" */
export type ObserveStdoutParsed = { id: number; skipped: boolean };

/** get_compile_audit が Rust CLI から受け取る JSON 配列の要素。
 *  フィールドは Rust ContextLogEntry（id / target_id / granularity / payload / created_at）に対応。
 *  【v3 差分・バグ修正】v3 の formatAuditEntry は e.input_files / e.input_command を参照するが
 *  Rust 出力に該当フィールドは存在しない。legixy では payload JSON を parse して
 *  target_files / command を取り出す（SUPP-LGX-007 §2-9 [要決定]→ 方針(a) 採用）。 */
export type ContextLogEntry = {
  id: number;
  target_id: string;
  granularity: string | null;
  payload: string; // JSON 文字列（CTX が書き込んだ形式）
  created_at: string;
};

/** get_compile_audit の payload JSON 内部構造（Rust CTX audit_logger の書込形式）。 */
export type AuditPayload = {
  command?: string;
  target_files?: string[];
  targets?: string[];
  granularity?: string;
  upstream_count?: number;
  layer_count?: number;
  additional_count?: number;
  custom_count?: number;
  unresolved_count?: number;
};
```

## 3. 公開 API surface（凍結、HR7）

### 3.1 Rust crate `legixy-feedback`

| 関数 / 型 | シグネチャ | 不変条件 | 冪等性 | 同期/非同期 |
|---|---|---|---|---|
| `ObservationRecorder::record` | `fn record(obs: &NewObservation, db: &Connection) -> Result<RecordResult, FeedbackError>` | related_ids を distinct→昇順 sort→JSON 化して DB の dedup キーとする。同一 (category, related_ids_json) かつ status IN ('pending','analyzing') が存在すれば INSERT スキップ（FB-INV-1/REQ.11）。並行 INSERT 競合は UNIQUE 制約違反→SELECT fallback で吸収（REQ.11/MCP-INV-3） | yes（同一 obs を複数回呼んでも最初の 1 件のみ INSERT） | 同期 |
| `AutoObserver::from_check_results` | `fn from_check_results(report: &CheckReport) -> Vec<NewObservation>` | フィルタ規則（severity=Ok 除外 / FileExistence×Error 除外 / DocumentId×Warning 除外 / 既知 5 カテゴリのみ）に従い NewObservation 列を生成。message には mask_api_key を適用（NFR SEC.05）。read-only | yes | 同期 |
| `drift_from_embed_error` | `fn drift_from_embed_error(err: &EmbedError, node_id: &str) -> Option<NewObservation>` | EmbedError::ContextualRetrievalFailed の場合のみ drift カテゴリの NewObservation を返す。message は必ず mask_api_key を通す | yes | 同期 |
| `ProposalAnalyzer::analyze` | `fn analyze(db: &Connection) -> Result<Vec<Proposal>, FeedbackError>` | Pessimistic Claim（single tx で pending→analyzing）→ カテゴリ別変換 → semantic_key dedup → INSERT → observation 状態更新（resolved/skipped/pending Claim Release）。戻り値は新規 INSERT した Proposal のみ（重複排除スキップ分を含まない）。FB-INV-5 維持 | no（実行ごとに行数変化） | 同期 |
| `ProposalManager::approve` | `fn approve(proposal_id: i64, db: &Connection) -> Result<(), FeedbackError>` | 単一 tx: CAS `UPDATE proposals SET status='approved' WHERE id=? AND status='pending'`（更新行数判定）→ `kind == "add_custom_edge"` の場合のみ custom_edges に INSERT → approve tx 内で対応 observation を resolved へ UPDATE（FB-INV-2）。終端状態への再操作は `InvalidProposalStatus` → exit 1 | no | 同期 |
| `ProposalManager::reject` | `fn reject(proposal_id: i64, reason: &str, db: &Connection) -> Result<(), FeedbackError>` | reason trim 後 0 文字は `EmptyRejectReason` → exit 1（GAP-LGX-124。【v3 差分】v3 は is_empty() のみ）。CAS `UPDATE proposals SET status='rejected' WHERE id=? AND status='pending'`（行数 0 → `InvalidProposalStatus`）。observation 状態は pending に戻す（SPEC REQ.08 "reject または一時的失敗 → pending"） | no | 同期 |
| `FeedbackCli::run_feedback` | `fn run_feedback(db: &Connection, check_report: &CheckReport) -> Result<FeedbackReport, FeedbackError>` | AutoObserver でフィルタ → 各 NewObservation を record → FeedbackReport 集計。stdout 出力は legixy-cli 層が担う | no（check_report により変化） | 同期 |
| `FeedbackCli::run_analyze` | `fn run_analyze(db: &Connection) -> Result<Vec<Proposal>, FeedbackError>` | ProposalAnalyzer::analyze の薄いラッパ | no | 同期 |
| `FeedbackCli::list_proposals` | `fn list_proposals(db: &Connection, status_filter: Option<ProposalStatus>) -> Result<Vec<ProposalSummary>, FeedbackError>` | status_filter が None の場合は全件（`--status` 省略時相当）。ORDER BY id | yes（read-only） | 同期 |
| `FeedbackCli::approve` | `fn approve(db: &Connection, proposal_id: i64) -> Result<(), FeedbackError>` | ProposalManager::approve の薄いラッパ | no | 同期 |
| `FeedbackCli::reject` | `fn reject(db: &Connection, proposal_id: i64, reason: &str) -> Result<(), FeedbackError>` | ProposalManager::reject の薄いラッパ | no | 同期 |
| `ContextAuditReader::recent` | `fn recent(db: &Connection, limit: usize) -> Result<Vec<ContextLogEntry>, FeedbackError>` | context_log を id DESC LIMIT limit で返す。limit 範囲は CLI 層（1..=50）で強制 | yes（read-only） | 同期 |
| `ContextAuditReader::by_target` | `fn by_target(db: &Connection, target_id: &str, limit: usize) -> Result<Vec<ContextLogEntry>, FeedbackError>` | target_id フィルタ + id DESC LIMIT limit | yes（read-only） | 同期 |

**CAS 実装詳細（FB-INV-2 / SUPP-LGX-007 §2-7）:**

approve / reject では fetch-then-update ではなく `UPDATE ... WHERE status='pending'` の更新行数判定（CAS）を採用する:

```
1. UPDATE proposals SET status='{new}', decided_at=datetime('now'), decided_reason=? WHERE id=? AND status='pending'
2. updated_rows == 1 → 成立
3. updated_rows == 0 → ProposalNotFound または InvalidProposalStatus を判別するため SELECT で確認:
   - 行不在      → ProposalNotFound { id }
   - 行あり status != 'pending' → InvalidProposalStatus { id, expected:"pending", actual: <現在値> }
```

【v3 差分】v3 の reject は tx 外で fetch-then-update（非 CAS）。legixy は SPEC REQ.09 に従い CAS で実装する。

### 3.2 TypeScript MCP 転送層

| ツール | zod 入力スキーマ | CLI 変換 | 注意点 |
|---|---|---|---|
| `observe` | `category: z.enum(["compile_miss","review_correction","manual_note"])`, `message: z.string().min(1).trim()`, `related_ids?: z.array(z.string()).optional()`, `target_files?: z.array(z.string()).optional()`, `missing_doc?: z.string().optional()`, `source_glob?: z.string().optional()` | `observe <category> <message> [--related-id ...]` | stdout `"observation: id=N skipped=true\|false"` を parseObserveStdout で parse。`_meta["legixy/warnings"]` は exit 0 + stderr 非空の場合のみ付与（SPEC-LGX-009 REQ.03/13） |
| `get_compile_audit` | `limit?: z.number().int().min(1).max(50).optional()` | `audit [--limit N]` | stdout JSON 配列を parse → formatAuditEntry で Markdown 化（payload JSON から target_files/command を取り出す。【v3 差分】v3 は input_files/input_command 参照でフィールド不整合あり）。`_meta["anthropic/maxResultSizeChars"]=500000` を付与（SPEC-LGX-009 REQ.13） |

**observe の stdout 正規表現パターン（互換上重要、MCP-INV-2）:**
```
/^observation:\s*id=(\d+)\s+skipped=(true|false)/
```
形式変更は MCP 転送層のパースを壊すため凍結（LGX-COMPAT-001 §4.1）。

**formatAuditEntry（legixy 修正版）:**
```typescript
export function formatAuditEntry(e: ContextLogEntry): string {
  let payload: AuditPayload = {};
  try { payload = JSON.parse(e.payload) as AuditPayload; } catch { /* ignore */ }
  const files = payload.target_files ?? [];
  const command = payload.command ?? "(none)";
  return (
    `### #${e.id} (${e.created_at})\n` +
    `- Target: ${e.target_id || "(unresolved)"}\n` +
    `- Files: ${files.length > 0 ? files.join(", ") : "(none)"}\n` +
    `- Command: ${command}`
  );
}
```

## 4. module / package 構成

### 4.1 Rust crate `legixy-feedback`

```
legixy-feedback/
├── src/
│   ├── lib.rs          // Document ID: SRC-LGX-008（pub use・extern crate self・TC include）
│   ├── observer.rs     // ObservationStatus / Observation / NewObservation / AutoObserver / drift_from_embed_error
│   ├── recorder.rs     // ObservationRecorder（dedup INSERT・UNIQUE 制約 fallback・distinct+sort 正準化）
│   ├── analyzer.rs     // ProposalStatus / Proposal / ClaimedObservation / ProposalAnalyzer
│   │                   //   Pessimistic Claim / カテゴリ別変換 / semantic_key / observation→Resolved or Skipped
│   ├── manager.rs      // ProposalManager（approve CAS + observation resolved 連動 / reject CAS）
│   ├── audit.rs        // ContextAuditReader / ContextLogEntry（読取専用）
│   ├── cli.rs          // FeedbackCli ファサード / FeedbackReport / ProposalSummary
│   └── error.rs        // FeedbackError（thiserror）
└── Cargo.toml
```

依存方向（DAG、ADR-LGX-020）:
```
legixy-feedback → legixy-db    (Connection / schema 共有)
              → legixy-check (CheckReport / CheckResult / CheckCategory / Severity)
              → legixy-core  (共通エラー)
              → legixy-embed (EmbedError / mask_api_key)
legixy-cli → legixy-feedback  (FeedbackCli ファサード経由で CLI サブコマンドに組込む)
```

循環なし。legixy-feedback 自体は bin を含まず、legixy-cli が統合バイナリとして組み込む。

### 4.2 TypeScript MCP（ts-mcp）

```
ts-mcp/
├── src/
│   ├── server.ts          // MCP サーバ起動・ツール登録（observe / get_compile_audit を追加）
│   ├── engine.ts          // RustEngine / RustCliError（runText / タイムアウト管理 REQ.16）
│   ├── types.ts           // ObserveStdoutParsed / ContextLogEntry / AuditPayload
│   └── tools/
│       ├── observe.ts     // registerObserve / parseObserveStdout / formatObserveResult
│       └── get-compile-audit.ts  // registerGetCompileAudit / formatAuditEntry（payload parse 版）
└── tests/
    └── tools/
        ├── observe.test.ts         // Document ID: TC-LGX-009（TS MCP）
        └── get-compile-audit.test.ts
```

## 5. ライフタイム / 所有権 / 借用 方針

**Rust:**

- `ObservationRecorder::record` / `ProposalAnalyzer::analyze` / `ProposalManager::approve` / `ProposalManager::reject` / `ContextAuditReader::recent` / `by_target` はすべて `&Connection` を**借用**（所有権を取らない）。呼出し側（legixy-cli）が Connection を生成・保持し、各 FB 関数はスコープ内で借用する。
- `AutoObserver::from_check_results` は `&CheckReport` を借用し `Vec<NewObservation>` を所有で返す。
- `ProposalAnalyzer::analyze` 内の `ClaimedObservation` は内部一時型（`Vec` で所有）。外部公開しない。
- `Proposal` / `Observation` / `RecordResult` / `FeedbackReport` は所有権を持って返す（呼出し側の出力制御用）。
- `Arc`/`Mutex` 不要（単一スレッド逐次 §7。DB レベルの WAL + busy_timeout で並行制御）。

**TypeScript:**

- MCP ツールハンドラは `async` 関数、RustEngine は注入された依存として `readonly` で保持。
- ContextLogEntry / AuditPayload は `Readonly<T>` で定義し mutation を防ぐ。
- `any` 禁止。zod parse 後の型は discriminated union で扱う。

## 6. エラー伝播戦略

### 6.1 Rust

- crate 内: 各 module は失敗を `Result<_, FeedbackError>` で返す。`unwrap` / `expect` は禁止（rust.md §4）。
- 公開境界 (`legixy-cli`) への伝播: `FeedbackCli` の各関数が `FeedbackError` を返し、legixy-cli 層が `exit 1` / `exit 2` に変換（LGX-COMPAT-001 §3）。
- DB 破損判別（REQ.09 / SUPP-LGX-007 §2-11 方針 C 採用）: `Connection::open` は破損ファイルでも成功する場合があるため、書込系コマンド（observe / feedback / analyze / approve / reject）の初回 DB 操作で `SQLITE_CORRUPT` / `NotADatabase` エラーが返った場合を破損として `DbCorrupted` に変換し exit 1。読取系（proposals / audit）は同様に操作中のエラーで破損判別。コスト最小の遅延検出（毎回 `PRAGMA integrity_check` は避ける）。
- 不在と破損の判別: `Path::exists()` == false → 不在（新規作成して続行）、ファイル存在 + 操作エラー → 破損（exit 1・自動再生成禁止）。
- 部分成功（feedback コマンド）: 各 Observation の record 失敗は FeedbackError を上位に返す（FeedbackCli::run_feedback は 1 件目のエラーで返却。エラー集約の詳細は legixy-cli 層の責務）。
- stdout/stderr 分離: FeedbackReport の出力・WARNING メッセージは legixy-cli 層が stdout/stderr へ出力（legixy-feedback 自体は println! を持たない）。

### 6.2 TypeScript

- RustEngine の runText は非ゼロ exit で `RustCliError` を throw（engine.ts）。
- ツールハンドラは `try/catch` で `RustCliError` を捕捉し `isError: true` 応答を返す（throw を MCP 境界に漏らさない。typescript.md §3 Result 型方針）。
- `parseObserveStdout` は形式不一致時に `Error` を throw（呼出し側でキャッチして isError 応答）。
- AuditPayload の JSON parse 失敗は `isError: true` 応答として返す（v3 実測踏襲）。

## 7. 並行性 / 非同期境界

### 7.1 Rust（同期・単一スレッド）

- legixy-feedback の全関数は**同期・単一スレッド**。async なし。`tokio` / `rayon` は不要。
- 並行安全性は DB レベルで担保する（PRAGMA WAL + busy_timeout=5000）:
  - **observe の並行呼出し（REQ.11 / MCP-INV-3）**: UNIQUE INDEX `idx_obs_dedup` により並行 INSERT の片方が CONSTRAINT 違反 → SELECT-then-skip fallback（recorder.rs §2.1 参照）。busy_timeout=5000 ms でロック競合を吸収（NFR REL.07）。
  - **approve/reject の競合（FB-INV-2）**: CAS UPDATE（§3.1 CAS 実装詳細）。同時操作の敗者は updated_rows=0 → `InvalidProposalStatus` → exit 1。
- distinct 化（GAP-LGX-122 追加分）: record() 内部で sort 前に `dedup()` を適用して重複を除去する。v3 の recorder.rs は sort のみ（distinct なし）であり、legixy では distinct→sort→JSON の順で正準化する（【v3 差分】SPEC REQ.11 正準定義）。

### 7.2 TypeScript（非同期・Promise）

- ツールハンドラは `async` 関数で RustEngine.runText（子プロセス起動）を `await`。
- タイムアウト: RustEngine が SPEC-LGX-009 REQ.16 のタイムアウト（既定 30 秒）を実装。ツールハンドラは engine 層に委任。
- Node.js 単一スレッド上で並行リクエストは Promise 単位で interleave するが、Rust CLI は child process として分離されるためスレッドセーフ問題は生じない。

## 8. テスト分類

| 分類 | 内容 | 対応 TP |
|---|---|---|
| Unit (Rust) | ObservationRecorder::record の dedup（distinct/sort/JSON 正準化・UNIQUE fallback）、ProposalAnalyzer の Pessimistic Claim・skipped 終端・semantic_key 生成、ProposalManager の CAS（approve/reject 競合・EmptyRejectReason・終端再操作）、ObserveCategoryInput 検証（不正 → exit 2）、message/reason trim 空白拒否（exit 1） | TP-LGX-007, TP-LGX-018 |
| Integration (Rust) | feedback コマンド E2E（CheckReport → Observation 生成・skipped カウント）、analyze E2E（pending → Proposal 生成・Observation Resolved/Skipped）、approve/reject → CAS 確認・Observation resolved 連動、proposals フィルタ、audit limit 境界（1/10/50）、DB 破損時 exit 1 fixture、engine.db 不在の新規作成（正常） | TP-LGX-007, TP-LGX-018 |
| Property-based (Rust) | related_ids の distinct→sort 正準化の決定性（同一内容なら同一 JSON）、重複 observe の dedup 冪等性（N 回呼出しで 1 件のみ INSERT）。proptest 使用 | TP-LGX-007 C1 |
| Concurrent (Rust) | 並行 observe ストレステスト（同一 observation を複数 thread から同時送信 → 1 件のみ格納）、並行 approve/reject（CAS 成立 1 件・敗者 exit 1） | TP-LGX-007 C2 |
| Unit (TypeScript) | parseObserveStdout（正常・形式不正）、formatAuditEntry（payload parse 成功・失敗・空 entries）、zod スキーマ境界（category 不正値 reject・message min(1)・limit 0 / 負値 / 小数 reject）、exit 0 + stderr 非空 → `_meta["legixy/warnings"]` 付与、stderr 空 → フィールド省略 | TP-LGX-018（UC 観点）|
| Integration (TypeScript) | mock RustEngine での CLI argv 検証（observe の位置引数・フラグ変換、audit --limit 転送）、isError 応答（exit 1 / exit 2 / タイムアウト）の形式確認 | TP-LGX-018 |

## 9. ADR への参照

- **ADR-LGX-020**: crate 分割・共有型凍結（本 DD の crate 境界）
- ADR-LGX-004: 可観測性強化（exit 0 + stderr 非空を `_meta["legixy/warnings"]` で転送する判断、SPEC-LGX-009 REQ.03）
- ADR-LGX-005: engine.db 破損時保護（observation/proposal を再生成不能データとして保護）
- ADR-LGX-006: 承認系 CLI「人間のみ」二層強制（MCP 非露出 + CLAUDE.md ルール 5）
- ADR-LGX-010: MCP 子プロセスタイムアウト（SPEC-LGX-009 REQ.16）
- ADR-LGX-014: SPEC 準拠原則
- ADR-LGX-015: DB パス（engine.db 配置。ディレクトリ名矛盾 SUPP §2-4 [要決定] は人間裁定待ち）
- ADR-LGX-016: env 解決（バイナリパス・LGX_MCP_TIMEOUT_SEC）
- ADR-LGX-019: skipped 終端追加（SPEC-LGX-007 REQ.08 v0.6.0 / TRIAGE §4 #13）

## 10. 関連 NFR

- NFR-LGX-001.REL.07: SQLite busy_timeout=5000 ms（並行 observe の内部リトライ上限）
- NFR-LGX-001.REL.08: engine.db のネットワーク共有配置禁止
- NFR-LGX-001.SEC.02: 並行書き込み安全性（WAL + UNIQUE INDEX）
- NFR-LGX-001.SEC.04: DoS 防御（message 最大長なし。一般則）
- NFR-LGX-001.SEC.05: クレデンシャルマスキング（drift Observation の message に mask_api_key 適用必須）
- NFR-LGX-001.SEC.08: 単独開発者前提（approve/reject の改ざん耐性ガード要件なし）
- NFR-LGX-001.MAINT.05: テストコード不可侵（proposal が「テスト修正」提案を生成しても実施は人間 s1）
- NFR-LGX-001.OBS.01: フィードバックループ可観測性
- NFR-LGX-001.PERF.03: MCP レイテンシ（engine.ts の RustEngine が一次的に担保）

## 11. 凍結履歴

| Date | Version | Note |
|---|---|---|
| 2026-06-13 | 1.0 | 初版凍結。`legixy-feedback` の全公開型（ObservationStatus 4 値 / ProposalStatus 3 値 / Observation / NewObservation / Proposal / ContextLogEntry / FeedbackError 8 variant）と公開 API（record / analyze / approve / reject / list_proposals / recent / by_target / FeedbackCli ファサード群）、TypeScript 転送層（observe / get_compile_audit）を確定。v3 実測（lx-feedback crate）に整合し意図的差分（distinct+sort 正準化 / CAS 化 / ObservationStatus Resolved 終端 / Skipped 終端追加 / message・reason trim 空拒否 / formatAuditEntry payload parse 修正）を明示。crate 境界は ADR-LGX-020。DB 破損判別はオプション C（操作中の SQLITE_CORRUPT 捕捉）。HR7 凍結 |
