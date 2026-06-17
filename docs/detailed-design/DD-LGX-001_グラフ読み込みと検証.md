Document ID: DD-LGX-001

# DD-LGX-001: グラフ読み込みと検証（check）の詳細設計

**親 SEQD**: SEQD-LGX-001
**親 RBD**: RBD-LGX-001 / **親 UC**: UC-LGX-001
**対象言語**: Rust（CLI 本体）
**境界 API 凍結**: yes（本ドキュメント確定後、crate 間公開 API は変更不可。追加は許容、削除・改名・型変更は次版 SPEC 改訂。HR7）

> 言語別規律は `guides/language-stacks/rust.md`。crate 境界・共有型は **ADR-LGX-020** で凍結済（本 DD は参照する）。型は v3 実装（traceability-engine.v3 `crates/te-check`）に整合させ引数互換を保つ。

## 1. 対象範囲

- **主 crate**: `legixy-check`（検証カテゴリ・severity・finding 生成・CheckReport 集約）
- **依存 crate（共有型は ADR-LGX-020、再定義しない）**: `legixy-graph`（`TraceGraph` / `Node` / `Edge`）, `legixy-db`（`EmbeddingStore` — 意味検証用、Option）, `legixy-core`（`Id` / 共通エラー / `Severity` 基底）, 設定は `legixy-cli`/`legixy-core` の `Config`
- **公開 API surface**: 本 DD §3（`legixy-check` の crate 公開関数）
- **関連 SEQD**: SEQD-LGX-001

## 2. 型定義

### 2.1 主要データ型

```rust
// legixy-core（共有、ADR-LGX-020）
pub struct Id(String);              // {type}-{area}-{seq} or {id}#{subnode_hash}

// legixy-check
pub struct CheckResult {
    pub severity: Severity,
    pub category: CheckCategory,
    pub message: String,
    pub related_ids: Vec<Id>,
    pub location: Option<Location>, // file path + line（finding の発生位置）
}

pub struct CheckReport {
    pub findings: Vec<CheckResult>, // 安定ソート済（REQ.06: severity 降順 → category → related_ids）
    pub counts: SeverityCounts,     // { error, warning, info, ok }
}

pub struct Location { pub path: PathBuf, pub line: Option<usize> }
pub struct SeverityCounts { pub error: usize, pub warning: usize, pub info: usize, pub ok: usize }
```

### 2.2 列挙 / Sum 型

```rust
// legixy-core（共有）
pub enum Severity { Ok, Info, Warning, Error } // REQ.03。Ok は予約（finding 非発行、REQ.03/GAP-065）

// legixy-check（SPEC-004 REQ.15 割当表に 1:1。新カテゴリ追加は SPEC 改訂）
pub enum CheckCategory {
    FileExistence, DocumentId, ChainIntegrity, OrphanFile, GraphDag, Freshness,
    SubnodeIdFormat, SubnodeIdUniqueness, SubnodeParentIntegrity,
    SubnodePathConsistency, SubnodeDag, SubnodeIdCollision,
    UnresolvedEdge, IdRedefined, IdSemanticMismatch, IdSemanticDrift,
    SemanticSimilarity, // 全層 check のみ（--formal では発行しない）
}

pub enum CheckMode { Formal, Full } // --formal=Formal（形式層のみ）、無印=Full（意味層追加）
```

各 CheckCategory の severity は REQ.15 割当表で固定（不可能な severity 組合せは生成側で出さない）。config 由来助言（`{id}` 誤記 Warning 等）は検証 finding と分離（REQ.15 注記、ADR-LGX-019）。

### 2.3 エラー型

```rust
// legixy-check（実行時失敗 = exit 1。検証 finding とは別概念）
pub enum CheckError {
    GraphLoad(legixy_graph::GraphError), // graph.toml 破損・パース不能 → exit 1（REQ.04, TP-004 E3）
    ConfigLoad(legixy_core::ConfigError), // .legixy.toml 不在/破損 → exit 1
    Db(legixy_db::DbError),               // engine.db open 失敗等（意味層）
}
```

- 終了コードは `CheckError` → exit 1、引数構文誤り（clap）→ exit 2、`CheckReport.counts.error > 0` → exit 1、それ以外 → exit 0（REQ.04, LGX-COMPAT-001 §3）。
- 部分失敗継続（REQ.05）: 一部成果物ファイル読込失敗は `CheckError` に昇格させず `CheckResult{ severity: Error, category: FileExistence }` として `findings` に記録し他検査を継続。

## 3. 公開 API surface（凍結、HR7）

| 関数 | シグネチャ | 不変条件 | 冪等性 | 同期/非同期 |
|---|---|---|---|---|
| `legixy_check::run` | `fn run(graph: &TraceGraph, config: &Config, mode: CheckMode, store: Option<&EmbeddingStore>) -> Result<CheckReport, CheckError>` | 同一入力 → 同一 CheckReport（findings 順序含む、REQ.06）。read-only（graph/db を変更しない） | yes | 同期 |
| `legixy_check::exit_code` | `fn exit_code(report: &CheckReport) -> i32` | `report.counts.error > 0` ⇒ 1、else 0（REQ.04） | yes | 同期 |
| `legixy_check::CheckReport::to_json` | `fn to_json(&self) -> String` | JSON Lines（`--log-format=json`/`--json`、REQ.08。OBS.02 stdout） | yes | 同期 |

- `store` が `None`（embeddings 空・engine.db 不在）でも形式層は完走し、意味層は Info 1 件（embed 誘導）で非致命（REQ.02, FB-INV-4）。
- 引数の意味は LGX-COMPAT-001 §4 #3（`check [--formal]`）に凍結済。新規 finding カテゴリ追加は本表の API を破らない（加算的）。

## 4. module / package 構成

```
legixy-check/
├── src/
│   ├── lib.rs            // Document ID: SRC-LGX-001（run / exit_code / 再エクスポート）
│   ├── report.rs         // CheckReport / CheckResult / SeverityCounts / 安定ソート（REQ.06）
│   ├── category.rs       // CheckCategory / Severity 割当（REQ.15 表）
│   ├── formal/           // 形式層検査器（カテゴリ単位）
│   │   ├── file_existence.rs, document_id.rs, chain_integrity.rs,
│   │   ├── orphan_file.rs, graph_dag.rs, freshness.rs
│   │   ├── subnode.rs     // SubnodeId* / SubnodeDag / SubnodeIdCollision
│   │   └── id_checks.rs   // IdRedefined / IdSemanticMismatch（config opt-in）
│   ├── semantic/         // 意味層（store 必須。IdSemanticDrift / SemanticSimilarity）
│   └── error.rs          // CheckError
└── Cargo.toml
```

依存方向（DAG、ADR-LGX-020）: `legixy-check` → `legixy-graph` / `legixy-db` / `legixy-core`。循環なし。検査器は `formal/` 各 module が `&TraceGraph` を読み `Vec<CheckResult>` を返す純関数とし、`run` が集約する（部分失敗継続の合流点）。

## 5. ライフタイム / 所有権 / 借用 方針

- `run` は `&TraceGraph` / `&Config` / `Option<&EmbeddingStore>` を **借用**（所有権を取らない。read-only 判定、複数 check 同時実行で共有可）。
- `CheckReport` は所有を返す（呼び出し側が出力・exit 判定に使う）。
- `Id` は `legixy-core` 所有。検査器間で `&Id` 参照を回す。`'static` バウンド不要（呼び出しスコープ内で完結）。
- `Arc`/`Mutex` 不要（単一スレッド逐次。§7）。

## 6. エラー伝播戦略

- 内部: 各検査器は失敗を `CheckResult{ Error }` として返し panic しない。crate 公開境界の `run` は graph/config/db のロード失敗のみ `Err(CheckError)` を返す（実行時失敗 = exit 1）。
- 部分成功: ファイル読込失敗は finding 化して継続（REQ.05、ロールバック不要 = read-only）。
- panic 禁止: 検査器内の `unwrap`/`expect` を禁止（rust.md §4）。graph.toml パース失敗は `legixy-graph` が `Result` で返し `CheckError::GraphLoad` に変換。
- ユーザ通知: CheckReport=stdout、ログ=stderr（REQ.08, OBS.02）。

## 7. 並行性 / 非同期境界

- `check` は **同期・単一スレッド・read-only**。async なし。
- 検査器の並列化は将来最適化（本 DD では逐次。NFR PERF.02 のノード1,000+エッジ2,000で<500ms は逐次で充足見込み、benches で測定）。
- 並行アクセス（外部更新）整合性は対象外（NFR REL.07/08 の射程、TP-004 C1/C2）。

## 8. テスト分類

| 分類 | 内容 | 対応 TP |
|---|---|---|
| Unit | 各検査器（カテゴリ別 finding 生成・severity）、空グラフ・単一ノード境界 | TP-LGX-004, TP-LGX-011 |
| Integration | run の集約・部分失敗継続・exit code・--formal/無印の層差 | TP-LGX-004, TP-LGX-011 |
| Property-based | CheckReport の安定ソート決定性（同一入力→同一順序、REQ.06、proptest） | TP-LGX-004 D1 |
| Bench | ノード1,000+エッジ2,000 の check 応答時間（NFR PERF.02、criterion） | NFR-LGX-001 |

## 9. ADR への参照

- **ADR-LGX-020**: crate 分割・共有型凍結（本 DD の crate 境界）
- ADR-LGX-003: embedding 決定論モデル（意味層スコアの再現性、全層 check 冪等性）
- ADR-LGX-014: SPEC 準拠原則
- ADR-LGX-019: REQ.15 config 助言の射程（検証 finding と分離）

## 10. 関連 NFR

- NFR-LGX-001.PERF.02: check 性能予算（ノード1,000+エッジ2,000で<500ms）
- NFR-LGX-001.OBS.02: 出力先（CheckReport=stdout / ログ=stderr）
- NFR-LGX-001.OBS.05: 終了コード（0/1/2）
- NFR-LGX-001.OBS.06: CheckResult severity 4 段階
- NFR-LGX-001.REL.07/08: SQLite busy_timeout / engine.db 配置（意味層）

## 11. 凍結履歴

| Date | Version | Note |
|---|---|---|
| 2026-06-13 | 1.0 | 初版凍結。`legixy-check` の CheckCategory/Severity/CheckResult/CheckReport/CheckMode/CheckError 型と run/exit_code/to_json 公開 API を確定（v3 te-check 整合）。crate 境界は ADR-LGX-020。HR7 凍結 |
