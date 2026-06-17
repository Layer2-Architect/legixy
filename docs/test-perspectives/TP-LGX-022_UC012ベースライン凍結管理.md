Document ID: TP-LGX-022

# TP-LGX-022: UC-LGX-012「ベースライン凍結管理」観点（UC レベル）

> TP は **テストケース** ではなく **観点リスト**。UC レベル TP は「ユースケースのフロー記述に問いかける質問のリスト」として書く。SPEC レベル TP（TP-LGX-010）が「仕様が答えるか」を問うのに対し、UC レベル TP は「フローが先行成果物（親 SPEC）を観察可能なステップへ忠実かつ完全に具体化しているか」を問う。

**親**: UC-LGX-012
**ステータス**: green
**最終更新**: 2026-06-13

## 1. 対象スコープ

この TP は UC-LGX-012「ベースライン凍結管理」の全フロー（基本フロー Step 1〜7、代替フロー 1a/2a/4a/6a/6b、復旧フロー、事後条件）に UC レベル観点をぶつける。

- 対象: UC-LGX-012 全節（概要・アクター・事前条件・基本フロー・代替フロー・復旧フロー・事後条件・関連不変条件・関連 SPEC/NFR）
- 親 SPEC: SPEC-LGX-010（embedding 運用・監査）REQ.01（共通規約・exit 分類）/ REQ.02（snapshot ライフサイクル）/ REQ.06（list 安定出力・create 非決定性）/ REQ.07（ストレージ境界・DB 不在時非作成・非破壊性）
- 関連 SPEC §: LGX-COMPAT-001 §4 #8（snapshot の凍結済引数契約）・§7（サブコマンド排他・既定挙動）、NFR-LGX-001.OBS.02（ログ=stderr / 結果=stdout）/ OBS.05（終了コード 0/1/2）。UC が引く OBS.06（ユーザー向け構造化出力）は親 SPEC-LGX-010 の対応 NFR 行（OBS.02/OBS.05）には挙がらない参照（→ §3 注記 / GAP 検討）
- 委譲方針: snapshot のライフサイクル・セマンティクス（単一トランザクション複製・snapshot_id の不透明トークン性・label 非一意と最新優先解決・空ストア非永続・delete の text/json 別挙動・DB 不在 ≡ 空ストア・list の決定性）の**規定そのもの**は TP-LGX-010（green 確定済、総観点 71 / GREEN 71）が所有する。本 TP はそれらを再検証せず、「UC-012 のフロー記述が SPEC-010 の規定を観察可能なステップとして正しく具体化しているか」のみを問う。

## 2. 観点リスト

### 2.1 基本フロー（ステップ連鎖の整合）

- [ ] 観点 BF1: 各ステップの事後条件が後続ステップの前提を満たすか（Step1 create 受理 → Step2 単一 Tx 複製 → Step3 snapshot_id 発行・返却 → Step4 list 確認 → Step5 後続 drift 参照 → Step6 delete → Step7 exit 0 の連鎖整合）
- [ ] 観点 BF2: アクター入力の検証タイミングの段階区分（段階1 構文=サブコマンド/引数パース＝exit 2 / 段階2 形式=target の `snapshot_id`/`label:<L>` 判別 / 段階3 意味=label 解決・該当行有無）が UC で観察可能か
- [ ] 観点 BF3: 成功時事後条件（create=スナップショット領域への複製と snapshot_id 永続化 / delete=該当行除去 / いずれも exit 0）が外部観察可能で、後続 UC（UC-LGX-013 ドリフト対比の `--against snapshot:<L>` 基準点）の前提として参照可能か
- [ ] 観点 BF4: Step3 の snapshot_id 発行の出力契約（`snap-` プレフィクス・不透明トークン・text / `--json` 両表現）が UC レベルで観察可能か

### 2.2 代替フロー（分岐網羅）

- [ ] 観点 AF1: サブコマンド分岐の網羅性。`create` / `list` / `delete` の 3 必須サブコマンドが基本フローと代替フローで全て被覆され、サブコマンド省略（1a）が排他規約（exit 2）として明示されているか
- [ ] 観点 AF2: create の境界 case 網羅。空ストア（2a: WARNING + exit 0 + 非永続 + `node_count: 0`）が列挙され、非空ストアの基本フローと対比可能か。非永続が事後条件（list 不在）まで追跡されているか
- [ ] 観点 AF3: list の境界 case 網羅。0 件（4a: text 案内 / json 空配列 + exit 0）が列挙され、≥1 件の基本フロー（taken_at 降順一覧）と対比可能か
- [ ] 観点 AF4: delete の target 形態網羅。`snapshot_id` 指定（6b: 該当 0 件で text WARNING+exit 0 / json `deleted_rows:0`）と `label:<L>` 指定（6a: 重複 label を最新 1 件へ決定論的解決）の両形態が列挙されているか
- [ ] 観点 AF5: 各代替フローの事後条件収束。2a/4a/6b はいずれも exit 0（結果が空＝正常）へ収束し、1a のみ exit 2（使用法誤り）へ分岐する。この終了コードの収束/分岐が基本フロー Step7（exit 0）と一貫しているか
- [ ] 観点 AF6: 代替フローへの遷移条件が明示されているか（2a=「空ストア」、4a=「list 0 件」、6a=「同一 label が複数存在」、6b=「snapshot_id 指定で該当行 0 件」と発火条件が明示されているか）

### 2.3 例外フロー（失敗パス）

- [ ] 観点 EF1: delete の失敗パス網羅。SPEC-LGX-010.REQ.02 が規定する **`label:<L>` 解決失敗（不在 label）= ERROR(stderr) + exit 1** の例外パスが UC の代替/例外フローに列挙されているか（6a は重複時、6b は snapshot_id の 0 件時のみを扱う）
- [ ] 観点 EF2: create の書込み失敗パス。snapshot create は唯一の書込み系（REQ.07: engine.db 初期化 + スナップショット領域書込み）であり、複製トランザクション失敗・書込み I/O 失敗時の状態（部分複製の有無・exit code）が UC フローに定義されているか
- [ ] 観点 EF3: エラー時の状態が不変条件を満たすか（複製は単一トランザクション → 失敗時に部分スナップショットが残らない atomicity が UC 前提として担保されているか。embeddings 本体・graph.toml の不変が維持されるか）
- [ ] 観点 EF4: エラー時のユーザ通知（WARNING / ERROR / INFO の severity 区分と stderr 出力先）が UC フローで定義されているか。特に 2a の WARNING・6b の WARNING・EF1 の ERROR の出力先（stderr）が一貫しているか
- [ ] 観点 EF5: 部分成功・非永続の見逃し検出。空ストア create（2a）の非永続を運用者が誤認した場合の二次検出（list 不在）が復旧フローに定義され、観察可能化されているか

### 2.4 アクター遷移と権限

- [ ] 観点 AT1: アクター（運用者/設定管理者・設計者・QA リード）の権限・状態が一貫しているか。3 アクターとも同一の snapshot 操作権限（create/list/delete）で実行可能であることが UC で一貫しているか
- [ ] 観点 AT2: 責任境界。システムは凍結・一覧・削除の機械的実行のみを行い、「どの世代を廃棄するか」「いつ凍結するか」の判断はアクター責務（QA リードのリリース基準点判断等）であることの分担が明示されているか
- [ ] 観点 AT3: 不可逆操作の権限前提。delete は不可逆（復元手段なし）であり、誤削除に対する事前確認（delete 前の list 確認）がアクター責務として復旧フローに位置づけられているか
- [ ] 観点 AT4: snapshot list / 他読取系が embeddings 本体・graph.toml を変更しない読取専用前提が、並行操作（create 中の list 等）下でも UC レベルで成立しているか

### 2.5 データフロー

- [ ] 観点 DF1: 入出力データの分離。出力（snapshot_id / list 結果 = stdout、WARNING/ERROR/INFO = stderr）の stdout/stderr 分離が UC で観察可能か。`--json` 時の機械可読出力が stdout に保たれるか
- [ ] 観点 DF2: スナップショットデータのライフタイム。create で複製される行が content_hash / model_version（SCORE-INV-1）を保持し、後続 drift（UC-LGX-013）の `--against snapshot:<L>` 基準点として参照されるまでのデータ寿命が UC フローで整合するか
- [ ] 観点 DF3: 非破壊性のデータフロー。snapshot 操作（create/list/delete）が embeddings 本体の現行行・graph.toml・成果物ファイルを変更しないこと、engine.db 不在時に DB を新規作成しないこと（create を除く）が UC 事後条件と整合するか

### 2.6 領域固有観点（トレーサビリティエンジン / ベースライン凍結 UC）

- [ ] 観点 R1: snapshot_id の不透明トークン性が UC で保たれているか（`snap-` プレフィクスのみ外部契約、内部形式に依存しない delete target 受理が Step3/Step6 で一貫しているか）
- [ ] 観点 R2: label 解決規則の一貫性。`delete label:<L>`（6a）の「同一 label 複数時は taken_at 最新 1 件へ決定論的解決」が、UC-LGX-013 の `--against snapshot:<L>` と同一規則であることが明示されているか
- [ ] 観点 R3: 終了コード契約（0/1/2）が UC 事後条件と LGX-COMPAT-001 §4 #8 / §7・NFR-LGX-001.OBS.05 の凍結契約に一致するか（1a=2 / 2a・4a・6b=0 / EF1=1 の分類）
- [ ] 観点 R4: DB 不在時の挙動が UC 事後条件（engine.db 不在時に DB を新規作成しない・空ストア相当で正常終了）と REQ.07 の「DB 不在 ≡ 空ストア」導出に一致するか。create のみ DB 初期化を行う非対称が観察可能か
- [ ] 観点 R5: list の決定性。`snapshot list` が taken_at 降順で安定出力する一方、`create` の snapshot_id / taken_at は決定性対象外（REQ.06）である非対称が UC のフロー記述（Step3 vs Step4）で整合するか

## 3. RED / GREEN 判定

| 観点 | 判定 | 親 SPEC / UC §で回答（委譲先） | 関連 GAP |
|---|---|---|---|
| 2.1 BF1 ステップ連鎖整合 | GREEN | 基本フロー Step1〜7 が事前条件（init+migrate / embed 済）→ Step2 単一 Tx 複製 → Step3 snapshot_id 発行 → Step4 list → Step6 delete → Step7 exit と連鎖。各事後条件が後続前提を満たす（SPEC-LGX-010.REQ.02） | — |
| 2.1 BF2 段階区分の観察可能性 | GREEN | 1a（サブコマンド省略=exit 2、構文層）/ 6a・6b（target 判別・解決、意味層）で段階1/2/3 の差が観察可能。exit 分類は SPEC-LGX-010.REQ.01 へ委譲 | — |
| 2.1 BF3 成功時事後条件の観察可能性 | GREEN | 事後条件「create=複製と snapshot_id 永続化」「delete=該当行除去」+ Step5 が後続 UC-LGX-013 の `--against snapshot:<L>` 基準点として参照可。SPEC-LGX-010.REQ.02 へ委譲 | — |
| 2.1 BF4 snapshot_id 出力契約 | GREEN | Step3「`snap-` プレフィクス・不透明トークン・text / `--json` で返す」と明示。不透明トークン性は SPEC-LGX-010.REQ.02 へ委譲 | — |
| 2.2 AF1 サブコマンド分岐網羅 | GREEN | create（Step1-3）/ list（Step4）/ delete（Step6）を被覆。1a「サブコマンド省略=exit 2」で排他規約を明示（LGX-COMPAT-001 §7 へ委譲） | — |
| 2.2 AF2 create 境界 case 網羅 | GREEN | 2a「空ストア=WARNING+exit 0+非永続+`node_count:0`」を列挙し非空の基本フローと対比可。非永続は「list に現れない」まで追跡（SPEC-LGX-010.REQ.02 空ストア節へ委譲） | — |
| 2.2 AF3 list 境界 case 網羅 | GREEN | 4a「list 0 件=text 案内 / json 空配列 + exit 0」を列挙し基本フロー（降順一覧）と対比可（SPEC-LGX-010.REQ.02 list 節 / REQ.06 へ委譲） | — |
| 2.2 AF4 delete target 形態網羅 | GREEN | 6a（`label:<L>` 重複=最新解決）/ 6b（`snapshot_id` 該当 0 件=text WARNING / json `deleted_rows:0`）の両形態を列挙。解決規則は SPEC-LGX-010.REQ.02 へ委譲 | — |
| 2.2 AF5 代替フロー exit 収束 | GREEN | 2a/4a/6b は exit 0（結果が空＝正常）へ収束、1a のみ exit 2 へ分岐。SPEC-LGX-010.REQ.01「空ストア・欠如は exit 0」「構文誤りは exit 2」と一貫 | — |
| 2.2 AF6 遷移条件の明示 | GREEN | 2a「空ストアで create」/ 4a「list 0 件」/ 6a「同一 label が複数存在」/ 6b「snapshot_id 指定で該当行 0 件」と発火条件を明示 | — |
| 2.3 EF1 delete label 解決失敗パス | RED | UC-012 の delete 代替フローは 6a（label 重複）/ 6b（snapshot_id 0 件）のみを扱い、**`label:<L>` 解決失敗（不在 label）= ERROR(stderr) + exit 1**（SPEC-LGX-010.REQ.02 delete 節 / REQ.07 で規定済）の例外パスをフロー記述に列挙していない。6b（snapshot_id 0 件 = exit 0）と挙動が分岐するため観察可能性に関わる。【GENUINE 寄り: 不在 label と該当 0 件 snapshot_id で exit code が 1/0 に分岐し、UC のフロー記述だけでは判別不能】 | GAP-LGX-291 |
| 2.3 EF2 create 書込み失敗パス | RED | snapshot create は唯一の書込み系（SPEC-LGX-010.REQ.07: engine.db 初期化 + スナップショット領域書込み）だが、UC-012 は複製トランザクション失敗・書込み I/O 失敗時の挙動（exit code・部分複製の有無）をフローに定義していない。空ストア（2a）以外の create 失敗パスが空白。【WEAK: REQ.01 の実行時失敗=exit 1 + REQ.03 の単一 Tx atomicity へ委譲可。ただし書込み系特有の失敗パスは UC 完全性として明示価値あり】 | GAP-LGX-292 |
| 2.3 EF3 エラー時 atomicity・不変条件 | GREEN | Step2「単一トランザクションで複製」+ 事後条件「embeddings 本体・graph.toml は不変」「snapshot は embeddings 本体に触れない」。atomicity 規定は SPEC-LGX-010.REQ.02（単一 Tx）/ REQ.07（非破壊性）へ委譲 | — |
| 2.3 EF4 severity 区分・出力先 | GREEN | 2a/6b の WARNING（stderr）を明示。severity の stderr 統一は SPEC-LGX-010.REQ.01【v3 差分】/ NFR-LGX-001.OBS.02 へ委譲。EF1 の ERROR 出力先は GAP-LGX-291 に内包 | （→ EF1） |
| 2.3 EF5 非永続見逃しの二次検出 | GREEN | 復旧フロー「空ストア create の非永続の見逃し」で「list に現れないことで検出」「2a の WARNING / `node_count:0` が一次通知、list 不在が二次検出」と明示。観察可能化済 | — |
| 2.4 AT1 アクター権限の一貫性 | GREEN | 運用者/設計者/QA リードとも同一の snapshot create/list/delete 権限で実行。権限差は本質的に存在せず UC 記述と一貫 | — |
| 2.4 AT2 責任境界（実行 vs 判断） | GREEN | アクター節「マイルストーン毎の凍結と世代管理」「不要世代の廃棄判断」でシステム=機械的実行 / アクター=凍結時期・廃棄判断の分担を明示 | — |
| 2.4 AT3 不可逆操作の事前確認責務 | GREEN | 復旧フロー「delete は不可逆」「delete 前に `snapshot list` で対象を確認する」でアクター責務を明示 | — |
| 2.4 AT4 読取専用の並行整合性前提 | GREEN | 事後条件「`snapshot list` は読取専用」。並行アクセス整合性は NFR-LGX-001（SQLite busy_timeout、TP-LGX-010 が射程）へ委譲。読取専用前提で UC レベル整合 | — |
| 2.5 DF1 stdout/stderr 分離 | GREEN | Step3「text / `--json` で返す」（stdout）+ 2a/6b WARNING（stderr）。分離は SPEC-LGX-010.REQ.01 / NFR-LGX-001.OBS.02 へ委譲、UC が json を stdout で返すと整合 | — |
| 2.5 DF2 スナップショットデータ寿命 | GREEN | 関連不変条件 SCORE-INV-1「snapshot は content_hash / model_version を含む行を複製」+ Step5 で後続 drift の基準点として参照。データ寿命は SPEC-LGX-010.REQ.02 / §4 SCORE-INV-1 へ委譲 | — |
| 2.5 DF3 非破壊性データフロー | GREEN | 事後条件「embeddings 現行行・graph.toml・成果物ファイルは不変」「engine.db 不在時に DB を新規作成しない」が SPEC-LGX-010.REQ.07 と整合 | — |
| 2.6 R1 snapshot_id 不透明トークン性 | GREEN | Step3「内部形式は不透明トークン」+ Step6 delete が `snapshot_id` を受理。不透明性は SPEC-LGX-010.REQ.02 / LGX-COMPAT-001 §4 #8 へ委譲 | — |
| 2.6 R2 label 解決規則の一貫性 | GREEN | 6a「taken_at 最新の 1 件へ決定論的に解決（… drift の `--against snapshot:<L>` と同一規則）」と明示。規則は SPEC-LGX-010.REQ.02 へ委譲 | — |
| 2.6 R3 終了コード契約一致 | GREEN | 1a=2 / 2a・4a・6b=0 が LGX-COMPAT-001 §4 #8 / §7・NFR-LGX-001.OBS.05 の凍結契約に一致（EF1=1 は GAP-LGX-291 で別途）。TP-LGX-010 V 系で確立 | （→ EF1） |
| 2.6 R4 DB 不在時挙動の整合 | GREEN | 事後条件「engine.db 不在時に DB ファイルを新規作成しない（空ストア相当で正常終了）」が SPEC-LGX-010.REQ.07「DB 不在 ≡ 空ストア」「create のみ書込み系」へ委譲。create の DB 初期化非対称も REQ.07 で確立 | — |
| 2.6 R5 list 決定性 vs create 非決定性 | GREEN | Step4 list（taken_at 降順）vs Step3 create（snapshot_id / taken_at は作成時刻依存）の非対称が SPEC-LGX-010.REQ.06「list は安定出力 / create の snapshot_id・taken_at は決定性対象外」と整合 | — |

集計: **全 26 観点 / GREEN 24 / RED 2**（RED は EF1 / EF2）

## 4. ステータスの決定

RED 観点が 2 件（EF1 / GAP-LGX-291、EF2 / GAP-LGX-292）残存するため、本 TP のステータスは `**ステータス**: red`。

- EF1（delete `label:<L>` 解決失敗 = exit 1）は **GENUINE 寄り**。不在 label（exit 1）と該当 0 件 snapshot_id（exit 0、6b）で終了コードが分岐するが、UC のフロー記述だけではこの分岐を判別できない。新規 UC（2026-06-13 拡張）として親 SPEC REQ.02 に挙動規定はあるが UC フローへの具体化が欠落している典型例。
- EF2（create 書込み失敗パス）は **WEAK 候補**。REQ.01 の実行時失敗 = exit 1 + REQ.03（注: REQ.02 の単一 Tx）の atomicity へ委譲可能だが、唯一の書込み系コマンドの失敗パスが UC で空白なのは記述完全性の問題。
- 敵対的精査パスで GENUINE / WEAK / OUT_OF_SCOPE を確定し、裁定（UC フローへの追記 or drop）を経て close。全観点 GREEN 化後に本 TP を green へ更新する。

## 5. 観点ナレッジベース参照

この TP の生成時に参照したナレッジベース章節:

- `docs/perspectives/core-perspectives.md` §UC レベル観点（基本フロー / 代替フロー / 例外フロー / アクター遷移と権限 / データフロー）
- `docs/perspectives/core-perspectives.md` §汎用観点（永続化=トランザクション境界・部分書き込み回復 / 状態遷移=終端状態保証・atomicity / エラーハンドリング=部分成功の扱い・副作用解放）
- `docs/perspectives/core-perspectives.md` §領域固有観点（バージョン管理系: 履歴改変の検出・ロック中の他者編集に着想を得た snapshot ライフサイクル観点）
- 親 SPEC: SPEC-LGX-010.REQ.01/REQ.02/REQ.06/REQ.07、§4 SCORE-INV-1 / FB-INV-4
- 委譲先 TP: TP-LGX-010（embedding 運用・監査 SPEC レベル観点、green 確定済 71/71）
- LGX-COMPAT-001 §4 #8 / §7（snapshot 凍結済引数契約・サブコマンド排他）、NFR-LGX-001.OBS.02 / OBS.05（出力分離・終了コード）

UX 層観点（Undo/フォーカス/タッチ等）は CLI 管理コマンドには本質的に N/A のため、不可逆操作の事前確認 UX（AT3 に相当）以外はスキップした。

## 6. 履歴

| 日付 | 変更内容 |
|---|---|
| 2026-06-13 | 初版作成。UC レベル観点 26 件（GREEN 24 / RED 2）。GAP-LGX-291（EF1 delete label 解決失敗 exit 1 パス未列挙）/ GAP-LGX-292（EF2 create 書込み失敗パス未定義）を起票 |

## 7. 解消（2026-06-13、敵対的精査裁定後）

本 TP が起票した GAP[UC] は全て closed。内訳: **WEAK=方針B（委譲容認）** / **REFUTED=棄却** / **GENUINE=UC 修正で解消**（A/B/C、人間承認 2026-06-13）。§3 表の判定列は初版（起票時）の draft 判定を保持する（精査の履歴として温存）。全 RED 観点は上記裁定で解消したため本 TP は **green**。各 GAP の最終状態は当該 GAP ファイル（§5）と docs/spec-change-proposals/2026-06-13_uc-gap-adversarial-review.md を参照。
