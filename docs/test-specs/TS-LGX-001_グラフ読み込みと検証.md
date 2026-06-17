Document ID: TS-LGX-001

# TS-LGX-001: グラフ読み込みと検証（check）のテスト仕様

> TS は **上流 TP の翻訳**。ゼロから観点を考えない。各 TP 観点を DD-LGX-001 で確定した型・関数シグネチャに即した具体的な入力・期待出力・前提条件へ展開する。

**親 DD**: DD-LGX-001
**継承 TP**: TP-LGX-004（TP[SPEC] 検証、51 観点）, TP-LGX-011（TP[UC] UC-001 フロー）

## 1. 対象範囲

このテスト仕様がカバーする DD-LGX-001 の関数 / 型:

- DD-LGX-001 §3 `legixy_check::run(graph: &TraceGraph, config: &Config, mode: CheckMode, store: Option<&EmbeddingStore>) -> Result<CheckReport, CheckError>`
- DD-LGX-001 §3 `legixy_check::exit_code(report: &CheckReport) -> i32`
- DD-LGX-001 §3 `CheckReport::to_json(&self) -> String`
- DD-LGX-001 §2 型: `CheckResult` / `CheckReport` / `SeverityCounts` / `Severity` / `CheckCategory`(17) / `CheckMode`{Formal,Full} / `CheckError`{GraphLoad,ConfigLoad,Db} / `Location`

委譲（本 TS 対象外）: embedding スコア算出の数値妥当性（→ TS-LGX-007）、性能予算 PERF.02（→ bench / NFR-LGX-001）、並行アクセス整合性 REL.07/08（→ NFR）、サブノード ID 生成式（→ TS-LGX-003）。本 TS は「check が SPEC-004 の規定を DD-001 の型で正しく具体化しているか」を検証する。

## 2. ケース一覧

### ケース 1: 空グラフ（ノード 0 / エッジ 0）→ exit 0

- **観点出典**: TP-LGX-004 §2.1 B1（空グラフの挙動と終了コード）
- **分類**: Unit
- **前提**: `TraceGraph` がノード 0・エッジ 0。`mode = CheckMode::Formal`、`store = None`
- **入力**: `run(&empty_graph, &config, CheckMode::Formal, None)`
- **期待**: `Ok(report)` かつ `report.counts == SeverityCounts{error:0, warning:0, info:0, ok:_}`。`exit_code(&report) == 0`
- **境界条件**: 検証対象ゼロ件 = 正常終了（エラーではない）

### ケース 2: 単一ノード・孤立（エッジ 0）→ OrphanFile 非該当

- **観点出典**: TP-LGX-004 §2.1 B2（単一ノード・孤立ノードの扱い）, R1
- **分類**: Unit
- **前提**: graph.toml に登録された 1 ノード・エッジ 0。ファイル実体は存在
- **入力**: `run(&single_node_graph, &config, CheckMode::Formal, None)`
- **期待**: `OrphanFile` finding を発行しない（OrphanFile は graph.toml **未登録**ファイル対象）。`counts.error == 0`、`exit_code == 0`
- **境界条件**: 孤立ノード（エッジなし登録済）は検証対象外、未登録ファイルとは区別

### ケース 3: graph.toml 未登録ファイルの存在 → OrphanFile finding

- **観点出典**: TP-LGX-004 §2.11 R1（ChainIntegrity vs OrphanFile 差分）
- **分類**: Integration
- **前提**: docs 配下に graph.toml に登録されていない成果物ファイルが 1 件
- **入力**: `run(&graph, &config, CheckMode::Formal, None)`
- **期待**: `findings` に `CheckResult{ category: OrphanFile, ... }` を含む。severity は REQ.15 割当表に従う
- **境界条件**: OrphanFile = 未登録ファイル / ChainIntegrity = chain エッジ整合（検出対象の差分）

### ケース 4: chain エッジ不整合 → ChainIntegrity finding

- **観点出典**: TP-LGX-004 §2.11 R1, TP-LGX-011 §2.1 BF1（ステップ連鎖整合）
- **分類**: Integration
- **前提**: chain order が定義されているが、ある下流ノードに親 chain エッジが欠落
- **入力**: `run(&graph, &config, CheckMode::Formal, None)`
- **期待**: `findings` に `category: ChainIntegrity` の finding。`exit_code` は当該 severity が Error なら 1

### ケース 5: severity=Error が 1 件以上 → exit 1（severity↔exit 一意対応）

- **観点出典**: TP-LGX-004 §2.2 E6（severity↔exit 対応）, §2.3 S1
- **分類**: Unit
- **前提**: `CheckReport.counts.error == 1`（他は任意）
- **入力**: `exit_code(&report)`
- **期待**: `== 1`
- **境界条件**: `counts.error > 0 ⇒ 1`、`== 0 ⇒ 0`（Warning/Info/Ok は exit に影響しない）

### ケース 6: Warning/Info のみ（Error 0）→ exit 0

- **観点出典**: TP-LGX-004 §2.2 E6, §2.3 S3（UnresolvedEdge=Warning は G1 非阻害）
- **分類**: Unit
- **前提**: `counts == {error:0, warning:2, info:1, ok:_}`（例: UnresolvedEdge×2）
- **入力**: `exit_code(&report)`
- **期待**: `== 0`
- **境界条件**: Warning は G1 ゲートを阻害しない

### ケース 7: 部分失敗継続（一部ファイル読込失敗）

- **観点出典**: TP-LGX-004 §2.2 E1/E2, TP-LGX-011 §2.3 EF4
- **分類**: Integration
- **前提**: 登録ノードのうち 1 件のファイル実体が読込失敗、他は正常
- **入力**: `run(&graph, &config, CheckMode::Formal, None)`
- **期待**: `Err` に昇格せず `Ok(report)`。`findings` に `CheckResult{ severity: Error, category: FileExistence }` を含み、他ノードの検査結果も `findings` に存在（継続したこと）
- **境界条件**: ファイル読込失敗は finding 化（`CheckError` に昇格させない、REQ.05）

### ケース 8: graph.toml 破損・パース不能 → Err(GraphLoad) → exit 1

- **観点出典**: TP-LGX-004 §2.2 E3
- **分類**: Integration
- **前提**: graph.toml がパース不能（実行時失敗 = 構文層 clap 誤りではない）
- **入力**: `run(...)`（graph ロード段で失敗）
- **期待**: `Err(CheckError::GraphLoad(_))`。呼出側で exit 1 へ変換
- **境界条件**: 実行時失敗 = exit 1（引数構文誤り exit 2 と区別）

### ケース 9: `--formal`（Formal）と無印（Full）の層差

- **観点出典**: TP-LGX-004 §2.2 E4/E5, TP-LGX-011 §2.1 BF2
- **分類**: Integration
- **前提**: 同一グラフ。`store = None`（embeddings 空相当）
- **入力**: `run(.., CheckMode::Formal, None)` と `run(.., CheckMode::Full, None)`
- **期待**: Formal は `SemanticSimilarity` を発行しない。Full は `store=None` のとき意味層を Info 1 件（embed 誘導、非致命）で完走。両者とも `counts.error` は形式層由来のみで一致
- **境界条件**: `--formal`=段階1+2 / 無印=段階1+2+3

### ケース 10: store=None（engine.db 不在・embeddings 空）→ 意味層 Info 1 件・非致命

- **観点出典**: TP-LGX-004 §2.1 B3, §2.6 P2, §2.8 L1（初回実行誘導）
- **分類**: Unit
- **前提**: `mode = Full`, `store = None`
- **入力**: `run(&graph, &config, CheckMode::Full, None)`
- **期待**: `Ok(report)`。意味層は `Severity::Info` の finding 1 件（embed --all 未実行誘導）。`counts.error` は意味層により増えない。`exit_code` 非影響（FB-INV-4）
- **境界条件**: DB 不在 = embeddings 空の上位ケース、同一の degraded・非致命帰結

### ケース 11: SUBNODE-INV 違反は必ず Error

- **観点出典**: TP-LGX-004 §2.3 S2
- **分類**: Unit
- **前提**: SubnodeId 系（IdFormat/Uniqueness/ParentIntegrity/Dag のいずれか）違反を含むグラフ
- **入力**: `run(...)`
- **期待**: 当該 `CheckResult.severity == Severity::Error`。`counts.error >= 1`、`exit_code == 1`。`SubnodeDag` 違反の場合は `category == SubnodeDag`（SUBNODE-INV-4、サブノード関与エッジのサイクル）であり、グラフ全体サイクルの `GraphDag`（CTX-INV-4、ケース 19）とは別カテゴリ
- **境界条件**: INV-1/2/3/4/6 違反 = 必ず Error（REQ.07）。DAG はサブノード関与エッジ（SubnodeDag）に限定し、グラフ全体エッジのサイクルは GraphDag に分離（ケース 19 で別途検証）

### ケース 12: SubnodeIdCollision は Warning（自動生成縮退のみ）

- **観点出典**: TP-LGX-004 §2.3 S4, §2.11 R7, §2.5 V4（v3 後方互換）
- **分類**: Unit
- **前提**: 自動生成サブノード同士の ID 縮退が発生
- **入力**: `run(...)`
- **期待**: `CheckResult{ category: SubnodeIdCollision, severity: Severity::Warning }`。明示ノード衝突は本カテゴリの対象外。`exit_code == 0`（G1 非阻害）
- **境界条件**: Uniqueness(INV-3=Error) と Collision(自動縮退=Warning) の役割分担

### ケース 13: IdRedefined / IdSemanticMismatch / IdSemanticDrift は既定 OFF

- **観点出典**: TP-LGX-004 §2.3 S5, §2.5 V5（デフォルト OFF 互換）
- **分類**: Unit
- **前提**: config でこれらの opt-in 検査が未設定（既定 false）
- **入力**: `run(...)`
- **期待**: これらカテゴリの finding を発行しない（既定 OFF）
- **境界条件**: opt-in 検査の既定無効による後方互換

### ケース 14: CheckReport の安定ソート決定性（property）

- **観点出典**: TP-LGX-004 §2.10 D1（formal 冪等性）
- **分類**: Property-based（proptest）
- **生成器**: 任意の `Vec<CheckResult>` を構築するが、tiebreaker を必ず exercise する制約を課す。(a) **同一 severity を必ず複数含む**（例: `Severity::Error` を 2 件以上生成し、第1キー〔severity 降順〕だけでは順序が確定しない状態を保証）、(b) 同一 severity 内で `category`（CheckCategory 17 種から複数値）を変動させ第2キー（category）の tiebreaker を発火、(c) 同一 (severity, category) 内で `related_ids`（複数の `Id` 列）を変動させ第3キー（related_ids）の tiebreaker を発火。この `Vec<CheckResult>` から、要素を任意順に並べ替えた複数の同値入力を構築する
- **不変条件**: 同一の finding 集合に対し `run(.., Formal, None)`（および `CheckReport` 構築）は常に同一順序の `findings` を返す。ソートキーは severity 降順 → category → related_ids（REQ.06）。検証は (i) `findings` の `(severity, category, related_ids)` タプル列が入力並び替えに依らず完全一致、(ii) 加えて `to_json` 出力がバイト一致、の両方を確認する（タプル列一致は第2・第3キーの安定性を直接 assert し、バイト一致だけでは見逃しうる category/related_ids tiebreaker の取りこぼしを塞ぐ）
- **反例ハンドリング**: shrink して最小の順序不一致例（特に同一 severity 内で category または related_ids の順序が崩れる最小例）を記録

### ケース 15: 終了コード契約 0/1/2（LGX-COMPAT-001 凍結）

- **観点出典**: TP-LGX-004 §2.5 V1/V2
- **分類**: Contract
- **前提**: (a) Error 0 件、(b) Error 1 件以上、(c) 引数構文誤り（clap 層）
- **入力**: それぞれ `exit_code` / CLI ディスパッチ
- **期待**: (a)→0、(b)→1、(c)→2。値の意味的不正（破損 graph 等）は exit 1（exit 2 ではない）
- **境界条件**: exit 2 は構文層限定、意味的不正は exit 1

### ケース 16: 出力先分離（CheckReport=stdout / ログ=stderr）

- **観点出典**: TP-LGX-004 §2.9 O1/O2
- **分類**: Integration
- **前提**: 任意のグラフ
- **入力**: CLI 実行（`check` / `check --log-format=json`）
- **期待**: CheckReport は stdout、診断ログは stderr。`--log-format=json` で JSON Lines 出力（`to_json`）
- **境界条件**: stdout/stderr のチャネル分離（OBS.02）

### ケース 17: finding message の情報量（related_ids / location）

- **観点出典**: TP-LGX-004 §2.9 O3
- **分類**: Unit
- **前提**: 任意の finding を発生させる入力
- **入力**: 発行された `CheckResult`
- **期待**: `related_ids` が非空で当該 ID を含み、`location`（path + 行）が特定可能。message が「何が・どこで・どの ID」を含む
- **境界条件**: 観測可能性（O3）

### ケース 18: read-only 不変（check はグラフ/DB を変更しない）

- **観点出典**: TP-LGX-004 §2.4 C1, TP-LGX-011 §2.3 EF2
- **分類**: Property/Integration
- **前提**: 任意の入力（成功・部分失敗いずれも）
- **入力**: `run(...)` 実行前後の graph / engine.db のハッシュ
- **期待**: 実行前後で graph・engine.db が不変（借用のみ、§5 read-only）。エラー時も中間状態破壊なし
- **境界条件**: 借用（`&TraceGraph` / `Option<&EmbeddingStore>`）による read-only 保証

### ケース 19: GraphDag（グラフ全体サイクル）→ category=GraphDag / severity=Error（SubnodeDag と区別）

- **観点出典**: TP-LGX-004 §2.11 R3（DAG 違反 severity）, §2.3 S1（severity 割当完全性）, TP-LGX-011 §2.6 R2（DAG=CTX-INV-4 の形式検査対応）
- **分類**: Integration
- **前提**: **通常ノード**（サブノードを含まない）から成るグラフに、chain / custom / parentchild の**全エッジ種別を含むグラフ全体のサイクル**を構成する fixture（例: `A --chain--> B --custom--> C --parentchild--> A`）。サブノード関与エッジは一切含まない（SubnodeDag を発火させないため）。`mode = CheckMode::Formal`、`store = None`
- **入力**: `run(&cyclic_graph, &config, CheckMode::Formal, None)`
- **期待**: `findings` に `CheckResult{ category: CheckCategory::GraphDag, severity: Severity::Error, .. }` を 1 件以上含む（CTX-INV-4 違反、SPEC-LGX-004.REQ.01 / REQ.15 割当表で Error 固定）。`counts.error >= 1`、`exit_code(&report) == 1`。同入力で `SubnodeDag` カテゴリの finding は**発行されない**（サブノード関与エッジが無いため）
- **境界条件**: GraphDag（CTX-INV-4、グラフ全体・全エッジ種別）と SubnodeDag（SUBNODE-INV-4、サブノード関与エッジ限定、ケース 11）はカテゴリが別。v3 はグラフ全体サイクルも `SubnodeDag` 名で報告していたが、legixy では `GraphDag` に分離する（【v3 差分】SPEC-LGX-004.REQ.01 注記）。本ケースは名目 mapping ではなく、実際に `category == GraphDag` を assert して両者の分離を検証する

### ケース 20: DocumentId（不一致 / 行欠落）→ いずれも severity=Error

- **観点出典**: TP-LGX-004 §2.11 R2（DocumentId 不一致 severity）, §2.3 S1（severity 割当完全性）, TP-LGX-011 §2.6 R2（Document ID 一致の形式検査対応）
- **分類**: Integration
- **前提**: 2 つの fixture を用意する。(a) **不一致**: ファイル先頭の `Document ID:` 行の値が graph.toml に登録された当該ノードの ID と異なる。(b) **行欠落**: ファイルに `Document ID:` 行自体が存在しない。両 fixture とも他カテゴリ違反を含まず DocumentId のみを発火させる。`mode = CheckMode::Formal`
- **入力**: 各 fixture に対し `run(&graph, &config, CheckMode::Formal, None)`
- **期待**: いずれの fixture でも `findings` に `CheckResult{ category: CheckCategory::DocumentId, severity: Severity::Error, .. }` を含む（SPEC-LGX-004.REQ.15: 不一致=Error 固定 / 行欠落=Error 固定、人間裁定 2026-06-10・v3 実挙動と一致・ハードルール 4(c) と整合）。両ケースとも `counts.error >= 1`、`exit_code == 1`
- **境界条件**: 不一致・行欠落の**両方**が Error（Warning ではない）。REQ.15 割当表が 2 行（DocumentId 不一致 / DocumentId 行欠落）でいずれも Error に pin している点を、両 fixture それぞれで `severity == Error` を assert して検証する（ケース 11 の「S1 系」曖昧 mapping を解消する DocumentId 専用ケース）

## 3. 観点カバレッジ表

| TP § | 観点 | カバーする TS ケース |
|---|---|---|
| TP-004 §2.1 B1 空グラフ | 境界値 | ケース 1 |
| TP-004 §2.1 B2 単一/孤立 | 境界値 | ケース 2 |
| TP-004 §2.1 B3 embeddings 空 | 境界値 | ケース 10 |
| TP-004 §2.1 B4/B5/B6/B7 閾値・drift 境界 | 境界値 | TS-LGX-007 へ委譲（意味層スコア） |
| TP-004 §2.1 B8 性能規模 | 境界値 | NFR-LGX-001 / bench へ委譲 |
| TP-004 §2.2 E1/E2 部分失敗継続 | エラー | ケース 7 |
| TP-004 §2.2 E3 graph 破損 | エラー | ケース 8 |
| TP-004 §2.2 E4/E5 モデル不在・非致命 | エラー | ケース 9, 10 |
| TP-004 §2.2 E6 severity↔exit | エラー | ケース 5, 6 |
| TP-004 §2.3 S1 severity 割当完全性 | 状態 | ケース 5, 11, 12（カテゴリ別 severity） |
| TP-004 §2.3 S2 INV 違反 Error | 状態 | ケース 11 |
| TP-004 §2.3 S3 UnresolvedEdge Warning | 状態 | ケース 6 |
| TP-004 §2.3 S4 Collision 境界 | 状態 | ケース 12 |
| TP-004 §2.3 S5 Id 系 severity/既定 | 状態 | ケース 13 |
| TP-004 §2.3 S6 INV-5 対象外 | 状態 | TS-LGX-003 へ委譲（生成側保証） |
| TP-004 §2.3 S7 Ok 使用条件 | 状態 | ケース 1（counts.ok）, 14（シリアライズ） |
| TP-004 §2.4 C1/C2 並行性 | 並行 | ケース 18（read-only）+ NFR REL.07 委譲 |
| TP-004 §2.5 V1/V2 終了コード契約 | 互換 | ケース 15 |
| TP-004 §2.5 V3 新カテゴリ G1 非阻害 | 互換 | ケース 6, 12 |
| TP-004 §2.5 V4/V5 後方互換 | 互換 | ケース 12, 13 |
| TP-004 §2.5 V6 設定ソース | 互換 | SPEC-008.REQ.13 委譲（config 探索） |
| TP-004 §2.6 P1/P2/P3 永続化 | 永続化 | ケース 10（P2）/ TS-LGX-007（P1/P3 drift） |
| TP-004 §2.7 I1〜I4 入力検証 | 入力 | ケース 11（I1）/ TS-LGX-003・007 委譲 |
| TP-004 §2.8 L1/L2 ライフサイクル | ライフ | ケース 10（L1）|
| TP-004 §2.9 O1〜O4 観測性 | 観測 | ケース 16（O1/O2）, 17（O3）, 9（O4 生スコア非出力） |
| TP-004 §2.10 D1/D2 冪等性 | 決定性 | ケース 14 |
| TP-004 §2.11 R1 Chain/Orphan 差分 | 領域 | ケース 2, 3, 4 |
| TP-004 §2.11 R2 DocumentId severity | 領域 | ケース 20（不一致=Error / 行欠落=Error） |
| TP-004 §2.11 R3 DAG severity | 領域 | ケース 19（GraphDag=Error）, 11（SubnodeDag=Error） |
| TP-004 §2.11 R4 SemanticSimilarity 対象 | 領域 | TS-LGX-007 委譲 |
| TP-004 §2.11 R5/R6 check/report・drift 責務 | 領域 | ケース 9（責務境界）/ TS-LGX-010・013 |
| TP-004 §2.11 R7 Uniqueness/Collision 分担 | 領域 | ケース 11, 12 |
| TP-011 §2.1 BF1 ステップ連鎖整合 | UC フロー | ケース 4（chain エッジ整合で連鎖の事後条件→前提を検証） |
| TP-011 §2.1 BF2 段階区分（formal/full）の観察可能性 | UC フロー | ケース 9（Formal=段階1+2 / Full=段階1+2+3 の層差を観察） |
| TP-011 §2.1 BF3 成功時事後条件（stdout 出力・exit） | UC フロー | ケース 16（CheckReport=stdout）+ ケース 1（exit 0）/ ケース 5（exit 1） |
| TP-011 §2.1 BF4 9 サブ検証の実行モデル（全件実行・部分失敗継続） | UC フロー | ケース 7（部分失敗継続で他検査結果も findings に存在） |
| TP-011 §2.2 AF1 分岐網羅（opt-in ON/OFF） | UC フロー | ケース 13（opt-in 既定 OFF）。ON 側数値判定は TS-LGX-007 委譲 |
| TP-011 §2.2 AF2 前提崩壊系の exit 収束 | UC フロー | ケース 8（graph 破損→GraphLoad→exit 1）+ ケース 15（exit 契約） |
| TP-011 §2.2 AF3 遷移条件の明示（発火条件） | UC フロー | ケース 13（opt-in 既定 OFF の発火条件＝config 設定で観察）。フロー記述自体は UC-LGX-001 所有 |
| TP-011 §2.3 EF1 破損・パース不能の失敗パス | UC フロー | ケース 8（graph.toml 破損→Err(GraphLoad)→exit 1） |
| TP-011 §2.3 EF2 エラー時状態の不変条件保持 | UC フロー | ケース 18（read-only、エラー時も中間状態破壊なし） |
| TP-011 §2.3 EF3 エラー通知（severity 区分） | UC フロー | ケース 17（finding message 情報量）+ severity 区分は ケース 5/6/11/12 |
| TP-011 §2.3 EF4 部分失敗継続のフロー反映 | UC フロー | ケース 7（読込失敗を FileExistence Error finding 化し継続） |
| TP-011 §2.4 AT1 アクター権限の一貫性 | UC フロー | ケース 18（read-only＝開発者/CI 同一権限で実行可） |
| TP-011 §2.4 AT2 責任境界（検出 vs 是正） | UC フロー | ケース 17（finding=システム検出。是正はアクター責務で TS 対象外） |
| TP-011 §2.4 AT3 実行中外部更新の整合性前提 | UC フロー | NFR-LGX-001.REL.07（SQLite busy_timeout）へ委譲（並行アクセス） |
| TP-011 §2.5 DF1 入出力型・stdout/stderr 分離 | UC フロー | ケース 16（CheckReport=stdout / ログ=stderr） |
| TP-011 §2.5 DF2 embedding ライフタイム（不在＝非致命） | UC フロー | ケース 10（store=None で意味層 Info 1 件・非致命） |
| TP-011 §2.5 DF3 エラー時データ解放 | UC フロー | ケース 18（read-only・永続リソース確保解放を伴わない） |
| TP-011 §2.6 R1 formal/意味層の責務境界 | UC フロー | ケース 9（Formal/Full の責務差）。意味層数値は TS-LGX-007 委譲 |
| TP-011 §2.6 R2 形式検査カテゴリ対応 | UC フロー | ケース 3（OrphanFile）, 4（ChainIntegrity）, 19（GraphDag）, 20（DocumentId）, 11（Subnode 系） |
| TP-011 §2.6 R3 opt-in 既定 OFF の観察可能性 | UC フロー | ケース 13（IdRedefined/Mismatch/Drift 既定 OFF） |
| TP-011 §2.6 R4 終了コード契約一致（0/1/2） | UC フロー | ケース 15（exit 0/1/2 契約） |
| TP-011 §2.6 R5 severity 4 段階一致 | UC フロー | ケース 11（Error）, 12（Warning）, 6（Warning/Info）, 5（Error→exit）。Ok は ケース 1（counts.ok） |

> 継承 TP 観点はすべて本テーブルで TS ケースまたは明示委譲先に mapping 済み（人間ゲート判断対象）。TP-011 の全 22 観点（BF1〜4 / AF1〜3 / EF1〜4 / AT1〜3 / DF1〜3 / R1〜5）を実カバレッジ単位で 1:1 対応させた（名目 mapping ではなく、当該観点を実際に exercise するケースへ割当）。意味層スコア・性能・並行・サブノード生成式は責務上 TS-LGX-003/007 / NFR へ委譲し、本 TS は legixy-check の finding 生成・severity・exit・決定性・read-only に集中する。
