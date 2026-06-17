Document ID: TP-LGX-014

# TP-LGX-014: UC-LGX-004「粒度制御付きコンテキスト解決」観点（UC レベル）

> TP は **テストケース** ではなく **観点リスト**。UC レベル TP は「ユースケースのフロー記述に問いかける質問のリスト」として書く。SPEC レベル TP（TP-LGX-003）が「仕様が答えるか」を問うのに対し、UC レベル TP は「フローが先行成果物（親 SPEC）を観察可能なステップへ忠実かつ完全に具体化しているか」を問う。

**親**: UC-LGX-004
**ステータス**: green
**最終更新**: 2026-06-13

## 1. 対象スコープ

この TP は UC-LGX-004「粒度制御付きコンテキスト解決」の全フロー（基本フロー Step 1〜4、代替フロー 1a/3a/4-A/4-B/4-C/4-D、事後条件）に UC レベル観点をぶつける。

- 対象: UC-LGX-004 全節（概要・アクター・事前条件・基本フロー・代替フロー・事後条件・関連要求・関連不変条件）
- 親 SPEC: SPEC-LGX-003（コンテキスト解決）REQ.01〜REQ.20
- 関連 SPEC §: SPEC-LGX-003.REQ.03（粒度制御）・REQ.08（サブノード親への解決）・REQ.11（決定論的整列）・REQ.13（大規模返却エラー）・REQ.15（outline_only）・REQ.16（sections フィルタ）・REQ.17（depth_limit）・REQ.18（フラグ組合せ優先順位）
- 関連不変条件: CTX-INV-1（決定論保証）・MCP-INV-1（Agent Surface 限定）・MCP-INV-2（忠実な転送）
- 委譲方針: `compile_context` の上流解決セマンティクス（グラフ走査・レイヤーガイドライン解決・CTX-INV-1 の規定そのもの）は TP-LGX-003（green 確定済）が所有する。また UC-LGX-004 が「UC-LGX-002 基本フロー 2〜5 と同様」と委譲する部分の検証はすでに TP-LGX-012（UC-002 UCレベル TP）が担う。本 TP は「UC-004 が SPEC-003 の粒度制御規定（REQ.03/REQ.08/REQ.11/REQ.15〜18）を観察可能なステップへ忠実・完全に具体化しているか」のみを問う。

## 2. 観点リスト

### 2.1 基本フロー（ステップ連鎖の整合）

- [ ] 観点 BF1: 事前条件「上流成果物にサブノードが存在する」の検証タイミングが基本フロー内で観察可能か（Step 3 実行時に検出されるのか、事前の入力検証で弾くのかが不明確）
- [ ] 観点 BF2: Step 2「UC-LGX-002 の基本フロー 2〜5 と同様」という委譲記述の後、Step 3 への入力（上流成果物コレクション）が観察可能な形で受け渡されているか（委譲後の接続点が明示されているか）
- [ ] 観点 BF3: Step 4 の返却構造（subnode_id / anchor / content / drift_score を含む UpstreamArtifact）が、UC-LGX-002 の事後条件（ContextResult 構造）と整合しているか。拡張フィールドが基本フロー事後条件に反映されているか
- [ ] 観点 BF4: `--granularity subnode` 指定時の成功事後条件が外部観察可能か（サブノード単位展開が完了した観察可能な状態として定義されているか）

### 2.2 代替フロー（分岐網羅）

- [ ] 観点 AF1: `--granularity` の全 case 網羅。明示分岐（1a=document・基本フロー=subnode）の 2 値が REQ.03 の仕様範囲（document/subnode のみ）と一致しているか。値が 2 値以外の場合（不正値）のフローが未記述
- [ ] 観点 AF2: 代替フロー 3a「サブノードが存在しない上流成果物はドキュメント全体として返却（fallback）」の発火条件の明示。どの時点でサブノード有無を判定し、どの単位（ノード単位）で fallback が適用されるかが不明確
- [ ] 観点 AF3: 代替フロー 4-A（outline-only）・4-B（sections フィルタ）・4-C（depth_limit）・4-D（subnode 展開）の適用順序が明示されているか。REQ.18 のフラグ組合せ優先順位マトリクス（sections→outline の順）が UC フローに反映されているか
- [ ] 観点 AF4: 代替フロー 4-D「subnode 指定時に子サブノードを個別 UpstreamArtifact として展開」と基本フロー Step 3（エッジを辿り関連サブノードを特定）の整合。4-D の記述が Phase 2 Block B 以降の正準挙動として基本フローと矛盾しないか
- [ ] 観点 AF5: 代替フロー 1a「`--granularity document` の場合 UC-LGX-002 と同一の動作」の事後条件が UC-LGX-002 の事後条件と完全に一致するか（monitoring entry / 決定論保証 / 監査ログ含め収束しているか）

### 2.3 例外フロー（失敗パス）

- [ ] 観点 EF1: Step 3 実行中の失敗パスの欠落。サブノードへのエッジ辿り（Step 3a）やサブノード本文抽出（Step 3b）が失敗する場合（サブノードファイル破損・content_range 範囲外・エッジ不整合）のフローが列挙されていない
- [ ] 観点 EF2: 大規模返却エラー（REQ.13: 500,000 文字超過）の取り扱い。`--granularity subnode` でサブノード展開後に超過した場合の例外パスが UC に示されていない
- [ ] 観点 EF3: Step 4 の drift_score 付与（Step 3c）が失敗した場合（embedding 不在・スコア計算エラー）のフロー。スコア付与失敗時にサブノード artifact をどう扱うかが未定義

### 2.4 アクター遷移と権限

- [ ] 観点 AT1: アクター（Claude Code / 開発者）の権限・状態が 1a（document fallback）と基本フロー（subnode）で一貫しているか。粒度変更がアクターの権限要件を変えないことが UC で観察可能か
- [ ] 観点 AT2: 事前条件「上流成果物にサブノードが存在する」が成立しない状態でアクターが `--granularity subnode` を実行した場合（サブノード皆無のグラフ）の責任境界。システムが 3a fallback を自動適用するか、アクターへエラー通知が必要か

### 2.5 データフロー

- [ ] 観点 DF1: 入出力データの拡張フィールドの完全性。基本フロー Step 4 が subnode_id / anchor / content / drift_score を含む UpstreamArtifact を返すと記述しているが、サブノードが存在しない場合（3a fallback）の UpstreamArtifact フォーマット（subnode_id=null/absent か）が未定義
- [ ] 観点 DF2: `--granularity subnode` 時の UpstreamArtifact の整列規則。REQ.11「親ドキュメント ID 辞書順 + アンカー出現順」がフロー記述に反映されているか
- [ ] 観点 DF3: 監査ログ（context_log）への granularity 記録（REQ.07）。UC 基本フローに監査ログ記録ステップが存在しない（UC-002 は Step 7 で記録）。granularity=subnode の記録がフローに観察可能か

### 2.6 領域固有観点（サブノード粒度制御 / コンテキスト解決）

- [ ] 観点 R1: 代替フロー 4-D と基本フロー Step 3 の Phase 依存関係の可視化。「Phase 2 Block B 以降で正準」という条件付き挙動が UC フロー上でどう識別されるか（現在の実装フェーズで有効な分岐が不明確）
- [ ] 観点 R2: 事後条件「Phase 2 Block B 以降、`--granularity subnode` は親ドキュメントを返さず子サブノードを個別 artifact に展開する」が基本フロー Step 4 の返却記述（subnode_id / anchor / content / drift_score を含む）と整合しているか。基本フローは「上流成果物の UpstreamArtifact に subnode_id 等を含めて返却」だが、事後条件は「親ドキュメントを返さず子サブノードを個別展開」と記述しており、粒度の違いが観察可能な形で一致しているか
- [ ] 観点 R3: CTX-INV-1（決定論保証）の具体化。基本フロー内に「同一入力 → 同一サブノード取得順序」を保証するステップが観察可能か（REQ.11 整列の UC への反映）
- [ ] 観点 R4: MCP-INV-1（新ツール追加なし）との整合。`--granularity subnode` がオプション引数として実装されており新 MCP ツールを追加していないことが UC フローから確認可能か（REQ.06 の具体化）
- [ ] 観点 R5: 代替フロー 4-B（sections フィルタ）の `--granularity document` 時の無視挙動（REQ.18）が UC フローに明示されているか。4-B の記述は「`--granularity subnode` 経路で展開される子サブノードのうち」と条件付きだが、document 粒度でフィルタが無視されることが観察可能か

## 3. RED / GREEN 判定

| 観点 | 判定 | 親 SPEC / UC §で回答（委譲先） | 関連 GAP |
|---|---|---|---|
| 2.1 BF1 事前条件の検証タイミング | RED | UC-004 では「上流成果物にサブノードが存在する」が事前条件として記載されるが、Step 3 実行時に動的に判定される（3a fallback 発動）のか、それとも実行前に検証されるのかがフロー記述に現れない。SPEC-LGX-003 は subnode 有無を動的判定（REQ.08・fallback）として規定するが UC のフロー記述がそれを観察可能に具体化していない。【GENUINE: UC フロー記述の問題であり SPEC 委譲では解消しない】 | GAP-LGX-211 |
| 2.1 BF2 委譲後の接続点の観察可能性 | GREEN | UC-004 Step 2「UC-LGX-002 の基本フロー 2〜5 と同様」という委譲は上流解決の共通部分を UC-002 に委譲するのみで、Step 3 への入力は「上流成果物」として自然に定まる。SPEC-LGX-003.REQ.02/REQ.08 で上流成果物コレクション取得が規定済。接続点の観察可能性は UC-002 委譲で成立 | — |
| 2.1 BF3 返却構造と ContextResult の整合 | GREEN | UC-004 事後条件「UC-LGX-002 の事後条件と同じ（決定論保証、監査ログ）」と明記。サブノード拡張フィールド（subnode_id / anchor / content / drift_score）の追加は SPEC-LGX-003.REQ.03 / LGX-EXT-001 §5.1 の拡張として UC-002 構造に加算的であり矛盾しない。委譲容認 | — |
| 2.1 BF4 成功事後条件の観察可能性 | GREEN | 事後条件「サブノードモードでは、ドキュメント全体ではなく関連セクションのみが返却される」で外部観察可能。CTX-INV-1 はSPEC-LGX-003.REQ.04 / UC-004 事後条件「UC-LGX-002 の事後条件と同じ」で継承済 | — |
| 2.2 AF1 granularity 不正値フローの欠落 | RED | UC-004 は `--granularity document`（1a）と `--granularity subnode`（基本フロー）の 2 値のみを記述。不正値（document/subnode 以外）の失敗パスが UC フローに列挙されていない。SPEC-LGX-003.REQ.03（2値のみ）+ LGX-COMPAT-001 §1（構文誤りは exit 2）で終了コードは確定しているが、UC のフロー記述としての明示がない。【WEAK: SPEC-003 / LGX-COMPAT-001 委譲で実質解決。UC への明示列挙は任意】 | GAP-LGX-212 |
| 2.2 AF2 fallback 発火条件の不明確さ | RED | 代替フロー 3a「サブノードが存在しない上流成果物はドキュメント全体として返却（fallback）」の発火条件が「対象ノード単位でサブノード有無を確認し、無ければドキュメント全体を採用」であることが UC フロー記述から読み取れない。SPEC-LGX-003 は動的 fallback を規定するが（Phase 2 Block B 後も維持と UC が明記）、ノード単位か探索単位かの粒度が観察不能。【GENUINE: フロー記述レベルの問題。委譲で解消しない】 | GAP-LGX-213 |
| 2.2 AF3 フラグ適用順序の非記述 | RED | 代替フロー 4-A〜4-D が列挙されているが、適用される順序（sections フィルタ → outline 化という REQ.18 の優先順位マトリクス）が UC フローに観察可能な形で示されていない。組合せ指定時に何が先に適用されるかがフロー記述から不明。【WEAK: SPEC-LGX-003.REQ.18 フラグ組合せマトリクスに委譲可能。UC への明示は任意】 | GAP-LGX-214 |
| 2.2 AF4 4-D と基本フローの整合 | GREEN | 4-D は「Phase 2 Block B、観察事項 1 解消」として `--granularity subnode` 指定時の正準挙動（親ドキュメント全文ではなく子サブノードを個別展開）を規定。基本フロー Step 3（エッジを辿り関連サブノードを特定）と同一の動作を Phase 依存で明確化したものであり矛盾しない。SPEC-LGX-003.REQ.03 末尾の記述（subnode=該当サブノードの本文のみ）と整合 | — |
| 2.2 AF5 1a の事後条件収束 | GREEN | 代替フロー 1a「UC-LGX-002 と同一の動作」は事後条件として UC-LGX-002 の決定論保証・監査ログ記録へ収束することが明示済。UC-004 事後条件「UC-LGX-002 の事後条件と同じ」と一貫 | — |
| 2.3 EF1 Step 3 実行中の失敗パス欠落 | RED | UC-004 の代替フロー / 例外フロー節には Step 3 の処理失敗（サブノードファイル破損・content_range 範囲外・エッジ不整合）が列挙されていない。SPEC-LGX-003.REQ.20（上流部分欠損は部分成功・exit 0）が適用されると考えられるが、UC フローとしての明示がない。【GENUINE: UC フロー記述の欠落。SPEC 委譲は間接的】 | GAP-LGX-215 |
| 2.3 EF2 大規模返却エラーの例外パス | GREEN | SPEC-LGX-003.REQ.13（500,000 文字超過時エラー + 提案文）は `--granularity subnode` 指定時にも適用される。REQ.13 は granularity に依らない共通制約として UC-LGX-004 の「UC-LGX-002 の事後条件と同じ」委譲で継承済（UC-LGX-002 の事後条件は SPEC-LGX-003 全体を継承）。委譲容認 | — |
| 2.3 EF3 drift_score 付与失敗 | GREEN | UC-004 Step 3c「ドリフトスコア（エッジごと）を付与する」の失敗は、embedding 不在・スコア計算エラーを含む。SPEC-LGX-003.REQ.20（上流部分欠損は部分成功・exit 0）+ UC-LGX-007 の drift 検出仕様（embedding 不在はスキップ）が委譲先として成立。UC のフローとしてはスコア付与失敗の明示列挙は任意の範囲。委譲容認 | — |
| 2.4 AT1 アクター権限の粒度間一貫性 | GREEN | Claude Code / 開発者ともに `legixy context` を同一権限で実行する。`--granularity` はオプション引数であり権限要件を変えない。MCP-INV-1（新ツール追加なし）と一貫。UC 記述と整合 | — |
| 2.4 AT2 サブノード皆無時の責任境界 | GREEN | 代替フロー 3a（サブノードが存在しない上流成果物はドキュメント全体として返却）により、システムが自動で fallback を適用する。アクターへのエラー通知は不要とフローが規定。事後条件に「サブノードが存在しない場合のみ親全文へ fallback」と明記。委譲容認 | — |
| 2.5 DF1 fallback 時の UpstreamArtifact フォーマット | RED | 基本フロー Step 4 は subnode_id / anchor / content / drift_score を含む UpstreamArtifact を返すと記述するが、代替フロー 3a（fallback でドキュメント全体を返却）時の同フォーマット（subnode_id=null/absent か、drift_score の有無）が未定義。フローとデータ構造の整合性が観察不能。【GENUINE: UC フロー記述レベルの欠落】 | GAP-LGX-216 |
| 2.5 DF2 整列規則の UC への反映 | GREEN | SPEC-LGX-003.REQ.11（subnode 粒度時は「親ドキュメント ID 辞書順 + アンカー出現順」整列）は TP-LGX-003（green）で確立済。UC-004 がその規定を観察可能なステップとして再記述する義務はなく委譲容認 | — |
| 2.5 DF3 監査ログ記録ステップの欠落 | RED | UC-LGX-002 は Step 7「システムが context_log に監査ログを記録する」を基本フローに明示するが、UC-LGX-004 の基本フローにはその相当ステップが存在しない。事後条件「UC-LGX-002 の事後条件と同じ（…監査ログ）」でカバーされるとも読めるが、フロー記述として監査ログ記録（granularity=subnode の記録含む）のステップが観察可能化されていない。【WEAK: UC-002 の「事後条件と同じ」委譲で解消可。明示列挙は任意】 | GAP-LGX-217 |
| 2.6 R1 Phase 依存の挙動の可視化 | GREEN | UC-004 各代替フロー（4-A〜4-D）に「Phase 2 Block B」ラベルが明記されており、Phase 依存であることは可視化されている。現フェーズで有効な分岐を識別する責務は実装フェーズ管理（Phase 2 Block B 到達判定）に委譲され、UC の責務範囲外 | — |
| 2.6 R2 基本フローと事後条件の記述整合 | GREEN | 基本フロー Step 4「UpstreamArtifact に subnode_id / anchor / content / drift_score を含めて返却」は「個別 artifact への展開」と等価。事後条件「親ドキュメントを返さず子サブノードを個別 artifact に展開」と記述の粒度差はあるが意味的に整合。SPEC-LGX-003.REQ.03（subnode=該当サブノードの本文のみ）を超える矛盾は見られない | — |
| 2.6 R3 決定論保証の具体化 | GREEN | UC-004 事後条件「UC-LGX-002 の事後条件と同じ（決定論保証、監査ログ）」が CTX-INV-1 を継承。REQ.11（サブノード粒度の整列規則）は TP-LGX-003 green で確立済。UC フローへの明示再記述は不要。委譲容認 | — |
| 2.6 R4 MCP-INV-1 の UC からの確認可能性 | GREEN | UC-004 基本フロー Step 1「`legixy context <files> --granularity subnode` を実行」と明記。既存コマンドのオプション引数として粒度制御を提供しており新 MCP ツール追加なし。SPEC-LGX-003.REQ.06 / MCP-INV-1 委譲で成立 | — |
| 2.6 R5 sections フィルタの document 粒度時の無視挙動 | GREEN | UC-004 代替フロー 4-B「`--granularity subnode` 経路で展開される子サブノードのうち」という条件付き記述により、document 粒度時には 4-B が発動しないことが暗黙的に含意される。SPEC-LGX-003.REQ.16/REQ.18（sections×document は無視）へ委譲で成立 | — |

集計: **全 22 観点 / GREEN 15 / RED 7**（RED は BF1 / AF1 / AF2 / AF3 / EF1 / DF1 / DF3）

## 4. ステータスの決定

RED 観点が 7 件残存するため、本 TP のステータスは `**ステータス**: red`。

- BF1（GAP-LGX-211）・AF2（GAP-LGX-213）・EF1（GAP-LGX-215）・DF1（GAP-LGX-216）は GENUINE 候補（UC フロー記述の問題で SPEC 委譲では解消しない）。
- AF1（GAP-LGX-212）・AF3（GAP-LGX-214）・DF3（GAP-LGX-217）は WEAK 候補（親 SPEC が既に答えており、UC フロー記述への明示反映が任意か必須かの裁定が必要）。
- 敵対的精査パスで GENUINE / WEAK / OUT_OF_SCOPE を確定し、WEAK 確定分は人間裁定（UC フローへの追記 or drop）を経て close。全観点 GREEN 化後に本 TP を green へ更新する。

## 5. 観点ナレッジベース参照

この TP の生成時に参照したナレッジベース章節:

- `docs/perspectives/core-perspectives.md` §UC レベル観点（基本フロー / 代替フロー / 例外フロー / アクター遷移と権限 / データフロー）
- `docs/perspectives/core-perspectives.md` §汎用観点（エラーハンドリング / 状態遷移 / 入力検証 / ライフサイクル）
- 親 SPEC: SPEC-LGX-003.REQ.01〜REQ.20（全要求）、特に REQ.03/REQ.08/REQ.11/REQ.13/REQ.15〜REQ.18
- 委譲先 TP: TP-LGX-003（コンテキスト解決 SPEC レベル観点、green 確定済）
- 兄弟 UC: UC-LGX-002（コンテキスト解決 基本、UC-004 が事後条件・委譲先として参照）
- 関連不変条件: CTX-INV-1（決定論保証）・MCP-INV-1（Agent Surface 限定）・MCP-INV-2（忠実な転送）

UX 層観点（Undo/フォーカス/タッチ等）は CLI/MCP コンテキスト解決コマンドには本質的に N/A のためスキップした。

## 6. 履歴

| 日付 | 変更内容 |
|---|---|
| 2026-06-13 | 初版作成。UC レベル観点 22 件（GREEN 15 / RED 7）。GAP-LGX-211（BF1 事前条件検証タイミング）/ GAP-LGX-212（AF1 不正 granularity 値フロー欠落）/ GAP-LGX-213（AF2 fallback 発火条件不明確）/ GAP-LGX-214（AF3 フラグ適用順序非記述）/ GAP-LGX-215（EF1 Step3 失敗パス欠落）/ GAP-LGX-216（DF1 fallback 時 UpstreamArtifact フォーマット未定義）/ GAP-LGX-217（DF3 監査ログ記録ステップ欠落）を起票 |

## 7. 解消（2026-06-13、敵対的精査裁定後）

本 TP が起票した GAP[UC] は全て closed。内訳: **WEAK=方針B（委譲容認）** / **REFUTED=棄却** / **GENUINE=UC 修正で解消**（A/B/C、人間承認 2026-06-13）。§3 表の判定列は初版（起票時）の draft 判定を保持する（精査の履歴として温存）。全 RED 観点は上記裁定で解消したため本 TP は **green**。各 GAP の最終状態は当該 GAP ファイル（§5）と docs/spec-change-proposals/2026-06-13_uc-gap-adversarial-review.md を参照。
