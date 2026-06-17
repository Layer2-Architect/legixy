Document ID: TS-LGX-009

# TS-LGX-009: プロジェクト初期化とマイグレーションのテスト仕様

> TS は **上流 TP の翻訳**。ゼロから観点を考えない。各 TP 観点を DD-LGX-009 で確定した型・関数シグネチャ（`legixy-mig` crate）に即した具体的な入力・期待出力・前提条件へ展開する。

**親 DD**: DD-LGX-009
**継承 TP**: TP-LGX-008（TP[SPEC] マイグレーション、51 観点）, TP-LGX-019（TP[UC] UC-009 フロー、22 観点）

## 1. 対象範囲

このテスト仕様がカバーする DD-LGX-009 の関数 / 型:

- DD-LGX-009 §3 `legixy_mig::init(project_root: &Path, force: bool) -> Result<InitReport, MigError>`
- DD-LGX-009 §3 `legixy_mig::migrate(src: &Path, dst: &Path, opts: MigrateOpts) -> Result<MigrationReport, MigError>`
- DD-LGX-009 §3 `legixy_mig::detect_version(project_root: &Path) -> Result<DetectedVersion, MigError>`
- DD-LGX-009 §3 `legixy_mig::backup_file(path: &Path) -> Result<BackupName, MigError>`
- DD-LGX-009 §3 `legixy_mig::atomic_write(path: &Path, content: &[u8]) -> Result<(), MigError>`
- DD-LGX-009 §3 `legixy_mig::parse_matrix(content: &str, config: &MigrationConfig) -> Result<ArtifactIdSet, MigError>`
- DD-LGX-009 §3 `legixy_mig::generate_id_map(artifact_set: &ArtifactIdSet, existing_refs: &[String], config: &MigrationConfig, policy: UnmappedIdPolicy) -> Result<MigrationIdMap, MigError>`
- DD-LGX-009 §3 `legixy_mig::MigrationReport::to_json(&self) -> String`
- DD-LGX-009 §2 型: `InitReport` / `MigrationReport` / `DetectedVersion` / `MigrationConfig` / `ArtifactIdSet` / `ArtifactItem` / `MigrationIdMap` / `IdMapping` / `MigrationSummaryJson` / `BackupName` / `ProjectVersion`{V0_1_0,Legixy,Unknown} / `IdMapConfidence`{High} / `UnmappedIdPolicy`{Abort,SkipEdge} / `ChainConfigVariant`{Single,MultiArea} / `MigrateOpts` / `MigOutputFormat`{Markdown,Json} / `MigError`（14 variant）

委譲（本 TS 対象外）:
- 性能予算・大規模入力境界（TP-008 §2.1 B-3 / §2.4 並行性ホットパス）→ NFR-LGX-001（migrate は PERF 予算対象外、SPEC-008.REQ.03 明記）/ bench へ委譲
- 並行アクセス整合性・二重 migrate 排他（TP-008 §2.4 C-1/C-2/C-3、§2.5 P-6 ネットワーク FS）→ NFR-LGX-001.REL.07/REL.08（SQLite WAL / busy_timeout）へ委譲。Step 2（Docker / ネットワーク FS Warning）は本版スコープ外
- graph.toml 整合性検証**本体**（CTX-INV-2 のチェック側実装）→ SPEC-LGX-002 / TS-LGX-001（check）へ委譲。本 TS は `graph_gen.rs` の出力妥当性検証（`TraceGraph::validate()` 呼び出し → `OutputGraphInvalid`）の発火のみを検証
- ロールバック手段・前方互換（旧バイナリ読込）（TP-008 §2.6 V-3/V-6）→ OUT_OF_SCOPE（片方向移行、運用手順）
- CLI 引数ディスパッチ・グローバル opt 受理・clap exit 2（TP-008 §2.10 F-1/F-2/F-3、TP-019 AF3/R4）→ `legixy-cli` 統合 / E2E（Contract ケース 25 で exit 規約のみ確認、引数パース本体は legixy-cli へ委譲）

本 TS は「init / migrate が SPEC-008（v0.7.1）の規定を DD-009 の `legixy-mig` 型で正しく具体化しているか」を検証する。

## 2. ケース一覧

### ケース 1: 空ディレクトリでの init → 5 成果物 + 8 ディレクトリ生成 / exit 0

- **観点出典**: TP-019 §2.1 BF1/BF5, TP-008 §2.8 L-1/L-2/L-5
- **分類**: Integration
- **前提**: legixy 管理生成物（`.legixy.toml` / `.trace-engine.toml` / `graph.toml` / `.legixy/engine.db`）が一切存在しない空ディレクトリ
- **入力**: `init(&empty_root, false)`
- **期待**: `Ok(InitReport)`。`report.created_files` に `.legixy.toml`・`docs/traceability/graph.toml`（空）・8 ディレクトリ各 `.gitkeep`（`docs/specs/` `docs/usecases/` `docs/robustness/` `docs/sequence/` `docs/detailed-design/` `docs/test-specs/` `tests/` `src/`）を含む。`report.engine_db_path == <root>/.legixy/engine.db`（絶対パス、実在）。`report.skipped_files` は空。`.legixy.toml` は ICONIX 8 typecode（SPEC/UC/RB/SEQ/DD/TS/TC/SRC）+ `[id.document_id]` を含み chain `UC→RB→SEQ→DD→TS→TC→SRC`（単段）。init 直後に `check --formal` が 0 ERROR
- **境界条件**: 生成物 0 件状態 → 完全初期化（REQ.07）。二段化（RBA/RBD 等）ではなく単段 8 ディレクトリ

### ケース 2: legixy 生成物が 1 つでも既存 → AlreadyExists（force=false）

- **観点出典**: TP-008 §2.1 B-5（一部のみ存在 init の境界、GAP-LGX-143）, §2.2 E-5, TP-019 §2.2 AF1（4 種既存判定）
- **分類**: Integration
- **前提**: `.legixy.toml` / `.trace-engine.toml` / `docs/traceability/graph.toml` / `.legixy/engine.db` の **いずれか 1 つ**が存在（ICONIX 8 ディレクトリやユーザドキュメントのみの存在は対象外）
- **入力**: `init(&partial_root, false)`（4 種それぞれを既存とした 4 入力で別ケース展開）
- **期待**: `Err(MigError::AlreadyExists { path })`（exit 1）。`path` は検出した既存生成物を指す。既存ファイルは無変更。診断メッセージに `use --force to overwrite`
- **境界条件**: 「既存」判定対象は **legixy 管理生成物の 4 種のみ**。`.trace-engine.toml` 既存（旧プロジェクトへの後付け init）も AlreadyExists に収束。8 ディレクトリ/ユーザファイルの存在は判定対象外

### ケース 3: init --force → 既存生成物を REQ.02a 命名で退避後に上書き

- **観点出典**: TP-008 §2.2 E-5, §2.5 P-3, §2.11 D-3（backup-before-overwrite）, TP-019 §2.6 R6
- **分類**: Integration
- **前提**: `.legixy.toml` と `graph.toml` が既存。`force=true`
- **入力**: `init(&existing_root, true)`
- **期待**: `Ok(InitReport)`。上書き前に既存 `.legixy.toml` / `graph.toml` が `<元名>.bak.{epoch}` で退避済（退避ファイル実在・原本内容を保持）。`.gitkeep` は不足分のみ補完し、既存ユーザファイルには触れない。`report.created_files` に新規 `.legixy.toml` 等を含む
- **境界条件**: backup-before-overwrite 順序（退避 → 上書き）。`--force` の破壊範囲は legixy 生成物のみ

### ケース 4: backup_file の退避名 = `<元名>.bak.{epoch}`

- **観点出典**: TP-008 §2.5 P-3, §2.11 D-3, TP-019 §2.6 R3, SPEC-008.REQ.02a
- **分類**: Unit
- **前提**: 退避対象ファイル 1 件が存在。同一秒内に他の退避なし
- **入力**: `backup_file(&path)`
- **期待**: `Ok(BackupName { path })`。`path` は `<元名>.bak.{unix epoch 秒}` 形式。退避ファイルが実在し元ファイル内容のコピー。既存退避ファイルを上書きしない
- **境界条件**: 固定名 `.bak`（v3 実装の固定退避）は採用しない。epoch 秒サフィックス必須

### ケース 5: backup_file の同一秒内衝突 → 連番 `.bak.{epoch}.{seq}`

- **観点出典**: TP-008 §2.5 P-5（`.bak` 衝突方針、GAP-LGX-153）, TP-019 §2.6 R3
- **分類**: Unit
- **前提**: 同一エポック秒内で同一ファイルを 2 回退避（1 回目で `.bak.{epoch}` が既存）
- **入力**: `backup_file(&path)` を 2 回（同一秒シミュレート）
- **期待**: 1 回目 `<元名>.bak.{epoch}`、2 回目 `<元名>.bak.{epoch}.{seq}`（連番付与、`seq` は 1 以上の最小未使用値）。**既存退避ファイルを上書きしない**（両世代とも残存）
- **境界条件**: 退避自体の上書きも非破壊性違反。連番フォーマットは `.bak.{epoch}.{seq}`（DD §11 確定）

### ケース 6: 2 回 migrate で退避ファイル 2 世代が保全される

- **観点出典**: TP-008 §2.5 P-5, §2.11 D-3, SPEC-008.REQ.02a 検証方法
- **分類**: Integration
- **前提**: 同一 v0.1.0 プロジェクトに対し migrate を 2 回実行（2 回目は既に legixy だが `.trace-engine.toml` 等の退避が発生する経路を含む fixture）
- **入力**: `migrate(&src, &dst, opts)` を 2 回
- **期待**: 退避ファイルが **2 世代とも残存**（epoch 差または連番差で別名）。legixy が機械的に削除しない
- **境界条件**: 退避は累積保持（削除はユーザ判断）

### ケース 7: atomic_write は temp+fsync+rename で確定（中断耐性）

- **観点出典**: TP-008 §2.5 P-2（graph/id-map atomic 書込、GAP-LGX-152）, TP-019 §2.3 EF2, SPEC-008.REQ.02
- **分類**: Integration
- **前提**: 出力先パスに旧版ファイルが存在
- **入力**: `atomic_write(&path, new_content)`
- **期待**: `Ok(())`。確定後 `path` の内容は `new_content` と完全一致。中間で `{path}.tmp.{epoch}` を経由（直接上書きしない）。`File::sync_all()` 後に `rename(2)`。読み手は常に完全な旧版か新版のみを観測（半端な書き込みなし）
- **境界条件**: 部分書き込みの不可視性。rename 前中断で旧版が無傷

### ケース 8: atomic_write の冪等収束（中断後再実行で同一最終状態）

- **観点出典**: TP-008 §2.5 P-4（冪等性）, §2.11 D-2（再開戦略）, TP-019 §2.3 EF2, SPEC-008.REQ.02 再開戦略
- **分類**: Property/Integration
- **前提**: 同一 `content` に対する複数回の `atomic_write`（tmp 残存・rename 失敗注入を含む）
- **入力**: `atomic_write(&path, content)` を中断点を変えて複数回
- **期待**: 何回実行しても最終状態は同一（`path` == `content`）。残存 tmp は再実行で上書き/掃除され不整合を残さない。rename 失敗時は `Err(MigError::AtomicWriteFailed { path, source })`
- **境界条件**: resume なし全やり直し方式（中間状態を持たず冪等収束）

### ケース 9: detect_version の網羅判定（user_version 0/3 × [graph] 有無 × 矛盾）

- **観点出典**: TP-008 §2.6 V-1/V-2（バージョン判定根拠、GAP-LGX-154）, TP-019 §2.6 R2, SPEC-008.REQ.09
- **分類**: Unit
- **前提**: engine.db の `PRAGMA user_version`（0 または 3）と `.legixy.toml`/`.trace-engine.toml` の `[graph]` セクション有無を組み合わせた fixture 群
- **入力**: `detect_version(&project_root)` を各組合せで
- **期待**:
  - `user_version=3`（権威ソース）→ `DetectedVersion { kind: ProjectVersion::Legixy, evidence: "user_version=3" 相当 }`
  - `user_version=0` かつ legixy 追加カラムあり → 二次判定で legixy 寄り、`[graph]` セクションありなら Legixy
  - `user_version=0` かつ `[graph]` なし・マーカ欠落 → `ProjectVersion::V0_1_0`（最も保守的解釈）
  - 矛盾（engine.db は legixy だが `.legixy.toml` は v0.1.0 形式）→ `Err(MigError::VersionMismatch { config_version, db_version })`（exit 1）
- **境界条件**: user_version=3 が一次根拠。マーカ欠落は V0_1_0、矛盾は Error。`evidence` に判定根拠を含む

### ケース 10: parse_matrix 空入力（抽出 0 件）→ 空 ArtifactIdSet・正常

- **観点出典**: TP-008 §2.1 B-1（空プロジェクト migrate、GAP-LGX-141）, §2.7 I-1（matrix.md 形式検証、GAP-LGX-158）, TP-019 §2.3 EF1（空入力 vs 破損の区別）
- **分類**: Unit
- **前提**: matrix.md が想定構造でない/ノードを含まず抽出 0 件。`MigrationConfig` は `[id.chain]` 等を正常に保持（`is_multi_area=false`）
- **入力**: `parse_matrix(&empty_or_malformed_content, &config)`
- **期待**: `Ok(ArtifactIdSet { items: vec![] })`（0 件）。**Error にしない**（v0.1.0 が許容した構造の幅を狭めない）。後段で空 graph.toml + exit 0 + stderr Info（移行対象 0 件）へ収束
- **境界条件**: 「対象が無い」（パース成功 + 抽出 0 件 = 空入力正常）と「対象が壊れている」（REQ.03a Error、ケース 13/14）を厳密に区別

### ケース 11: parse_matrix 単一ノード抽出（SUPP-008 §2.5 抽出規則）

- **観点出典**: TP-008 §2.1 B-2（単一ノード変換）, §2.7 I-1, TP-019 §2.1 BF2
- **分類**: Unit
- **前提**: matrix.md に成果物 1 件（`|` 始まり行、先頭行 ID はノード化しない、`-`/空は不在）。`config.id_pattern` / `config.matrix_section` 設定済
- **入力**: `parse_matrix(&single_node_content, &config)`
- **期待**: `Ok(ArtifactIdSet)` で `items.len() == 1`。`ArtifactItem { typecode, id_str, path }` が抽出規則どおり。重複排除済
- **境界条件**: 件数非依存の一般則。先頭行 ID 非ノード化・`-`/空 = 不在の抽出規則

### ケース 12: parse_matrix の `[id.chain]` 欠落 → ChainConfigMissing（破損 Error）

- **観点出典**: TP-008 §2.7 I-2（[id.chain] 欠落/不正、GAP-LGX-158）, §2.2 E-2, TP-019 §2.3 EF1
- **分類**: Unit
- **前提**: `MigrationConfig` 構築段で `[id.chain]`（単数形）も `[id.chains]`（複数形）も存在しない / 存在する側の `order` が欠落・不正
- **入力**: chain config 欠落の TOML から `MigrationConfig` 抽出 → `parse_matrix`（または config_parse 段）
- **期待**: `Err(MigError::ChainConfigMissing)`（exit 1）。chain エッジを暗黙に 0 本として続行しない（構造情報の黙殺禁止）
- **境界条件**: 空入力（抽出 0 件 = 正常）と chain 定義欠落（破損 = Error）の区別。order 不正も破損扱い

### ケース 13: `[id.chains]`+`[id.areas]` multi-area 変種を受理（両表記）

- **観点出典**: TP-008 §2.7 I-2, §2.6 V-4, ADR-LGX-018 #15（M-4 SPEC-008 v0.7.1）
- **分類**: Unit
- **前提**: v0.1.0 設定が `[id.chain].order`（単数形）ではなく `[id.chains]` + `[id.areas]`（複数形・multi-area 変種）を使用
- **入力**: 単数形版と複数形版の 2 種 TOML から `MigrationConfig` を抽出 → `parse_matrix`
- **期待**: **両表記とも受理**。複数形版は `MigrationConfig.is_multi_area == true`（`ChainConfigVariant::MultiArea`）、単数形版は `false`（`Single`）。いずれもエラーにせずノード/エッジ生成材料として等価に機能
- **境界条件**: ADR-LGX-018 #15 の両受理。multi-area フラグの区別

### ケース 14: 破損 source（壊れた feedback.db / 不正 TOML）→ Error + 原本無傷

- **観点出典**: TP-008 §2.2 E-2（source 破損）, §2.11 D-4（破損を黙って引き継がない、GAP-LGX-144）, TP-019 §2.3 EF1（REQ.03a）, SPEC-008.REQ.03a
- **分類**: Integration
- **前提**: 破損 fixture 群:（a）open/クエリ失敗または必須テーブル（observations/proposals/custom_edges）欠落の `.trace-engine/feedback.db`、（b）TOML パース失敗の `.legixy.toml`、（c）`[id.chain].order` 欠落
- **入力**: `migrate(&corrupt_src, &dst, opts)` を各破損で
- **期待**: それぞれ `Err`:（a）→ `SchemaIncompatible { table, detail }` または `Sqlite(_)`、（b）→ `TomlParse(_)` または `ConfigCorrupt { detail }`、（c）→ `ChainConfigMissing`。いずれも exit 1。**原本（feedback.db / `.legixy.toml` / 既存 graph.toml）は無変更**（atomic 確定未到達のため）。部分移行なし
- **境界条件**: 「対象が壊れている」= Error + 原本温存（REQ.03a）。空入力（ケース 10）との診断メッセージ上の区別

### ケース 15: 出力 graph.toml 妥当性検証失敗 → OutputGraphInvalid（atomic 確定前）

- **観点出典**: TP-008 §2.11 D-4（移行後 graph の妥当性）, TP-019 §2.3 EF1, SPEC-008.REQ.03a（出力の妥当性保証）
- **分類**: Integration
- **前提**: 入力から生成した `TraceGraph` がパース不能/ID 一意性違反となる経路（`graph_gen.rs` が `TraceGraph::validate()` 失敗）
- **入力**: `migrate(...)`（graph_gen 段）
- **期待**: `Err(MigError::OutputGraphInvalid { detail })`（exit 1）。atomic rename **前**に検出され graph.toml 原本は無変更。壊れた入力から壊れた出力を確定しない（CTX-INV-2 保全）
- **境界条件**: 出力妥当性は確定前検証。委譲: 整合性検証本体は SPEC-002 / TS-001、本 TS は発火のみ

### ケース 16: generate_id_map の SHA-256 基本動作（High 確信度）

- **観点出典**: TP-008 §2.6 V-8（ID 書換え+マッピング生成）, §2.7 I-5（id-map 形式）, TP-019 §2.6 R5, SPEC-008.REQ.11
- **分類**: Unit
- **前提**: `ArtifactIdSet`（複数 ArtifactItem、衝突なし）。`config.seq_digits` 設定済。`policy = UnmappedIdPolicy::Abort`
- **入力**: `generate_id_map(&artifact_set, &existing_refs, &config, UnmappedIdPolicy::Abort)`
- **期待**: `Ok(MigrationIdMap)`。各 `IdMapping { old_id, new_id, confidence: IdMapConfidence::High }`。`new_id` は SHA-256 入力 `"{path}\n{typecode}"` の先頭 `seq_digits` 桁から確定。旧→新が全単射
- **境界条件**: ハッシュ入力フォーマット `"{path}\n{typecode}"` 固定。通常ケースは High のみ

### ケース 17: generate_id_map 全単射違反（旧 ID 重複/新 ID 衝突/全体一意性違反）→ IdBijectionViolation

- **観点出典**: TP-008 §2.7 I-5（一意性/重複検出、GAP-LGX-159）, §2.6 V-8, SPEC-008.REQ.11 全単射保証
- **分類**: Unit
- **前提**: 3 種の違反 fixture:（a）同一旧 ID に複数新 ID（曖昧性）、（b）複数旧 ID が同一新 ID（多対一 SHA-256 衝突）、（c）書き換え後 graph 全体の新 ID 一意性（SUBNODE-INV-3）違反
- **入力**: `generate_id_map(...)` を各違反で（`--dry-run` 相当でも実施）
- **期待**: いずれも `Err(MigError::IdBijectionViolation { detail })`（exit 1）。v3 実装の桁伸長による衝突回避は**不採用**（Error にする）。`detail` に違反種別を含む
- **境界条件**: 全 3 違反種を Error。--dry-run でも同検証を実施

### ケース 18: マッピング不可 ID — 既定 abort / --skip-unmapped で SkipEdge

- **観点出典**: TP-008 §2.2 E-6（マッピング不可 ID、GAP-LGX-146）, SPEC-008.REQ.11 マッピング不可処理
- **分類**: Unit
- **前提**: `existing_refs` に `artifact_set` で解決できない旧 ID 参照を含む（dangling 候補）
- **入力**:（a）`generate_id_map(.., UnmappedIdPolicy::Abort)`、（b）`generate_id_map(.., UnmappedIdPolicy::SkipEdge)`
- **期待**:（a）→ `Err(MigError::UnmappedIds { ids })`（exit 1、graph.toml/id-map とも不変・非破壊性優先）。（b）→ `Ok`、当該エッジを**除外**して継続（旧 ID を残置せず dangling 参照を防止）
- **境界条件**: 既定 = Abort（非破壊性優先）。継続 opt-in（`--skip-unmapped`）時はエッジ除外（残置しない）

### ケース 19: migrate v0.1.0 fixture → graph.toml / id-map / engine.db 生成

- **観点出典**: TP-019 §2.5 DF1（入出力対応）, §2.1 BF2/BF5, TP-008 §2.6 V-8, SPEC-008.REQ.03/04/07
- **分類**: Integration / E2E fixture
- **前提**: V3/old.source/（v0.1.0 設定 multi-area 変種・`.trace-engine/feedback.db` user_version=0・matrix.md 実例）を fixture に使用。`opts.dry_run=false`
- **入力**: `migrate(&v01_fixture, &dst, MigrateOpts { dry_run: false, format: Markdown, unmapped_policy: Abort })`
- **期待**: `Ok(MigrationReport)`。`files_written` に `graph.toml`・`.legixy/migration-id-map.toml`・変換済 `.legixy.toml` を含む。`id_map_path == .legixy/migration-id-map.toml`。`engine.db`（`.legixy/engine.db`）に feedback.db の observations/proposals/custom_edges が統合（`report.tables_copied`・`report.rows_copied` が転記実績を反映）。生成 graph.toml はドキュメントノードのみ（サブノードなし、REQ.05）
- **境界条件**: 移行元 DB = `.trace-engine/feedback.db` → 移行先 = `.legixy/engine.db`（ADR-015）。3 テーブル統合

### ケース 20: 確定順序 — DB コミット先行 → graph.toml/id-map/config の atomic 確定

- **観点出典**: TP-019 §2.1 BF3（処理順序と REQ.02 確定順序整合、GAP-LGX-261）, TP-008 §2.5 P-1/P-2, SPEC-008.REQ.02 確定順序
- **分類**: Integration
- **前提**: migrate の各段（DB 変換・graph.toml 生成・id-map 生成・config 移行）。DB コミット後・平文確定前で観測
- **入力**: `migrate(...)`（各確定点でスナップショット取得）
- **期待**: engine.db のトランザクション `tx.commit()` 完了が graph.toml/id-map/config の `atomic_write` より**先行**。中間状態は「DB のみ新」の 1 形態に限定（「DB のみ新・平文のみ新」の不整合は発生しない）
- **境界条件**: 確定順序 = DB 先行 → atomic 平文。中断時の中間状態の単一化

### ケース 21: DB コミット後中断の再実行収束（resume なし全やり直し）

- **観点出典**: TP-008 §2.3 S-3（部分 migrate からの収束）, §2.11 D-2（再開戦略）, §2.5 P-1, SPEC-008.REQ.02 再開戦略
- **分類**: Integration
- **前提**: DB コミット済・graph.toml 未確定の中間状態を作る（rename 前で中断注入）
- **入力**: 中断後に `migrate(...)` を再実行
- **期待**: 再実行で同一最終状態へ収束（graph.toml/id-map/config 確定、engine.db は INSERT OR IGNORE で冪等）。進捗記録を持たず全やり直しで安全収束。原本破壊なし
- **境界条件**: resume なし全やり直し方式（STATE-INV-1 と整合、各段冪等）

### ケース 22: --dry-run は一切書き込まない（全単射検証は実施）

- **観点出典**: TP-008 §2.1 B-4（dry-run 等価性）, §2.9 O-4（dry-run 出力構造）, TP-019 §2.2 AF2, SPEC-008.REQ.06/REQ.11
- **分類**: Integration
- **前提**: v0.1.0 fixture。`opts.dry_run=true`
- **入力**: `migrate(&src, &dst, MigrateOpts { dry_run: true, .. })`
- **期待**: **一切のファイル書き込みなし**（graph.toml/id-map/engine.db/config 不変、退避も発生しない）。ただし全単射検証（ケース 17）は実施され、違反時は `IdBijectionViolation`。`MigrationReport` は実 migrate と同一の変更集合（`files_written` 等が「書き込まれる予定」を表す）
- **境界条件**: dry-run 非書込。検証だけは実行（事前確認）。実 migrate との変更集合等価

### ケース 23: 既に legixy → no-op（exit 0 + stderr Info + 空サマリ）

- **観点出典**: TP-008 §2.3 S-2（already-migrated no-op）, §2.11 D-1（再 migrate 収束）, SPEC-008.REQ.06, DD §6 no-op 規定
- **分類**: Integration
- **前提**: 既に legixy 形式（detect_version → Legixy）のプロジェクト
- **入力**: `migrate(&legixy_src, &dst, opts)`
- **期待**: `Ok(MigrationReport)` で no-op（`files_written` 空・`ids_rewritten_count==0`・`backup_paths` 空）。exit 0。stderr に Info「既に legixy 形式、変更なし」を **1 コマンドにつき 1 回**。stdout に空サマリ
- **境界条件**: no-op の exit 0 + 空サマリ収束。Info は 1 回のみ（DD §11 §2.18 解釈）

### ケース 24: 成功サマリ stdout / 診断 stderr 分離（REQ.08）+ to_json スキーマ

- **観点出典**: TP-008 §2.2 E-1（段階別エラー型・提示）, §2.9 O-1/O-2/O-3, TP-019 §2.1 BF4, SPEC-008.REQ.08, NFR OBS.02
- **分類**: Integration / Contract
- **前提**: 成功 migrate（`format=Json`）と失敗 migrate の両方
- **入力**: 成功時 `MigrationReport::to_json()` / CLI 実行の stdout・stderr
- **期待**: 成功時、変更サマリ（生成/更新ファイル一覧・書き換え ID 件数・バックアップ場所）は **stdout**。診断・進捗・Warning は **stderr**。`to_json()` は `MigrationSummaryJson` スキーマ（`files_written` / `ids_rewritten_count` / `id_map_path` / `backups` / `warnings`）の JSON。失敗時は失敗段階・バックアップ場所・リカバリ手順を提示（USE.02）
- **境界条件**: stdout/stderr チャネル分離（OBS.02）。JSON スキーマは `MigrationSummaryJson` に束縛

### ケース 25: 終了コード契約 0/1/2（LGX-COMPAT-001 §3 凍結）

- **観点出典**: TP-008 §2.10 F-4（終了コード規約）, TP-019 §2.2 AF3/§2.6 R4, SPEC-008 / LGX-COMPAT-001 §3
- **分類**: Contract
- **前提**:（a）init/migrate 成功、（b）`MigError`（実行時失敗: AlreadyExists / V01NotFound / ConfigCorrupt / IdBijectionViolation 等）、（c）`--from` 省略等の引数誤り（clap 層）
- **入力**: それぞれ CLI ディスパッチ / `MigError` → exit 変換
- **期待**:（a）→ 0、（b）→ 1、（c）→ 2。値の意味的不正（破損 source・破損出力）は exit 1（exit 2 ではない）
- **境界条件**: exit 2 は clap 構文層限定、`MigError` 系の実行時失敗は exit 1。代替フロー ERROR は exit 1 に収束

### ケース 26: custom_edges 継承（v0.1.0 にあれば転記、なければ継承なし）

- **観点出典**: TP-008 §2.7 I-3（custom_edges 不在/差異）, M-3 SPEC-008 v0.7.1（feedback.db 統合）
- **分類**: Integration
- **前提**:（a）`custom_edges` を持つ feedback.db、（b）`custom_edges` テーブルを持たない feedback.db
- **入力**: `migrate(...)` を各 fixture で
- **期待**:（a）→ custom エッジを engine.db / graph.toml へ転記（v3 lx-mig 同様の copy 挙動、孤児検出は check 側責務）。（b）→ 継承なしで明示（Error にしない）。`report.tables_copied` が実際にコピーしたテーブルを反映
- **境界条件**: custom_edges 不在は継承なしで正常（破損ではない）

### ケース 27: vectors.bin 不在時はスキップ + Warning（Phase 1 ドキュメントノードのみ）

- **観点出典**: TP-019 §2.5 DF2（vectors.bin 不在分岐、GAP-LGX-268）, TP-008 §2.8 L-3, SPEC-008.REQ.05, DD §11 §2.10（A 案: Skip + Warning）
- **分類**: Integration
- **前提**:（a）vectors.bin 不在の v0.1.0 fixture、（b）vectors.bin 存在の fixture
- **入力**: `migrate(...)` を各 fixture で
- **期待**:（a）→ スキップして正常終了（exit 0）、`report.warnings` に vectors.bin スキップの非致命警告（stderr 経由）。（b）→ embeddings インポート（または同 A 案で Skip + Warning、DD 確定方針に従う）。いずれも graph.toml はドキュメントノードのみ（サブノードなし、REQ.05）
- **境界条件**: vectors.bin 不在 = 非致命 Warning + 継続。Phase 1 はドキュメントノードのみ

### ケース 28: 設定ファイル探索順 4 ケース（.legixy.toml / .trace-engine.toml フォールバック）

- **観点出典**: TP-008 §2.6 V-7（config 探索 4 ケース）, §2.9 O-1（移行 Info 一度だけ）, TP-019 §2.6 R1（探索順 UC 可視性、GAP-LGX-270）, SPEC-008.REQ.13
- **分類**: Integration
- **前提**: 4 ケース fixture:（a）`.legixy.toml` のみ、（b）`.trace-engine.toml` のみ、（c）両方、（d）両方なし
- **入力**: migrate/detect_version の config 探索段（`MigrationConfig` 抽出のソース解決）
- **期待**:（a）→ `.legixy.toml` 使用、`.trace-engine.toml` 無視。（b）→ `.trace-engine.toml` 読込 + **一度だけ Info**（`.legixy.toml` 移行案内、継続・Error にしない）。migrate 時は `.legixy.toml` 生成し旧ファイルを `.bak` 退避。（c）→ `.legixy.toml` 優先・不一致時 Warning。（d）→ 未初期化として init 誘導
- **境界条件**: 最初に見つかった 1 ファイルのみ使用。Info は一度だけ。スキーマは両ファイル同一

### ケース 29: parse_matrix の決定性（property）

- **観点出典**: TP-008 §2.1 B-2 系の一般則, DD §8 Property-based（parse_matrix 決定性）
- **分類**: Property-based（proptest）
- **生成器**: 任意の matrix.md 風 content（`|` 始まり行・先頭行・`-`/空セルをランダムに含む）と整合的な `MigrationConfig`（`id_pattern` / `matrix_section`）
- **不変条件**: 同一 `(content, config)` 入力に対し `parse_matrix` は常に同一の `ArtifactIdSet`（`items` の順序・内容・重複排除がバイト一致レベルで決定的）
- **反例ハンドリング**: shrink して最小の非決定例を記録

### ケース 30: V01NotFound — migrate 対象が見つからない（代替フロー 2b）

- **観点出典**: TP-008 §2.2 E-4（--from 不在/不読）, TP-019 §2.2 AF3（代替フロー 2b）, SPEC-008.REQ.06
- **分類**: Integration
- **前提**: `--from <PATH>` が指すパスに v0.1.0 プロジェクトが存在しない（または読めない）
- **入力**: `migrate(&nonexistent_src, &dst, opts)`
- **期待**: `Err(MigError::V01NotFound { path })`（exit 1）。診断メッセージで原本の保全状態を示す。`--from` 自体の省略（引数誤り）は clap 層 exit 2（ケース 25c へ）
- **境界条件**: パス不在（実行時失敗 exit 1）と `--from` 省略（構文誤り exit 2）の区別

## 3. 観点カバレッジ表

### 3.1 TP-LGX-008（TP[SPEC]、51 観点）

| TP § | 観点 | カバーする TS ケース |
|---|---|---|
| TP-008 §2.1 B-1 空プロジェクト migrate | 境界値 | ケース 10（空入力 → 空 graph.toml 経路）|
| TP-008 §2.1 B-2 単一ノード変換 | 境界値 | ケース 11, 29 |
| TP-008 §2.1 B-3 大規模入力境界 | 境界値 | NFR-LGX-001 / bench へ委譲（PERF 予算対象外）|
| TP-008 §2.1 B-4 dry-run 等価性 | 境界値 | ケース 22 |
| TP-008 §2.1 B-5 一部のみ存在 init | 境界値 | ケース 2 |
| TP-008 §2.2 E-1 段階別エラー型・提示 | エラー | ケース 24 |
| TP-008 §2.2 E-2 source 破損 | エラー | ケース 14 |
| TP-008 §2.2 E-3 ディスクフル/権限 | エラー | ケース 7, 8（AtomicWriteFailed）, 14（BackupFailed 系）|
| TP-008 §2.2 E-4 --from 不在/不読 | エラー | ケース 30 |
| TP-008 §2.2 E-5 init 既存/--force 退避 | エラー | ケース 2, 3 |
| TP-008 §2.2 E-6 マッピング不可 ID | エラー | ケース 18 |
| TP-008 §2.3 S-1 状態遷移と判定基準 | 状態 | ケース 9（バージョン判定に集約、GAP-154）|
| TP-008 §2.3 S-2 already-migrated no-op | 状態 | ケース 23 |
| TP-008 §2.3 S-3 部分 migrate 収束 | 状態 | ケース 21 |
| TP-008 §2.3 S-4 auto 初回のみ発火 | 状態 | ケース 9（バージョン判定に集約、GAP-154）/ legixy-cli auto 経路へ委譲 |
| TP-008 §2.3 S-5 commit までが完了 | 状態 | ケース 24（STATE-INV-2 運用支援サマリ）|
| TP-008 §2.4 C-1/C-2/C-3 並行性 | 並行 | NFR REL.07/REL.08 へ委譲（SQLite WAL/busy_timeout）|
| TP-008 §2.5 P-1 中断で DB 壊れない | 永続化 | ケース 21（DB トランザクション中断耐性）|
| TP-008 §2.5 P-2 graph/id-map atomic 書込 | 永続化 | ケース 7, 20 |
| TP-008 §2.5 P-3 失敗時原本 .bak 保護 | 永続化 | ケース 3, 4, 14 |
| TP-008 §2.5 P-4 冪等性 | 永続化 | ケース 8, 21, 22 |
| TP-008 §2.5 P-5 .bak 衝突方針 | 永続化 | ケース 5, 6 |
| TP-008 §2.5 P-6 ネットワーク FS Warning | 永続化 | NFR REL.08 / Step 2 へ委譲 |
| TP-008 §2.6 V-1 双方バージョン不整合 Error | 互換 | ケース 9（矛盾 → VersionMismatch）|
| TP-008 §2.6 V-2 バージョン判定根拠 | 互換 | ケース 9 |
| TP-008 §2.6 V-3 前方互換（旧バイナリ読込）| 互換 | OUT_OF_SCOPE（片方向移行）|
| TP-008 §2.6 V-4 [matrix] 意味変更 | 互換 | ケース 13（変種受理）/ ケース 19（config 変換）|
| TP-008 §2.6 V-5 将来拡張・段階適用 | 互換 | DD §3 detect_version 構造 / 実装レビューへ委譲（REQ.10）|
| TP-008 §2.6 V-6 ロールバック手段 | 互換 | OUT_OF_SCOPE（.bak 復元 + Git revert 運用手順）|
| TP-008 §2.6 V-7 config 探索 4 ケース | 互換 | ケース 28 |
| TP-008 §2.6 V-8 ID 書換え+マッピング生成 | 互換 | ケース 16, 17, 19 |
| TP-008 §2.6 V-9 --from/--to 意味 | 互換 | ケース 25, 30（PATH 意味確定、GAP-157 裁定済）|
| TP-008 §2.7 I-1 matrix.md 形式検証 | 入力 | ケース 10, 11 |
| TP-008 §2.7 I-2 [id.chain] 欠落/不正 | 入力 | ケース 12, 13 |
| TP-008 §2.7 I-3 custom_edges 不在/差異 | 入力 | ケース 26 |
| TP-008 §2.7 I-4 --format 指定 | 入力 | ケース 24（to_json / format=Json）|
| TP-008 §2.7 I-5 id-map 一意性/重複検出 | 入力 | ケース 16, 17 |
| TP-008 §2.8 L-1 init 直後 check 0 ERROR | ライフ | ケース 1 |
| TP-008 §2.8 L-2 空 graph + 初期 DB 妥当 | ライフ | ケース 1 |
| TP-008 §2.8 L-3 Phase 1 ドキュメントノードのみ | ライフ | ケース 19, 27 |
| TP-008 §2.8 L-4 ステートレス維持 | ライフ | ケース 21（resume なし全やり直し = STATE-INV-1）|
| TP-008 §2.8 L-5 init 既定は単段 ICONIX | ライフ | ケース 1 |
| TP-008 §2.9 O-1 移行 Info 一度だけ | 観測 | ケース 23, 28 |
| TP-008 §2.9 O-2 失敗ログの情報量 | 観測 | ケース 24 |
| TP-008 §2.9 O-3 backup/id-map 追跡可能 | 観測 | ケース 24（サマリ）, 16（id-map）|
| TP-008 §2.9 O-4 dry-run 出力構造 | 観測 | ケース 22 |
| TP-008 §2.10 F-1 init 引数一致 | 境界 API | ケース 1, 3（force 契約）/ legixy-cli へ委譲（引数パース本体）|
| TP-008 §2.10 F-2 migrate 引数一致 | 境界 API | ケース 22, 30 / legixy-cli へ委譲 |
| TP-008 §2.10 F-3 グローバル opt 受理 | 境界 API | legixy-cli 統合へ委譲 |
| TP-008 §2.10 F-4 終了コード規約 | 境界 API | ケース 25 |
| TP-008 §2.10 F-5 設定二層区別の明示 | 境界 API | ケース 28（.legixy.toml vs .trace-engine.toml）|
| TP-008 §2.11 D-1 再 migrate no-op 収束 | 領域 | ケース 23 |
| TP-008 §2.11 D-2 部分 migrate 再開戦略 | 領域 | ケース 8, 21 |
| TP-008 §2.11 D-3 backup-before-overwrite 順序 | 領域 | ケース 3, 4 |
| TP-008 §2.11 D-4 破損を黙って引き継がない | 領域 | ケース 14, 15 |
| TP-008 §2.11 D-5 matrix.md→読取専用ビュー遷移 | 領域 | ケース 19（config 変換 REQ.04）/ SPEC-002 COMPAT.05 へ委譲 |

### 3.2 TP-LGX-019（TP[UC]、22 観点）

| TP § | 観点 | カバーする TS ケース |
|---|---|---|
| TP-019 §2.1 BF1 init ステップ連鎖整合 | 基本フロー | ケース 1 |
| TP-019 §2.1 BF2 migrate Step2→Step3 前提充足 | 基本フロー | ケース 11, 19 |
| TP-019 §2.1 BF3 処理順序と確定順序整合 | 基本フロー | ケース 20 |
| TP-019 §2.1 BF4 Step7 変更サマリ具体化 | 基本フロー | ケース 24 |
| TP-019 §2.1 BF5 init 生成物と REQ.07 対応 | 基本フロー | ケース 1（engine.db 含む 5 成果物）|
| TP-019 §2.2 AF1 init 既存判定分岐網羅 | 代替フロー | ケース 2（4 種既存）|
| TP-019 §2.2 AF2 --dry-run / --to 既定 | 代替フロー | ケース 22 / LGX-COMPAT-001 §4 へ委譲（--to 既定）|
| TP-019 §2.2 AF3 代替フロー exit コード収束 | 代替フロー | ケース 25, 30 |
| TP-019 §2.3 EF1 破損入力失敗パス | 例外フロー | ケース 14, 15 |
| TP-019 §2.3 EF2 Step3〜6 各段階失敗 | 例外フロー | ケース 7, 8, 20 |
| TP-019 §2.3 EF3 init 部分生成失敗中間状態 | 例外フロー | ケース 2, 3（既存判定 + 退避で被覆）|
| TP-019 §2.4 AT1 書込権限前提 | アクター | ケース 7, 14（書込失敗 = AtomicWriteFailed/BackupFailed）|
| TP-019 §2.4 AT2 Git commit 責任境界 | アクター | ケース 24（サマリ = STATE-INV-2 運用支援）/ SPEC §4 へ委譲 |
| TP-019 §2.5 DF1 migrate 入出力対応 | データフロー | ケース 19 |
| TP-019 §2.5 DF2 vectors.bin 不在分岐 | データフロー | ケース 27 |
| TP-019 §2.5 DF3 init matrix.md 生成 | データフロー | ケース 1（init 生成物 / GAP-269 UC 裁定済、§7 解消）|
| TP-019 §2.6 R1 設定ファイル探索順 | 領域固有 | ケース 28 |
| TP-019 §2.6 R2 バージョン検出ステップ | 領域固有 | ケース 9 |
| TP-019 §2.6 R3 退避命名の可視性 | 領域固有 | ケース 4, 5, 6 |
| TP-019 §2.6 R4 exit コード観察可能性 | 領域固有 | ケース 25 |
| TP-019 §2.6 R5 ID マッピング可視性 | 領域固有 | ケース 16, 17, 19 |
| TP-019 §2.6 R6 --force オプション可視性 | 領域固有 | ケース 3 |

> 継承 TP（TP-008 全 51 観点 + TP-019 全 22 観点）はすべて本テーブルで TS ケースまたは明示委譲先に mapping 済み（漏れゼロ、人間ゲート判断対象）。性能（B-3）・並行性（C-1/2/3, P-6）・前方互換/ロールバック（V-3/V-6）・将来拡張（V-5）・CLI 引数パース本体（F-1/2/3）は責務上 NFR / legixy-cli / 将来要求 / OUT_OF_SCOPE へ委譲し、本 TS は `legixy-mig` の init/migrate/detect_version/backup_file/atomic_write/parse_matrix/generate_id_map の入出力・エラー型・exit 規約・確定順序・冪等性・全単射・退避命名・空入力 vs 破損の区別に集中する。

## 4. テスト技法選択

- **同値分割**: detect_version の入力空間を user_version(0/3) × [graph](有/無) × 矛盾(有/無) に分割（ケース 9）。config 探索を 4 ケース（.legixy.toml/.trace-engine.toml の有無組合せ）に分割（ケース 28）。マッピング policy を Abort/SkipEdge に分割（ケース 18）
- **境界値分析**: init の「既存生成物 0 件」（ケース 1）vs「1 つでも存在」（ケース 2）の境界。parse_matrix の抽出 0 件（空入力 = 正常、ケース 10）vs chain 定義欠落（破損 = Error、ケース 12）の境界。backup の同一秒内 1 個目（ケース 4）vs 2 個目衝突（連番、ケース 5）
- **状態遷移**: 未初期化 → init 済 → migrate 済（ケース 1/19/23）。DB コミット後・平文確定前の中間状態 → 再実行収束（ケース 20/21）
- **Property-based**: parse_matrix の決定性（同一入力 → 同一 ArtifactIdSet、ケース 29）。atomic_write の冪等収束（ケース 8）
- **Contract**: 終了コード 0/1/2 凍結（ケース 25）、to_json の MigrationSummaryJson スキーマ（ケース 24）

## 5. テスト基盤

- 言語: Rust（`legixy-mig` crate）
- フレームワーク: cargo test
- Property-based: proptest（parse_matrix 決定性・atomic_write 冪等収束）
- fixture: `V3/old.source/`（v0.1.0 multi-area 変種・`.trace-engine/feedback.db` user_version=0・matrix.md 実例）+ 破損 fixture 群（壊れた feedback.db / 不正 TOML / order 欠落 / 全単射違反）+ tempdir（init / atomic_write / backup_file）
- モック: なし（実 FS = tempdir、実 SQLite = in-memory または tempdir engine.db）。中断注入は rename 前フック / tmp 残存ファイルの事前配置で擬似

## 6. 関連 TC

| TS ケース | 対応 TC | 場所 |
|---|---|---|
| ケース 1, 2, 3 | TC-LGX-009（init 系）| `legixy-mig/tests/init.rs` |
| ケース 4, 5, 6 | TC-LGX-009（backup 系）| `legixy-mig/tests/backup.rs` / `src/backup.rs` unit |
| ケース 7, 8 | TC-LGX-009（atomic 系）| `legixy-mig/tests/atomic.rs` / `src/atomic.rs` unit |
| ケース 9 | TC-LGX-009（version 系）| `legixy-mig/src/migrate/version.rs` unit |
| ケース 10, 11, 12, 13, 29 | TC-LGX-009（parse_matrix 系）| `legixy-mig/src/migrate/matrix.rs` unit / proptest |
| ケース 14, 15 | TC-LGX-009（破損検出系）| `legixy-mig/tests/migrate_corrupt.rs` |
| ケース 16, 17, 18 | TC-LGX-009（id_map 系）| `legixy-mig/src/migrate/id_map.rs` unit |
| ケース 19, 20, 21, 22, 23, 26, 27 | TC-LGX-009（migrate 統合系）| `legixy-mig/tests/migrate.rs` |
| ケース 24, 25, 28, 30 | TC-LGX-009（出力/exit/探索系）| `legixy-mig/tests/migrate.rs` / legixy-cli E2E |
