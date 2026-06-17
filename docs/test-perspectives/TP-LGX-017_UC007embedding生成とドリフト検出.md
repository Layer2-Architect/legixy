Document ID: TP-LGX-017

# TP-LGX-017: UC-LGX-007「embedding 生成とドリフト検出」観点（UC レベル）

> TP は **テストケース** ではなく **観点リスト**。UC レベル TP は「ユースケースのフロー記述に問いかける質問のリスト」として書く。SPEC レベル TP（TP-LGX-006）が「仕様が答えるか」を問うのに対し、UC レベル TP は「フローが先行成果物（親 SPEC）を観察可能なステップへ忠実かつ完全に具体化しているか」を問う。

**親**: UC-LGX-007
**ステータス**: green
**最終更新**: 2026-06-13

## 1. 対象スコープ

この TP は UC-LGX-007「embedding 生成とドリフト検出」の全フロー（embed 基本フロー Step 1〜4 + サブノード分岐、drift 基本フロー Step 1〜4、代替フロー 2a/3b、事後条件、関連不変条件）に UC レベル観点をぶつける。

- 対象: UC-LGX-007 全節（概要・アクター・事前条件・基本フロー・代替フロー・事後条件・関連不変条件）
- 親 SPEC: SPEC-LGX-006（embedding 生成・格納・ドリフト検出）REQ.01〜REQ.12、SCORE-INV-1/SCORE-INV-2
- 関連 SPEC §: SPEC-LGX-010.REQ.03（drift コマンドの運用仕様・ベースライン選択・モデル解決順・終了コード・JSON 出力）、SPEC-LGX-004.REQ.02（check 内 Drift 報告との書き分け）、LGX-COMPAT-001 §4 #4/§4 #5（embed/drift の凍結済み引数契約）
- 委譲方針: embedding 生成エンジンの内部ロジック（ONNX 推論パイプライン・mean pooling・L2 正規化・content_hash 正規化・トランザクション境界・bulk API など**規定そのもの**）は TP-LGX-006（green 確定済）が所有する。drift の運用コマンド詳細（ベースライン選択・`--against` 解釈・model_version 不一致判定・JSON スキーマ）は SPEC-LGX-010.REQ.03 が所有する。本 TP は「UC-LGX-007 のフロー記述が SPEC-LGX-006/SPEC-LGX-010 の規定を観察可能なステップとして忠実かつ完全に具体化しているか」のみを問う。

## 2. 観点リスト

### 2.1 基本フロー（ステップ連鎖の整合）

- [ ] 観点 BF1: embed フロー各ステップの事後条件連鎖。Step1（コマンド受理） → Step2（graph.toml からノード取得） → Step3（各ノードのハッシュ計算→スキップ判定→前処理→推論→格納） → Step4（`--subnodes` 分岐）の順序で、各ステップの事後条件が後続ステップの前提を満たすか
- [ ] 観点 BF2: drift フロー各ステップの事後条件連鎖。Step1（コマンド受理） → Step2（現在 embedding 生成） → Step3（既存 embedding との比較） → Step4（閾値判定・レビュー推奨）の連鎖整合。特に Step2（embedding 生成）完了前に Step3（比較）が開始しないことが UC フローで観察可能か
- [ ] 観点 BF3: 成功時事後条件の観察可能性。embed の「engine.db の embeddings テーブルが更新される」「モデルバージョンが記録される（SCORE-INV-2）」が外部観察可能（後続の check/drift/report 等の前提として参照可能）か
- [ ] 観点 BF4: `--subnodes` 追加処理の連鎖。Step4「サブノードの embedding も生成する」が Step3 と同一ロジックを踏むことが UC フローで観察可能か、または独立ステップとして事後条件が明示されているか

### 2.2 代替フロー（分岐網羅）

- [ ] 観点 AF1: 代替フロー 2a（ONNX モデル不在）の分岐完全性。「ERROR を報告する」だけで終了コードの定義が UC に無い。SPEC-LGX-006.REQ.02 は「モデル解決失敗 = exit 1 + 試行パスを stderr 通知」と規定するが、UC フロー記述にこれが反映されているか
- [ ] 観点 AF2: 代替フロー 3b（`--all` 時のハッシュ比較スキップ）の事後収束。「ハッシュ比較をスキップして全ノードを再生成する」が、Step3b の事後条件として「全ノードの embedding が更新される」に収束することが観察可能か（SPEC-LGX-006.REQ.02 `--force` / `--all` の組合せとの整合）
- [ ] 観点 AF3: 分岐の網羅性。UC に明示されている代替フロー（2a / 3b）が、SPEC-LGX-006.REQ.02 が規定する他の分岐（個別ノード指定 `--node`、`--force`、空テキスト skip、サブノード `--subnodes`）を網羅しているか。drift 側の代替フロー（ベースライン不在、`--against` 指定、モデル解決失敗、現行ファイル欠落）が UC に存在しないか

### 2.3 例外フロー（失敗パス）

- [ ] 観点 EF1: embed の各ステップでの失敗パス網羅。graph.toml 読込失敗（Step2）・個別ノード推論失敗（Step3）・格納失敗（Step3e）の失敗パスが UC に定義されているか。SPEC-LGX-006.REQ.08 は「ノード単位 1 トランザクション・失敗時は当該 Tx のみ rollback して後続継続」と規定するが、UC フローにこの部分失敗継続挙動が現れていない
- [ ] 観点 EF2: drift の失敗パス網羅。現行ファイル欠落（SPEC-LGX-010.REQ.03 = ERROR + exit 1）、ベースライン不在（INFO + exit 0）、モデル解決失敗（exit 1）など drift 特有の失敗パスが UC に定義されているか
- [ ] 観点 EF3: エラー時の状態不変条件。embed は書き込み操作だが、ノード単位 Tx ロールバックにより「失敗したノードは未更新状態のまま」という事後条件が UC フローで担保されているか（SCORE-INV-1 freshness の整合）
- [ ] 観点 EF4: エラー時のユーザ通知（stderr/stdout・終了コード）が UC フローで定義されているか。代替フロー 2a は「ERROR を報告する」のみで出力先・終了コードが未定義

### 2.4 アクター遷移と権限

- [ ] 観点 AT1: アクター（開発者 / CI システム）の権限の一貫性。embed は engine.db への書き込みを伴う書き込み操作であり、check（読み取り専用）と権限要件が異なる点が UC で考慮されているか
- [ ] 観点 AT2: drift の責任境界。Step4「リンクされた成果物のレビューを推奨する」はシステムが「推奨」を出力するだけであり、実際のレビュー・修正はアクター責務であることが UC フローで明示されているか
- [ ] 観点 AT3: `--subnodes` 指定権限。Phase 2 Block F 以降でのみ意味を持つ `--subnodes` が、フェーズ未達の状態で実行された場合の挙動が UC フローで成立しているか（前提条件の不足）

### 2.5 データフロー

- [ ] 観点 DF1: embed の入出力データフロー。入力（graph.toml + 各成果物ファイル本文 + ONNX モデル）→ 出力（engine.db embeddings テーブル更新 + stdout/stderr）のフローが UC で観察可能か。SPEC-LGX-006.REQ.03 の 7 必須格納項目が事後条件として参照可能か
- [ ] 観点 DF2: drift の入出力データフロー。入力（`<artifact_id>` + engine.db 既存 embedding + ONNX モデル）→ 出力（drift 値 + レビュー推奨 = stdout/stderr）。SPEC-LGX-010.REQ.03 は「`--json` 時のスキーマ」「ベースライン不在時の null 返却」を規定するが、UC フロー Step3〜4 でこれらが反映されているか
- [ ] 観点 DF3: SCORE-INV-1（ハッシュ一致保証）のデータフロー整合。Step3b（content_hash 比較）の skip ロジックが UC で観察可能か（SPEC-LGX-006.REQ.02 のスキップ判定、SCORE-INV-1 との整合）

### 2.6 領域固有観点（embedding / drift UC）

- [ ] 観点 R1: embed コマンドと drift コマンドのフロー分離。UC-LGX-007 は embed と drift を同一 UC 内に記述しているが、両者の事前条件・事後条件・フローが明確に分離されているか（embed は生成・格納、drift は読取・比較・報告の責務）
- [ ] 観点 R2: SCORE-INV-2（モデルバージョン一致）の事後条件への反映。UC の事後条件「モデルバージョンが記録される（SCORE-INV-2）」が SPEC-LGX-006.REQ.10（model_version 複合キー生成・完全一致判定）の規定と整合するか、または観察可能な外部状態として定義されているか
- [ ] 観点 R3: drift フロー Step2「現在の embedding を生成する」の意味。drift が「現行ファイル内容から embedding をその場生成する」（engine.db への永続化なし）なのか「既存 embedding を読み取る」なのかが UC フローで曖昧。SPEC-LGX-010.REQ.03 は「現行ファイル内容から生成した embedding」と明示するが、UC Step2 の記述では不分明
- [ ] 観点 R4: drift の閾値判定フロー。Step4「ドリフトが閾値以上の場合、リンクされた成果物のレビューを推奨する」において、閾値の出所（`.legixy.toml` 設定値）・閾値未満の場合の明示的な事後条件・`--json` 出力形式が UC フロー記述から観察可能か（SPEC-LGX-010.REQ.03 へ委譲できるか、UC 固有の記述漏れか）
- [ ] 観点 R5: 凍結済み引数契約との整合。UC Step1 の引数記述（`embed [--all] [--subnodes]`、`drift <artifact_id>`）が LGX-COMPAT-001 §4 #4/§4 #5 の凍結済み引数（`--force`・`--node` の加算的拡張を含む）と整合するか

## 3. RED / GREEN 判定

| 観点 | 判定 | 親 SPEC / UC §で回答（委譲先） | 関連 GAP |
|---|---|---|---|
| 2.1 BF1 embed ステップ連鎖 | GREEN | Step1→2→3（ハッシュ→スキップ→前処理→推論→格納）の連鎖はSPEC-LGX-006.REQ.02（変更有無判定）+ REQ.08（Tx 境界）+ REQ.01（推論パイプライン）で規定済み。UC フローの記述は SPEC の処理順と整合 | — |
| 2.1 BF2 drift ステップ連鎖 | GREEN | Step1→2（現行生成）→3（既存と比較）→4（閾値判定）の連鎖は SPEC-LGX-010.REQ.03（drift 値定義・ベースライン選択）と整合。「生成完了前に比較」は逐次処理前提で UC フロー上問題なし | — |
| 2.1 BF3 embed 成功時事後条件の観察可能性 | GREEN | 事後条件「embeddings テーブル更新」「モデルバージョン記録（SCORE-INV-2）」は外部観察可能（後続 check / drift / report の前提として参照可）。格納項目の詳細は SPEC-LGX-006.REQ.03 へ委譲 | — |
| 2.1 BF4 `--subnodes` 連鎖の観察可能性 | GREEN | Step4「サブノードの embedding も生成する」は Step3 と同一フロー（ハッシュ→スキップ→推論→格納）の繰り返しを示す。SPEC-LGX-006.REQ.09/REQ.12 へ委譲。UC フロー上の連鎖として成立 | — |
| 2.2 AF1 代替フロー 2a の終了コード未定義 | RED | UC の代替フロー 2a「ONNX モデルが存在しない場合、ERROR を報告する」は出力先・終了コードを記述しない。SPEC-LGX-006.REQ.02 は「exit 1 + 試行パスを stderr 通知」と規定するが、UC フロー記述への反映が欠落。【WEAK: SPEC-006.REQ.02 への委譲で解決可。UC への明示記述は任意か必須かは人間裁定】 | GAP-LGX-241 |
| 2.2 AF2 代替フロー 3b の事後収束 | GREEN | 3b「ハッシュ比較をスキップして全ノードを再生成する」は Step3 の b ステップ（比較スキップ）から d（推論）→ e（格納）への収束として読める。SPEC-LGX-006.REQ.02 `--force` 仕様（hash 一致でも強制再生成）へ委譲。事後収束として成立 | — |
| 2.2 AF3 代替フロー分岐の非網羅 | RED | UC-LGX-007 の代替フローは 2a/3b の 2 件のみ。drift コマンド側の代替フロー（ベースライン不在=INFO+exit 0・`--against` 指定・モデル解決失敗・現行ファイル欠落=ERROR+exit 1・model_version 不一致=exit 1）が UC フローに一切記述されていない。SPEC-LGX-010.REQ.03 が詳細を規定するが、UC フロー記述への反映が欠落している（drift フロー分岐が観察不能）。【GENUINE: UC フロー記述の構造的欠落。drift 運用コマンドの代替フローは UC レベルで定義されるべき事項】 | GAP-LGX-242 |
| 2.3 EF1 embed 部分失敗継続の欠落 | RED | UC Step3 は各ノードの処理ステップ（a→b→c→d→e）を列挙するが、「1 ノード失敗時に当該 Tx のみ rollback して後続継続」という部分失敗継続挙動（SPEC-LGX-006.REQ.08 規定）が UC フローに現れない。事後条件（更新件数・失敗件数・部分的 CheckReport）が観察不能。【WEAK: SPEC-006.REQ.08 委譲で解決可。UC への明示は任意か必須かは裁定待ち】 | GAP-LGX-243 |
| 2.3 EF2 drift 失敗パスの欠落 | RED | drift の基本フローに失敗パスが定義されていない（代替フロー AF3 で指摘の通り、drift の代替フローが UC に存在しない）。EF2 は AF3（GAP-LGX-242）と根が同じため、GAP-LGX-242 に統合 | GAP-LGX-242 |
| 2.3 EF3 エラー時の不変条件保持 | GREEN | embed は Tx ロールバック（SPEC-LGX-006.REQ.08）で失敗ノードを未更新状態に保つ。SCORE-INV-1 freshness は engine.db の content_hash 管理で維持。UC フロー上の状態整合は SPEC-006.REQ.08/REQ.05 へ委譲で成立 | — |
| 2.3 EF4 エラー通知（出力先・終了コード）の未定義 | RED | AF1（GAP-LGX-241）と同根。UC 全体として embed/drift のエラー通知（stderr/stdout・終了コード 0/1/2）が UC フロー・事後条件に記述されていない。AF1 の GAP-LGX-241 に統合 | GAP-LGX-241 |
| 2.4 AT1 embed の書き込み権限 | GREEN | embed が engine.db 書き込みを伴うことは事後条件「embeddings テーブルが更新される」から観察可能。権限管理の詳細は NFR-LGX-001.SEC.01/REL.06 へ委譲。UC フロー上の前提として engine.db 存在（なければ自動作成）が事前条件に明示されている | — |
| 2.4 AT2 drift の責任境界 | GREEN | Step4「レビューを推奨する」はシステムが出力するのみ（観察可能な出力行為）であり、是正はアクター責務であることが UC フロー上で分担として成立。SPEC-LGX-010.REQ.03 への委譲で確認 | — |
| 2.4 AT3 `--subnodes` のフェーズ前提 | GREEN | `--subnodes` は UC Step4 で「指定された場合」という条件付きで記述。Phase 2 有効化は SPEC-LGX-006.REQ.09 が所有する範囲外フロー制御であり、UC レベルでの条件付き記述として整合 | — |
| 2.5 DF1 embed の入出力データフロー | GREEN | 入力（graph.toml + 成果物ファイル本文 + ONNX モデル）→ 出力（engine.db 更新）が UC 事前条件・事後条件と Step2〜3 の記述から観察可能。格納 7 項目は SPEC-LGX-006.REQ.03 へ委譲 | — |
| 2.5 DF2 drift の JSON/null 出力フロー | GREEN | drift Step4 の「レビューを推奨する」は観察可能な出力。`--json` スキーマ・ベースライン不在時の null 返却は SPEC-LGX-010.REQ.03 へ委譲で回答。UC フロー記述のデータフローとして成立（詳細は SPEC-010 が所有） | — |
| 2.5 DF3 SCORE-INV-1 スキップロジック | GREEN | Step3b「engine.db の既存ハッシュと比較し、変更がなければスキップする（SCORE-INV-1）」でスキップ判定のデータフローが UC で観察可能。不変条件引用も明示 | — |
| 2.6 R1 embed / drift フローの分離明確性 | GREEN | UC は embed（§基本フロー: embedding 生成）と drift（§基本フロー: ドリフト検出）を見出しで分離して記述。事前条件・事後条件も embed の事後状態（embeddings テーブル更新）が drift の前提（既存 embedding の参照）となる構造を暗示。責務境界は SPEC-LGX-006/SPEC-LGX-010 へ委譲 | — |
| 2.6 R2 SCORE-INV-2 の事後条件整合 | GREEN | 事後条件「モデルバージョンが記録される（SCORE-INV-2）」は SPEC-LGX-006.REQ.10（model_version 複合キー生成・完全一致判定）の規定と整合する宣言として機能。詳細は SPEC-006.REQ.10 へ委譲で成立 | — |
| 2.6 R3 drift Step2「現在の embedding 生成」の曖昧性 | RED | UC drift フロー Step2「システムが対象成果物の現在の embedding を生成する」は、「現行ファイル内容からその場で生成（engine.db への永続化なし）」なのか「既存 embedding を engine.db から読み取る」なのかが UC フローから不分明。SPEC-LGX-010.REQ.03 は「現行ファイル内容から生成した embedding」と明示するが、UC フロー記述は両解釈が可能。事後条件にも記述が無い。【GENUINE: UC フロー記述の観察可能性欠如。drift の本質的動作（on-the-fly 生成）が UC から読み取れない】 | GAP-LGX-244 |
| 2.6 R4 drift の閾値出所・閾値未満の事後条件 | RED | UC drift Step4「ドリフトが閾値以上の場合、リンクされた成果物のレビューを推奨する」において、①閾値の出所（設定ファイル参照）②閾値**未満**の場合の明示的な事後条件（「推奨なし」かどうか）が UC フローから観察不能。SPEC-LGX-010.REQ.03 は閾値設定を NFR/設定ファイルへ委譲しているが、UC フロー上の観察可能な分岐定義として欠落。【WEAK: SPEC-010.REQ.03 + NFR への委譲で解決可能。UC への明示は任意か必須かは裁定待ち】 | GAP-LGX-245 |
| 2.6 R5 凍結済み引数契約との整合 | GREEN | UC Step1 の `embed [--all] [--subnodes]` は LGX-COMPAT-001 §4 #4 の `[--all]`（凍結）+ `[--subnodes]`（加算的拡張）と整合。`drift <artifact_id>` は LGX-COMPAT-001 §4 #5（凍結）と整合。`--force` / `--node` / `--against` 等の追加引数は SPEC-LGX-006.REQ.02 / SPEC-LGX-010.REQ.03 が所有する加算的拡張として UC レベルでは委譲で成立 | — |

集計: **全 22 観点 / GREEN 15 / RED 7**（EF2 は AF3=GAP-LGX-242 に統合、EF4 は AF1=GAP-LGX-241 に統合。実質 GAP は 5 件：GAP-LGX-241/242/243/244/245）

## 4. ステータスの決定

RED 観点が 5 件（AF1=GAP-LGX-241、AF3=GAP-LGX-242、EF1=GAP-LGX-243、R3=GAP-LGX-244、R4=GAP-LGX-245）残存するため、本 TP のステータスは `**ステータス**: red`。

- GAP-LGX-242（drift 代替フロー欠落）・GAP-LGX-244（drift Step2 の観察可能性欠如）は GENUINE 寄り。UC フロー記述の構造的欠落であり、SPEC-010.REQ.03 への委譲では UC フロー記述の問題は解決しない。
- GAP-LGX-241（exit コード記述欠落）・GAP-LGX-243（embed 部分失敗継続の欠落）・GAP-LGX-245（閾値未満事後条件の欠落）は WEAK 候補。親 SPEC が既に答えており、UC フローへの明示反映が任意か必須かの裁定が必要。
- 敵対的精査パスで GENUINE / WEAK / OUT_OF_SCOPE を確定し、WEAK 確定分は人間裁定（UC フローへの追記 or drop）を経て close。全観点 GREEN 化後に本 TP を green へ更新する。

## 5. 観点ナレッジベース参照

この TP の生成時に参照したナレッジベース章節:

- `docs/perspectives/core-perspectives.md` §UC レベル観点（基本フロー / 代替フロー / 例外フロー / アクター遷移と権限 / データフロー）
- `docs/perspectives/core-perspectives.md` §汎用観点（エラーハンドリング / 状態遷移 / 永続化）
- 親 SPEC: SPEC-LGX-006.REQ.01〜REQ.12、SCORE-INV-1/SCORE-INV-2
- 委譲先 TP: TP-LGX-006（embedding SPEC レベル観点、green 確定済）
- 関連 SPEC: SPEC-LGX-010.REQ.03（drift コマンド詳細）、SPEC-LGX-004.REQ.02（check 内 Drift との書き分け）
- LGX-COMPAT-001 §4 #4/§4 #5（embed/drift の凍結済み引数契約）

UX 層観点（Undo/フォーカス/タッチ等）は CLI コマンドには本質的に N/A のため、エラー通知 UX（EF4 統合済み）以外はスキップした。

## 6. 履歴

| 日付 | 変更内容 |
|---|---|
| 2026-06-13 | 初版作成。UC レベル観点 22 件（GREEN 15 / RED 7、EF2/EF4 を AF3/AF1 に統合し実質 GAP 5 件）。GAP-LGX-241（AF1 exit コード未定義）/ GAP-LGX-242（AF3 drift 代替フロー欠落）/ GAP-LGX-243（EF1 embed 部分失敗継続欠落）/ GAP-LGX-244（R3 drift Step2 曖昧性）/ GAP-LGX-245（R4 閾値未満事後条件欠落）を起票 |

## 7. 解消（2026-06-13、敵対的精査裁定後）

本 TP が起票した GAP[UC] は全て closed。内訳: **WEAK=方針B（委譲容認）** / **REFUTED=棄却** / **GENUINE=UC 修正で解消**（A/B/C、人間承認 2026-06-13）。§3 表の判定列は初版（起票時）の draft 判定を保持する（精査の履歴として温存）。全 RED 観点は上記裁定で解消したため本 TP は **green**。各 GAP の最終状態は当該 GAP ファイル（§5）と docs/spec-change-proposals/2026-06-13_uc-gap-adversarial-review.md を参照。
