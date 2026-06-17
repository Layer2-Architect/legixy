Document ID: TP-LGX-012

# TP-LGX-012: UC-LGX-002「コンテキスト解決」観点（UC レベル）

> TP は **テストケース** ではなく **観点リスト**。UC レベル TP は「ユースケースのフロー記述に問いかける質問のリスト」として書く。SPEC レベル TP（TP-LGX-003）が「仕様が答えるか」を問うのに対し、UC レベル TP は「フローが先行成果物（親 SPEC）を観察可能なステップへ忠実かつ完全に具体化しているか」を問う。

**親**: UC-LGX-002
**ステータス**: green
**最終更新**: 2026-06-13

## 1. 対象スコープ

この TP は UC-LGX-002「コンテキスト解決（compile_context）」の全フロー（基本フロー Step 1〜7、代替フロー 2a/3a/4-A/4-B/4-C、事後条件）に UC レベル観点をぶつける。

- 対象: UC-LGX-002 全節（概要・アクター・事前条件・基本フロー・代替フロー・事後条件・関連不変条件・関連要求）
- 親 SPEC: SPEC-LGX-003 REQ.01〜REQ.20（コンテキスト解決）
- 関連 SPEC §: LEGIXY-SPEC-001 §10（CTX-INV-1〜4）、LGX-EXT-001 §5.1 / §5.4（Phase 2 Block B）、LGX-EXT-002 §3.2（CACHE-INV-2）、SPEC-LGX-009 REQ.15（MCP depth≥1 制約）、LGX-COMPAT-001 §3 / §4（終了コード凍結契約）
- 委譲方針: compile_context の検証/処理セマンティクスそのもの（決定論保証・バイト単位整列・キャッシュブレーク点・サイズ上限・並行排他・フラグ組合せ優先順位の**規定そのもの**）は TP-LGX-003（green 確定済）が所有する。本 TP はそれらを再検証せず、「UC-LGX-002 のフロー記述が SPEC-LGX-003 の規定を観察可能なステップとして正しく具体化しているか」のみを問う。

## 2. 観点リスト

### 2.1 基本フロー（ステップ連鎖の整合）

- [ ] 観点 BF1: 各ステップの事後条件が後続ステップの前提を満たすか（Step1 コマンド受理 → Step2 逆引き → Step3 グラフ走査 → Step4 レイヤールール解決 → Step5 カスタムエッジ解決 → Step6 ContextResult 返却 → Step7 監査ログ記録の連鎖整合）
- [ ] 観点 BF2: Step1 に登場する `--command <intent>` フラグが UC の関連要求（SPEC-LGX-003.REQ.01〜REQ.20）のいずれかに対応しているか。SPEC に未定義のフラグが UC フローに先行して出現しており、フロー記述の逸脱・将来拡張への言及漏れが無いか
- [ ] 観点 BF3: Step6 の ContextResult 構造（targets / upstream / layer_documents / custom_documents の 4 フィールド）と SPEC-LGX-003.REQ.10 が規定する 5 セクション配置（Layer Guidelines / Additional Guidelines / キャッシュブレーク点マーカ / Upstream Artifacts / Target Node Metadata）の対応関係が UC フローで観察可能か。UC の返却フィールド定義がセクション順序・構造と整合しているか
- [ ] 観点 BF4: 成功時の事後条件（CTX-INV-1 決定論保証 + MCP-INV-4 監査ログ完全性）が外部観察可能か。後続 UC（UC-LGX-007 フィードバックループ等）の前提として参照可能か

### 2.2 代替フロー（分岐網羅）

- [ ] 観点 AF1: 代替フロー 2a の「ファイルがどのノードにも対応しない場合、targets の artifact_id を null として返す」と SPEC-LGX-003.REQ.20「指定された起点のうち graph.toml に未登録のものは無視し、残りの起点で解決して exit 0」の整合性。REQ.20 は「無視して残りで解決」を規定するが、UC は「artifact_id を null として返す」を記述しており、全起点未登録時・部分未登録時の挙動差が観察可能か
- [ ] 観点 AF2: 代替フロー 3a（上流不在の場合 upstream=空配列）が SPEC-LGX-003.REQ.02 / REQ.10（上流ゼロでもセクション構成は維持）と整合するか。空 upstream 時も ContextResult 構造全体が返却されることが UC フローで観察可能か
- [ ] 観点 AF3: Phase 2 Block B の代替フロー（4-A outline-only / 4-B sections / 4-C depth）の各発火条件が明示されているか。SPEC-LGX-003.REQ.18 が規定するフラグ組合せ優先順位（sections×document で sections 無視 / outline×sections で sections→outline の順）がフロー記述から読み取れるか
- [ ] 観点 AF4: 代替フロー 4-C の `--depth N` に N=0 を指定した場合（CLI=空集合 exit 0 / MCP=zod reject。SPEC-LGX-003.REQ.17）の分岐が UC フローで観察可能か

### 2.3 例外フロー（失敗パス）

- [ ] 観点 EF1: 返却本文が 500,000 文字を超えた場合（SPEC-LGX-003.REQ.13 大規模返却エラー）の例外フロー。この失敗パスが UC フロー記述に存在しない。フロー上に相当する分岐・終端が定義されているか
- [ ] 観点 EF2: Step7 の context_log 書込失敗時の挙動（SPEC-LGX-003.REQ.19 本処理優先・別トランザクション・exit 0 + stderr Warning）が UC フローに反映されているか。Step7 後の終端状態が「本処理成功・記録欠落も exit 0」として観察可能か
- [ ] 観点 EF3: エラー時のユーザ通知（サイズ超過エラーの提案文 / stderr Info 診断 / stderr Warning 診断）の出力先（stdout/stderr）と終了コードの不変性が UC フローで観察可能か
- [ ] 観点 EF4: SPEC-LGX-003.REQ.20 が規定する「上流連鎖途中の欠損（エッジ先ノードのファイル不在等）を飛ばして残りを返す部分成功・exit 0・欠損の決定論的記録」が UC フローに反映されているか

### 2.4 アクター遷移と権限

- [ ] 観点 AT1: アクター（Claude Code 経由 / 開発者 CLI 直接実行）の権限・状態が一貫しているか。両アクターが同一の context コマンドを同一権限で実行可能であることが UC で一貫しているか
- [ ] 観点 AT2: 責任境界。システムは上流コンテキストの解決と返却のみを行い、返却されたコンテキストを使って何を生成するかはアクター（Claude Code / 開発者）責務であることの分担が明示されているか
- [ ] 観点 AT3: `--command <intent>` が意図として渡される場合、その intent がシステム側で上流解決の結果に影響する処理（フィルタリング・スコアリング等）として機能するのかどうかが UC フローに未明示である。アクターの入力が解決ロジックに影響する範囲が観察可能か

### 2.5 データフロー

- [ ] 観点 DF1: 入出力データの型・制約。入力（target_files パス配列 + 各オプション引数）→ 出力（ContextResult 構造体 = targets + upstream + layer_documents + custom_documents）のフィールド定義が SPEC-LGX-003.REQ.02 / REQ.10 の 5 セクション構成と対応しているか
- [ ] 観点 DF2: Step7 で context_log に記録される情報の範囲（SPEC-LGX-003.REQ.07「granularity カラム追加」、SPEC-LGX-007.REQ.06 所管）が UC フローで観察可能か。監査ログに何が記録されるかがフロー記述から読み取れるか
- [ ] 観点 DF3: engine.db が存在しない状態（FB-INV-4 / SPEC-LGX-003 §4）での動作（graph.toml のみで上流返却・context_log 記録スキップ）が UC の事前条件・例外フローで明示されているか

### 2.6 領域固有観点（コンテキスト解決 UC）

- [ ] 観点 R1: `--granularity` パラメータ（SPEC-LGX-003.REQ.01/REQ.03）が UC のフローに現れない。UC-LGX-002 が粒度制御を UC-LGX-004（compile_context の粒度別フロー）へ委譲しているとすれば委譲先が明示されているか。委譲なしに SPEC 要求をフロー外に置いているなら UC の不完全性
- [ ] 観点 R2: CTX-INV-1（決定論保証）が事後条件「同じ入力に対して常に同じ結果が返される」として記述されている。SPEC-LGX-003.REQ.14（バイト単位決定論）まで強化された決定論の詳細は TP-LGX-003 に委譲済みとして UC フロー記述の範囲での整合が成立するか
- [ ] 観点 R3: CTX-INV-3（カスタムエッジ独立性）が代替フロー・事後条件に現れない。Step5「カスタムエッジに基づく追加文書を解決する」がカスタムエッジの追加によって上流エッジが変化しないこと（独立性）をフロー上で保証しているか
- [ ] 観点 R4: MCP-INV-2（忠実な転送）が関連不変条件に挙げられているが、UC フローのどのステップで MCP 転送の忠実性（SPEC-LGX-009 REQ.04 による snake→kebab 変換、`_meta` 付与）が観察可能か。CLI 直接実行と MCP 経由実行でアクターの違いがフロー上の差として現れるか

## 3. RED / GREEN 判定

| 観点 | 判定 | 親 SPEC / UC §で回答（委譲先） | 関連 GAP |
|---|---|---|---|
| 2.1 BF1 ステップ連鎖整合 | GREEN | 基本フロー Step1〜7 が事前条件（`.legixy.toml`/`graph.toml` 存在）→ Step2（逆引き）→ Step3（走査）→ Step4/5（ガイドライン解決）→ Step6（返却）→ Step7（ログ記録）と連鎖。各事後条件が後続前提を満たす | — |
| 2.1 BF2 `--command` フラグの SPEC 対応 | RED | UC-LGX-002 Step1 に `--command <intent>` が登場するが SPEC-LGX-003.REQ.01〜REQ.20 のいずれにも `--command` への要求定義が存在しない。SPEC に未定義のフラグがフローに先行出現しており、フロー記述が親 SPEC を忠実に具体化しているかが検証不能。【GENUINE: SPEC と UC フロー記述の乖離】 | GAP-LGX-191 |
| 2.1 BF3 ContextResult 構造と5セクション対応 | RED | UC Step6 の返却フィールド（targets / upstream / layer_documents / custom_documents）は SPEC-LGX-003.REQ.10 が規定する 5 セクション（Layer Guidelines / Additional Guidelines / キャッシュブレーク点マーカ / Upstream Artifacts / Target Node Metadata）と名称・構成が一致しない。REQ.10 の5セクション配置順序が ContextResult フィールド定義と対応しているかが UC フローで観察不能。【WEAK: TP-LGX-003 D-03 が GREEN 確定済。UC 記述の粒度の問題であり SPEC 規定はある】 | GAP-LGX-192 |
| 2.1 BF4 成功時事後条件の観察可能性 | GREEN | 事後条件「同じ入力に対して常に同じ結果（CTX-INV-1）」 + 「context_log に記録（MCP-INV-4）」を明示。外部観察可能・後続 UC 前提として参照可。詳細は SPEC-LGX-003.REQ.04/REQ.07/TP-LGX-003 D-01/LOG-01 へ委譲 | — |
| 2.2 AF1 代替フロー 2a と REQ.20 の整合 | RED | 代替フロー 2a「ファイルがどのノードにも対応しない場合、targets の artifact_id を null として返す」は SPEC-LGX-003.REQ.20「未登録起点は無視して残りで解決し exit 0」と挙動記述が乖離する。全起点未登録（空 upstream exit 0）時と部分未登録時の差がフローで観察できない。【GENUINE: SPEC 規定（REQ.20）とフロー記述の直接矛盾】 | GAP-LGX-193 |
| 2.2 AF2 代替フロー 3a と上流空時の ContextResult 構造 | GREEN | 代替フロー 3a「upstream を空配列として返す」は SPEC-LGX-003.REQ.02 / REQ.10 整合（上流ゼロでもセクション構成維持）。詳細は TP-LGX-003 L-01（GREEN 確定済）へ委譲 | — |
| 2.2 AF3 Phase2 代替フロー発火条件とフラグ組合せ | RED | 代替フロー 4-A/4-B/4-C は各フラグの個別動作を記述するが SPEC-LGX-003.REQ.18 が規定するフラグ組合せ優先順位（sections×document で sections 無視 / outline×sections で sections 先・outline 後 / depth 直交）がフロー記述に現れない。複数フラグ同時指定時の挙動が観察不能。【WEAK: TP-LGX-003 D-08 が GREEN 確定済。UC フロー記述への明示反映の要否が未裁定】 | GAP-LGX-194 |
| 2.2 AF4 `--depth 0` の CLI/MCP 差分 | GREEN | SPEC-LGX-003.REQ.17（CLI=空集合 exit 0 / MCP=zod reject）は TP-LGX-003 B-03/L-04（GREEN 確定済）で答えられている。UC の代替フロー 4-C 「上流走査を N 階層に制限」はこの境界を記述していないが、CLI/MCP 受理差の詳細は MCP 層（SPEC-LGX-009）および終了コード凍結契約（LGX-COMPAT-001 §1）への委譲で整合 | — |
| 2.3 EF1 大規模返却エラー（REQ.13）の例外フロー | RED | SPEC-LGX-003.REQ.13（500,000 文字超過時のエラー返却・切り捨て禁止）に対応する例外フロー分岐が UC-LGX-002 の代替フロー・例外フローに一切存在しない。この失敗パスは UC の通常フロー（Step6 返却）中に発生しうる重要な分岐であり、フロー記述の欠落。【GENUINE: SPEC 規定に対応する UC フロー記述が無い】 | GAP-LGX-195 |
| 2.3 EF2 context_log 書込失敗時の終端状態 | RED | SPEC-LGX-003.REQ.19（本処理優先・別トランザクション・exit 0 + stderr Warning）が Step7 の失敗パスとして UC フローに反映されていない。Step7 後の「記録失敗 = exit 0 継続」という終端状態が観察不能。【WEAK: TP-LGX-003 E-03（GAP-LGX-041 closed）で確定済。UC への明示反映の要否が未裁定】 | GAP-LGX-196 |
| 2.3 EF3 エラー通知の出力先と終了コード不変性 | GREEN | SPEC-LGX-003.REQ.16/REQ.17【v3 差分】（Info 診断は stderr・stdout/終了コード不変）は TP-LGX-003 LOG-04（GREEN 確定済）で答えられている。UC 事後条件「同じ入力に対して同じ結果」と整合 | — |
| 2.3 EF4 上流部分欠損の部分成功フロー（REQ.20） | RED | SPEC-LGX-003.REQ.20「上流連鎖途中の欠損を飛ばして残りを返す部分成功・exit 0・欠損の決定論的記録」が UC フローに反映されていない。基本フロー Step3（グラフ走査）中に欠損が発生した場合の継続挙動が UC から観察不能。【WEAK: TP-LGX-003 E-05（GAP-LGX-043 closed）で確定済。UC への明示反映の要否が未裁定】 | GAP-LGX-197 |
| 2.4 AT1 アクター権限の一貫性 | GREEN | Claude Code（MCP 経由）/ 開発者（CLI 直接）とも同一の context コマンドを実行。権限差は本質的に存在せず（読み取り + ログ書込のみ）UC 記述と一貫 | — |
| 2.4 AT2 責任境界（解決 vs 利用） | GREEN | UC 概要「参照すべき上流成果物を取得する」・Step6「返却する」でシステム=解決・アクター=利用の分担を示している。明示的な分担記述としては薄いが SPEC 委譲で整合 | — |
| 2.4 AT3 `--command` が解決ロジックに影響するか | RED | UC Step1 に `--command <intent>` が登場するが、この intent が上流解決（Step2〜5）に影響するかどうかが UC フローで不明。SPEC-LGX-003 に `--command` の定義がないため委譲先が存在せず、アクターの入力とシステムの処理範囲が観察不能。BF2 と同根だが観点は独立（アクター側の影響範囲の明示が焦点）。【GENUINE: SPEC-UC 間の乖離で委譲先不在】 | （GAP-LGX-191 に統合） |
| 2.5 DF1 入出力データの型・制約 | GREEN | 入力（target_files 配列 + オプション引数）→ 出力（ContextResult 4 フィールド）。詳細な型・制約は SPEC-LGX-003.REQ.01/REQ.02/REQ.10 + TP-LGX-003 各観点（GREEN 確定済）に委譲。UC フロー記述の粒度としては適切 | — |
| 2.5 DF2 context_log 記録範囲 | GREEN | Step7「context_log に監査ログを記録する」で記録ステップが観察可能。記録項目の詳細（timestamp / target_files / 返却ノード / granularity カラム）は SPEC-LGX-003.REQ.07 + SPEC-LGX-007.REQ.06 へ委譲（TP-LGX-003 LOG-02 GREEN 確定済） | — |
| 2.5 DF3 engine.db 不在時の動作 | GREEN | 事前条件に engine.db の存在要件が記載されていない（graph.toml のみ必須）。SPEC-LGX-003 §4 FB-INV-4（DB 不在時も graph.toml のみで上流を返す）+ TP-LGX-003 P-01（GREEN 確定済）に委譲。UC の事前条件記述と整合 | — |
| 2.6 R1 `--granularity` の UC フロー記述 | RED | SPEC-LGX-003.REQ.01/REQ.03 が規定する `--granularity` パラメータ（document/subnode の 2 値）が UC-LGX-002 の基本フロー・代替フローに現れない。UC-LGX-004 への委譲であれば明示が必要。委譲先の記述がなく、SPEC 要求とフロー記述の不対応が観察不能。【GENUINE: フロー記述が SPEC REQ を具体化していない】 | GAP-LGX-198 |
| 2.6 R2 決定論保証の UC フロー整合 | GREEN | 事後条件「同じ入力に対して常に同じ結果（CTX-INV-1）」が SPEC-LGX-003.REQ.04/REQ.14 の委譲先として機能。バイト単位決定論の詳細は TP-LGX-003 D-01/D-02（GREEN 確定済）へ委譲 | — |
| 2.6 R3 CTX-INV-3 カスタムエッジ独立性のフロー整合 | GREEN | Step5「カスタムエッジに基づく追加文書を解決する」でカスタムエッジ処理をステップとして明示。独立性（CTX-INV-3）の詳細は SPEC-LGX-003.REQ.05 + TP-LGX-003 D-07（GREEN 確定済）へ委譲。UC フロー記述の粒度として整合 | — |
| 2.6 R4 MCP-INV-2 忠実転送のフロー観察可能性 | GREEN | UC の関連不変条件に MCP-INV-2 を挙げているが、忠実転送の詳細（snake→kebab 変換、`_meta` 付与）は SPEC-LGX-009.REQ.04/REQ.15 + TP-LGX-003 F-02/F-03（GREEN 確定済）に委譲。アクター（Claude Code MCP / 開発者 CLI）の記述でフロー上の差異の存在を示しており委譲として整合 | — |

集計: **全 22 観点 / GREEN 14 / RED 8**（RED は BF2 / BF3 / AF1 / AF3 / EF1 / EF2 / EF4 / R1。AT3 は BF2 に統合のため独立 GAP なし）

## 4. ステータスの決定

RED 観点が 7 件残存（BF2 / BF3 / AF1 / AF3 / EF1 / EF2 / EF4 / R1。うち AT3 は BF2 に統合）するため、本 TP のステータスは `**ステータス**: red`。

- GENUINE 候補: BF2（`--command` SPEC 未定義）/ AF1（代替フロー 2a と REQ.20 の直接矛盾）/ EF1（REQ.13 大規模返却エラーの例外フロー欠落）/ R1（`--granularity` フロー未記述）。いずれも親 SPEC に規定があるかフロー記述との乖離が明確。
- WEAK 候補: BF3（ContextResult 構造と5セクション対応）/ AF3（フラグ組合せフロー記述）/ EF2（context_log 書込失敗フロー反映）/ EF4（上流部分欠損フロー反映）。TP-LGX-003 が GREEN 確定済であり UC フロー記述への明示反映の要否が人間裁定待ち。
- 全観点 GREEN 化後に本 TP を green へ更新する。

## 5. 観点ナレッジベース参照

この TP の生成時に参照したナレッジベース章節:

- `docs/perspectives/core-perspectives.md` §UC レベル観点（基本フロー / 代替フロー / 例外フロー / アクター遷移と権限 / データフロー）
- `docs/perspectives/core-perspectives.md` §汎用観点（エラーハンドリング / 状態遷移 / ロギング・観測性）
- 親 SPEC: SPEC-LGX-003 REQ.01〜REQ.20、LEGIXY-SPEC-001 §10（CTX-INV-1〜4）
- 委譲先 TP: TP-LGX-003（コンテキスト解決 SPEC レベル観点、green 確定済）
- LGX-COMPAT-001 §3 / §4（終了コード凍結契約）、SPEC-LGX-009 REQ.15（MCP 受理範囲）

UX 層観点（Undo/フォーカス等）は CLI/MCP コンテキスト解決コマンドには本質的に N/A のためスキップした。

## 6. 履歴

| 日付 | 変更内容 |
|---|---|
| 2026-06-13 | 初版作成。UC レベル観点 22 件（GREEN 14 / RED 8）。GAP-LGX-191〜198 を起票 |

## 7. 解消（2026-06-13、敵対的精査裁定後）

本 TP が起票した GAP[UC] は全て closed。内訳: **WEAK=方針B（委譲容認）** / **REFUTED=棄却** / **GENUINE=UC 修正で解消**（A/B/C、人間承認 2026-06-13）。§3 表の判定列は初版（起票時）の draft 判定を保持する（精査の履歴として温存）。全 RED 観点は上記裁定で解消したため本 TP は **green**。各 GAP の最終状態は当該 GAP ファイル（§5）と docs/spec-change-proposals/2026-06-13_uc-gap-adversarial-review.md を参照。
