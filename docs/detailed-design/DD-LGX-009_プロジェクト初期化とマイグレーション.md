Document ID: DD-LGX-009

# DD-LGX-009: プロジェクト初期化とマイグレーションの詳細設計

**親 SEQD**: SEQD-LGX-009
**親 RBD**: RBD-LGX-009 / **親 UC**: UC-LGX-009
**対象言語**: Rust（CLI 本体）
**境界 API 凍結**: yes（本ドキュメント確定後、crate 間公開 API は変更不可。追加は許容、削除・改名・型変更は次版 SPEC 改訂。HR7）

> 言語別規律は `guides/language-stacks/rust.md`。crate 境界・共有型は **ADR-LGX-020** で凍結済（本 DD は参照する）。型は v3 実装（traceability-engine.v3 `crates/lx-mig`）に整合させ引数互換を保つ。

## 1. 対象範囲

- **主 crate**: `legixy-mig`（init・migrate 統括・v0.1.0 変換・atomic 書込・確定順序・feedback.db→engine.db 移行）
- **依存 crate（共有型は ADR-LGX-020、再定義しない）**:
  - `legixy-core`（`Id` / 共通エラー / `ConfigError` / `Severity` 基底）
  - `legixy-graph`（`TraceGraph` / `Node` / `Edge` — 出力妥当性検証用）
  - `legixy-db`（`open_engine_db` / `EmbeddingStore` / `DbError` — engine.db スキーマ初期化・PRAGMA 設定）
  - `legixy-cli`（CLI 引数ディスパッチ・終了コード）
- **公開 API surface**: 本 DD §3（`legixy-mig` の crate 公開関数・型）
- **関連 SEQD**: SEQD-LGX-009

## 2. 型定義

### 2.1 主要データ型

```rust
// legixy-mig: init 結果サマリ（SEQD-LGX-009 §1 基本フロー / REQ.07）
pub struct InitReport {
    /// 新規生成したファイル一覧（stdout 変更サマリ用、REQ.08）
    pub created_files: Vec<PathBuf>,
    /// 既に存在しスキップしたファイル（force=false 時は AlreadyExists で早期中断のため通常空）
    pub skipped_files: Vec<PathBuf>,
    /// 生成した engine.db の絶対パス（.legixy/engine.db、REQ.07）
    pub engine_db_path: PathBuf,
}

// legixy-mig: migrate 実行結果サマリ（SEQD-LGX-009 §2 基本フロー / REQ.08）
pub struct MigrationReport {
    /// 生成・更新したファイル一覧（REQ.08 成功時変更サマリ: stdout）
    pub files_written: Vec<PathBuf>,
    /// 書き換えた ID の件数（REQ.08, REQ.11）
    pub ids_rewritten_count: usize,
    /// id-map ファイルの配置パス（REQ.11: .legixy/migration-id-map.toml）
    pub id_map_path: PathBuf,
    /// 退避ファイルのパス一覧（REQ.02a: <元名>.bak.{epoch}）
    pub backup_paths: Vec<PathBuf>,
    /// 非致命警告（vectors.bin スキップ等、REQ.08: stderr 経由で別途出力）
    pub warnings: Vec<String>,
    /// コピーしたテーブル名（DB 移行レポート用）
    pub tables_copied: Vec<String>,
    /// コピーしたレコード総数
    pub rows_copied: usize,
}

// legixy-mig: プロジェクトバージョン検出結果（SEQD-LGX-009 §2 / REQ.09）
pub struct DetectedVersion {
    pub kind: ProjectVersion,
    /// 判定根拠（"user_version=3", "[graph] section present" 等）
    pub evidence: String,
}

// legixy-mig: v0.1.0 設定から抽出したマイグレーション設定（SEQD-LGX-009 §2 / REQ.03）
pub struct MigrationConfig {
    /// chain order（[id.chain].order または [id.chains] 変種から抽出）
    pub chain_order: Vec<String>,
    /// independent typecodes（[id.chain].independent から抽出）
    pub independent: Vec<String>,
    /// matrix.md 所在（[matrix].file または既定値）
    pub matrix_file: PathBuf,
    /// [matrix].section（パース対象節名）
    pub matrix_section: Option<String>,
    /// ID パターン文字列（[id].pattern）
    pub id_pattern: String,
    /// area コード（[id].area）
    pub id_area: String,
    /// seq_digits（[id].seq_digits）
    pub seq_digits: usize,
    /// multi-area 変種（[id.chains] + [id.areas] 使用フラグ、SUPP-008 §2.4）
    pub is_multi_area: bool,
    /// 元の TOML 全文（設定変換処理 REQ.04 で additive 変更に使用）
    pub raw_toml: toml::Value,
}

// legixy-mig: 成果物 ID 集合（マトリクスパース結果、SEQD-LGX-009 §2）
pub struct ArtifactIdSet {
    /// ノード（typecode, id_str）のコレクション（重複排除済）
    pub items: Vec<ArtifactItem>,
}

pub struct ArtifactItem {
    pub typecode: String,
    pub id_str: String,
    pub path: Option<String>,
}

// legixy-mig: ID マッピング表（REQ.11 / SEQD-LGX-009 §2）
pub struct MigrationIdMap {
    pub mappings: Vec<IdMapping>,
}

pub struct IdMapping {
    pub old_id: String,
    pub new_id: String,
    pub confidence: IdMapConfidence,
}

// legixy-mig: --format の出力形式（LGX-COMPAT-001 §4 #2 凍結）
pub struct MigrationSummaryJson {
    pub files_written: Vec<String>,
    pub ids_rewritten_count: usize,
    pub id_map_path: String,
    pub backups: Vec<String>,
    pub warnings: Vec<String>,
}

// legixy-mig: init --force 退避ファイル命名で使用する epoch + 連番
pub struct BackupName {
    pub path: PathBuf,    // <元名>.bak.{epoch} または <元名>.bak.{epoch}.{seq}
}
```

### 2.2 列挙 / Sum 型

```rust
// legixy-mig: プロジェクトバージョン種別（REQ.09）
pub enum ProjectVersion {
    V0_1_0,
    Legixy,
    Unknown,
}

// legixy-mig: ID マッピング確信度（REQ.11 / SUPP-008 §2.7）
// NOTE: v3 実装の High/Warn と異なり、本実装では衝突は MigError::IdBijectionViolation で Error にする。
//       High のみが通常ケース。Warn は将来拡張用予約。
pub enum IdMapConfidence {
    High,    // 衝突なし（唯一の SHA-256 先頭 seq_digits 桁で確定）
}

// legixy-mig: マッピング不可 ID の継続フラグ（REQ.11 / SUPP-008 §2.15）
// NOTE: --from/--to/--dry-run/--format は凍結契約。本フラグは追加引数として定義（追加は凍結違反外）。
pub enum UnmappedIdPolicy {
    Abort,          // 既定（REQ.11: 非破壊性優先）
    SkipEdge,       // --skip-unmapped 指定時: 当該エッジを除外して継続（dangling 防止）
}

// legixy-mig: [id.chains] 変種の受理モード（SUPP-008 §2.4 / ADR-LGX-018#15）
pub enum ChainConfigVariant {
    Single,       // [id.chain].order（単数形）
    MultiArea,    // [id.chains]（複数形）+ [id.areas]
}
```

### 2.3 エラー型

```rust
// legixy-mig（実行時失敗 = exit 1。使用法誤り = clap で exit 2。REQ.06/07 検証方法、SUPP-008 §2.14）
#[derive(Debug, thiserror::Error)]
pub enum MigError {
    // --- init 固有 ---
    #[error("project already initialized at {path:?} (use --force to overwrite)")]
    AlreadyExists { path: PathBuf },

    // --- migrate 固有 ---
    #[error("v0.1.0 project not found at {path:?}")]
    V01NotFound { path: PathBuf },

    #[error("version mismatch: config={config_version}, db={db_version}")]
    VersionMismatch {
        config_version: String,
        db_version: String,
    },

    // 設定破損・必須セクション欠落（REQ.03a / SEQD-LGX-009 例外 E2）
    #[error("source config corrupt: {detail}")]
    ConfigCorrupt { detail: String },

    // [id.chain] / [id.chains] 双方なし（REQ.03: 破損として Error）
    #[error("chain config missing or invalid: neither [id.chain] nor [id.chains] found")]
    ChainConfigMissing,

    // REQ.03a: feedback.db 必須テーブル欠落
    #[error("schema incompatible: table {table}, detail: {detail}")]
    SchemaIncompatible { table: String, detail: String },

    // REQ.11 全単射違反（GAP-LGX-159 / SUPP-008 §2.7）
    #[error("id bijection violation: {detail}")]
    IdBijectionViolation { detail: String },

    // REQ.11 マッピング不可 ID（既定 abort、--skip-unmapped で SkipEdge）
    #[error("unmapped id(s) detected: {ids:?}")]
    UnmappedIds { ids: Vec<String> },

    // REQ.03a: 出力 graph.toml の妥当性検証失敗（SEQD-LGX-009 例外 E1）
    #[error("output graph validation failed: {detail}")]
    OutputGraphInvalid { detail: String },

    // 退避ファイル生成失敗（REQ.02a）
    #[error("backup failed for {path:?}: {source}")]
    BackupFailed {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    // atomic rename 失敗（REQ.02: temp+fsync+rename 方式）
    #[error("atomic write failed for {path:?}: {source}")]
    AtomicWriteFailed {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    // --- 共有エラー ---
    #[error("db error: {0}")]
    Db(#[from] legixy_db::DbError),

    #[error("sqlite error: {0}")]
    Sqlite(#[from] rusqlite::Error),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("toml parse error: {0}")]
    TomlParse(#[from] toml::de::Error),

    #[error("toml serialize error: {0}")]
    TomlSerialize(#[from] toml::ser::Error),

    #[error("config load error: {0}")]
    ConfigLoad(#[from] legixy_core::ConfigError),
}
```

- 終了コード: `MigError` → exit 1、`--from` 省略等の引数誤り（clap）→ exit 2、成功 → exit 0（LGX-COMPAT-001 §3, SUPP-008 §2.14）。
- `MigError::AlreadyExists` / `V01NotFound` は診断メッセージで原本の保全状態を示す（REQ.08）。

## 3. 公開 API surface（凍結、HR7）

| 関数 | シグネチャ | 不変条件 | 冪等性 | 同期/非同期 |
|---|---|---|---|---|
| `legixy_mig::init` | `fn init(project_root: &Path, force: bool) -> Result<InitReport, MigError>` | `force=false` かつ legixy 管理生成物（.legixy.toml / .trace-engine.toml / graph.toml / engine.db）の**いずれかが存在する場合** `AlreadyExists`。`force=true` は存在ファイルを REQ.02a 命名で退避後に上書き。生成物は SPEC REQ.07 の 8 ディレクトリ + .legixy/ + graph.toml + engine.db | 部分的（force=true 時に退避番号が異なる） | 同期 |
| `legixy_mig::migrate` | `fn migrate(src: &Path, dst: &Path, opts: MigrateOpts) -> Result<MigrationReport, MigError>` | 移行元が v0.1.0 でない場合 `VersionMismatch` または no-op（既に legixy、SUPP-008 §2.19）。DB コミット先行 → graph.toml / id-map / config の atomic 確定順（REQ.02）。移行失敗時は原本保全（REQ.03a）。`dry_run=true` の場合は一切書き込まない | yes（DB INSERT OR IGNORE + atomic rename の冪等性） | 同期 |
| `legixy_mig::detect_version` | `fn detect_version(project_root: &Path) -> Result<DetectedVersion, MigError>` | PRAGMA user_version を一次根拠（REQ.09）、[graph] セクション有無を二次判定、マーカ欠落 → V0_1_0、矛盾 → `VersionMismatch` | yes | 同期 |
| `legixy_mig::backup_file` | `fn backup_file(path: &Path) -> Result<BackupName, MigError>` | 退避名は `<元名>.bak.{unix epoch 秒}`（REQ.02a）。同一秒内衝突時は `.bak.{epoch}.{seq}` で連番付与（SUPP-008 §2.12）。既存退避ファイルを上書きしない | no（epoch/seq が異なる） | 同期 |
| `legixy_mig::atomic_write` | `fn atomic_write(path: &Path, content: &[u8]) -> Result<(), MigError>` | 一時ファイル `{path}.tmp.{epoch}` に全量書き出し → `File::sync_all()` → `std::fs::rename()`（REQ.02 / SPEC-LGX-002.REQ.13 方式統一）。rename 失敗は `AtomicWriteFailed` | yes（中断後に再実行で同じ最終状態に収束） | 同期 |
| `legixy_mig::parse_matrix` | `fn parse_matrix(content: &str, config: &MigrationConfig) -> Result<ArtifactIdSet, MigError>` | SUPP-008 §2.5 の抽出規則（\| 始まり行、先頭行 ID はノード化しない、`-` / 空は不在）。0 件は空正常（REQ.03 空入力）。[id.chain] 欠落は `ChainConfigMissing`（REQ.03）。[id.chains] 変種は `MigrationConfig.is_multi_area=true` で受理（SUPP-008 §2.4 / ADR-LGX-018#15） | yes | 同期 |
| `legixy_mig::generate_id_map` | `fn generate_id_map(artifact_set: &ArtifactIdSet, existing_refs: &[String], config: &MigrationConfig, policy: UnmappedIdPolicy) -> Result<MigrationIdMap, MigError>` | SHA-256 ハッシュ入力 `"{path}\n{typecode}"`（SUPP-008 §2.7）。旧 ID 重複 / 新 ID 衝突 / graph 全体一意性違反は `IdBijectionViolation`（REQ.11 GAP-LGX-159、v3 実装の桁伸長で回避する方式は**不採用**）。マッピング不可 ID は `policy` に従い Abort または SkipEdge | yes（--dry-run でも検証実施） | 同期 |
| `legixy_mig::MigrationReport::to_json` | `fn to_json(&self) -> String` | `MigrationSummaryJson` スキーマの JSON 出力（REQ.08 / `--format json`）。stdout に出力。診断・進捗は stderr（NFR OBS.02） | yes | 同期 |

### MigrateOpts

```rust
pub struct MigrateOpts {
    pub dry_run: bool,
    pub format: MigOutputFormat,
    pub unmapped_policy: UnmappedIdPolicy,
}

pub enum MigOutputFormat {
    Markdown,
    Json,
}
```

- `init` の引数 `force` は LGX-COMPAT-001 §4 #1（`init [--force]`）の凍結契約どおり。
- `migrate` の `--from/--to/--dry-run/--format` は LGX-COMPAT-001 §4 #2 凍結。`--skip-unmapped` は新規追加引数（凍結既存引数の意味変更なし。HR7 加算的追加として許容）。
- `legixy_mig::detect_version` / `backup_file` / `atomic_write` は `legixy-cli` から呼び出す内部公開 API（crate 間 public、但し crate 外ユーザ向けでなく `legixy-cli` のみが direct 利用）。

## 4. module / package 構成

```
legixy-mig/
├── src/
│   ├── lib.rs            // Document ID: SRC-LGX-009（init / migrate 再エクスポート）
│   ├── error.rs          // MigError（thiserror）
│   ├── init/
│   │   ├── mod.rs        // init() エントリ、ProjectInitializer 統括
│   │   ├── checker.rs    // 既存生成物検査（.legixy.toml / .trace-engine.toml / graph.toml / engine.db 検査、GAP-LGX-143 / REQ.07）
│   │   ├── template.rs   // .legixy.toml テンプレート生成（SUPP-008 §2.21、ICONIX 8 typecode + [id.document_id]）
│   │   └── db_init.rs    // legixy-db 経由 engine.db 初期化・PRAGMA user_version=3 設定
│   ├── migrate/
│   │   ├── mod.rs        // migrate() エントリ、MigratorOrchestrator 統括
│   │   ├── version.rs    // detect_version()（REQ.09：PRAGMA user_version 一次 + [graph] 二次 + 矛盾 Error）
│   │   ├── config_parse.rs // MigrationConfig 抽出（REQ.03/03a：[id.chain] / [id.chains] 変種受理）
│   │   ├── matrix.rs     // parse_matrix()（SUPP-008 §2.5 抽出規則: ArtifactIdSet）
│   │   ├── graph_gen.rs  // 有向グラフ表現生成・出力妥当性検証（REQ.03a）
│   │   ├── id_map.rs     // generate_id_map()（REQ.11 / 全単射 Error / SHA-256 方式）
│   │   ├── config_conv.rs // .legixy.toml 変換（REQ.04：[graph] 追加、[semantic].vector_store 削除等）
│   │   ├── db_mig.rs     // DbMigrator（feedback.db → engine.db 移行、REQ.01/02/03a）
│   │   └── commit.rs     // 移行確定処理（REQ.02：DB コミット先行 → atomic_write graph.toml / id-map / config）
│   ├── backup.rs         // backup_file()（REQ.02a：<元名>.bak.{epoch}[.{seq}]、v3 実装の固定 .bak とは別実装）
│   ├── atomic.rs         // atomic_write()（REQ.02：.tmp.{epoch} → fsync → rename）
│   └── report.rs         // InitReport / MigrationReport / MigrationSummaryJson / MigOutputFormat
└── Cargo.toml
```

依存方向（DAG、ADR-LGX-020）:
```
legixy-mig → legixy-core / legixy-graph / legixy-db
legixy-cli → legixy-mig（CLI 引数ディスパッチ）
```
循環なし。`legixy-graph` は `migrate/graph_gen.rs` での `TraceGraph` 構築と出力妥当性検証（REQ.03a）のみで使用。

## 5. ライフタイム / 所有権 / 借用 方針

- `init(project_root: &Path, force: bool)` は `project_root` を**借用**（所有権を取らない）。`InitReport` は所有を返す。
- `migrate(src: &Path, dst: &Path, opts: MigrateOpts)` も `src` / `dst` を借用。`MigrationReport` は所有を返す。
- `MigrationConfig.raw_toml: toml::Value` は `config_parse.rs` が所有。`config_conv.rs` は `&mut toml::Value` で additive 変更（[graph] 追加等）を行い、直接 `atomic_write` で確定する。
- `ArtifactIdSet` / `MigrationIdMap` は所有型。`migrate/mod.rs` が所有し各サブ関数へ `&` 参照を渡す。
- `rusqlite::Connection` は `db_mig.rs` が所有。トランザクション `tx` は `Connection::transaction()` で借用し、コミット後に `commit()`（drop で自動 rollback）。`commit.rs` の DB コミット先行順序は `db_mig.rs` の `tx.commit()` 完了を確認してから `atomic_write` を呼ぶことで実現（REQ.02）。
- `Arc`/`Mutex` 不要（単一スレッド逐次。§7）。`'static` バウンド不要（呼び出しスコープ内で完結）。

## 6. エラー伝播戦略

- 内部: 各 module の失敗は `MigError` の具体 variant として返す。`?` + `From` 変換で伝播（`thiserror`）。
- 破損検出（REQ.03a）: `config_parse.rs` での TOML パース失敗 → `MigError::TomlParse`、必須セクション欠落 → `MigError::ConfigCorrupt` / `ChainConfigMissing`。`db_mig.rs` での必須カラム欠落 → `MigError::SchemaIncompatible`。いずれも原本は無変更（`commit.rs` の atomic 確定が未到達のため）。
- 全単射違反（REQ.11 GAP-LGX-159）: `id_map.rs` が旧 ID 重複・新 ID 衝突・graph 全体一意性違反を検出して `MigError::IdBijectionViolation`。`--dry-run` でも同検証を実施。
- 出力妥当性検証（REQ.03a）: `graph_gen.rs` が `atomic_write` 前に `TraceGraph` のパース可能性と ID 一意性を検証（`legixy_graph::TraceGraph::validate()`）。失敗は `MigError::OutputGraphInvalid`。
- パニック禁止: `unwrap`/`expect` を本番コードで禁止（rust.md §4）。
- 部分失敗なし: migrate は全 or 中断（REQ.02 非破壊性。DB コミット後・平文確定前の中断は再実行で回復、REQ.02 再開戦略）。
- ユーザ通知: 成功サマリ (`InitReport` / `MigrationReport`) = stdout、診断・進捗・Warning = stderr（NFR OBS.02、REQ.08）。
- no-op（既に legixy）: exit 0 + stderr に Info「既に legixy 形式、変更なし」+ stdout に空サマリ（SUPP-008 §2.19 推奨案採用）。

## 7. 並行性 / 非同期境界

- `init` / `migrate` ともに**同期・単一スレッド**。`async` なし。
- 並行アクセス排他（REQ.02 GAP-LGX-150/151）: engine.db については **SQLite WAL の読取一貫性**に委任。graph.toml は **atomic rename** で常に完全な旧版か新版のみを読み手が観測。**専用ロックファイル等の明示排他は設けない**（ADR-LGX-011）。
- 二重 migrate 競合時は SQLite 書込ロックが事実上の排他となり、敗者は `busy_timeout=5000ms` 超過後に `MigError::Sqlite`（rusqlite::Error::SqliteFailure）→ exit 1（SUPP-008 §2.3: NFR REL.07）。

## 8. テスト分類

| 分類 | 内容 | 対応 TP |
|---|---|---|
| Unit | `parse_matrix`（空入力・[id.chain] 欠落 Error・[id.chains] 変種受理）、`detect_version`（user_version 0/3・[graph] 有無・矛盾 Error 全組み合せ）、`backup_file`（同一秒内衝突時の連番付与）、`generate_id_map`（全単射違反 Error・SHA-256 基本動作） | TP-LGX-009 |
| Integration | `init`（空ディレクトリ・既存ファイル Error・--force 退避・8 ディレクトリ+.gitkeep 生成・init 直後 check --formal が 0 ERROR）、`migrate`（v0.1.0 fixture → graph.toml / id-map / engine.db の生成確認）、破損 fixture（壊れた feedback.db / 不正 TOML / order 欠落 → Error + 原本無傷）、DB コミット後中断の再実行収束、--dry-run 非書込、2 回 migrate で退避ファイル 2 世代保全（REQ.02a） | TP-LGX-009 |
| Property-based | `parse_matrix` の決定性（同一入力 → 同一 ArtifactIdSet、proptest） | TP-LGX-009 |
| E2E fixture | V3/old.source/ を fixture に使用（v0.1.0 設定 multi-area 変種・feedback.db user_version=0・matrix.md 実例） | TP-LGX-009 |

## 9. ADR への参照

- **ADR-LGX-020**: crate 分割・共有型凍結（本 DD の crate 境界）
- ADR-LGX-011: migrate 並行排他リスク受容（SQLite ロックを事実上の排他とし、明示ロックファイルを設けない。REQ.02 GAP-LGX-150/151）
- ADR-LGX-018 #15: [id.chains] 変種（multi-area モード）の受理（SUPP-008 §2.4）
- ADR-LGX-014: SPEC 準拠原則
- ADR-LGX-015: DB パス（`.legixy/engine.db`、v3 の `.trace-engine/engine.db` から名称変更）
- ADR-LGX-016: env バイナリ解決・モデルディレクトリ

## 10. 関連 NFR

- NFR-LGX-001.COMPAT.04/05: v0.1.0 → legixy データマイグレーション互換性
- NFR-LGX-001.REL.01: 非破壊性（失敗時の原本保全、atomic 書込）
- NFR-LGX-001.REL.07: SQLite busy_timeout=5000ms（二重 migrate 競合時の敗者 exit 1）
- NFR-LGX-001.REL.08: engine.db はローカル FS 配置（Docker REQ.12）
- NFR-LGX-001.OBS.02: 成功サマリ=stdout / 診断=stderr（REQ.08）
- NFR-LGX-001.OBS.05: 終了コード 0/1/2
- NFR-LGX-001.SEC.04: OOM 防止（悪意入力。migrate は PERF 予算対象外 REQ.03）
- NFR-LGX-001.SEC.08: 単独開発者前提（並行排他リスク受容の根拠）
- NFR-LGX-001.USE.02: マイグレーション失敗時の段階・バックアップ場所・リカバリ手順提示（REQ.08）

## 11. 凍結履歴

| Date | Version | Note |
|---|---|---|
| 2026-06-13 | 1.0 | 初版凍結。`legixy-mig` の MigError / InitReport / MigrationReport / DetectedVersion / MigrationConfig / ArtifactIdSet / MigrationIdMap / MigrateOpts と init / migrate / detect_version / backup_file / atomic_write / parse_matrix / generate_id_map / MigrationReport::to_json 公開 API を確定。v3 lx-mig 整合（SUPP-008 §2 底本）。crate 境界は ADR-LGX-020。HR7 凍結。[要決定] 10 件のうち DD で確定したもの: §2.12 連番形式（`.bak.{epoch}.{seq}`）/ §2.15 継続フラグ名（`--skip-unmapped`）/ §2.16 JSON スキーマ（MigrationSummaryJson）/ §2.18「一度だけ Info」解釈（1 コマンドにつき 1 回）/ §2.19 no-op 出力（exit 0 + stderr Info + 空サマリ）。未解決: §2.1 SPEC 文言修正（人間承認候補）/ §2.4 [id.chains] 変種の SPEC 改訂提案 / §2.6 custom_edges 継承意味論（B 案: v3 同様転記 + check 側で検出）/ §2.10 vectors.bin（A 案: Skip + Warning 採用）/ §2.17 ネットワーク FS 判定（Step 2 延期） |
