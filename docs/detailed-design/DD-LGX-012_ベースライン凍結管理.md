Document ID: DD-LGX-012

# DD-LGX-012: ベースライン凍結管理（snapshot create / list / delete）の詳細設計

**親 SEQD**: SEQD-LGX-012
**親 RBD**: RBD-LGX-012 / **親 UC**: UC-LGX-012
**対象言語**: Rust（CLI 本体）
**境界 API 凍結**: yes（本ドキュメント確定後、crate 間公開 API は変更不可。追加は許容、削除・改名・型変更は次版 SPEC 改訂。HR7）

> 言語別規律は `guides/language-stacks/rust.md`。crate 境界・共有型は **ADR-LGX-020** で凍結済（本 DD は参照する）。型は v3 実装（traceability-engine.v3.chg_to_lexigy `crates/lx-cli/src/commands/snapshot.rs` + `crates/lx-embed/src/store.rs`）に整合させ引数互換を保つ。

## 1. 対象範囲

- **主 crate**: `legixy-embed`（`EmbeddingStore`・スナップショット API: `create_snapshot` / `list_snapshots` / `delete_snapshot` / `resolve_snapshot_id_by_label`）および `legixy-db`（`embedding_snapshots` テーブルスキーマ・DB 接続・DB パス解決）
- **依存 crate（共有型は ADR-LGX-020、再定義しない）**: `legixy-core`（`Id` / 共通エラー基底）、`legixy-cli`（clap による `snapshot create / list / delete` サブコマンドのディスパッチ。`SnapshotAction` enum、グローバルオプション `--project-root` / `--json` / `--models-dir` の受理）
- **公開 API surface**: 本 DD §3（`legixy-embed` crate の snapshot 系 public 関数）
- **関連 SEQD**: SEQD-LGX-012

## 2. 型定義

### 2.1 主要データ型

```rust
// legixy-embed — スナップショット行の読取結果（list 用）
pub struct SnapshotMeta {
    pub snapshot_id: String,       // "snap-" プレフィクス付き不透明トークン（SPEC REQ.02）
    pub label: Option<String>,     // None = --label 未指定
    pub node_count: usize,         // スナップショット内のノード行数
    pub taken_at: String,          // SQLite datetime('now') = UTC "YYYY-MM-DD HH:MM:SS"（秒精度、SUPP-010 S-2）
}

// legixy-embed — create 結果の集約型（create 後の出力用）
pub struct SnapshotCreateResult {
    pub snapshot_id: String,
    pub label: Option<String>,
    pub node_count: usize,         // 0 のとき空ストア非永続（SPEC REQ.02 2a）
}

// legixy-embed — delete 結果（delete 後の出力用）
pub struct SnapshotDeleteResult {
    pub snapshot_id: String,
    pub deleted_rows: usize,       // 0 = 該当行なし（exit 0）、1+ = 削除成功
}

// legixy-db — 全コマンド共通の DB 接続（ADR-LGX-015 パス解決）
// DB パスは legixy-db 内部で ".legixy/engine.db" 正準 → ".trace-engine/engine.db" 読取フォールバック
```

> `EmbeddingRow`（`legixy-embed`）は ADR-LGX-020 より `legixy-embed` 所有の共有型。スナップショット行は `embedding_snapshots` テーブルから読む際も `EmbeddingRow` 形式に正規化して返す（`load_snapshot_embedding` は UC-LGX-013 drift 用途、本 DD 範囲外）。

### 2.2 列挙 / Sum 型

```rust
// legixy-cli（clap derive、v3 lx-cli/src/main.rs L250-263 の SnapshotAction と同型）
#[derive(Debug, clap::Subcommand)]
pub enum SnapshotAction {
    /// embeddings ストアの現行全行を凍結する
    Create {
        /// ベースラインのラベル（省略可）
        #[arg(long)]
        label: Option<String>,
    },
    /// 凍結済みベースラインを一覧表示する
    List,
    /// ベースラインを削除する（snapshot_id または label:LABEL）
    Delete {
        /// 削除対象（snapshot_id または "label:LABEL"）
        target: String,
    },
}
// SnapshotAction 未指定 = clap が exit 2 を自動付与（SPEC REQ.02 1a）

// legixy-embed — label 解決の結果
pub enum LabelResolveResult {
    Resolved(String),       // 解決済み snapshot_id
    NotFound,               // 該当 label 0 件（delete 6c = exit 1 / drift 曖昧形式 = exit 0 で分岐）
}
```

### 2.3 エラー型

```rust
// legixy-embed（snapshot 操作の実行時失敗 = exit 1）
#[derive(Debug, thiserror::Error)]
pub enum SnapshotError {
    #[error("DB エラー: {0}")]
    Db(#[from] legixy_db::DbError),                 // DB open / クエリ失敗 → exit 1

    #[error("label '{label}' に該当するスナップショットがありません")]
    LabelNotFound { label: String },                 // delete 6c → exit 1（SPEC REQ.02）

    #[error("create トランザクション失敗: {0}")]
    TransactionFailed(legixy_db::DbError),           // 単一トランザクション失敗 → exit 1（SEQD 例外フロー）
}

// legixy-cli（snapshot サブコマンドのディスパッチ層）
// SnapshotError を受け取り legixy-core の CLI 終了コード規約へ変換する。
// 引数構文誤り（clap）→ exit 2、SnapshotError → exit 1
```

- **終了コードの規約（v3 正準整合、SPEC REQ.01/02）**:
  - `exit 2`: clap 構文層（サブコマンド省略・型不正・未知フラグ）
  - `exit 1`: `SnapshotError`（DB 失敗・`LabelNotFound`・トランザクション失敗）
  - `exit 0`: 空ストア create（2a）、list 0 件（4a）、delete 該当 0 件（6b）、正常完了
- `unwrap` / `expect` 禁止（rust.md §4）。`?` 演算子で `SnapshotError` へ変換する。

## 3. 公開 API surface（凍結、HR7）

| 関数 | シグネチャ | 不変条件 | 冪等性 | 同期/非同期 |
|---|---|---|---|---|
| `legixy_embed::snapshot::create` | `fn create(store: &EmbeddingStore, snapshot_id: &str, label: Option<&str>) -> Result<SnapshotCreateResult, SnapshotError>` | 単一トランザクション。node_count=0 のとき DB への永続なし（SPEC REQ.02 2a）。snapshot_id は `snap-` プレフィクス付き一意トークン（呼出側が生成）。read-only: `embeddings` テーブルを変更しない | no（呼び出すごとに新行を生成する） | 同期 |
| `legixy_embed::snapshot::list` | `fn list(store: &EmbeddingStore) -> Result<Vec<SnapshotMeta>, SnapshotError>` | taken_at 降順 + 同時刻タイブレーク snapshot_id DESC の安定整列（SPEC REQ.02/06。SUPP-010 S-5）。read-only | yes | 同期 |
| `legixy_embed::snapshot::delete` | `fn delete(store: &EmbeddingStore, snapshot_id: &str) -> Result<SnapshotDeleteResult, SnapshotError>` | 単一トランザクション。deleted_rows=0 は SnapshotError ではなく Ok として返す（exit 0 パス、SPEC REQ.02 6b）。read-only: `embeddings` テーブルを変更しない | no（誤って呼べば異なる結果になりうる） | 同期 |
| `legixy_embed::snapshot::resolve_label` | `fn resolve_label(store: &EmbeddingStore, label: &str) -> Result<LabelResolveResult, SnapshotError>` | taken_at DESC + snapshot_id DESC で 1 件に決定論的解決（SPEC REQ.02 6a / REQ.06。SUPP-010 S-5）。DB エラーのみ Err | yes | 同期 |
| `legixy_embed::snapshot::generate_snapshot_id` | `fn generate_snapshot_id() -> String` | `snap-{epoch_ms 13 桁 16 進}-{8 桁 16 進乱数}` 形式。一意性保証なし（衝突時は `create` トランザクション失敗で検出し呼出側がリトライ。SUPP-010 S-4 v3 方式踏襲）。`snap-` プレフィクスは SPEC 凍結 | no | 同期 |

### 3.1 DB パス解決の規約（ADR-LGX-015、SUPP-010 C-3/C-4）

`legixy-db` の `open_engine_db(project_root: &Path, access: DbAccess) -> Result<Connection, DbError>` を経由する。内部解決順:

1. `<project_root>/.legixy/engine.db`（正準、SPEC/LGX-EXT-001 §4.3）
2. `<project_root>/.trace-engine/engine.db`（正準パス不在時の**読取専用**フォールバック、ADR-LGX-015）

`snapshot create` / `delete` は `DbAccess::Write`（正準パスへ書込。不在時は`.legixy/` ディレクトリと `engine.db` を新規作成しスキーマ初期化。SPEC REQ.07）。`snapshot list` は `DbAccess::Read`（正準パス不在時フォールバック。それも不在なら空ストア相当 = `Vec::new()` を返し exit 0。FB-INV-4）。

v0.1.0 DB 検出ゲート（SUPP-010 C-5）は `legixy-cli` の `autodetect::gate` が担当し、`legixy-embed` の snapshot API は関知しない。

### 3.2 出力形式の規約

`legixy-cli` の snapshot コマンドハンドラが `SnapshotCreateResult` / `Vec<SnapshotMeta>` / `SnapshotDeleteResult` を受け取り、`--json` フラグに応じて出力する。形式は v3 実装（snapshot.rs L56-117）を正準とする:

| 操作 | text モード | `--json` モード |
|---|---|---|
| create 成功（node_count > 0） | `snapshot created:` + snapshot_id / label / nodes 行（v3 正準） | `{"snapshot_id", "label", "node_count"}` |
| create 空ストア（node_count = 0） | `WARNING:` stderr + snapshot_id + nodes=0 stderr行 | `{"snapshot_id", "label", "node_count": 0, "warning": "ストアが空のため永続化されません。`embed --all` を先に実行してください"}` |
| list（1 件以上） | ヘッダ + 80 桁罫線 + `{:<32} {:<20} {:>6} {}` 固定幅表（label 無しは `-`） | `[{"snapshot_id","label","node_count","taken_at"}, ...]`（pretty print） |
| list（0 件） | `（スナップショットはありません。\`snapshot create\` で作成してください）` | `[]` |
| delete 成功（deleted_rows > 0） | `snapshot '{id}' を削除しました（{n} 行）` stdout | `{"snapshot_id", "deleted_rows": n}` |
| delete 該当 0 件（6b）| `WARNING:` stderr + exit 0 | `{"snapshot_id", "deleted_rows": 0}`（WARNING なし） |
| delete label 解決失敗（6c） | `ERROR:` stderr + exit 1 | exit 1（JSON 出力なし） |

> **診断メッセージの出力先**: INFO / WARNING / ERROR はすべて **stderr**（NFR-LGX-001.OBS.02、SPEC REQ.01【v3 差分】）。結果（snapshot_id / 一覧 / 削除確認）は stdout。
>
> **warning 文言（SUPP-010 S-8）**: v3 の英語文言を廃し「ストアが空のため永続化されません。`embed --all` を先に実行してください」を採用（NFR-LGX-001.OBS.04 日本語 primary）。

## 4. module / package 構成

```
legixy-embed/
├── src/
│   ├── lib.rs              // Document ID: SRC-LGX-??? （pub use snapshot::*; を含む）
│   ├── store.rs            // EmbeddingStore 本体（load_all / upsert / upsert_with_subnode_meta 等、ADR-LGX-020）
│   ├── snapshot/
│   │   ├── mod.rs          // Document ID: SRC-LGX-??? （generate_snapshot_id / create / list / delete / resolve_label）
│   │   └── types.rs        // SnapshotMeta / SnapshotCreateResult / SnapshotDeleteResult / LabelResolveResult
│   ├── snapshot_error.rs   // SnapshotError（thiserror）
│   └── ...（embedder / drift / similarity 等は他 UC の主管）
└── Cargo.toml

legixy-db/
├── src/
│   ├── lib.rs              // Document ID: SRC-LGX-??? （open_engine_db / DbAccess / DbError）
│   ├── schema.rs           // embedding_snapshots テーブル定義（§5 SQL）
│   └── connection.rs       // DB 接続・WAL 設定・スキーマ初期化（正準パス / フォールバック解決）
└── Cargo.toml

legixy-cli/（参照のみ、主管は legixy-cli crate）
└── src/
    └── commands/
        └── snapshot.rs     // Document ID: SRC-LGX-??? （run_create / run_list / run_delete）
```

依存方向（DAG、ADR-LGX-020）: `legixy-cli` → `legixy-embed` → `legixy-db` → `legixy-core`。循環なし。

## 5. ライフタイム / 所有権 / 借用 方針

- `create` / `list` / `delete` / `resolve_label` は `&EmbeddingStore` を**借用**（read-only 判定との整合。ただし `create` / `delete` は SQLite トランザクションを内部実行する。`Connection` は内部 mutability を持つため `&self` で `unchecked_transaction` を呼ぶ）。
- `SnapshotCreateResult` / `Vec<SnapshotMeta>` / `SnapshotDeleteResult` は**所有を返す**（呼出側が出力・exit 判定に使う）。
- `snapshot_id: String` は `generate_snapshot_id()` が所有を生成し `create` へ `&str` として渡す。
- `EmbeddingStore` は `Connection` を所有（単一スレッド。§7）。`Arc<Mutex<EmbeddingStore>>` は不要。

## 6. エラー伝播戦略

- **内部**: `store.rs` の SQL 操作は `rusqlite::Error` を `legixy_db::DbError`（`From<rusqlite::Error>` 実装）へ変換し `snapshot_error.rs` の `SnapshotError::Db` へ `?` 伝播。
- **公開境界**: `legixy-embed::snapshot::*` は `Result<_, SnapshotError>` を返す。`legixy-cli` の snapshot コマンドハンドラが `SnapshotError` を受け取り終了コードへ変換（`LabelNotFound` → exit 1、`Db` / `TransactionFailed` → exit 1）。
- **delete 6b（該当 0 件）**: `SnapshotDeleteResult.deleted_rows = 0` は `Ok` として返す（エラーではない）。コマンド層が WARNING を stderr へ出力して exit 0。
- **空ストア create（2a）**: `create` は `SnapshotCreateResult{node_count: 0}` を `Ok` として返す（スキップは API 内部で決定。DB への書込なし）。コマンド層が WARNING を stderr へ出力。
- **トランザクション失敗**: `create` / `delete` の `tx.commit()?` が `Err` になった場合 `SnapshotError::TransactionFailed` として伝播。SQLite は ROLLBACK を自動保証（アトミック性維持）。
- **panic 禁止**: `unwrap` / `expect` を禁止（rust.md §4）。`rusqlite::optional()` を活用して `NOT FOUND` を `Ok(None)` で表現。

## 7. 並行性 / 非同期境界

- `snapshot create` / `list` / `delete` はすべて**同期・単一スレッド**。async なし。
- `create` のトランザクションは SQLite の `unchecked_transaction` + `commit`（v3 正準）で完結。WAL モード（NFR-LGX-001.PERF.07）により他の read 操作と競合しない。
- 複数アクターが同一 label に同時 delete を行う場合の競合は NFR-LGX-001（WAL busy_timeout = 5000ms、ADR-LGX-015）の射程であり本 DD では追加設計しない。
- `generate_snapshot_id()` の `rand_seed()` は `Instant::elapsed()` と `SystemTime::now()` の XOR 合成（v3 正準方式）。同一ミリ秒内衝突は `PRIMARY KEY(snapshot_id, node_id)` 違反として `TransactionFailed` に昇格し、コマンド層がリトライするか exit 1 を返す（SUPP-010 S-4 の衝突可能性を了知したうえで v3 方式踏襲）。

## 8. テスト分類

| 分類 | 内容 | 対応 TP |
|---|---|---|
| Unit | `generate_snapshot_id()` の `snap-` プレフィクス検証・13+8 桁 16 進形式検証 | TP-LGX-009 |
| Unit | `create` 正常系（node_count > 0）: DB の `embedding_snapshots` 行を確認 | TP-LGX-009 |
| Unit | `create` 空ストア（2a）: `node_count=0`・DB に行なし（非永続確認） | TP-LGX-009 |
| Unit | `list` taken_at 降順・同時刻 snapshot_id DESC タイブレーク安定整列 | TP-LGX-009 |
| Unit | `list` 0 件: `Vec::new()` を返す | TP-LGX-009 |
| Unit | `delete` 成功: `deleted_rows > 0`・DB に行なし | TP-LGX-009 |
| Unit | `delete` 該当 0 件（6b）: `Ok(SnapshotDeleteResult{deleted_rows: 0})`（エラー非発生） | TP-LGX-009 |
| Unit | `resolve_label` 同一 label 複数存在（6a）: taken_at DESC + snapshot_id DESC で 1 件 | TP-LGX-009 |
| Unit | `resolve_label` label 不在（6c）: `LabelResolveResult::NotFound` | TP-LGX-009 |
| Integration | `create → list → delete` E2E（DB ファイル・テーブル状態）| TP-LGX-009 |
| Integration | create トランザクション失敗: 途中状態の行が残らない（ロールバック検証） | TP-LGX-009 |
| Integration | engine.db 不在で `list`（Read）: 空配列・exit 0（FB-INV-4。DBAccess::Read パス） | TP-LGX-009 |
| Integration | engine.db 不在で `create`（Write）: `.legixy/` 作成 + スキーマ初期化 + exit 0 | TP-LGX-009 |
| E2E（CLI）| `--json` 出力スキーマ（create / list / delete の各 JSON 形式。空ストア warning フィールド含む） | TP-LGX-009 |
| E2E（CLI）| exit code: 2（サブコマンド省略）/ 1（label 不在）/ 0（id 不在・空ストア） | TP-LGX-009 |

## 9. ADR への参照

- **ADR-LGX-020**: crate 分割・共有型凍結（本 DD の crate 境界・`legixy-embed` / `legixy-db` の責務分担）
- **ADR-LGX-015**: DB パス正準化（`.legixy/engine.db` 正準 + `.trace-engine/engine.db` 読取フォールバック。`snapshot list` の DBAccess::Read フォールバック挙動）
- **ADR-LGX-007**: 非有限スコア / model_version 照合ポリシー（`snapshot` 系は類似度計算を行わないため本ポリシーの直接適用外。`embedding_snapshots` の `model_version` 列は UC-LGX-013 drift 側が照合に利用する）
- ADR-LGX-003: embedding 決定論モデル（content_hash / model_version を複製することで SCORE-INV-1 を保持。§S-1 の SELECT 列と整合）
- ADR-LGX-014: SPEC 準拠原則
- ADR-LGX-016: 環境変数命名規約（`LGX_MODELS_DIR` / `TE_MODELS_DIR` は drift UC-013 の主管。snapshot には影響なし）

## 10. 関連 NFR

- NFR-LGX-001.OBS.02: 出力先（結果=stdout / 診断 INFO/WARNING/ERROR=stderr）
- NFR-LGX-001.OBS.04: エラーメッセージ日本語 primary（warning 文言・error 文言）
- NFR-LGX-001.OBS.05: 終了コード（0=OK / 1=Error / 2=使用法誤り）
- NFR-LGX-001.PERF.07: WAL モード必須（SQLite WAL + busy_timeout 5000ms、snapshot トランザクション競合の緩和）
- NFR-LGX-001.REL.07/08: SQLite busy_timeout / engine.db 配置

## 11. 付録: embedding_snapshots テーブルスキーマ（legixy-db 所管）

v3 実測（`crates/lx-db/src/schema.rs` L166-179、SUPP-010 S-1）を踏襲する:

```sql
CREATE TABLE IF NOT EXISTS embedding_snapshots (
    snapshot_id       TEXT NOT NULL,
    label             TEXT NULL,
    node_id           TEXT NOT NULL,
    embedding         BLOB NOT NULL,          -- f32 little-endian 直列化
    embedding_dim     INTEGER NOT NULL,
    model_version     TEXT NOT NULL,
    content_hash      TEXT NOT NULL,
    taken_at          TEXT NOT NULL DEFAULT (datetime('now')),  -- UTC "YYYY-MM-DD HH:MM:SS"（秒精度）
    PRIMARY KEY (snapshot_id, node_id)
);
CREATE INDEX IF NOT EXISTS idx_snapshots_label
    ON embedding_snapshots (label);
```

`list_snapshots` クエリ（taken_at DESC + snapshot_id DESC タイブレーク、SUPP-010 S-5 推奨案を採用）:

```sql
SELECT snapshot_id, label, COUNT(*), MAX(taken_at)
FROM embedding_snapshots
GROUP BY snapshot_id, label
ORDER BY MAX(taken_at) DESC, snapshot_id DESC
```

`resolve_snapshot_id_by_label` クエリ（同一規則で 1 件決定）:

```sql
SELECT snapshot_id FROM embedding_snapshots
WHERE label = ?1
ORDER BY taken_at DESC, snapshot_id DESC
LIMIT 1
```

> **同時刻タイブレーク（SUPP-010 S-5）**: SQLite `datetime('now')` は秒精度のため同一秒内に複数スナップショットが存在しうる。`ORDER BY taken_at DESC, snapshot_id DESC` を `list` と `resolve_label` 両方に適用することで決定論的整列を保証する（SPEC REQ.02/REQ.06「label 解決と list 降順安定出力に同一規則」を満足）。

## 12. 凍結履歴

| Date | Version | Note |
|---|---|---|
| 2026-06-13 | 1.0 | 初版凍結。`legixy-embed` の `SnapshotMeta` / `SnapshotCreateResult` / `SnapshotDeleteResult` / `LabelResolveResult` / `SnapshotError` 型と `create` / `list` / `delete` / `resolve_label` / `generate_snapshot_id` 公開 API を確定。DB パス解決は ADR-LGX-015 委任。タイブレーク規則（taken_at DESC + snapshot_id DESC）を §11 SQL で凍結。warning 文言（空ストア）日本語版を §3.2 で凍結。crate 境界は ADR-LGX-020。HR7 凍結 |
