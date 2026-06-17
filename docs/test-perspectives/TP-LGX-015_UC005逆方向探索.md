Document ID: TP-LGX-015

# TP-LGX-015: UC-LGX-005「逆方向探索」観点（UC レベル）

> TP は **テストケース** ではなく **観点リスト**。UC レベル TP は「ユースケースのフロー記述に問いかける質問のリスト」として書く。SPEC レベル TP（TP-LGX-005）が「仕様が答えるか」を問うのに対し、UC レベル TP は「フローが先行成果物（親 SPEC）を観察可能なステップへ忠実かつ完全に具体化しているか」を問う。

**親**: UC-LGX-005
**ステータス**: green
**最終更新**: 2026-06-13

## 1. 対象スコープ

この TP は UC-LGX-005「逆方向探索（investigate）」の全フロー（基本フロー Step 1〜5、代替フロー 1a/3a、事後条件）に UC レベル観点をぶつける。

- 対象: UC-LGX-005 全節（概要・アクター・事前条件・基本フロー・代替フロー・事後条件・関連仕様）
- 親 SPEC: SPEC-LGX-005（グラフ走査）REQ.01〜REQ.10
- 関連 SPEC §: SPEC-LGX-002（グラフ基盤: BFS エッジ種別 / 決定論 REQ.08）、SPEC-LGX-006（drift スコア算出・drift_threshold）、LGX-COMPAT-001 §3/§4（`investigate` 引数・終了コード凍結）、LEGIXY-SPEC-001 §4（investigate エンジン機能）/ §5（双方向探索）
- 委譲方針: BFS 走査セマンティクス（visited 制御・深度 BFS・決定論・DAG 破れ安全性）は TP-LGX-005（SPEC-005 レベル、green 確定済）が所有する。drift スコアの算出・SCORE-INV-1 の規定は TP-LGX-006（SPEC-006 レベル、green 確定済）が所有する。本 TP はそれらを再検証せず、「UC-005 のフロー記述が SPEC-005 の規定を観察可能なステップとして正しく具体化しているか」のみを問う。

## 2. 観点リスト

### 2.1 基本フロー（ステップ連鎖の整合）

- [ ] 観点 BF1: 各ステップの事後条件が後続ステップの前提を満たすか（Step1 コマンド受理 → Step2 逆方向 BFS 走査 → Step3 ドリフトスコア参照 → Step4 疑わしいノードマーク → Step5 結果返却の連鎖整合）
- [ ] 観点 BF2: 事前条件（`graph.toml` と `engine.db` が存在する）がフロー各ステップの前提を実際に成立させるか。とくに Step3 でドリフトスコアを参照するには `engine.db`（embedding 済み）が必要であり、事前条件がその保証を担うことが観察可能か
- [ ] 観点 BF3: 成功時の事後条件（「走査結果が標準出力に表示される」「グラフの状態は変更されない（読み取り専用）」）が外部観察可能で、かつ後続 UC の前提として参照可能か

### 2.2 代替フロー（分岐網羅）

- [ ] 観点 AF1: 分岐の網羅性。UC が列挙する代替フロー（1a：`--drift-threshold` 未指定時 / 3a：embedding 未生成時）が SPEC-LGX-005 の前提（drift_threshold 省略・engine.db 不在）の全 case を被覆しているか。`--max-depth` 未指定（REQ.04 既定=無制限）という代替フローが UC に存在しないことは問題がないか
- [ ] 観点 AF2: 代替フロー 1a の事後条件収束。「設定ファイルの `drift_threshold` を使用する」という振る舞いは、基本フロー Step4（閾値判定）と同じ出力形式（suspicious_nodes 含む結果）に収束するか。収束先が UC フローで観察可能か
- [ ] 観点 AF3: 代替フロー 3a の事後条件収束。「ドリフトスコアなしで走査結果のみ返す」という振る舞いは具体的にどのような出力（suspicious_nodes が空 or 省略、depth_map・visited は通常通り）に収束するか。出力フォーマット差分が UC フローで観察可能か
- [ ] 観点 AF4: 代替フロー 3a への遷移条件の明示。「embedding が未生成の場合」は engine.db 全体の不在か、起点の embedding のみ未生成か、全ノードの embedding が未生成か。遷移発火条件が観察可能か

### 2.3 例外フロー（失敗パス）

- [ ] 観点 EF1: `graph.toml` が存在しない・破損している場合の失敗パスが UC のフロー記述に列挙されているか。事前条件「graph.toml が存在する」が崩壊したときの終了コードと出力先（stderr）が観察可能か
- [ ] 観点 EF2: `engine.db` が存在しない場合、代替フロー 3a「ドリフトスコアなしで走査結果のみ返す」で処理されるのか、それとも exit 1 の失敗パスに落ちるのか。この分岐が事前条件と代替フロー 3a の組合せで一貫しているか
- [ ] 観点 EF3: 起点 `start_ids` に存在しない ID が渡されたときの挙動（SPEC-LGX-005.REQ.05「空結果・非エラー」）が UC フローに反映されているか。特に exit コード（0 か 1 か）が観察可能か

### 2.4 アクター遷移と権限

- [ ] 観点 AT1: アクター（開発者 CLI / Linear Agent コンテナ）の権限・状態が一貫しているか。investigate が読み取り専用であり両アクターが同一権限で実行可能であることが UC で一貫しているか
- [ ] 観点 AT2: 責任境界。システムは探索と提示のみを行い、疑わしいノードの確認・修正はアクター責務であることの分担が UC フローで明示されているか（Step5 の「返却」が終端であり是正アクションを含まないことの確認）

### 2.5 データフロー

- [ ] 観点 DF1: 入出力データの型・制約。入力（`start_ids` + `--drift-threshold` / 設定ファイル値）→ 出力（visited / suspicious_nodes / depth_map の 3 フィールド）の構造が UC Step5 で観察可能か。stdout/stderr の分離が定義されているか
- [ ] 観点 DF2: `--drift-threshold` 値のデータフロー。CLI 引数値と設定ファイル値の優先規則（CLI 指定 > 設定ファイル既定）が 1a の代替フローと合わせて観察可能か
- [ ] 観点 DF3: suspicious_nodes の順序保証（スコア降順）が UC Step5 で明示され、かつ SPEC-LGX-005.REQ.03（BFS 決定論）の visited 順と独立していることが観察可能か

### 2.6 領域固有観点（グラフ走査・investigate UC）

- [ ] 観点 R1: UC の「逆方向（上流方向）に BFS 走査」（Step2）が SPEC-LGX-005.REQ.02（逆方向走査: `to`→`from` を辿る）へ忠実に対応しているか。「上流方向」という表現がエッジの逆方向 (`to`→`from`) と同義であることが観察可能か
- [ ] 観点 R2: `--max-depth` オプションの存在が UC フロー（Step1 のコマンド行）に現れていない（省略されている）ことは問題か。SPEC-LGX-005.REQ.04（max_depth 制御）が investigate に適用される旨が UC から委譲可能か
- [ ] 観点 R3: 走査結果の 3 フィールド（visited / suspicious_nodes / depth_map）が SPEC-LGX-005.REQ.09（走査結果の情報と出力フォーマット: ID 走査順 / タイプ / パス / 深度 / 使用エッジ）を漏れなく被覆しているか。`suspicious_nodes`（investigate 固有）は LEGIXY-SPEC-001 §4 / §5 へ委譲できるか
- [ ] 観点 R4: 終了コード契約。UC の事後条件に終了コード（0/1/2）の明示がないが、LGX-COMPAT-001 §3/§4（`investigate` の終了コード凍結）への委譲で観察可能か
- [ ] 観点 R5: MCP 非公開（SPEC-LGX-005.REQ.10、MCP-INV-1）が UC アクター欄（CLI のみ）で整合しているか

## 3. RED / GREEN 判定

| 観点 | 判定 | 親 SPEC / UC §で回答（委譲先） | 関連 GAP |
|---|---|---|---|
| 2.1 BF1 ステップ連鎖整合 | GREEN | Step1→2（コマンド受理→BFS）→Step3（drift スコア参照）→Step4（閾値判定）→Step5（出力）が事後条件 → 前提の連鎖を満たす。BFS 走査そのものは SPEC-LGX-005.REQ.02/03 へ委譲 | — |
| 2.1 BF2 事前条件とステップ前提の整合 | RED | Step3「各エッジのドリフトスコアを参照する」には engine.db（embedding 済み）が必要だが、事前条件「engine.db が存在する」が Step3 の前提を保証するとは限らない（engine.db が存在しても embedding 未生成の場合は 3a へ）。事前条件と代替フロー 3a の関係が UC フローに観察可能な形で定義されておらず、「engine.db が存在する」という事前条件の意味が曖昧。【GENUINE: 事前条件の意味定義がフローの分岐条件（3a）と衝突する可能性がある — SPEC 側でなく UC 記述のギャップ】 | GAP-LGX-221 |
| 2.1 BF3 成功時事後条件の観察可能性 | GREEN | 事後条件「走査結果が標準出力に表示される」が外部観察可能。読み取り専用は SPEC-LGX-005 §1.2 + SPEC-LGX-005.REQ.02 へ委譲 | — |
| 2.2 AF1 分岐網羅（drift-threshold / max-depth） | RED | 代替フロー 1a（drift-threshold 省略）/ 3a（embedding 未生成）は定義されているが、`--max-depth` 未指定時（SPEC-LGX-005.REQ.04: 省略=無制限）が UC フローに現れていない。基本フローの Step1 コマンド行にも `--max-depth` が記載されていない。SPEC-005.REQ.04 へ委譲可能とも取れるが、他の investigate 固有引数（`--drift-threshold`）は 1a で明示しているため、扱いの一貫性に欠く。【WEAK: SPEC-LGX-005.REQ.04 委譲容認の可能性。引数体系は LGX-COMPAT-001 §4 #12 へ委譲済みとも解釈できる】 | GAP-LGX-222 |
| 2.2 AF2 代替フロー 1a の収束 | GREEN | 1a「設定ファイルの `drift_threshold` を使用する」は、設定値を差し替えるだけで Step4 以降の基本フロー（疑わしいノードマーク → Step5 出力）に収束する。収束先は基本フローと同一出力形式。drift_threshold 解決順（CLI > 設定ファイル）は SPEC-LGX-006 / LGX-COMPAT-001 §6 へ委譲 | — |
| 2.2 AF3 代替フロー 3a の出力差分観察可能性 | RED | 3a「ドリフトスコアなしで走査結果のみ返す」は振る舞いを宣言しているが、出力フォーマット差分（suspicious_nodes が空配列か省略か、depth_map・visited は通常通りか）が UC フローで観察可能化されていない。SPEC-LGX-005.REQ.09 の出力フォーマットは走査結果全般を定義しているが、embedding 不在時の suspicious_nodes の扱いは明示されていない。【GENUINE: SPEC-005.REQ.09 が embedding 不在ケースの出力差分を規定していないため委譲先がない。UC フロー記述ギャップかつ親 SPEC にも答えがない】 | GAP-LGX-223 |
| 2.2 AF4 代替フロー 3a の遷移条件明示 | RED | 「embedding が未生成の場合」という遷移条件が曖昧。engine.db 全体不在・一部ノードの embedding 不在・全ノード embedding 不在で挙動が異なりうる。SPEC-LGX-006 / LEGIXY-SPEC-001 §4 に委譲できるが、UC フロー記述として発火条件が観察不能。【WEAK: SPEC-006 / LEGIXY-SPEC-001 §4 で答えがあれば委譲可。実際に SPEC を確認できていないため粒度を落として RED にする】 | GAP-LGX-224 |
| 2.3 EF1 graph.toml 不在・破損の失敗パス | RED | 事前条件「graph.toml が存在する」が崩壊した場合（不在 or 破損）の失敗パスが UC のフロー記述に列挙されていない。SPEC-LGX-005.REQ.05（起点不在=空結果）と SPEC-LGX-004.REQ.04（graph.toml 破損=exit 1）がそれぞれ答えているが、investigate UC のフロー記述に反映されていない。TP-LGX-011 の GAP-LGX-189 と同種。【WEAK: 親 SPEC（SPEC-004/005）委譲で解決可。UC への明示反映は任意の可能性】 | GAP-LGX-225 |
| 2.3 EF2 engine.db 不在と事前条件・3a の一貫性 | GREEN | 事前条件「engine.db が存在する」と代替フロー 3a（embedding 未生成時はスコアなし走査）との関係は、BF2/AF4 で RED 化済み。engine.db 不在 → 代替フロー 3a に収束すると読めば基本フローの EF としての失敗パスは不要とも判断できる。ただし BF2/AF4 の GAP 解消後に再評価する | — |
| 2.3 EF3 起点 ID 不在時の挙動 | RED | `start_ids` に存在しない ID が渡されたときの挙動（空結果・exit 0）が UC フロー記述に現れていない。SPEC-LGX-005.REQ.05「起点不在=空結果・非エラー」で規定済みだが、UC-005 の例外フロー（または代替フロー）に明示されていない。終了コード（exit 0）の観察可能性も未定義。【WEAK: SPEC-LGX-005.REQ.05 への委譲で解決可】 | GAP-LGX-226 |
| 2.4 AT1 アクター権限の一貫性 | GREEN | 開発者 CLI / Linear Agent コンテナともに読み取り専用 investigate を同一権限で実行。権限差は本質的に存在せず UC 記述と一貫。読み取り専用は事後条件「グラフの状態は変更されない」で明示 | — |
| 2.4 AT2 責任境界（探索 vs 是正） | GREEN | Step5 が「結果を返却する」で終端しており、是正アクションをアクター責務とする分担が観察可能。Linear Agent コンテナへの結果渡しも「返却」として一貫 | — |
| 2.5 DF1 入出力データ型・stdout/stderr 分離 | GREEN | 入力（start_ids + drift-threshold/設定値）→ 出力（visited/suspicious_nodes/depth_map）の構造が Step5 で明示。stdout/stderr 分離は SPEC-LGX-005.REQ.09（出力フォーマット）+ LGX-COMPAT-001 §3 へ委譲。3 フィールドの観察可能性は観点 R3 で評価 | — |
| 2.5 DF2 drift-threshold 優先規則 | GREEN | 基本フロー Step1 が CLI 引数 `--drift-threshold <val>` を明示し、代替フロー 1a が省略時に設定ファイル値を使用すると明示。CLI 指定 > 設定ファイル既定の優先規則が観察可能。解決の詳細は SPEC-LGX-006 / LGX-COMPAT-001 §6 へ委譲 | — |
| 2.5 DF3 suspicious_nodes のスコア降順保証 | GREEN | UC Step5「suspicious_nodes: ドリフト閾値以上のノード（スコア降順）」で順序保証を明示。visited 走査順との独立性は SPEC-LGX-005.REQ.03（BFS 決定論、TP-LGX-005 で GREEN）へ委譲 | — |
| 2.6 R1 逆方向 BFS と「上流方向」の対応 | GREEN | UC Step2「有向グラフを逆方向（上流方向）に BFS 走査する」が SPEC-LGX-005.REQ.02（逆方向走査）を具体化。括弧書き「上流方向」は説明語であり「逆方向」＝ `to`→`from` との対応は SPEC-LGX-005.REQ.01 の一般則（順方向 = from→to）の裏面として成立 | — |
| 2.6 R2 --max-depth の UC 記載省略 | GREEN（委譲） | UC Step1 のコマンド行に `--max-depth` が記載されていないことは AF1 で別途 RED 化。本観点は「委譲として整合するか」を問う。SPEC-LGX-005.REQ.04 が impact/investigate 両方に適用されると明記（§3 の CLI インターフェース REQ.07 が「どちらも --max-depth を受け付ける」と規定）しており、UC 省略はあるが SPEC 委譲として整合する。AF1 GAP の裁定後に確定 | （→ AF1） |
| 2.6 R3 出力 3 フィールドと REQ.09 の整合 | GREEN | visited（走査順）= REQ.09「到達したノードの ID 一覧（走査順）」/ depth_map = REQ.09「深度情報」/ suspicious_nodes = LEGIXY-SPEC-001 §4 investigate 固有出力。REQ.09 の「使用エッジ情報」は 3 フィールドのいずれとも対応していないが、SPEC-005.REQ.09 自体を変更する必要はなく investigate UC フローに「使用エッジ」の不在は委譲判断できる（visit 系 UC と運用目的の差） | — |
| 2.6 R4 終了コード契約 | GREEN | UC の事後条件に終了コードの明示はないが、SPEC-LGX-005.REQ.07（CLI インターフェース）+ LGX-COMPAT-001 §4 #12（`investigate` の引数・終了コード凍結）への委譲で観察可能。EF3 の exit 0（起点不在時）は別途 RED 化済み | — |
| 2.6 R5 MCP 非公開の UC アクター整合 | GREEN | UC アクター欄「開発者（CLI）/ Linear Agent コンテナ（不具合分析時）」が CLI のみを対象とし MCP アクターを含まない。SPEC-LGX-005.REQ.10（Admin Surface 限定・MCP-INV-1）と整合 | — |

集計: **全 22 観点 / GREEN 14 / RED 8**

## 4. ステータスの決定

RED 観点が 8 件残存するため、本 TP のステータスは `**ステータス**: red`。

- BF2（GENUINE）/ AF3（GENUINE）: 親 SPEC にも答えがないか、UC フロー記述と親 SPEC 規定が衝突している可能性がある真正ギャップ候補。人間裁定が特に必要。
- AF1（WEAK）/ AF4（WEAK）/ EF1（WEAK）/ EF3（WEAK）: 親 SPEC が既に答えており、UC フロー記述への反映が任意か必須かの裁定待ち。
- 全 RED 観点の GAP 解消（人間裁定 → A: UC 追記 or B: drop）を経て全 GREEN になれば本 TP を green へ更新する。

## 5. 観点ナレッジベース参照

この TP の生成時に参照したナレッジベース章節:

- `docs/perspectives/core-perspectives.md` §UC レベル観点（基本フロー / 代替フロー / 例外フロー / アクター遷移と権限 / データフロー）
- `docs/perspectives/core-perspectives.md` §汎用観点（エラーハンドリング / 状態遷移）
- 親 SPEC: SPEC-LGX-005.REQ.01〜REQ.10
- 委譲先 TP: TP-LGX-005（グラフ走査 SPEC レベル観点、green 確定済）
- LEGIXY-SPEC-001 §4（investigate エンジン機能）/ §5（双方向探索）
- LGX-COMPAT-001 §3/§4 #12（終了コード・引数凍結契約）

UX 層観点（Undo/フォーカス/タッチ等）は CLI 走査コマンドには本質的に N/A のためスキップした。

## 6. 履歴

| 日付 | 変更内容 |
|---|---|
| 2026-06-13 | 初版作成。UC レベル観点 22 件（GREEN 14 / RED 8）。GAP-LGX-221〜226 を起票（RED 6 件; BF2/AF3 は GENUINE、AF1/AF4/EF1/EF3 は WEAK） |

## 7. 解消（2026-06-13、敵対的精査裁定後）

本 TP が起票した GAP[UC] は全て closed。内訳: **WEAK=方針B（委譲容認）** / **REFUTED=棄却** / **GENUINE=UC 修正で解消**（A/B/C、人間承認 2026-06-13）。§3 表の判定列は初版（起票時）の draft 判定を保持する（精査の履歴として温存）。全 RED 観点は上記裁定で解消したため本 TP は **green**。各 GAP の最終状態は当該 GAP ファイル（§5）と docs/spec-change-proposals/2026-06-13_uc-gap-adversarial-review.md を参照。
