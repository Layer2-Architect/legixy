Document ID: TS-LGX-010

# TS-LGX-010: トレーサビリティ健全性監査（report）のテスト仕様

> TS は **上流 TP の翻訳**。ゼロから観点を考えない。各 TP 観点を DD-LGX-010 で確定した型・関数シグネチャに即した具体的な入力・期待出力・前提条件へ展開する。

**親 DD**: DD-LGX-010
**継承 TP**: TP-LGX-010（TP[SPEC] embedding 運用・監査、71 観点。4 コマンド横断のため本 TS は `report` 関連観点のみ展開・他コマンドは委譲）, TP-LGX-020（TP[UC] UC-010 report フロー、22 観点）

## 1. 対象範囲

このテスト仕様がカバーする DD-LGX-010 の関数 / 型（report コマンド層に限定）:

- DD-LGX-010 §3 `legixy_embed::run_report(graph: &TraceGraph, store: &EmbeddingStore, config: &Config) -> Result<ReportOutput, ReportError>`
- DD-LGX-010 §3 `legixy_embed::ReportOutput::to_text(&self) -> String`
- DD-LGX-010 §3 `legixy_embed::ReportOutput::to_json(&self) -> String`
- DD-LGX-010 §2.1 型（v1.2）: `ReportOutput`{links, candidates, summary, **skipped: usize**} / `ReportSummary`{total_links, total_candidates, min/max/mean_link_score: Option\<f32\>}。`SkipWarning`{skip_count, reasons} / `SkipReasonSummary`{missing_embedding, dim_mismatch, non_finite_score} は **bulk エンジン（DD-LGX-007 §6）所有の集約型**（report は参照のみ・再集計しない、ADR-LGX-021 §2.3 / DD-010 §2.1・§6）
- DD-LGX-010 §2.2 `EdgeKind::to_json_str(&self) -> &'static str`（"chain"/"custom"/"parent_child"）/ `ReportFormat`{Text, Json}
- DD-LGX-010 §2.3 `ReportError`{GraphLoad, ConfigLoad, Db}（exit 1 規約）
- DD-LGX-010 §3.2 出力先・終了コード表 / 集約 Warning の stderr 文言（C-7 確定）

委譲（本 TS 対象外）:
- **bulk similarity エンジンのスコア算出ケース**（`compute_edge_scores` / `compute_link_candidates` の端点 embedding 不在 skip・次元不一致 skip・非有限スコア skip の**算出ロジックそのもの**、`EdgeScore` / `CandidateScore` の生成、**skip 検出・理由別集計（`SkipReasonSummary`）・stderr 集約 Warning の出力**）→ **DD-LGX-007 所有（ADR-LGX-021 §2.3 / DD-010 §6・v1.2）→ TS-LGX-007 へ委譲 or stderr 観察**。本 TS は `run_report` がエンジン出力を `ReportOutput` へ集約する責務（summary 集計・`skipped = 試行エッジ数 − links.len()` の件数算出・出力契約）に集中する。**理由別内訳（missing/dim/non_finite）の正確性はエンジン（DD-007）由来であり本 TS は所有しない**。
- 意味層スコアの数値妥当性（cosine 値域・閾値境界）→ TS-LGX-007 へ委譲
- 性能予算 PERF.02（N×E の応答時間 bench）→ NFR-LGX-001 / criterion へ委譲
- 並行アクセス整合性（SEC.02 / REL.06/07、WAL・busy_timeout）→ NFR-LGX-001 へ委譲
- snapshot / drift / calibrate コマンド固有観点（TP-LGX-010 の B1〜B8/E2〜E9/S1〜S6/V3〜V7/I1〜I3/D2/D3/D5 等）→ TS-LGX-011（calibrate）/ TS-LGX-012（snapshot）/ TS-LGX-013（drift）へ委譲

本 TS は「`run_report` / `to_text` / `to_json` が SPEC-LGX-010 REQ.04（計測専用・閾値判定なし）を DD-010 の型で正しく具体化しているか」を検証する。

## 2. ケース一覧

### ケース 1: 空ストア（embeddings 0 件）→ 空 ReportOutput・exit 0

- **観点出典**: TP-LGX-010 §2.1 B2 相当（report の空ストア）, §2.2 E1, TP-LGX-020 §2.2 AF1/AF2（2a 空テーブル exit 0）
- **分類**: Integration
- **前提**: `EmbeddingStore::load_all()` が空ベクトル（DB 不在 ≡ 空ストア、TP-LGX-010 S2）。`graph` は任意（ノード有でも可）。`config` は既定
- **入力**: `run_report(&graph, &empty_store, &config)`
- **期待**: `Ok(ReportOutput{ links: vec![], candidates: vec![], summary: ReportSummary{ total_links: 0, total_candidates: 0, min_link_score: None, max_link_score: None, mean_link_score: None }, skipped: 0 })`（DD-010 v1.2、`skipped: usize`）。呼出側 exit 0（DD-010 §3.2 空ストア行）
- **境界条件**: 検証対象ゼロ件 = 正常終了（`ReportError` ではない）。`min/max/mean` は `None`（links 0 件、REQ.04）

### ケース 2: 空ストア text 出力 → stdout に INFO 文（DD §3.2 空ストア text 行）

- **観点出典**: TP-LGX-010 §2.8 L1/L2（INFO=stderr 原則 / stdout 機械可読性）, TP-LGX-020 §2.2 AF1
- **分類**: Integration
- **前提**: ケース 1 の `ReportOutput`（空）。`ReportFormat::Text`
- **入力**: `report_output.to_text()`（CLI 経由で stdout へ）
- **期待**: **stdout** に空ストア INFO 文（空ストアの案内、日本語、NFR-LGX-001 OBS.04）。`exit 0`。結果セクション（links/candidates/summary）は空のまま 0 件として表現。出力チャネルは **stdout 一本**（DD-010 §3.2「空ストア」行で stderr は `—`＝INFO を stderr へ出さない、v3 report.rs 実測準拠）
- **境界条件**: L1「INFO=stderr 原則」に対する**境界的例外**。空ストア INFO は report 本文（計測結果欄が 0 件である旨の案内）として **stdout** に出る確定挙動（DD-010 §3.2 / v3 実測）。「INFO は常に stderr」という曖昧 assertion を**書かない**（L1 一般原則ではなく DD §3.2 の空ストア行を一次根拠とする）。空ストアでも `to_text` は panic せず案内文を返す（degraded・非致命）

### ケース 3: 空ストア --json 出力 → 空構造 JSON（3 キー・統計 null）

- **観点出典**: TP-LGX-010 §2.9 F3, §2.8 L2, TP-LGX-020 §2.4 AT3（CI の --json 連携）
- **分類**: Contract
- **前提**: ケース 1 の `ReportOutput`（空）。`ReportFormat::Json`
- **入力**: `report_output.to_json()`
- **期待**: バイト一致で
  ```json
  {"links":[],"candidates":[],"summary":{"total_links":0,"total_candidates":0,"min_link_score":null,"max_link_score":null,"mean_link_score":null}}
  ```
  （DD-010 §3.2 空ストア --json 行の凍結文字列。`Option<f32>` の `None` → JSON `null`）。`exit 0`
- **境界条件**: `--json` の stdout に `warnings` フィールドを**付加しない**（C-7、ケース 9 で別途検証）。空構造でも 3 キー（links/candidates/summary）は必ず存在

### ケース 4: links あり → summary 統計値の正確性（min/max/mean）

- **観点出典**: TP-LGX-010 §2.10 D7（算出可能エッジのみ）, DD-010 §8「ReportSummary 集計」
- **分類**: Unit
- **前提**: `compute_edge_scores` が複数 `EdgeScore`（例: score = [0.20, 0.80, 0.50]）を返す状況を `run_report` が受ける（エンジン出力は所与・委譲）
- **入力**: `run_report(&graph, &store, &config)` の戻り `ReportOutput.summary`
- **期待**: `total_links == 3`、`min_link_score == Some(0.20)`、`max_link_score == Some(0.80)`、`mean_link_score == Some(0.50)`（算術平均）。`total_candidates == candidates.len()`
- **境界条件**: links 1 件のとき min==max==mean==その値。集計は f32 算術（丸めは to_text の `{:.4}` で行い summary 生値は非丸め）

### ケース 5: links 1 件（最小非空）→ summary 単一値境界

- **観点出典**: TP-LGX-010 §2.1 境界値（最小非空）, §2.10 D7
- **分類**: Unit
- **前提**: `compute_edge_scores` が `EdgeScore` 1 件（score = 0.42）を返す
- **入力**: `run_report(...).summary`
- **期待**: `total_links == 1`、`min == max == mean == Some(0.42)`
- **境界条件**: 0 件（ケース 1）→ None と 1 件 → Some の境界。1 件で min/max/mean が同値

### ケース 6: to_text フォーマット準拠（ヘッダ・score={:.4}・統計行）

- **観点出典**: TP-LGX-010 §2.9 F3, DD-010 §8「to_text: v3 実測 text フォーマット（R-4）への準拠」
- **分類**: Unit
- **前提**: links / candidates / summary が非空の `ReportOutput`
- **入力**: `report_output.to_text()`
- **期待**: 3 セクション（links / candidates / summary）を含む。各 link 行のスコアは `score={:.4}` 形式（小数 4 桁固定）。summary 行に total_links / total_candidates / min/max/mean を表示。**診断 Warning は含まない**（DD-010 §3 不変条件: stderr 出力はコマンド層が担う）
- **境界条件**: text には skip 由来の WARNING 行を埋め込まない（集約 Warning はエンジンが stderr へ出力、stdout/stderr 分離、OBS.02）。`skipped` 件数は to_text 本文の summary 表現に含めてよいが WARNING 文ではない

### ケース 7: to_json 構造（3 キー・links.kind 文字列・非有限値非出力）

- **観点出典**: TP-LGX-010 §2.9 F3, §2.1 B9（非有限スコア非出力）, DD-010 §8「to_json: 3 キー構造・links.kind 文字列・非有限値非出力」
- **分類**: Contract
- **前提**: links に `EdgeKind` が Chain / Custom / ParentChild の各エッジを含む `ReportOutput`
- **入力**: `report_output.to_json()`
- **期待**: トップレベルキーは厳密に `{"links", "candidates", "summary"}` の 3 つ（`skipped` / `warnings` のいずれも JSON stdout に含まない、C-7・DD-010 §3.2 凍結スキーマ）。各 link の `kind` は `EdgeKind::to_json_str` により `"chain"` / `"custom"` / `"parent_child"` のいずれか（DD-010 §2.2）。`f32::is_finite()` を通らない値（NaN/±Inf）は JSON に出力されない（REQ.09、DD-010 §6: 非有限スコアの除外はエンジンが担い report links は有限スコアのみ）
- **境界条件**: links.kind の 3 文字列が v3 実測と一致。非有限値は summary（min/max/mean）にも links.score にも現れない

### ケース 8: スキップ発生時の `skipped` 件数算出（report 層）+ 集約 Warning stderr（エンジン由来）+ stdout クリーン

> **責務分割（DD-010 v1.2 / ADR-LGX-021 §2.3）**: skip 検出・理由別集計・stderr 集約 Warning は **bulk エンジン（DD-007、frozen signature `compute_edge_scores(graph, store) -> Result<Vec<EdgeScore>, EmbedError>`）が単独所有**。`run_report` は `skipped = 試行エッジ数 − links.len()` の**件数のみ算出**する。理由別内訳を充填した `SkipReasonSummary` を run_report が保持する旧前提は**撤回**。本ケースは下記 8a / 8b に分割する。

- **観点出典**: TP-LGX-010 §2.2 E10（集約 Warning 1 件 stderr）, TP-LGX-020 §2.6 R6（スキップ集約 Warning の UC 表現）, DD-010 §3「`skipped = 試行エッジ数 − links.len()`」/ §6 / §8「スキップ発生時の集約 Warning stderr 出力と stdout クリーン」
- **分類**: Integration

**ケース 8a（本 TS 所有・report 層）: `run_report` の `skipped` 件数算出**
- **前提**: エンジンが N 件をスキップし（X+Y+Z = N > 0）有限スコアの `links`（len = L）を返す状況。試行エッジ数 = L + N
- **入力**: `run_report(&graph, &store, &config)` の戻り `ReportOutput`
- **期待**: `ReportOutput == { links (len L), candidates, summary, skipped: N }`。すなわち `skipped == 試行エッジ数 − links.len()`。`exit 0`（スキップは `ReportError` に昇格しない、DD-010 §2.3）。`skipped > 0` でも summary 統計は `links`（算出済み有限スコア）のみから算出される
- **境界条件**: `ReportOutput` のトップレベルは `{links, candidates, summary, skipped}`（v1.2、`skip_warnings: Vec<SkipWarning>` フィールドは存在しない）。`skipped` は理由別内訳を**持たない**単一 `usize`。report 層は `is_finite` 等の理由別検査を二重実行しない

**ケース 8b（エンジン由来・委譲 or stderr 観察）: 集約 Warning stderr 1 件 + stdout クリーン**
- **前提**: ケース 8a と同じスキップ構成（内訳 missing X / dim Y / non_finite Z）。理由別集計と stderr 出力は bulk エンジン（DD-007 §6 / `SkipReasonSummary` 充填）が所有
- **入力**: CLI 実行（`report`）— stdout と stderr を分離取得
- **期待**: stderr にエンジン由来の集約 Warning 1 件（DD-010 §3.2 文言）:
  ```
  WARNING: N ペアをスキップしました（embedding 不在: X 件 / 次元不一致: Y 件 / 非有限スコア: Z 件）
  ```
  stdout には WARNING を一切含まず計測報告（text/JSON）のみ。`exit 0`
- **境界条件**: 集約 Warning 文言・理由別内訳（missing/dim/non_finite）の**正確性は DD-007（エンジン）所有 → TS-LGX-007 へ委譲**。本 TS はエンジン未配線の段階では stderr 文言を**観察的に**検査（CLI E2E、文言一致）し、内訳数値の単体妥当性は TS-007 に委ねる。出力チャネルは stderr 限定（report 層の stdout は不変）

### ケース 9: --json stdout に warnings フィールドなし（C-7 確定）

- **観点出典**: TP-LGX-010 §2.2 E10 / §2.9 F3, DD-010 §3.2 + §11 v1.0「C-7 確定: `--json` stdout に warnings フィールドなし、集約 Warning は stderr のみ」
- **分類**: Contract
- **前提**: スキップが発生（N > 0、`ReportOutput.skipped == N > 0`）。`ReportFormat::Json`
- **入力**: CLI 実行（`report --json`）— stdout と stderr を分離取得
- **期待**: stdout の JSON はトップレベルに `"warnings"` キーを**持たない**（`{"links","candidates","summary"}` の 3 キーのみ。`skipped` も JSON stdout には出力しない — DD-010 §3.2 空ストア --json 行の凍結スキーマは 3 キー固定）。集約 Warning は stderr へ（ケース 8b と同文言、エンジン由来）。`exit 0`
- **境界条件**: `--json` の機械可読性保全（v3 正準化）。stdout JSON のスキーマはスキップ有無に依存しない（warnings 欄は付かない）

### ケース 10: graph.toml 破損・パース不能 → Err(ReportError::GraphLoad) → exit 1

- **観点出典**: TP-LGX-010 §2.2 E1, TP-LGX-020 §2.3 EF1（Step2 graph.toml 破損失敗パス）, DD-010 §6「実行時失敗」
- **分類**: Integration
- **前提**: graph.toml がパース不能（実行時失敗 = clap 構文誤りではない）
- **入力**: `run_report(...)`（graph ロード段で失敗、`legixy_graph::GraphError` を包む）
- **期待**: `Err(ReportError::GraphLoad(_))`。呼出側で stderr に ERROR メッセージ（日本語、OBS.04）+ exit 1（DD-010 §2.3 / §3.2 ReportError 行）
- **境界条件**: 実行時失敗 = exit 1（引数構文誤り exit 2 と区別）。スキップ（exit 0）とは別概念

### ケース 11: engine.db open 失敗 → Err(ReportError::Db) → exit 1

- **観点出典**: TP-LGX-010 §2.2 E1, DD-010 §6「`load_all` の DB エラーは `?` で伝播させ `ReportError::Db` に変換」
- **分類**: Integration
- **前提**: engine.db は存在するが open 失敗（権限・破損等）。`EmbeddingStore::load_all` が `legixy_db::DbError` を返す
- **入力**: `run_report(...)`
- **期待**: `Err(ReportError::Db(_))`。stderr に ERROR + exit 1
- **境界条件**: **DB 不在**（ケース 1: 空ストア・exit 0）と **DB open 失敗**（本ケース: ReportError・exit 1）の区別

### ケース 12: .legixy.toml 不在/破損 → Err(ReportError::ConfigLoad) → exit 1

- **観点出典**: TP-LGX-010 §2.2 E1, DD-010 §2.3 ReportError::ConfigLoad
- **分類**: Integration
- **前提**: `.legixy.toml` が不在または破損（`legixy_core::ConfigError`）
- **入力**: `run_report(...)`（config ロード段で失敗）
- **期待**: `Err(ReportError::ConfigLoad(_))` + exit 1
- **境界条件**: 3 種の `ReportError` バリアント（GraphLoad / ConfigLoad / Db）がいずれも exit 1 に収束

### ケース 13: read-only 不変（report は graph / engine.db を変更しない）

- **観点出典**: TP-LGX-010 §2.5 P2（読取系非破壊）, TP-LGX-020 §2.3 EF2（エラー時状態不変、STATE-INV-1）, DD-010 §5/§7（借用・read-only）
- **分類**: Property/Integration
- **前提**: 任意の入力（成功・スキップ・エラーいずれも）
- **入力**: `run_report(...)` 実行前後の graph.toml / engine.db のバイトハッシュ
- **期待**: 実行前後で graph.toml・engine.db が不変（`&TraceGraph` / `&EmbeddingStore` / `&Config` の借用のみ、DD-010 §5）。`ReportError` 時も中間状態破壊なし
- **境界条件**: 借用による read-only 保証（書込みゼロ）。STATE-INV-1

### ケース 14: to_json / to_text の出力決定性（property）

- **観点出典**: TP-LGX-010 §2.10 D1（読取系決定性、REQ.06）, TP-LGX-020 §2.6 R5（SCORE-INV-1 決定性）, DD-010 §8「同一入力での to_json / to_text 出力バイト一致」
- **分類**: Property-based（proptest）
- **生成器**: 任意の `ReportOutput`（links: `Vec<EdgeScore>`・candidates: `Vec<CandidateScore>`・summary・`skipped: usize` をランダム生成。score は有限 f32 に制約）
- **不変条件**: 同一 `ReportOutput` に対し `to_json()` を複数回呼んでもバイト一致。`to_text()` も同様にバイト一致。`load_all` の node_id 昇順ロード（SCORE-INV-1）を前提に同一 graph+store からの `run_report` 出力が決定的
- **反例ハンドリング**: shrink して最小のバイト不一致例を記録

### ケース 15: 非有限スコア注入 fixture で to_json に NaN/Inf が現れない（property）

- **観点出典**: TP-LGX-010 §2.1 B9（NaN/±Inf 方針）, DD-010 §6「serde_json 前の明示検査必須」, DD-010 §8「非有限スコア注入 fixture」
- **分類**: Property-based（proptest）
- **生成器**: links の一部 score に NaN / +Inf / -Inf を意図的に注入した `ReportOutput`、および summary（min/max/mean）に非有限値が混入しうる構成
- **不変条件**: `to_json()` の出力文字列に `"NaN"` / `"Infinity"` / `"-Infinity"` / `nan` / `inf` が一切現れない。`f32::is_finite()` を通らない値は出力前に除外され、JSON は常に妥当（serde_json でパース可能）
- **反例ハンドリング**: 非有限値が JSON に漏れた最小例を shrink して記録

### ケース 16: report = 計測 / check = 判定 の責務非重複（severity 概念なし）

- **観点出典**: TP-LGX-010 §2.10 D4（計測/判定分離、REQ.04・SPEC-LGX-004 REQ.02）, TP-LGX-020 §2.6 R1（責務非重複の観察可能性）, DD-010 §1
- **分類**: Contract
- **前提**: 任意の `ReportOutput`
- **入力**: `ReportOutput` / `ReportSummary` / `to_json` / `to_text` の構造
- **期待**: `ReportOutput` のいずれの型・フィールド・JSON キーにも `severity` / `Ok/Info/Warning/Error` の判定概念が**存在しない**（report は閾値判定をしない計測専用）。出力は score / candidate / summary 統計のみ
- **境界条件**: check（severity 付き findings）との責務境界。report に severity が混入しないこと（TP-LGX-020 R3 の OBS.06 誤参照防止に対応）

### ケース 17: candidates の無向除外と閾値（run_report の集約面）

- **観点出典**: TP-LGX-010 §2.10 D7, DD-010 §8「compute_link_candidates: 無向除外 / 閾値境界」（算出ロジックは委譲、集約面のみ本 TS）
- **分類**: Unit
- **前提**: `compute_link_candidates` が `link_candidate_threshold`（既定 0.7）で抽出した `Vec<CandidateScore>` を `run_report` が受ける（閾値境界 ≥/< とエンジン無向除外ロジックは TS-LGX-007 委譲）
- **入力**: `run_report(...).candidates` と `.summary.total_candidates`
- **期待**: `total_candidates == candidates.len()`。`run_report` はエンジンが返した candidates をそのまま `ReportOutput.candidates` に格納し件数を summary に反映する（再フィルタしない）
- **境界条件**: 閾値判定・無向除外の正しさ自体は TS-LGX-007 で検証。本ケースは run_report の「エンジン出力を改変せず集約する」契約のみ

## 3. 観点カバレッジ表

### 3.1 TP-LGX-010（TP[SPEC]、report 関連観点）

| TP § | 観点 | カバーする TS ケース |
|---|---|---|
| TP-010 §2.1 B1〜B8 | snapshot/calibrate/drift の境界値 | TS-LGX-011/012/013 へ委譲（report 外コマンド） |
| TP-010 §2.1 B9 | report 統計の NaN/±Inf 非出力 | ケース 7, 15 |
| TP-010 §2.1 B10 | snapshot 保持境界 | TS-LGX-012 へ委譲 |
| TP-010 §2.2 E1 | 終了コード 3 分類（exit 0/1/2） | ケース 1, 10, 11, 12 |
| TP-010 §2.2 E2〜E9 | drift/snapshot/calibrate 固有エラー | TS-LGX-011/012/013 へ委譲 |
| TP-010 §2.2 E10 | スキップ時集約 Warning 1 件 stderr（**エンジン由来＝DD-007 所有**、文言観察はケース 8b / 内訳正確性は TS-LGX-007）。report 層の `skipped` 件数算出はケース 8a | ケース 8a（件数）, 8b（stderr 文言観察）, 9 |
| TP-010 §2.2 E11/E12 | create トランザクション/部分成功 | TS-LGX-012 委譲（E12 report 面の skip 件数はケース 8a） |
| TP-010 §2.3 S1〜S6 | snapshot ライフサイクル | TS-LGX-012 へ委譲 |
| TP-010 §2.3 S2 | DB 不在 ≡ 空ストア（read 系非作成） | ケース 1 |
| TP-010 §2.4 C1〜C4 | 並行性（WAL/busy_timeout） | NFR-LGX-001 へ委譲 |
| TP-010 §2.5 P1〜P5 | 永続化（create トランザクション/構造） | TS-LGX-012 委譲（P2 read 非破壊はケース 13） |
| TP-010 §2.6 V1/V2 | 引数体系・グローバルオプション維持 | LGX-COMPAT-001 §4 #6 凍結（CLI 層・DD §3.1） |
| TP-010 §2.6 V3〜V7 | snapshot_id/モデル解決 | TS-LGX-012/013 へ委譲 |
| TP-010 §2.7 I1〜I5 | drift/snapshot 入力検証 | TS-LGX-012/013 へ委譲 |
| TP-010 §2.8 L1/L2 | INFO/WARNING=stderr 原則 / 結果=stdout / json 機械可読性（**例外: 空ストア INFO は stdout＝DD §3.2 確定**、ケース 2） | ケース 2, 3, 8b, 9 |
| TP-010 §2.8 L3 | drift モデル解決失敗通知 | TS-LGX-013 へ委譲 |
| TP-010 §2.8 L4/L5/L6 | 監査証跡/ローカライズ/機密非混入 | NFR-LGX-001 へ委譲（OBS.04/SEC.05） |
| TP-010 §2.9 F1 | report の MCP 非公開（MCP-INV-1） | DD-010 §1（MCP-INV-1）/ TS-LGX-009 整合 |
| TP-010 §2.9 F2 | CLI 引数契約一致 | LGX-COMPAT-001 §4 #6（DD §3.1 凍結） |
| TP-010 §2.9 F3 | --json スキーマ定義 | ケース 3, 7, 9 |
| TP-010 §2.10 D1 | 読取系決定性（同一入力→同一バイト） | ケース 14 |
| TP-010 §2.10 D2/D3/D5 | snapshot/calibrate 決定性 | TS-LGX-011/012 へ委譲 |
| TP-010 §2.10 D4 | report=計測 / check=判定 責務非重複 | ケース 16 |
| TP-010 §2.10 D6 | bulk API consumer（類似度非再実装。skip 検出・理由別集計も非再実装＝DD-007 委譲） | ケース 4, 17（集約面）/ TS-LGX-007（算出・skip 集計） |
| TP-010 §2.10 D7 | report links は算出可能エッジのみ + 可視化（skip 件数 = `skipped`） | ケース 4, 5, 8a, 17 |
| TP-010 §2.10 D8 | drift 次元不一致 Error | TS-LGX-013 へ委譲 |

### 3.2 TP-LGX-020（TP[UC]、UC-010 report フロー）

| TP § | 観点 | カバーする TS ケース |
|---|---|---|
| TP-020 §2.1 BF1 | ステップ連鎖整合（受理→ロード→算出→出力→exit） | ケース 1, 4, 8a/8b（E2E 連鎖） |
| TP-020 §2.1 BF2 | Step2 graph パース / embeddings ロード独立性 | ケース 10（graph 失敗）, 11（DB 失敗）で独立性を検証 |
| TP-020 §2.1 BF3 | 成功時事後条件（stdout 報告・exit 0・db/graph 不変） | ケース 1, 13 |
| TP-020 §2.1 BF4 | text/JSON 排他分岐 | ケース 2, 3, 6, 7 |
| TP-020 §2.2 AF1 | 分岐網羅（空ストア/スキップ/NaN） | ケース 1（空）, 8a（スキップ件数）/ 8b（stderr）, 15（NaN） |
| TP-020 §2.2 AF2 | 代替フロー事後条件収束（2a exit0 / 3a exit1） | ケース 1, 10 |
| TP-020 §2.2 AF3 | 遷移条件明示（compute 失敗→exit1） | ケース 10, 11（ReportError） |
| TP-020 §2.3 EF1 | Step2 失敗パス（graph.toml 破損） | ケース 10 |
| TP-020 §2.3 EF2 | エラー時状態不変（STATE-INV-1） | ケース 13 |
| TP-020 §2.3 EF3 | エラー時通知の情報十分性 | ケース 10, 11, 12（ERROR メッセージ） |
| TP-020 §2.4 AT1 | アクター権限一貫性（read-only 同一権限） | ケース 13（read-only）|
| TP-020 §2.4 AT2 | 責任境界（計測 vs 是正） | ケース 16 |
| TP-020 §2.4 AT3 | CI の --json 連携 | ケース 3, 9 |
| TP-020 §2.5 DF1 | 入出力型・stdout/stderr 分離 | ケース 2, 3, 8a/8b |
| TP-020 §2.5 DF2 | link_candidate_threshold データフロー | ケース 17 |
| TP-020 §2.5 DF3 | エラー時データ解放（read-only） | ケース 13 |
| TP-020 §2.6 R1 | report/check 責務非重複 | ケース 16 |
| TP-020 §2.6 R2/R3 | UC 関連 SPEC/NFR 誤参照（REQ.10/OBS.06） | UC 修正で解消済（TP-020 §7）。severity 非混入はケース 16 で担保 |
| TP-020 §2.6 R4 | 終了コード契約一致（exit 0/1） | ケース 1, 10, 11, 12 |
| TP-020 §2.6 R5 | SCORE-INV-1 決定性 | ケース 14 |
| TP-020 §2.6 R6 | スキップ集約 Warning の表現（stderr 文言はエンジン由来＝ケース 8b 観察 / report 層の件数は 8a） | ケース 8a, 8b, 9 |

> 継承 TP 観点はすべて本テーブルで TS ケースまたは明示委譲先に mapping 済み（人間ゲート判断対象）。TP-LGX-010 は 4 コマンド横断 SPEC レベル TP のため、snapshot / drift / calibrate 固有観点は責務上 TS-LGX-011/012/013 へ、bulk similarity エンジンのスコア算出ロジックは ADR-LGX-021 §2.3 に従い TS-LGX-007 へ、並行・性能・機密・ローカライズは NFR-LGX-001 へ委譲する。本 TS は legixy-embed report コマンド層（`run_report` の集約・`ReportOutput`{links, candidates, summary, **skipped**}/`ReportSummary` の summary 集計・**`skipped = 試行エッジ数 − links.len()` の件数算出（ケース 8a）**・空ストア exit 0・空ストア INFO=stdout（ケース 2）・`--json` の warnings 欄なし C-7・to_text/to_json 決定性・非有限値非出力・read-only）に集中する。**skip 検出・理由別集計（`SkipReasonSummary`）・stderr 集約 Warning は bulk エンジン（DD-007 §6）所有のため、文言観察（ケース 8b）に留め内訳正確性は TS-LGX-007 へ委譲する**。

## 4. テスト技法選択

- 同値分割: ストア状態（空 / links 非空 / スキップ発生 / エラー）、出力形式（text / json）、ReportError バリアント（GraphLoad / ConfigLoad / Db）
- 境界値分析: links 件数 0（None）/ 1（min==max==mean）/ 複数（min/max/mean 算術）の summary 境界（ケース 1, 4, 5）
- Property-based: 出力決定性（バイト一致、ケース 14）、非有限値非出力（NaN/Inf injection、ケース 15）、read-only 不変（ケース 13）
- 状態遷移: なし（report は単一ステートレス計測。状態機械は snapshot 系 = TS-LGX-012 委譲）

## 5. テスト基盤

- 言語: Rust（CLI 本体、`legixy-embed` crate）
- フレームワーク: cargo test
- Property-based: proptest（決定性・非有限値・read-only）
- モック: `EmbeddingStore` / `compute_edge_scores` / `compute_link_candidates` のエンジン出力は fixture またはテスト用 store で供給（算出ロジック自体は TS-LGX-007 が検証）。stdout/stderr 分離は assert_cmd / 子プロセス capture（ケース 8/9）

## 6. 関連 TC

| TS ケース | 対応 TC | 場所 |
|---|---|---|
| ケース 1 | TC-LGX-NNN（empty_store_exit0） | crates/legixy-embed/tests/report_empty.rs |
| ケース 3 | TC-LGX-NNN（empty_json_contract） | crates/legixy-embed/tests/report_json.rs |
| ケース 4 | TC-LGX-NNN（summary_stats） | crates/legixy-embed/src/report.rs（#[cfg(test)]） |
| ケース 7 | TC-LGX-NNN（json_3keys_kind_finite） | crates/legixy-embed/tests/report_json.rs |
| ケース 8a | TC-LGX-NNN（skipped_count_report_layer） | crates/legixy-embed/src/report.rs（#[cfg(test)]） |
| ケース 8b | TC-LGX-NNN（skip_warning_stderr_engine_observed） | crates/legixy-cli/tests/report_skip.rs |
| ケース 9 | TC-LGX-NNN（json_no_warnings_C7） | crates/legixy-cli/tests/report_skip.rs |
| ケース 10 | TC-LGX-NNN（graph_load_err_exit1） | crates/legixy-embed/tests/report_error.rs |
| ケース 13 | TC-LGX-NNN（read_only_invariant） | crates/legixy-embed/tests/report_readonly.rs |
| ケース 14 | TC-LGX-NNN（output_determinism） | crates/legixy-embed/tests/report_property.rs |
| ケース 15 | TC-LGX-NNN（no_nonfinite_in_json） | crates/legixy-embed/tests/report_property.rs |
| ケース 16 | TC-LGX-NNN（no_severity_concept） | crates/legixy-embed/tests/report_json.rs |
