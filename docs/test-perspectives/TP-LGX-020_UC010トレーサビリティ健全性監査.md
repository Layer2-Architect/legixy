Document ID: TP-LGX-020

# TP-LGX-020: UC-LGX-010「トレーサビリティ健全性監査」観点（UC レベル）

> TP は **テストケース** ではなく **観点リスト**。UC レベル TP は「ユースケースのフロー記述に問いかける質問のリスト」として書く。SPEC レベル TP（TP-LGX-010）が「仕様が答えるか」を問うのに対し、UC レベル TP は「フローが先行成果物（親 SPEC）を観察可能なステップへ忠実かつ完全に具体化しているか」を問う。

**親**: UC-LGX-010
**ステータス**: green
**最終更新**: 2026-06-13

## 1. 対象スコープ

この TP は UC-LGX-010「トレーサビリティ健全性監査」の全フロー（基本フロー Step 1〜6、代替フロー 2a/3a、事後条件、関連不変条件）に UC レベル観点をぶつける。

- 対象: UC-LGX-010 全節（概要・アクター・事前条件・基本フロー・代替フロー・事後条件・関連不変条件・関連 SPEC/NFR）
- 親 SPEC: SPEC-LGX-010 REQ.04（report — トレーサビリティ健全性監査）、REQ.01（共通規約・終了コード）、REQ.06（出力の決定性）、REQ.07（ストレージ境界・非破壊性）、REQ.09（非有限スコア）
- 関連 SPEC §: SPEC-LGX-006 REQ.11（bulk similarity API — `compute_edge_scores` / `compute_link_candidates`）、NFR-LGX-001.OBS.02（ログ=stderr / 結果=stdout）、NFR-LGX-001.OBS.05（終了コード）
- 委譲方針: report の計測セマンティクス（エッジスコア算出・リンク候補抽出・summary 統計・スキップ・NaN 扱いの**規定そのもの**）は TP-LGX-010（green 確定済）が所有する。本 TP はそれらを再検証せず、「UC-010 のフロー記述が SPEC-LGX-010 REQ.04 の規定を観察可能なステップとして正しく具体化しているか」のみを問う。

## 2. 観点リスト

### 2.1 基本フロー（ステップ連鎖の整合）

- [ ] 観点 BF1: 各ステップの事後条件が後続ステップの前提を満たすか（Step1 コマンド受理 → Step2 graph.toml パース + embeddings ロード → Step3 スコア算出 → Step4/5 出力 → Step6 exit の連鎖整合）
- [ ] 観点 BF2: Step2 の「graph.toml パース」と「embeddings テーブルから全件ロード」が独立操作として観察可能か。いずれか一方の失敗が他方に干渉しないか（3a は Step3 の compute 失敗を扱うが Step2 の部分失敗が不明）
- [ ] 観点 BF3: 成功時事後条件（stdout に text or JSON 報告出力・exit 0・engine.db/graph.toml 不変）が外部観察可能で、後続 UC（UC-008 フィードバックループ連携）の前提として参照可能か
- [ ] 観点 BF4: text モード（Step4）と `--json` モード（Step5）が排他的分岐として UC のフロー記述で観察可能か。`--json` 指定の有無をどのステップで判定するかがフローに現れているか

### 2.2 代替フロー（分岐網羅）

- [ ] 観点 AF1: 分岐の網羅性。UC が列挙する代替フロー（2a 空 embeddings テーブル / 3a compute 失敗）が SPEC-LGX-010 REQ.04 の主要分岐（空ストア / スキップ発生 / NaN 混入）を被覆しているか
- [ ] 観点 AF2: 各代替フローの事後条件収束。2a（空テーブル）の exit 0 と 3a（compute 失敗）の exit 1 が SPEC-LGX-010 REQ.01 の終了コード分類（exit 0 = 空結果 / exit 1 = 実行エラー）へ正しく収束しているか
- [ ] 観点 AF3: 代替フローへの遷移条件が明示されているか。2a「embeddings テーブルが空の場合」の発火条件は明示されているが、3a「`compute_edge_scores` / `compute_link_candidates` が失敗した場合」の発火条件として「anyhow エラーコンテキスト付き」の通知方式と exit 1 が明示されているか

### 2.3 例外フロー（失敗パス）

- [ ] 観点 EF1: Step2 の失敗パス（graph.toml 不在・破損・パース不能）が UC フローに定義されているか。3a は Step3 の compute 失敗のみを扱い、Step2 の graph.toml 読込失敗は代替フローに列挙されていない。SPEC-LGX-010 REQ.04 + REQ.07 はこれを規定しているか確認が必要。【WEAK 寄り: check と同様のパターンだが report 固有のパス明示が任意か必須かは裁定待ち】
- [ ] 観点 EF2: エラー時の状態が不変条件を満たすか。report は読み取り専用（STATE-INV-1）であり engine.db/graph.toml を変更しない。エラー時の状態破壊が原理的に発生しないことが UC 前提として担保されているか
- [ ] 観点 EF3: エラー時のユーザ通知方式が定義されているか。3a「anyhow エラーコンテキスト付きで exit 1」の stderr 通知は記述されているが、通知の情報十分性（何が失敗したかを特定できるか）がフロー記述に現れているか

### 2.4 アクター遷移と権限

- [ ] 観点 AT1: アクター（プロジェクトリード / QA リード / 設計者 / CI システム）の権限・状態が一貫しているか。report が読み取り専用で全アクターが同一権限で実行可能であることが UC で一貫しているか
- [ ] 観点 AT2: 責任境界。システムは計測のみを行い（閾値判定しない）、アクターがレビュー結果を基に `observe` 記録を行う選択肢が事後条件に明示されているか（check/report の責務非重複が UC 事後条件で観察可能か）
- [ ] 観点 AT3: CI システムアクターの `--json` 出力連携が基本フローで観察可能か。CI が `--json` 出力を上流ツールへ渡して差分監視するユースケースが Step4/5 の分岐として適切に表現されているか

### 2.5 データフロー

- [ ] 観点 DF1: 入出力データの型・制約。入力（graph.toml + embeddings テーブル + `.legixy.toml` の `link_candidate_threshold`）→ 出力（text 報告 or JSON = stdout / INFO/WARNING = stderr）の分離が UC で観察可能か
- [ ] 観点 DF2: `link_candidate_threshold` のデータフロー。事前条件「`.legixy.toml` の `semantic.link_candidate_threshold` が設定されている（既定 0.7）」が基本フロー Step3 の `compute_link_candidates` に到達するまでの経路がフロー記述に現れているか
- [ ] 観点 DF3: エラー時のデータ解放保証。読み取り専用・メモリ上計算のみで永続リソース確保解放を伴わないことが UC 前提と整合するか（SPEC-LGX-010 REQ.07 との整合）

### 2.6 領域固有観点（トレーサビリティエンジン / report UC）

- [ ] 観点 R1: report（計測）と check（判定）の責務非重複が UC レベルで観察可能か。事後条件に「CheckResult severity 概念なし」「閾値判定なし」が明示されているか（SPEC-LGX-010 REQ.04 の責務境界が UC に反映されているか）
- [ ] 観点 R2: UC の「関連 SPEC / NFR」参照の正確性。UC-010 は「SPEC-LGX-006 REQ.10（report コマンド）」と記載しているが、SPEC-LGX-006 REQ.10 はモデル更新時再計算の規定であり report コマンドの定義は SPEC-LGX-010 REQ.04 である。この誤参照が UC フローの根拠連鎖を断絶させていないか
- [ ] 観点 R3: UC の「関連 SPEC / NFR」に「NFR-LGX-001.OBS.06（ユーザー向け構造化出力）」が挙がっているが、OBS.06 は CheckResult の severity（`check` コマンド用）であり `report` コマンドには適用されない。この誤参照が UC フローの出力定義を歪めていないか
- [ ] 観点 R4: 終了コード契約（exit 0 / exit 1）が UC 事後条件と SPEC-LGX-010 REQ.01 の凍結契約で一致するか。特に 2a（空テーブル exit 0）が「結果が空の正常終了」として REQ.01 の exit 0 定義と整合するか
- [ ] 観点 R5: SCORE-INV-1（決定性保証）が UC フローで観察可能か。Step3 の算出が「同一入力 → 同一出力」（bulk API の決定論的走査順 + cosine 決定性）として表現されているか、または SPEC-LGX-010 REQ.06 / SPEC-LGX-006 REQ.11 への委譲が明示されているか
- [ ] 観点 R6: スキップ発生時の集約 Warning（stderr）が UC 代替フローに現れているか。端点 embedding 不在・次元不一致によるスキップ（SPEC-LGX-010 REQ.04「集約 Warning 1 件を stderr」）が基本フロー / 代替フローのどこに位置づけられているか

## 3. RED / GREEN 判定

| 観点 | 判定 | 親 SPEC / UC §で回答（委譲先） | 関連 GAP |
|---|---|---|---|
| 2.1 BF1 ステップ連鎖整合 | GREEN | 基本フロー Step1〜6 が事前条件（init+embed済）→ Step2 ロード → Step3 算出 → Step4/5 出力 → Step6 exit と連鎖。各事後条件が後続前提を満たす | — |
| 2.1 BF2 Step2 部分失敗の独立性 | RED | UC Step2「graph.toml をパースし、embeddings テーブルから全件をロードする」が単一ステップとして記述されており、両操作の独立性（一方失敗が他方に干渉しない）がフローに現れていない。SPEC-LGX-010 REQ.04 は report の主要失敗パスのみ規定し、Step2 独立性の明示はない。【WEAK: report の読み取り専用性（REQ.07）から原理的に独立だが UC フロー記述への明示が任意か必須か裁定待ち】 | GAP-LGX-271 |
| 2.1 BF3 成功時事後条件の観察可能性 | GREEN | 事後条件「標準出力に報告が出力される」「engine.db / graph.toml は不変」+ Step6「exit 0 で終了」。外部観察可能・後続 UC 前提として参照可。STATE-INV-1 + SPEC-LGX-010 REQ.07 へ委譲 | — |
| 2.1 BF4 text/JSON 分岐の観察可能性 | GREEN | 基本フロー Step4「text モード: 人間可読な階層表示」/ Step5「`--json` モード: 構造化 JSON」として代替表現が記述されている。`--json` 指定は SPEC-LGX-010 REQ.01（グローバルオプション）+ LGX-COMPAT-001 §3 へ委譲 | — |
| 2.2 AF1 分岐網羅（空ストア / スキップ / NaN） | RED | UC の代替フローは 2a（空テーブル）/ 3a（compute 失敗）の 2 件のみ。端点 embedding 不在・次元不一致によるスキップ発生時の集約 Warning（SPEC-LGX-010 REQ.04 明示）が代替フローに列挙されていない。NaN/Inf スキップ（REQ.09）も同様。【WEAK: スキップは正常フロー内処理として親 SPEC へ委譲で解決可。UC への列挙は任意の可能性】 | GAP-LGX-272 |
| 2.2 AF2 代替フロー事後条件収束 | GREEN | 2a（exit 0）= 「結果が空」正常終了として SPEC-LGX-010 REQ.01 の exit 0 定義に収束。3a（exit 1）= 実行エラーとして REQ.01 に収束。収束整合 | — |
| 2.2 AF3 遷移条件の明示 | GREEN | 2a「embeddings テーブルが空の場合」発火条件明示。3a「compute_edge_scores / compute_link_candidates が失敗した場合」発火条件 + anyhow エラーコンテキスト付き exit 1 明示。遷移条件は十分 | — |
| 2.3 EF1 Step2 失敗パス（graph.toml 破損等） | RED | UC-010 の代替フローは 2a（embeddings 空）/ 3a（compute 失敗）のみ。Step2 の graph.toml 不在・破損・パース不能の失敗パスが列挙されていない。SPEC-LGX-010 REQ.07 は graph.toml 非破壊性を規定するが、入力としての graph.toml の破損失敗パスは REQ.04 に明示なし。【WEAK: check 系と同様のパターン（GAP-LGX-189 類似）。実装は失敗するが UC への明示列挙が任意か必須か裁定待ち】 | GAP-LGX-273 |
| 2.3 EF2 エラー時状態の不変条件保持 | GREEN | report は読み取り専用（STATE-INV-1）。engine.db/graph.toml を変更しない。エラー時の状態破壊が原理的に発生しない。SPEC-LGX-010 REQ.07 + STATE-INV-1 へ委譲 | — |
| 2.3 EF3 エラー時通知の情報十分性 | GREEN | 3a「anyhow エラーコンテキスト付きで exit 1」と通知方式が記述されている。情報十分性の詳細（何が失敗したかを特定できるか）は SPEC-LGX-010 REQ.04 の根拠として v3 実測 + NFR-LGX-001.OBS.04 へ委譲（TP-LGX-010 観点 L3 相当は GREEN 確定済） | — |
| 2.4 AT1 アクター権限の一貫性 | GREEN | プロジェクトリード / QA リード / 設計者 / CI とも読み取り専用 report を同一権限で実行。権限差は本質的に存在せず UC 記述と一貫 | — |
| 2.4 AT2 責任境界（計測 vs 是正） | GREEN | 事後条件「アクターがレビュー結果を基に `observe` で観察事項を記録する選択肢がある」でシステム=計測 / アクター=是正判断の分担を明示。check/report の責務非重複（SPEC-LGX-010 REQ.04）へ委譲 | — |
| 2.4 AT3 CI アクターの --json 連携 | GREEN | アクター定義「CI システム（PR 毎に `--json` 出力を上流ツールへ渡して差分監視）」が基本フロー Step5 の `--json` モード出力と対応。連携経路は観察可能 | — |
| 2.5 DF1 入出力データ型・stdout/stderr 分離 | GREEN | 入力（graph.toml + embeddings テーブル + link_candidate_threshold）→ 出力（text/JSON = stdout / INFO = stderr）。分離は SPEC-LGX-010 REQ.04 + NFR-LGX-001.OBS.02 へ委譲。UC Step4/5 が stdout 出力と整合 | — |
| 2.5 DF2 link_candidate_threshold のデータフロー | GREEN | 事前条件「`.legixy.toml` の `semantic.link_candidate_threshold` が設定されている（既定 0.7）」→ Step3b「類似度 ≥ link_candidate_threshold」の参照で連鎖している。データフロー整合 | — |
| 2.5 DF3 エラー時データ解放保証 | GREEN | 読み取り専用・メモリ上計算のみで永続リソース確保解放を伴わない。SPEC-LGX-010 REQ.07 + STATE-INV-1 で担保。データ解放保証は N/A 相当（Rust リソース管理は DD へ委譲） | — |
| 2.6 R1 report/check 責務非重複の観察可能性 | GREEN | UC 概要「閾値判定を行わない計測レポート」+ 事後条件（severity 言及なし）で check との責務非重複が観察可能。SPEC-LGX-010 REQ.04 の責務境界（判定しない）が UC に反映されている | — |
| 2.6 R2 UC 関連 SPEC 参照の正確性（REQ.10 誤参照） | RED | UC-010「関連 SPEC / NFR」に「SPEC-LGX-006 REQ.10（report コマンド）」と記載しているが、SPEC-LGX-006 REQ.10 はモデル更新時再計算（model_version 複合キー生成）の規定であり report コマンドの定義ではない。report コマンドは SPEC-LGX-010 REQ.04 で定義される。この誤参照により UC フローの根拠連鎖（UC 基本フロー Step3 の算出関数引用「te_embed::compute_edge_scores」 / 「te_embed::compute_link_candidates」）が SPEC-LGX-006 REQ.11（bulk similarity API）に委譲されるべきものが誤 SPEC へ向いている。【GENUINE: 根拠参照の誤りは下流 RBA/DD の参照誤りに連鎖する可能性がある。UC の「関連 SPEC」節修正が必要】 | GAP-LGX-274 |
| 2.6 R3 UC 関連 NFR 参照の正確性（OBS.06 誤参照） | RED | UC-010「関連 SPEC / NFR」に「NFR-LGX-001.OBS.06（ユーザー向け構造化出力）」が挙がっているが、OBS.06 は「CheckResult の severity（Ok/Info/Warning/Error の 4 段階。DD-LGX-001 §2.4）」であり check コマンド用の定義である。report は severity 概念を持たない（SPEC-LGX-010 REQ.04 の責務境界）。OBS.06 の誤参照が「report も severity を持つ」という誤解を下流に伝播させる可能性がある。【GENUINE: report の出力定義に severity 概念が混入する恐れがあり修正が必要。正しい参照は NFR-LGX-001.OBS.02（stdout/stderr 分離）+ OBS.05（終了コード）】 | GAP-LGX-275 |
| 2.6 R4 終了コード契約一致 | GREEN | 事後条件「報告が stdout に出力される」+ Step6「exit 0 で終了」、2a「exit 0 で終了」、3a「exit 1」が SPEC-LGX-010 REQ.01 の終了コード分類（exit 0=空結果含む正常 / exit 1=実行エラー）と一致。TP-LGX-010 観点 E1 で確立 | — |
| 2.6 R5 SCORE-INV-1 決定性保証の観察可能性 | GREEN | 関連不変条件「SCORE-INV-1（決定性保証）: 同一入力 → 同一出力（bulk API の決定論的走査順 + cosine_similarity の決定性で担保）」と明示。SPEC-LGX-010 REQ.06 + SPEC-LGX-006 REQ.11 への委譲が UC で明示されている | — |
| 2.6 R6 スキップ時集約 Warning の UC 表現 | RED | 端点 embedding 不在・次元不一致によるスキップ発生時の「集約 Warning 1 件（スキップ件数と代表理由）を stderr に出力」（SPEC-LGX-010 REQ.04）が UC の基本フロー / 代替フローのどこにも現れていない。正常計算経路内で発生する部分スキップは AF1 と重複する観点。【WEAK: スキップは正常フロー内の計算エンジン挙動として SPEC-LGX-010 REQ.04 / SPEC-LGX-006 REQ.11 へ委譲で解決可。UC フローへの列挙は任意の可能性】 | GAP-LGX-272（AF1 と統合） |

集計: **全 22 観点 / GREEN 14 / RED 6（R6 は AF1 の GAP-LGX-272 と統合、実質独立 GAP は 5 件）**

## 4. ステータスの決定

RED 観点が 6 件（BF2 / AF1+R6 / EF1 / R2 / R3 / — ただし R6 は AF1 と同一 GAP）残存するため、本 TP のステータスは `red`。

- BF2: WEAK（Step2 独立性は原理的に保証されているが UC フロー記述への明示が任意か必須か裁定待ち）
- AF1+R6: WEAK（スキップ集約 Warning は親 SPEC へ委譲で解決可。UC への列挙は任意の可能性）
- EF1: WEAK（check 系と同様のパターン。GAP-LGX-189/190 と同種の UC フロー粒度問題）
- R2: GENUINE（SPEC-LGX-006 REQ.10 は誤参照。正しくは SPEC-LGX-010 REQ.04 + SPEC-LGX-006 REQ.11。下流参照誤りに連鎖する可能性あり）
- R3: GENUINE（NFR-LGX-001.OBS.06 は check 用。report に severity を混入させる誤参照）

GENUINE 2 件（R2 / R3）は UC の「関連 SPEC / NFR」節の修正で解決可能。WEAK 3 件（BF2 / AF1+R6 / EF1）は人間裁定（UC フローへの追記 or drop）を経て close。全観点 GREEN 化後に本 TP を green へ更新する。

## 5. 観点ナレッジベース参照

この TP の生成時に参照したナレッジベース章節:

- `docs/perspectives/core-perspectives.md` §UC レベル観点（基本フロー / 代替フロー / 例外フロー / アクター遷移と権限 / データフロー）
- `docs/perspectives/core-perspectives.md` §汎用観点（境界値=空ストア・終了コード境界 / エラーハンドリング / 状態遷移 / ロギング・観測性）
- 親 SPEC: SPEC-LGX-010.REQ.01/REQ.04/REQ.06/REQ.07/REQ.09、SPEC-LGX-006.REQ.11（bulk similarity API）
- 委譲先 TP: TP-LGX-010（embedding 運用・監査 SPEC レベル観点、green 確定済）
- LGX-COMPAT-001 §3/§4 #6（終了コード・引数契約）
- NFR-LGX-001.OBS.02/OBS.05（stdout/stderr 分離・終了コード）

UX 層観点（Undo/フォーカス/タッチ等）は CLI 計測コマンドには本質的に N/A のため、エラー UX（通知の情報十分性 EF3）以外はスキップした。

## 6. 履歴

| 日付 | 変更内容 |
|---|---|
| 2026-06-13 | 初版作成。UC レベル観点 22 件（GREEN 14 / RED 6）。GAP-LGX-271（BF2 Step2 独立性）/ GAP-LGX-272（AF1+R6 スキップ集約 Warning 未列挙）/ GAP-LGX-273（EF1 graph.toml 破損失敗パス未列挙）/ GAP-LGX-274（R2 SPEC-LGX-006 REQ.10 誤参照）/ GAP-LGX-275（R3 NFR-LGX-001.OBS.06 誤参照）を起票 |

## 7. 解消（2026-06-13、敵対的精査裁定後）

本 TP が起票した GAP[UC] は全て closed。内訳: **WEAK=方針B（委譲容認）** / **REFUTED=棄却** / **GENUINE=UC 修正で解消**（A/B/C、人間承認 2026-06-13）。§3 表の判定列は初版（起票時）の draft 判定を保持する（精査の履歴として温存）。全 RED 観点は上記裁定で解消したため本 TP は **green**。各 GAP の最終状態は当該 GAP ファイル（§5）と docs/spec-change-proposals/2026-06-13_uc-gap-adversarial-review.md を参照。
