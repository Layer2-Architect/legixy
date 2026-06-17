Document ID: TS-LGX-008

# TS-LGX-008: フィードバックループ（observe / feedback / analyze / approve / reject / proposals / audit）のテスト仕様

> TS は **上流 TP の翻訳**。ゼロから観点を考えない。各 TP 観点を DD-LGX-008 で確定した型・関数シグネチャに即した具体的な入力・期待出力・前提条件へ展開する。

**親 DD**: DD-LGX-008
**継承 TP**: TP-LGX-007（TP[SPEC] フィードバックループ、49 観点）, TP-LGX-018（TP[UC] UC-008 フロー、22 観点）

## 1. 対象範囲

このテスト仕様がカバーする DD-LGX-008 の関数 / 型:

- DD-LGX-008 §3.1 `ObservationRecorder::record(obs: &NewObservation, db: &Connection) -> Result<RecordResult, FeedbackError>`
- DD-LGX-008 §3.1 `AutoObserver::from_check_results(report: &CheckReport) -> Vec<NewObservation>`
- DD-LGX-008 §3.1 `drift_from_embed_error(err: &EmbedError, node_id: &str) -> Option<NewObservation>`
- DD-LGX-008 §3.1 `ProposalAnalyzer::analyze(db: &Connection) -> Result<Vec<Proposal>, FeedbackError>`
- DD-LGX-008 §3.1 `ProposalManager::approve(proposal_id: i64, db: &Connection) -> Result<(), FeedbackError>`
- DD-LGX-008 §3.1 `ProposalManager::reject(proposal_id: i64, reason: &str, db: &Connection) -> Result<(), FeedbackError>`
- DD-LGX-008 §3.1 `FeedbackCli::run_feedback` / `run_analyze` / `list_proposals` / `approve` / `reject`
- DD-LGX-008 §3.1 `ContextAuditReader::recent(db, limit)` / `by_target(db, target_id, limit)`
- DD-LGX-008 §3.2 TS MCP 転送層 `parseObserveStdout` / `formatAuditEntry`（zod スキーマ・CLI 変換）
- DD-LGX-008 §2 型: `NewObservation` / `Observation` / `Proposal` / `RecordResult` / `FeedbackReport` / `ProposalSummary` / `ContextLogEntry` / `ObservationStatus`{Pending,Analyzing,Resolved,Skipped} / `ProposalStatus`{Pending,Approved,Rejected} / `ObserveCategoryInput`(3) / `FeedbackCategory`(5) / `FeedbackError`(8 variant)

委譲（本 TS 対象外）:
- observe 並行 1 件格納の負荷検証（C1）・approve/reject CAS 競合の並行決着（C2/C3）→ ケース化はするが **負荷ストレス側面は NFR-LGX-001 REL.07 / SEC.02 へ委譲**（本 TS はロジック単位の単発検証）
- SQLite busy_timeout 上限超過時の失敗方針（E4）→ NFR-LGX-001 REL.07 へ委譲
- engine.db のネットワーク共有検出（P4）→ NFR-LGX-001 REL.08 へ委譲
- kill / 電源断回復（P5）→ STATE-INV-1 / NFR REL.01 へ委譲
- compile_context の全呼出し context_log 記録（O1, REQ.06）・get_compile_audit 加工なし転送本体（O2）・監査ログ順序契約（O4）→ context_log 書込本体は **SPEC-LGX-003 / TS-LGX-003 主導**（本 TS は `ContextAuditReader` 読取と `formatAuditEntry` 整形のみ検証）
- テスト不可侵 role 別書込制御（D3, MAINT.05）→ パイプラインフック（legixy 外）へ委譲
- 機密マスキング（O5, SEC.05）→ `mask_api_key` 適用は本 TS で検証（drift message）、ダンプ検査の網羅は NFR SEC.05 へ委譲

本 TS は「フィードバックループが SPEC-007 v0.6.0 の規定（状態モデル 4 値・3 値、skipped 終端、trim 空拒否、不正カテゴリ exit2、dedup、CAS）を DD-008 の型で正しく具体化しているか」を検証する。

## 2. ケース一覧

### ケース 1: message が trim 後 0 文字（空文字 / 空白のみ）→ exit 1（【v3 差分】）

- **観点出典**: TP-LGX-007 §2.1 B1（message 境界）, §2.7 I（入力検証）
- **分類**: Unit
- **前提**: `NewObservation{ category:"manual_note", message:"   ", ... }`（trim 後 0 文字）。CLI 層の検証経路
- **入力**: observe の message 値 `""` / `"   "` / `"\t\n"`（3 同値）
- **期待**: `FeedbackError::EmptyObservationMessage` → exit 1。`observations` への INSERT は発生しない
- **境界条件**: 【v3 差分】v3 は `is_empty()` のみ（空白のみは通過）→ legixy は `trim().is_empty()` で拒否。空文字列 = 構文上 valid な位置引数のため exit 2 ではなく exit 1

### ケース 2: message が trim 後 1 文字以上（境界下限）→ 受理

- **観点出典**: TP-LGX-007 §2.1 B1（境界値: 最小許容）
- **分類**: Unit
- **前提**: `NewObservation{ message:" a ", ... }`（trim 後 1 文字）
- **入力**: `record(&obs, &db)`
- **期待**: `Ok(RecordResult{ id: >0, skipped: false })`。message は **無加工**で保存（前後空白・改行・Unicode を正規化しない、REQ.01）
- **境界条件**: trim 判定は受理可否のみ。保存値はサニタイズしない（最大長なし）

### ケース 3: 同一 (category, related_ids) の重複 observe → 2 件目は skipped=true（dedup）

- **観点出典**: TP-LGX-007 §2.4 C1, §2.8 L2, FB-INV-1（REQ.11）
- **分類**: Integration
- **前提**: 1 件目 INSERT 後 status='pending'。2 件目は同一 category・同一 related_ids（message は相違でも可）
- **入力**: `record(&obs1, &db)` 後に `record(&obs2, &db)`
- **期待**: 1 件目 `RecordResult{ skipped:false }`、2 件目 `RecordResult{ skipped:true }`（INSERT スキップ）。`observations` 行数 1。dedup キーは distinct→昇順 sort→JSON した related_ids（message 非包含）
- **境界条件**: 適用範囲 `status IN ('pending','analyzing')`。resolved/skipped 後は同一キーで再受理（ケース 14 と接続）

### ケース 4: related_ids の distinct→sort 正準化の決定性（property）

- **観点出典**: TP-LGX-007 §2.1 B2, §2.7 I2（related_id 多数/重複/不在）, C1 正準化
- **分類**: Property-based（proptest）
- **生成器**: 任意の `Vec<String>`（重複・順不同・存在しないノード ID を含む）を related_ids に持つ `NewObservation` の同値集合
- **不変条件**: 同一「内容」（集合として等価）の related_ids は、要素順・重複に関わらず常に同一の正準 JSON に正規化される（distinct→昇順 sort→JSON）。これにより同一 dedup キーへ収束する。related_id の **実在検証は行わない**（未登録 ID も受理して保存、REQ.01）
- **反例ハンドリング**: shrink して正準化が不一致になる最小 Vec を記録

### ケース 5: 重複 observe の dedup 冪等性（property）

- **観点出典**: TP-LGX-007 §2.4 C1, FB-INV-1, DD §8 Property C1
- **分類**: Property-based（proptest）
- **生成器**: 同一 `NewObservation` を N 回（N=1..20 ランダム）`record` する操作列
- **不変条件**: N 回呼出し後も `observations` テーブルの該当キー行数は常に 1（最初の 1 件のみ INSERT、残りは `skipped:true`）
- **反例ハンドリング**: shrink して最小 N で行数 >1 になる例を記録

### ケース 6: 不正 category 値 → exit 2（clap ValueEnum、【v3 差分】）

- **観点出典**: TP-LGX-007 §2.2 E1, §2.7 I1, §2.5 V2, TP-LGX-018 §2.4（Surface 入口検証）
- **分類**: Contract（CLI 層）
- **前提**: `ObserveCategoryInput` は 3 値（`compile_miss`/`review_correction`/`manual_note`）。CLI 入口
- **入力**: `observe foobar "msg"`（凍結 3 値以外の category）
- **期待**: clap ValueEnum 違反 → **exit 2**（使用法誤り）。DB INSERT は発生しない
- **境界条件**: 【v3 差分】v3 は `category: String` 無検証で保存後 analyze で死蔵。legixy は二層検証（MCP zod enum + CLI ValueEnum）の CLI 側。正当 3 値は exit 0

### ケース 7: AutoObserver フィルタ規則（CheckReport → NewObservation 列）

- **観点出典**: TP-LGX-018 §2.1 BF3（check カテゴリ→observation カテゴリ）, §2.6 R1
- **分類**: Unit
- **前提**: `CheckReport` に severity=Ok / FileExistence×Error / DocumentId×Warning / 既知 5 カテゴリ（ChainIntegrity/LinkCandidate/Drift/OrphanFile/SemanticSimilarity）を混在
- **入力**: `from_check_results(&report)`
- **期待**: severity=Ok を除外 / FileExistence×Error を除外 / DocumentId×Warning を除外 / 既知 5 カテゴリのみ `NewObservation` 化。read-only（report 不変）。message に `mask_api_key` 適用
- **境界条件**: 既知 5 カテゴリ = `FeedbackCategory`（REQ.01 の 3 値 `ObserveCategoryInput` とは別集合、混同禁止）

### ケース 8: drift_from_embed_error は ContextualRetrievalFailed のみ Some

- **観点出典**: TP-LGX-007 §2.9 O5, SEC.05（マスキング）, TP-LGX-018 §2.1 BF3
- **分類**: Unit
- **前提**: `EmbedError::ContextualRetrievalFailed`（drift 該当）と他の `EmbedError` variant
- **入力**: `drift_from_embed_error(&err, "SPEC-LGX-007")`
- **期待**: ContextualRetrievalFailed のとき `Some(NewObservation{ category:"drift", ... })`、他 variant は `None`。message は必ず `mask_api_key` を通過（API キー文字列が混入しない）
- **境界条件**: drift カテゴリ生成は EmbedError 種別で分岐。マスキング必須（SEC.05）

### ケース 9: analyze の Pessimistic Claim と変換可能カテゴリ → Resolved 遷移

- **観点出典**: TP-LGX-007 §2.3 S4, TP-LGX-018 §2.1 BF4/BF5, §2.6 R1
- **分類**: Integration
- **前提**: pending observation（category=chain_integrity、変換規則 `add_chain_entry` あり）。proposal 未生成
- **入力**: `analyze(&db)` → 生成 Proposal を `approve(proposal.id, &db)`
- **期待**: analyze で pending→analyzing（single tx Claim）→ `add_chain_entry` Proposal を INSERT（status=pending）。戻り値は新規 INSERT した Proposal のみ。approve tx 内で対応 observation を **Resolved** へ UPDATE（FB-INV-2 連動。【v3 差分】v3 は approve が observation に波及しない）
- **境界条件**: category→kind 変換: chain_integrity→add_chain_entry / link_candidate→add_link / drift→update_doc

### ケース 10: 変換規則なしカテゴリ → Skipped 終端（【v3 差分】、永久再 claim 解消）

- **観点出典**: TP-LGX-007 §2.3 S4, TP-LGX-018 §2.6 R2（skipped 発火条件）
- **分類**: Integration
- **前提**: pending observation（category=orphan_file または semantic_similarity。Proposal 変換規則を持たない）
- **入力**: `analyze(&db)`（複数回連続実行）
- **期待**: 当該 observation は analyzing 経由で `ObservationStatus::Skipped`（終端）へ。Proposal は生成しない。**2 回目以降の analyze は skipped を再取込しない**（pending↔analyzing 往復＝永久再 claim が起きない）
- **境界条件**: 【v3 差分】v3 は変換不能を pending に戻し死蔵。SPEC-007 v0.6.0（ADR-LGX-019）で skipped 終端追加。Skipped は不可逆・終端

### ケース 11: analyze 中の単一 observation 処理失敗 → Claim Release（pending 復帰）

- **観点出典**: TP-LGX-007 §2.3 S4, TP-LGX-018 §2.2 AF2（claim release 発火条件）
- **分類**: Integration
- **前提**: analyzing 状態の observation の変換処理が失敗（`AnalyzeFailed` 相当）
- **入力**: `analyze(&db)`（処理中に当該 observation で失敗注入）
- **期待**: 当該 observation は `analyzing` → **`pending`** へ戻る（Claim Release、再分析対象）。`FeedbackError::AnalyzeFailed{ observation_id, detail }`。Skipped/Resolved にはしない
- **境界条件**: reject または一時的失敗 → pending（SPEC REQ.08）。終端 2 値（Resolved/Skipped）との区別

### ケース 12: 同一 semantic_key の pending proposal が既存 → INSERT 抑止（FB-INV-5）

- **観点出典**: TP-LGX-007 §2.4 C3, FB-INV-5（REQ.09）
- **分類**: Integration
- **前提**: 既に status='pending' の Proposal が semantic_key（例 `add_chain_entry:{missing_id}`）で存在
- **入力**: 同一 semantic_key を生む observation で `analyze(&db)` 再実行
- **期待**: 新規 INSERT を抑止。`analyze` 戻り値（新規 Proposal）に当該キーを含まない。proposals 行数は増えない
- **境界条件**: semantic_key 3 形式: `add_chain_entry:{id}` / `add_link:{a}:{b}`（辞書順ソート） / `update_doc:{id}`。pending 限定（終端後は再生成可）

### ケース 13: approve の状態遷移（pending → approved・終端不可逆・CAS）

- **観点出典**: TP-LGX-007 §2.3 S1/S2/S3, §2.4 C2, TP-LGX-018 §2.3 EF3
- **分類**: Unit/Integration
- **前提**: (a) status='pending' の Proposal、(b) 既に approved/rejected な Proposal
- **入力**: (a) `approve(id, &db)`、(b) 終端状態へ再 `approve(id, &db)`
- **期待**: (a) CAS `UPDATE ... WHERE status='pending'` updated_rows=1 → `Ok(())`、observation Resolved 連動。(b) updated_rows=0 → `InvalidProposalStatus{ id, expected:"pending", actual:"approved"|"rejected" }` → **exit 1**
- **境界条件**: 許容遷移は pending→{approved|rejected} のみ。approved→rejected / rejected→approved / approved→approved 等は全て禁止（終端不可逆）

### ケース 14: 状態遷移の網羅（許容遷移 / 禁止遷移の全列挙、property/状態機械）

- **観点出典**: TP-LGX-007 §2.3 S1/S2/S3/S4, TP-LGX-018 §2.1 BF4
- **分類**: Property/状態遷移
- **前提**: `ObservationStatus`(4) と `ProposalStatus`(3) の各状態を起点
- **入力**: 各状態 × 各操作（observe/analyze取込/proposal approve/reject/claim release）の全組合せ
- **期待（observation 許容）**: (無)→Pending / Pending→Analyzing / Analyzing→Resolved（approve 経由） / Analyzing→Skipped（変換不能） / Analyzing→Pending（claim release）。**禁止**: Resolved/Skipped からの遷移（終端不可逆）、Pending→Resolved 直行、Pending→Skipped 直行
- **期待（proposal 許容）**: (無)→Pending / Pending→Approved / Pending→Rejected。**禁止**: Approved/Rejected からの全遷移、Pending↔ 以外
- **境界条件**: 終端状態（Resolved/Skipped、Approved/Rejected）への遷移試行は CAS 行数 0 → exit 1 または no-op で観測。`from_str`/`as_str` ラウンドトリップ一致

### ケース 15: reject の reason が trim 後 0 文字 → EmptyRejectReason exit 1（【v3 差分】）

- **観点出典**: TP-LGX-007 §2.1 B5（reason 境界）, §2.2 E2, TP-LGX-018 §2.5 DF3
- **分類**: Unit
- **前提**: status='pending' の Proposal。reason = `""` / `"  "` / `"\n"`（trim 後 0 文字）
- **入力**: `reject(id, "  ", &db)`
- **期待**: `FeedbackError::EmptyRejectReason` → exit 1。CAS UPDATE を実行せず（status 不変）
- **境界条件**: 【v3 差分】v3 は `is_empty()` のみ（空白のみ通過）。legixy は `trim().is_empty()`。reason は監査証跡のため空理由を「指定あり」とみなさない

### ケース 16: reject 成功 → rejected・observation pending 復帰・decided_reason 格納

- **観点出典**: TP-LGX-007 §2.3 S1, §2.9 O3, TP-LGX-018 §2.5 DF3
- **分類**: Integration
- **前提**: status='pending' Proposal（対応 observation は analyzing）。reason = `"重複提案"`
- **入力**: `reject(id, "重複提案", &db)`
- **期待**: CAS で Proposal status='rejected'、`decided_at` = datetime('now')、`decided_reason` = "重複提案"。対応 observation は **pending** に戻す（SPEC REQ.08 "reject → pending"。approve の Resolved とは非対称）
- **境界条件**: approve→observation Resolved / reject→observation pending（連動先が異なる）

### ケース 17: 不在 proposal-id への approve/reject → ProposalNotFound exit 1

- **観点出典**: TP-LGX-007 §2.1 B3, §2.2 E2, §2.7 I3, TP-LGX-018 §2.3 EF3
- **分類**: Unit
- **前提**: 受理済み i64（構文上 valid）だが proposals に存在しない id（0 / 負 / i64 上限 / 未登録）
- **入力**: `approve(99999, &db)` / `reject(99999, "r", &db)`
- **期待**: CAS updated_rows=0 → SELECT で行不在確認 → `ProposalNotFound{ id }` → **exit 1**（意味的不正）
- **境界条件**: i64 パース失敗（構文誤り）は CLI 層 exit 2。受理済み i64 の値不正（不在）は exit 1（区別。LGX-COMPAT-001 §3）

### ケース 18: 並行 observe で同一キー → 1 件のみ INSERT（UNIQUE 制約 fallback）

- **観点出典**: TP-LGX-007 §2.4 C1, §2.6 P2, MCP-INV-3（REQ.11）, DD §8 Concurrent
- **分類**: Concurrent（Integration）
- **前提**: 同一 (category, related_ids) の observation を複数スレッドから同時 `record`。UNIQUE INDEX `idx_obs_dedup`
- **入力**: N スレッドが同時に `record(&same_obs, &db)`
- **期待**: 1 件のみ INSERT 成立、他は UNIQUE 制約違反 → SELECT fallback で `skipped:true`。`BUSY` を呼出側に返さず内部吸収（busy_timeout=5000）
- **境界条件**: 負荷上限・タイムアウト超過挙動は **NFR-LGX-001 REL.07 / SEC.02 へ委譲**。本ケースはロジック（1 件格納）の検証

### ケース 19: 並行 approve vs reject → CAS で 1 つだけ成立・敗者 exit 1

- **観点出典**: TP-LGX-007 §2.4 C2, FB-INV-2, TP-LGX-018 §2.3 EF3
- **分類**: Concurrent（Integration）
- **前提**: 同一 pending Proposal に approve と reject を同時実行
- **入力**: thread A `approve(id)` / thread B `reject(id, "r")`
- **期待**: CAS `WHERE status='pending'` により updated_rows=1 が片方のみ成立。敗者は updated_rows=0 → `InvalidProposalStatus{ expected:"pending", actual:<勝者値> }` → **exit 1**。最終状態は approved または rejected の一方
- **境界条件**: last-writer ではなく first-writer（CAS）。原子性の負荷側面は NFR REL.07 へ委譲

### ケース 20: feedback E2E（CheckReport → Observation 生成・skipped カウント）

- **観点出典**: TP-LGX-018 §2.1 BF1/BF2/BF3, §2.2 AF1, §2.4 AT1
- **分類**: Integration
- **前提**: `CheckReport`（既知 5 カテゴリ + 除外対象を含む）。空グラフ含む 2 fixture
- **入力**: `run_feedback(&db, &check_report)`
- **期待**: `FeedbackReport{ observations_created, observations_skipped }`。AutoObserver でフィルタ → 各 record → 集計。dedup で既存と衝突した分は `observations_skipped` にカウント。該当カテゴリ 0 件（代替フロー 1a）→ created=0, skipped=0, exit 0
- **境界条件**: feedback の入力は CheckReport を **引数で受け取る**（BF2 ギャップの DD 確定: 自動読取でなく引数）。空 CheckReport → 生成 0 件は正常終了

### ケース 21: analyze E2E（pending 0 件 → proposal 0 件・exit 0）

- **観点出典**: TP-LGX-018 §2.2 AF1/AF3, §2.6 R3, TP-LGX-007 §2.8 L1
- **分類**: Integration
- **前提**: observations テーブルが空（または pending 0 件）
- **入力**: `run_analyze(&db)`
- **期待**: `Ok(vec![])`（生成 proposal 0 件）。exit 0。エラーではない
- **境界条件**: 空入力 → proposal 0 件は既定正常挙動

### ケース 22: proposals 一覧の status フィルタ（None=全件 / Some=該当のみ）

- **観点出典**: TP-LGX-007 §2.1 B4, §2.7 I4, §2.5 V4, TP-LGX-018 §2.6 R4
- **分類**: Unit/Integration
- **前提**: pending / approved / rejected が混在する proposals。0 件状態も別 fixture
- **入力**: `list_proposals(&db, None)` / `list_proposals(&db, Some(ProposalStatus::Pending))`
- **期待**: None → 全件（`ORDER BY id`）。`Some(Pending)` → pending のみ。0 件状態 → 空 Vec（エラーでない）。`ProposalSummary` の各フィールド充足。read-only
- **境界条件**: `--status` 値域外（pending/approved/rejected 以外）は CLI 層 ValueEnum で exit 2（I4、ケース 6 と同一規約）

### ケース 23: audit limit 境界（recent / by_target、1 / 10 / 50）

- **観点出典**: TP-LGX-007 §2.10 F3（--limit 1..=50）, §2.9 O2/O4
- **分類**: Integration
- **前提**: context_log に 60 件超のエントリ
- **入力**: `recent(&db, 1)` / `recent(&db, 10)` / `recent(&db, 50)`、`by_target(&db, "SPEC-LGX-007", 10)`
- **期待**: `recent` は id DESC LIMIT N で N 件（または総数未満ならその数）。`by_target` は target_id フィルタ + id DESC LIMIT N。返却順は id DESC（O4）。read-only
- **境界条件**: limit 範囲 1..=50 は CLI 層強制（既定 10）。0 / 51 / 負 / 小数 は CLI 層 exit 2（範囲外 = 構文層）。境界 1（下限）/ 50（上限）を別検証

### ケース 24: engine.db 不在 → 新規作成して続行（正常、FB-INV-4 区別）

- **観点出典**: TP-LGX-007 §2.6 P1, TP-LGX-018 §2.5 DF2, §2.3 EF1
- **分類**: Integration
- **前提**: engine.db ファイルが存在しない（`Path::exists()==false`）。書込系コマンド（observe/feedback/analyze）
- **入力**: 不在 DB に対し初回 record / analyze
- **期待**: CREATE TABLE で新規作成して続行（exit 0）。**破損とは区別**（不在=正常）
- **境界条件**: FB-INV-4 は上流グラフ無効化の話で本 TS スコープ外（→ DF2 は委譲）。本ケースは「不在 = 新規作成」の正常系のみ

### ケース 25: engine.db 破損 → DbCorrupted exit 1（自動再生成禁止）

- **観点出典**: TP-LGX-007 §2.2 E5, §2.6 P1, TP-LGX-018 §2.3 EF1
- **分類**: Integration
- **前提**: ファイルは存在するが破損（不正バイト列）。書込系/読取系コマンドの初回 DB 操作
- **入力**: 破損 fixture に対し record / analyze / approve / list_proposals / recent
- **期待**: 初回操作で `SQLITE_CORRUPT` / `NotADatabase` を捕捉 → `FeedbackError::DbCorrupted{ detail }` → **exit 1**。**自動再生成しない**（証跡保護、再生成不能データ）
- **境界条件**: 不在（新規作成）と破損（exit 1）を `Path::exists()` + 操作エラーで判別。126（Admin 整合性優先）方針

### ケース 26: context_log 書込失敗時の挙動（ベストエフォート、analyze は欠落検査せず）

- **観点出典**: TP-LGX-007 §2.9 O6（REQ.06 GAP-LGX-139）, ADR-LGX-004
- **分類**: Integration
- **前提**: DB は存在するが context_log へ書込不能（ロック競合超過・ディスク満杯相当）。本ケースは **読取側 + analyze の完全性注記**を検証
- **入力**: 欠落のある context_log に対し `recent(&db, limit)` / `run_analyze(&db)`
- **期待**: `recent` は存在するエントリのみ返す（欠落を補完・検出しない）。`analyze` は context_log の完全性を**検査・報告しない**（responsibilities は observations→proposal 変換のみ、ADR-LGX-004 / SPEC REQ.03 注記）。本体処理は成功扱い
- **境界条件**: 書込本体（best-effort + stderr Warning）は SPEC-LGX-003.REQ.19 / TS-LGX-003 主導 → **委譲**。本 TS は読取・analyze 側の「欠落を前提に動くが検出しない」を検証

### ケース 27: read-only 不変（list_proposals / recent / by_target はDB を変更しない）

- **観点出典**: TP-LGX-007 §2.4 C, TP-LGX-018 §2.5 DF1, DD §5 借用方針
- **分類**: Property/Integration
- **前提**: 任意の proposals / context_log を持つ DB
- **入力**: `list_proposals` / `recent` / `by_target` 実行前後の DB ハッシュ
- **期待**: 実行前後で engine.db が不変（読取系は `&Connection` 借用のみ、§3.1 冪等性 yes）。複数回呼出しで同一結果
- **境界条件**: 借用（`&Connection`）による read-only。書込系（record/analyze/approve/reject）と区別

### ケース 28: TS MCP parseObserveStdout（正常 / 形式不正）

- **観点出典**: TP-LGX-018 §2.4 AT2, TP-LGX-007 §2.10 F2（観測性 MCP 転送）
- **分類**: Unit（TypeScript）
- **前提**: CLI stdout 契約 `"observation: id=N skipped=true|false"`、正規表現 `/^observation:\s*id=(\d+)\s+skipped=(true|false)/`
- **入力**: 正常 `"observation: id=42 skipped=false"`、形式不正 `"oops"` / `"observation: id=x skipped=maybe"`
- **期待**: 正常 → `{ id:42, skipped:false }`。形式不一致 → `Error` throw（呼出側で `isError:true` 応答に変換）
- **境界条件**: 正規表現パターンは互換凍結（MCP-INV-2、LGX-COMPAT-001 §4.1）。形式変更は転送破壊

### ケース 29: TS MCP zod 入力スキーマ境界（observe / get_compile_audit）

- **観点出典**: TP-LGX-007 §2.7 I1（category 二層検証）, §2.10 F3, TP-LGX-018 §2.4 AT2
- **分類**: Unit（TypeScript）
- **前提**: observe `category: z.enum([3値])`, `message: z.string().min(1).trim()`。get_compile_audit `limit: z.number().int().min(1).max(50).optional()`
- **入力**: category 不正値、message 空/trim 後空、limit=0 / 51 / -1 / 1.5（小数）
- **期待**: category 不正値 reject、message trim 後 min(1) 違反 reject、limit 0/51/負/小数 reject。境界 1 / 50 受理。limit 省略 → 既定（CLI 層 10）
- **境界条件**: MCP 層（zod）と CLI 層（ValueEnum / 範囲）の二層検証の MCP 側。最大 50 / 最小 1 の境界

### ケース 30: TS MCP formatAuditEntry（payload parse 成功 / 失敗 / 空、【v3 差分】）

- **観点出典**: TP-LGX-007 §2.9 O2, TP-LGX-018 §2.4 AT2
- **分類**: Unit（TypeScript）
- **前提**: `ContextLogEntry{ payload: JSON文字列 }`。`AuditPayload{ command?, target_files? }`
- **入力**: (a) `payload='{"command":"compile_context","target_files":["a","b"]}'`、(b) `payload='不正JSON'`、(c) `payload='{}'`（フィールド欠落）
- **期待**: (a) Markdown に Target/Files(`a, b`)/Command 反映。(b) JSON parse 失敗 → 空 payload 扱い（`{}`）で `Files:(none)` `Command:(none)`、throw しない。(c) 欠落フィールドは `(none)`
- **境界条件**: 【v3 差分】v3 は `e.input_files`/`e.input_command` 参照（Rust 出力に該当フィールドなし）→ legixy は payload を parse して `target_files`/`command` を取り出す（SUPP §2-9 方針 a）

### ケース 31: TS MCP `_meta` 付与（exit 0 + stderr 非空 → legixy/warnings、stderr 空 → 省略）

- **観点出典**: TP-LGX-007 §2.9 O（観測性 ADR-LGX-004）, TP-LGX-018 §2.4 AT2, SPEC-LGX-009 REQ.03/13
- **分類**: Unit/Integration（TypeScript）
- **前提**: mock RustEngine。observe / get_compile_audit
- **入力**: (a) exit 0 + stderr 非空、(b) exit 0 + stderr 空、(c) get_compile_audit
- **期待**: (a) `_meta["legixy/warnings"]` 付与。(b) 当該フィールド**省略**。(c) `_meta["anthropic/maxResultSizeChars"]=500000` 付与（REQ.13）
- **境界条件**: warnings 付与は exit 0 かつ stderr 非空の場合のみ（ADR-LGX-004）

### ケース 32: TS MCP isError 応答（exit 1 / exit 2 / タイムアウト）

- **観点出典**: TP-LGX-007 §2.2 E1/E2, §2.10 F4, TP-LGX-018 §2.3 EF1/EF3
- **分類**: Integration（TypeScript）
- **前提**: mock RustEngine が非ゼロ exit / タイムアウトで `RustCliError` throw
- **入力**: CLI が exit 1（意味的不正）/ exit 2（使用法誤り）/ タイムアウト（REQ.16 既定 30s）
- **期待**: ツールハンドラが `try/catch` で捕捉し `isError:true` 応答。throw を MCP 境界に漏らさない
- **境界条件**: exit 2=使用法誤り / exit 1=意味的不正（F4、COMPAT §3）。タイムアウトも isError

## 3. 観点カバレッジ表

### 3.1 TP-LGX-007（TP[SPEC]）

| TP § | 観点 | カバーする TS ケース |
|---|---|---|
| TP-007 §2.1 B1 message 境界 | 境界値 | ケース 1, 2 |
| TP-007 §2.1 B2 related_id 多数/重複/不在 | 境界値 | ケース 4 |
| TP-007 §2.1 B3 approve/reject id 境界 | 境界値 | ケース 17 |
| TP-007 §2.1 B4 proposals 空/大量 | 境界値 | ケース 22 |
| TP-007 §2.1 B5 reason 空/極端長 | 境界値 | ケース 15 |
| TP-007 §2.2 E1 不正 category exit 2 | エラー | ケース 6, 32 |
| TP-007 §2.2 E2 不在 proposal-id exit 1 | エラー | ケース 17, 32 |
| TP-007 §2.2 E3 approve 部分失敗ロールバック | エラー | ケース 13（CAS 原子性）, 19 |
| TP-007 §2.2 E4 ロック取得失敗方針 | エラー | NFR-LGX-001 REL.07 へ委譲 |
| TP-007 §2.2 E5 DB 破損時挙動 | エラー | ケース 25 |
| TP-007 §2.3 S1 不正遷移禁止 | 状態 | ケース 13, 14, 16 |
| TP-007 §2.3 S2 再 approve/reject | 状態 | ケース 13, 14 |
| TP-007 §2.3 S3 終端不可逆性 | 状態 | ケース 13, 14 |
| TP-007 §2.3 S4 observation 状態遷移 | 状態 | ケース 9, 10, 11, 14 |
| TP-007 §2.4 C1 並行 observe 1 件格納 | 並行 | ケース 18（+ NFR REL.07/SEC.02 委譲） |
| TP-007 §2.4 C2 approve/reject 並行決着 | 並行 | ケース 19（+ NFR REL.07 委譲） |
| TP-007 §2.4 C3 並行 analyze 重複抑止 | 並行 | ケース 12（semantic_key 一意） |
| TP-007 §2.4 C4 analyzing 中の同一キー observe | 並行 | ケース 3（dedup 範囲 pending/analyzing） |
| TP-007 §2.5 V1 observe 位置引数維持 | 互換 | ケース 6, 28（CLI 変換） |
| TP-007 §2.5 V2 CATEGORY 凍結 | 互換 | ケース 6 |
| TP-007 §2.5 V3 旧スキーマ継承 | 互換 | SPEC-LGX-008（migration）へ委譲 |
| TP-007 §2.5 V4 proposals 出力安定性 | 互換 | ケース 22（ProposalSummary フィールド） |
| TP-007 §2.6 P1 DB 不在時無効化 | 永続化 | ケース 24, 25（不在/破損区別） |
| TP-007 §2.6 P2 WAL+busy_timeout 排他 | 永続化 | ケース 18（+ NFR SEC.02 委譲） |
| TP-007 §2.6 P3 トランザクション境界 | 永続化 | ケース 9, 13, 16（単一 tx）, 3 |
| TP-007 §2.6 P4 ネットワーク共有検出 | 永続化 | NFR-LGX-001 REL.08 へ委譲 |
| TP-007 §2.6 P5 kill/電源断回復 | 永続化 | STATE-INV-1 / NFR REL.01 へ委譲 |
| TP-007 §2.7 I1 category 二層検証 | 入力 | ケース 6（CLI）, 29（MCP zod） |
| TP-007 §2.7 I2 related_id 形式/実在検証 | 入力 | ケース 4（無検証受理） |
| TP-007 §2.7 I3 proposal-id 型検証 exit 2 | 入力 | ケース 17（境界）, 32 |
| TP-007 §2.7 I4 proposals --status 値域 | 入力 | ケース 22, 29 |
| TP-007 §2.8 L1 空 observations の analyze | ライフ | ケース 21 |
| TP-007 §2.8 L2 解決済み再観測 | ライフ | ケース 3（終端後再受理）, 14 |
| TP-007 §2.8 L3 proposal retention | ライフ | 永続保持（自動パージなし）= ケース 13/16 終端後保持で観測 |
| TP-007 §2.8 L4 observation GC | ライフ | 同上（永続保持、自動 GC なし、人間裁定 2026-06-10） |
| TP-007 §2.9 O1 全 compile_context 記録 | 観測 | SPEC-LGX-003 / TS-LGX-003 へ委譲（書込本体） |
| TP-007 §2.9 O2 audit 加工なし転送 | 観測 | ケース 23（read 順序）, 30（formatAuditEntry） |
| TP-007 §2.9 O3 approve/reject 証跡記録 | 観測 | ケース 16（decided_at/decided_reason） |
| TP-007 §2.9 O4 監査ログ順序保証 | 観測 | ケース 23（id DESC）+ SPEC-LGX-003 主導 |
| TP-007 §2.9 O5 機密混入防止 | 観測 | ケース 8（mask_api_key）+ NFR SEC.05 委譲 |
| TP-007 §2.9 O6 記録失敗時の本体影響 | 観測 | ケース 26（analyze 非検査）+ SPEC-LGX-003 委譲 |
| TP-007 §2.10 F1 MCP 3 ツール限定 | 境界 API | SPEC-LGX-009 / TP-018 AT2 へ委譲（Admin 非露出） |
| TP-007 §2.10 F2 observe 引数変換忠実性 | 境界 API | ケース 28, 29 |
| TP-007 §2.10 F3 audit --limit 既定/上限 | 境界 API | ケース 23, 29 |
| TP-007 §2.10 F4 終了コード規約 | 境界 API | ケース 6, 17, 32 |
| TP-007 §2.11 D1 人間のみ CLI 強制 | 領域 | SPEC-LGX-009 / TP-018 AT1/AT3 へ委譲（MCP 非露出） |
| TP-007 §2.11 D2 Agent の approve/reject 不可 | 領域 | 同上（MCP 非露出） |
| TP-007 §2.11 D3 テスト不可侵 role 制御 | 領域 | パイプラインフック（legixy 外）/ MAINT.05 へ委譲 |
| TP-007 §2.11 D4 pending の context 不変性 | 領域 | ケース 12, 13（pending は終端化まで context 非影響） |

### 3.2 TP-LGX-018（TP[UC]）

| TP § | 観点 | カバーする TS ケース |
|---|---|---|
| TP-018 §2.1 BF1 フェーズ連鎖整合 | UC フロー | ケース 9, 20, 21 |
| TP-018 §2.1 BF2 feedback 入力受取方 | UC フロー | ケース 20（CheckReport 引数受取） |
| TP-018 §2.1 BF3 check→observation マッピング | UC フロー | ケース 7, 20 |
| TP-018 §2.1 BF4 Pessimistic Claim 観察 | UC フロー | ケース 9, 10, 11, 14 |
| TP-018 §2.1 BF5 approve/reject 後 observation 接続 | UC フロー | ケース 9（Resolved）, 16（pending） |
| TP-018 §2.2 AF1 代替フロー 1a 発火条件 | 代替 | ケース 20（該当 0 件 → created 0） |
| TP-018 §2.2 AF2 claim release 発火条件 | 代替 | ケース 11 |
| TP-018 §2.2 AF3 代替フロー事後条件 | 代替 | ケース 11（pending 復帰）, 21 |
| TP-018 §2.3 EF1 DB 破損時失敗パス | 例外 | ケース 25, 32 |
| TP-018 §2.3 EF2 observe 重複時振る舞い | 例外 | ケース 3, 18 |
| TP-018 §2.3 EF3 終端 proposal 再操作失敗 | 例外 | ケース 13, 17, 19, 32 |
| TP-018 §2.4 AT1 システムアクターと人間のみ実行 | アクター | ケース 20（feedback=引数受取の自動生成、CLI 起動は人間）+ SPEC-LGX-007 REQ.02 委譲 |
| TP-018 §2.4 AT2 Admin/Agent Surface 分離 | アクター | ケース 28, 29, 31, 32（MCP=Agent Surface）+ SPEC-LGX-009 委譲 |
| TP-018 §2.4 AT3 Claude Code の approve/reject 禁止 | アクター | SPEC-LGX-009（MCP 非露出）へ委譲 |
| TP-018 §2.5 DF1 テーブル別データフロー | データ | ケース 20, 9, 16, 27（read/write 別） |
| TP-018 §2.5 DF2 FB-INV-4 とフロー接続 | データ | ケース 24（不在=新規作成）+ SPEC-LGX-003 委譲 |
| TP-018 §2.5 DF3 --reason データフロー | データ | ケース 15, 16 |
| TP-018 §2.6 R1 orphan_file 変換先 | 領域 | ケース 7, 10（変換規則なし→skipped） |
| TP-018 §2.6 R2 skipped 発火条件 | 領域 | ケース 10, 14 |
| TP-018 §2.6 R3 feedback 事前条件 | 領域 | ケース 20, 24（DB 依存） |
| TP-018 §2.6 R4 proposals status フィルタ整合 | 領域 | ケース 22 |

> 継承 TP 観点（TP-007 49 + TP-018 22）はすべて本テーブルで TS ケースまたは明示委譲先に mapping 済み（人間ゲート判断対象）。並行負荷（C1/C2 のストレス側面・E4・P2/P4/P5）・context_log 書込本体（O1/O4 本体）・MCP 非露出強制（F1/D1/D2/AT3）・テスト不可侵 role 制御（D3）・migration（V3）は責務上 NFR / SPEC-LGX-003(TS-003) / SPEC-LGX-009 / パイプラインフック / SPEC-LGX-008 へ委譲し、本 TS は legixy-feedback の状態遷移・dedup・CAS・trim 拒否・不正カテゴリ exit・読取整形・MCP 転送パースに集中する。

## 4. テスト技法選択

- 同値分割: message/reason の trim 結果（0 文字 vs 1 文字以上）、category（凍結 3 値 vs 不正値）、observation/proposal の各 status クラス
- 境界値分析: message trim 後 0/1 文字（ケース 1/2）、audit limit 1/10/50/0/51（ケース 23/29）、approve/reject id（0/負/i64 上限/不在、ケース 17）
- Property-based: related_ids 正準化の決定性（ケース 4）、dedup 冪等性（ケース 5）、read-only 不変（ケース 27）
- 状態遷移: ObservationStatus 4 値 + ProposalStatus 3 値の許容遷移/禁止遷移の全網羅（ケース 14）。skipped 終端（ケース 10）・claim release（ケース 11）

## 5. テスト基盤

- 言語: Rust（legixy-feedback 本体） + TypeScript（ts-mcp 転送層）
- フレームワーク: cargo test（Rust）/ Vitest（TS MCP）
- Property-based: proptest（Rust、ケース 4/5/27）/ fast-check（TS は本 TS では未使用、境界は example test）
- モック: Rust は実 SQLite（in-memory または tempdir engine.db fixture、破損 fixture は不正バイト列）。TS は mock RustEngine（CLI argv 検証・stdout/stderr/exit 注入）

## 6. 関連 TC

| TS ケース | 対応 TC | 場所 |
|---|---|---|
| ケース 1, 2, 6 | TC-LGX-008（入力検証） | legixy-feedback/src/recorder.rs（#[cfg(test)]） |
| ケース 3, 4, 5, 18 | TC-LGX-008（dedup/正準化/並行） | legixy-feedback/src/recorder.rs / tests/ |
| ケース 7, 8 | TC-LGX-008（AutoObserver/drift） | legixy-feedback/src/observer.rs |
| ケース 9, 10, 11, 12, 14 | TC-LGX-008（analyze 状態遷移） | legixy-feedback/src/analyzer.rs / tests/ |
| ケース 13, 15, 16, 17, 19 | TC-LGX-008（CAS approve/reject） | legixy-feedback/src/manager.rs / tests/ |
| ケース 20, 21, 22 | TC-LGX-008（FeedbackCli E2E） | legixy-feedback/tests/ |
| ケース 23, 26, 27 | TC-LGX-008（audit/read-only） | legixy-feedback/src/audit.rs / tests/ |
| ケース 24, 25 | TC-LGX-008（DB 不在/破損） | legixy-feedback/tests/ |
| ケース 28, 29, 30, 31, 32 | TC-LGX-009（TS MCP） | ts-mcp/tests/tools/observe.test.ts / get-compile-audit.test.ts |
