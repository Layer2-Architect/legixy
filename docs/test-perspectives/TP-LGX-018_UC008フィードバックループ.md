Document ID: TP-LGX-018

# TP-LGX-018: UC-LGX-008「フィードバックループ」観点（UC レベル）

> TP は **テストケース** ではなく **観点リスト**。UC レベル TP は「ユースケースのフロー記述に問いかける質問のリスト」として書く。SPEC レベル TP（TP-LGX-007）が「仕様が答えるか」を問うのに対し、UC レベル TP は「フローが先行成果物（親 SPEC）を観察可能なステップへ忠実かつ完全に具体化しているか」を問う。

**親**: UC-LGX-008
**ステータス**: green
**最終更新**: 2026-06-13

## 1. 対象スコープ

この TP は UC-LGX-008「フィードバックループ（observation → proposal → approve/reject）」の全フロー（基本フロー: Observation 生成 / Proposal 生成 / 承認・却下、代替フロー 1a / 2a、事後条件）に UC レベル観点をぶつける。

- 対象: UC-LGX-008 全節（概要・アクター・事前条件・基本フロー・代替フロー・事後条件・関連不変条件）
- 親 SPEC: SPEC-LGX-007（フィードバックループ）REQ.01〜REQ.11、§4（FB-INV-1〜5、MCP-INV-1〜4）、§5（Surface 分離マトリクス）
- 関連 SPEC §: SPEC-LGX-003.REQ.19（context_log ベストエフォート書込 / FB-INV-4 主導）、LGX-COMPAT-001 §3 / §4.1（終了コード凍結契約 / observe 位置引数）
- 委譲方針: フィードバックループの検証セマンティクス（重複排除キー正準定義・状態遷移モデル・CAS 原子性・Surface 分離の強制手段・終了コード規約）は TP-LGX-007（green 確定済）が所有する。本 TP はそれらを再検証せず、「UC-LGX-008 のフロー記述が SPEC-LGX-007 の規定を観察可能なステップとして正しく具体化しているか」のみを問う。

## 2. 観点リスト

### 2.1 基本フロー（ステップ連鎖の整合）

- [ ] 観点 BF1: 各フェーズの事後条件が後続フェーズの前提を満たすか（Observation 生成 → Proposal 生成 → 承認・却下 の連鎖整合）
- [ ] 観点 BF2: `feedback` コマンドの入力（check 結果の受け取り方）が観察可能なステップとして記述されているか。check 結果（CheckReport）を引数として受け取るのか、自動的に読み取るのか、UC フローから判別できるか
- [ ] 観点 BF3: Observation 生成フェーズにおける check カテゴリ→observation カテゴリ（ChainIntegrity → chain_integrity 等）のマッピング完全性が UC で観察可能か
- [ ] 観点 BF4: analyze の Pessimistic Claim パターン（pending → analyzing → proposed/skipped）の各状態遷移が UC の一ステップとして観察可能か
- [ ] 観点 BF5: `approve <id>` / `reject <id>` 成功時の事後条件（承認・却下後の proposal status 変化）が後続フェーズ（Observation の resolved 化）と接続して観察可能か

### 2.2 代替フロー（分岐網羅）

- [ ] 観点 AF1: 代替フロー 1a「check 結果に該当カテゴリがない場合」の発火条件が明示されているか。チェック 4 カテゴリ（ChainIntegrity / LinkCandidate / Drift / OrphanFile）すべてが 0 件の場合のみか、一部 0 件でも発火するか
- [ ] 観点 AF2: 代替フロー 2a「analyze で処理中に失敗した場合、Observation を pending に戻す（claim release）」の「失敗」の発火条件が UC に明示されているか。どのような失敗（プロセス kill? DB エラー? アルゴリズム例外?）で claim release が起動するかが観察可能か
- [ ] 観点 AF3: 各代替フローの事後条件が定義されているか。1a（Observation 0 件）・2a（pending へのロールバック）が基本フロー事後条件に収束するか、または別事後条件を持つか

### 2.3 例外フロー（失敗パス）

- [ ] 観点 EF1: 各コマンド（feedback / observe / analyze / proposals / approve / reject）での engine.db 破損時の失敗パスが UC フローに定義されているか。SPEC-LGX-007.REQ.09 では破損 DB → exit 1 と規定されているが、UC フローへの反映が確認できるか
- [ ] 観点 EF2: `observe` コマンドでの重複（FB-INV-1 発動）時の失敗パス（または silent drop）が UC フローで観察可能か
- [ ] 観点 EF3: `approve` / `reject` が終端状態 proposal に対して再操作された場合の失敗パス（exit 1）が UC フローに反映されているか

### 2.4 アクター遷移と権限

- [ ] 観点 AT1: アクター 3 者（システム / Claude Code / 人間）の責務分担が各コマンドに一貫して対応しているか。特に `feedback` コマンドが「システム」帰属でも「人間のみ CLI 実行」（SPEC-LGX-007.REQ.02）であることとの矛盾がないか
- [ ] 観点 AT2: Admin Surface（feedback / analyze / proposals / approve / reject）と Agent Surface（observe MCP 経由）の分離が UC のアクター記述と整合しているか（§5 Surface 分離マトリクスとの一致）
- [ ] 観点 AT3: 承認・却下フェーズでの「人間のみ」制約（SPEC-LGX-007.REQ.05）が UC のアクター定義に反映されているか。Claude Code アクターが approve/reject を実行できないことが UC レベルで観察可能か

### 2.5 データフロー

- [ ] 観点 DF1: engine.db の入出力データフロー。各コマンドが engine.db のどのテーブル（observations / proposals / context_log）に読み書きするかが UC フローから追跡可能か
- [ ] 観点 DF2: 事後条件「engine.db がなくてもグラフ上流は正常に返される（FB-INV-4）」がフィードバックループ UC のフロー記述と接続しているか。本 UC のどのステップが欠落した場合でも上流グラフが正常稼働することが観察可能か
- [ ] 観点 DF3: `reject <id> --reason <reason>` の `--reason` パラメータのデータフロー（proposals テーブル reject_reason フィールドへの格納）が UC フローで観察可能か

### 2.6 領域固有観点（フィードバックループ UC）

- [ ] 観点 R1: Proposal 生成フェーズのカテゴリ別変換（chain_integrity → add_chain_entry / link_candidate → add_link / drift → update_doc）網羅性が UC で観察可能か。4 観測カテゴリ（基本フロー §Observation 生成）に対して 3 変換先しかなく、orphan_file の変換先が UC に明示されているか
- [ ] 観点 R2: `analyze` の「skipped」パス（pending → analyzing → skipped）の発火条件が UC フローに記述されているか。どの category / 条件で skipped になるか観察可能か
- [ ] 観点 R3: UC の事前条件が「engine.db が存在する」のみだが、各コマンドは engine.db 以外の依存（graph.toml / .legixy.toml）を前提とするか。Observation 生成時（feedback コマンド）に check 結果が必要な場合、その前提が UC 事前条件に含まれているか
- [ ] 観点 R4: `proposals` コマンドの「pending の Proposal 一覧」という限定が UC フロー記述と SPEC-LGX-007.REQ.04（status フィルタ: pending / approved / rejected）の整合性。UC では pending のみ表示と読めるが SPEC はフィルタ可能

## 3. RED / GREEN 判定

| 観点 | 判定 | 親 SPEC / UC §で回答（委譲先） | 関連 GAP |
|---|---|---|---|
| 2.1 BF1 フェーズ連鎖整合 | GREEN | UC の 3 フェーズ（Observation 生成 → Proposal 生成 → 承認・却下）は各フェーズが独立コマンドで区切られ連鎖整合。事前条件（engine.db 存在）は REQ.08/09 で確立。SPEC-LGX-007.REQ.01〜05 + §4 FB-INV-1〜3 へ委譲 | — |
| 2.1 BF2 feedback コマンドの入力受け取り方 | RED | UC 基本フロー §Observation 生成 Step1 は「check 結果から自動で Observation を生成する」と記述するが、check 結果をどのように受け取るか（直接引数? 自動実行して取得? 既存ファイル読み取り?）が観察可能なステップとして記述されていない。SPEC-LGX-007.REQ.02 は「check の結果や embedding から未対応の observation を生成する」と述べるが入力受取方法を規定しない。フロー記述レベルのギャップ。【GENUINE: feedback コマンドの入力インターフェースは下流 DD/SRC 設計に直結し、UC レベルで明示が必要】 | GAP-LGX-251 |
| 2.1 BF3 check カテゴリ→observation カテゴリのマッピング | GREEN | UC §Observation 生成 が「ChainIntegrity → chain_integrity / LinkCandidate → link_candidate / Drift → drift / OrphanFile → orphan_file」の 4 カテゴリマッピングを明示。SPEC-LGX-007.REQ.02 + REQ.08 へ委譲 | — |
| 2.1 BF4 Pessimistic Claim パターンの観察可能性 | GREEN | UC §Proposal 生成 Step2「pending → analyzing → proposed/skipped」で状態遷移を明示。状態モデルの完全性は SPEC-LGX-007.REQ.08（observation 状態モデル）+ REQ.09（proposal 状態モデル）+ TP-LGX-007 S4 へ委譲 | — |
| 2.1 BF5 approve/reject 後の observation 接続 | GREEN | UC §承認・却下 Step2/3 で approve/reject の対象が proposal であると明示。observation の resolved 化はフロー外（SPEC-LGX-007.REQ.08 の状態モデル：approve → resolved）へ委譲。UC レベルでは proposal の終端状態への遷移が観察可能 | — |
| 2.2 AF1 代替フロー 1a の発火条件 | RED | 代替フロー 1a「check 結果に該当カテゴリがない場合、Observation は生成されない」の発火条件が不明確。4 カテゴリすべてが 0 件の場合に限るのか、一部カテゴリが 0 件でも各カテゴリ個別に発火するのかが UC フローから判別できない。SPEC-LGX-007.REQ.02 も「該当カテゴリがない場合」の粒度を規定していない。【WEAK: 実装上は per-category に自然に処理されるが、UC フロー記述の曖昧さとして記録に値する。SPEC 委譲で解決可】 | GAP-LGX-252 |
| 2.2 AF2 代替フロー 2a の失敗発火条件 | RED | 代替フロー 2a「analyze で処理中に失敗した場合、Observation を pending に戻す（claim release）」の「失敗」の発火条件が UC フローに定義されていない。どのような失敗（プロセス強制終了・DB エラー・分析アルゴリズム例外）で claim release が起動するかが観察不能。SPEC-LGX-007.REQ.03 は失敗時の claim release を明示していない。【GENUINE: claim release の発火条件は observe → analyzing 状態管理の核心であり、UC レベルで発火条件が未定義だと RBA/DD での設計が根拠を持てない】 | GAP-LGX-253 |
| 2.2 AF3 代替フロー各事後条件 | GREEN | 1a（Observation 0 件 → analyze で対象なし → 自然に exit 0）は UC 基本フロー事後条件に収束。2a（pending ロールバック → 次回 analyze で再処理可能）は SPEC-LGX-007.REQ.03（pending 状態での再分析対象）+ REQ.08 の状態モデルへ委譲 | — |
| 2.3 EF1 DB 破損時失敗パス | GREEN | SPEC-LGX-007.REQ.09 が破損 DB → exit 1 を明確に規定済み。TP-LGX-007 E5（GREEN 確定済）へ委譲。UC フロー記述への明示列挙は任意（SPEC が答えを持つ）【WEAK 判定同等: 委譲で解決可】 | — |
| 2.3 EF2 observe 重複発動時の振る舞い | GREEN | FB-INV-1（Observation 冪等性）+ SPEC-LGX-007.REQ.11（同一キーは silent で INSERT 抑止）。TP-LGX-007 C1（GREEN 確定済）へ委譲。UC フローに失敗パスとして記述する必要はなく正常系の縮退として扱う | — |
| 2.3 EF3 終端 proposal への再操作失敗パス | GREEN | SPEC-LGX-007.REQ.05（pending 限定作用）+ REQ.09（終端状態への再操作 exit 1）。TP-LGX-007 S2/S3（GREEN 確定済）へ委譲。UC §承認・却下 では pending の Proposal のみを対象とすると読める | — |
| 2.4 AT1 「システム」アクターと人間のみ CLI 実行の整合 | RED | UC のアクター定義で `feedback` コマンドを「システム（自動 Observation 生成）」に帰属させているが、SPEC-LGX-007.REQ.02 は「人間のみが CLI で実行する」と規定する。「自動 Observation 生成」と「人間のみが実行する」の矛盾が UC フロー記述に存在する。アクター定義の整合性が破綻しており後続 RBA でのアクター責務が不明確になる。【GENUINE: アクター帰属の誤記または意図的な「システムが行う処理だが人間が起動する」の明示不足。いずれも UC フロー記述レベルのギャップ】 | GAP-LGX-254 |
| 2.4 AT2 Admin/Agent Surface 分離の UC 整合 | GREEN | UC アクター定義（システム / Claude Code / 人間）と SPEC-LGX-007 §5 Surface 分離マトリクスは整合。observe = Claude Code (MCP 経由) = Agent Surface、feedback/analyze/proposals/approve/reject = 人間 (CLI) = Admin Surface。TP-LGX-007 F1（GREEN 確定済）へ委譲 | — |
| 2.4 AT3 Claude Code の approve/reject 禁止の観察可能性 | GREEN | UC §承認・却下 がアクターを「人間（Proposal の承認・却下）」のみに帰属させており、Claude Code の当該コマンド実行禁止が UC レベルで観察可能。SPEC-LGX-007.REQ.05 + MCP-INV-1 + TP-LGX-007 D2（GREEN 確定済）へ委譲 | — |
| 2.5 DF1 engine.db テーブル別データフロー | GREEN | UC の各コマンド記述から暗黙にデータフローが追跡可能（observe → observations テーブル、analyze → observations + proposals テーブル、approve/reject → proposals テーブル）。詳細は SPEC-LGX-007.REQ.08/09 へ委譲 | — |
| 2.5 DF2 事後条件 FB-INV-4 とフロー接続 | RED | UC の事後条件「engine.db がなくてもグラフ上流は正常に返される（FB-INV-4）」はフィードバックループ UC の基本フロー・代替フローのどのステップとも接続していない。engine.db が存在しない場合の UC フローが記述されておらず（事前条件は「engine.db が存在する」のみ）、FB-INV-4 はフィードバックループ機能の無効化であり本 UC のスコープを超えている。事後条件が本 UC のフロー記述から導出できない。【WEAK: FB-INV-4 は SPEC-LGX-003 主導（SPEC-LGX-007 §4 記載）であり UC フロー記述外の性質を持つ。UC 事後条件としての適切性の人間裁定が必要】 | GAP-LGX-255 |
| 2.5 DF3 --reason パラメータのデータフロー | GREEN | UC §承認・却下 Step3「reject <id> --reason <reason>: Proposal を却下する（理由必須）」が `--reason` の必須性を明示。proposals テーブル reject_reason フィールドへの格納は SPEC-LGX-007.REQ.09 + REQ.05 へ委譲。TP-LGX-007 B5（GREEN 確定済）へ委譲 | — |
| 2.6 R1 orphan_file カテゴリの変換先 | RED | UC §Proposal 生成 Step3 のカテゴリ別変換は「chain_integrity → add_chain_entry / link_candidate → add_link / drift → update_doc」の 3 変換を記述するが、§Observation 生成 Step1 で定義した 4 観測カテゴリのうち「orphan_file」の変換先が UC フロー記述に存在しない。orphan_file category の observation が generate された場合の Proposal 種別が UC から判別できず、フロー記述として不完全。【GENUINE: orphan_file に対応する proposal 種別は SPEC-LGX-007 REQ.02/REQ.03 にも明示がなく、下流設計（RBA/DD）に未解決のままになる可能性がある】 | GAP-LGX-256 |
| 2.6 R2 analyze の「skipped」発火条件 | RED | UC §Proposal 生成 Step2「Pessimistic Claim パターン: pending → analyzing → proposed/skipped」は skipped という終端を示すが、どの条件で skipped となるかが UC フローに記述されていない。SPEC-LGX-007.REQ.03 も skipped の条件を明示していない（「v3 では不正 category → skipped」の記述はあるが category 凍結後の skipped 発生条件は未定義）。【GENUINE: skipped 状態の発火条件が未定義だと observation が永続的に skipped のまま残るケースへの対処が設計できない】 | GAP-LGX-257 |
| 2.6 R3 feedback コマンドの事前条件（check 結果依存） | GREEN | SPEC-LGX-007.REQ.02 は「check の結果や embedding から未対応の observation を生成する」と述べ、BF2 のギャップ（受け取り方）と区別して事前条件の必要性は SPEC 委譲で解決可。UC の事前条件「engine.db が存在する」は feedback コマンドの DB 依存として適切。check 実行自体は UC-LGX-001 の事後条件（CheckReport 出力）であり本 UC の事前条件としての明示は任意 | — |
| 2.6 R4 proposals コマンドの status フィルタと UC 記述の整合 | GREEN | UC §承認・却下 Step1「pending の Proposal 一覧を表示する」はデフォルト表示を記述。SPEC-LGX-007.REQ.04 は status フィルタ（pending/approved/rejected）を提供。UC は「pending がデフォルト」という最小記述であり矛盾はない。フィルタ機能の詳細は SPEC-LGX-007.REQ.04 + LGX-COMPAT-001 §4 へ委譲 | — |

集計: **全 22 観点 / GREEN 15 / RED 7**（RED は BF2 / AF1 / AF2 / AT1 / DF2 / R1 / R2）

## 4. ステータスの決定

RED 観点が 7 件残存するため、本 TP のステータスは `red`。

- GENUINE 4 件（BF2 / AF2 / AT1 / R1 / R2: feedback コマンド入力インターフェース・claim release 発火条件・アクター定義矛盾・orphan_file 変換先欠落・skipped 発火条件未定義）は下流設計（RBA/SEQA/DD）への波及リスクを持つ。
- WEAK 3 件（AF1 / DF2: 代替フロー 1a 発火条件粒度・FB-INV-4 事後条件接続）は SPEC または UC 追記の人間裁定が必要。
- GENUINE 確定分から人間裁定（UC フローへの追記 or drop）を経て close。全観点 GREEN 化後に本 TP を green へ更新する。

## 5. 観点ナレッジベース参照

この TP の生成時に参照したナレッジベース章節:

- `docs/perspectives/core-perspectives.md` §UC レベル観点（基本フロー / 代替フロー / 例外フロー / アクター遷移と権限 / データフロー）
- `docs/perspectives/core-perspectives.md` §汎用観点（状態遷移 / 並行性 / 永続化 / 入力検証）
- 親 SPEC: SPEC-LGX-007.REQ.01〜REQ.11、§4（FB-INV-1〜5、MCP-INV-1〜4、STATE-INV-1）、§5（Surface 分離マトリクス）
- 委譲先 TP: TP-LGX-007（フィードバックループ SPEC レベル観点、green 確定済）
- LGX-COMPAT-001 §3 / §4.1（終了コード凍結契約 / observe 位置引数）

UX 層観点（Undo/フォーカス/タッチ等）は CLI/MCP コマンドには本質的に N/A のため、アクター責務境界観点以外はスキップした。

## 6. 履歴

| 日付 | 変更内容 |
|---|---|
| 2026-06-13 | 初版作成。UC レベル観点 22 件（GREEN 15 / RED 7）。GAP-LGX-251（BF2 feedback 入力受取方法）/ GAP-LGX-252（AF1 代替フロー 1a 発火条件）/ GAP-LGX-253（AF2 claim release 発火条件）/ GAP-LGX-254（AT1 アクター定義矛盾）/ GAP-LGX-255（DF2 FB-INV-4 事後条件接続）/ GAP-LGX-256（R1 orphan_file 変換先欠落）/ GAP-LGX-257（R2 skipped 発火条件）を起票 |

## 7. 解消（2026-06-13、敵対的精査裁定後）

本 TP が起票した GAP[UC] は全て closed。内訳: **WEAK=方針B（委譲容認）** / **REFUTED=棄却** / **GENUINE=UC 修正で解消**（A/B/C、人間承認 2026-06-13）。§3 表の判定列は初版（起票時）の draft 判定を保持する（精査の履歴として温存）。全 RED 観点は上記裁定で解消したため本 TP は **green**。各 GAP の最終状態は当該 GAP ファイル（§5）と docs/spec-change-proposals/2026-06-13_uc-gap-adversarial-review.md を参照。
