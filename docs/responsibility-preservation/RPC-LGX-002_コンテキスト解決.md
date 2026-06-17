Document ID: RPC-LGX-002

# RPC-LGX-002: コンテキスト解決 chain の責務保存率検査

> RPC は **抽象責務集合（RBA + SEQA、UC 錨着）→ 具体責務集合（RBD + SEQD）** の保存性検査。詳細仕様は `11-responsibility-preservation-check.md`。VERDICT は §9 のエスカレーション規律に従う。

**対象 UC**: UC-LGX-002
**対象 RBA**: RBA-LGX-002
**対象 SEQA**: SEQA-LGX-002
**対象 RBD**: RBD-LGX-002
**対象 SEQD**: SEQD-LGX-002
**検査深度**: フル（§14.2: コンテキスト解決は MCP-INV-4 監査ログ完全性・決定論保証・部分成功継続を担う高クリティカリティ UC）
**検査日**: 2026-06-13
**Reviewer**: AI Reviewer（legixy DevProc_V4.1）

## 1. Abstract Responsibilities（UC ステップを一次アンカーとする）

RBA-LGX-002 の Boundary 7 / Control 8 / Entity 4（合計 19 ドメイン主語）と SEQA-LGX-002 の時系列展開から抽出。

| AR-ID | Source | Role | Subject | Responsibility | UC step |
|---|---|---|---|---|---|
| AR-001 | RBA | Boundary | コンテキスト解決コマンド受付窓口 | アクターからの `context <files>` 要求（オプション含む）を受け取る | UC-002 Step 1 |
| AR-002 | RBA | Control | コンテキスト解決統括処理 | コンテキスト解決要求を受け、起点解決・上流走査・ガイドライン解決・カスタム文書解決・結果整形・監査ログ記録を協調させる。部分成功を継続する | UC-002 Step 1–7 |
| AR-003 | RBA | Control | 起点解決処理 | 指定ファイルパスを graph.toml のノードに逆引きし、解決済み起点と未解決起点を分別する | UC-002 Step 2 / 代替 2a |
| AR-004 | RBA | Boundary | 設定ファイル境界 | `.legixy.toml` を供給する | UC-002 Step 1（前提） |
| AR-005 | RBA | Boundary | グラフ定義境界 | `graph.toml`（ノード・エッジ・カスタムエッジ情報）を供給する | UC-002 Step 2, 3, 5 |
| AR-006 | RBA | Entity | 解決済み起点 | ファイルパスから特定された対象ノード情報（未解決起点の記録を含む）を保持する | UC-002 Step 2 / 代替 2a |
| AR-007 | RBA | Control | 上流走査処理 | 解決済み起点から有向グラフを逆方向に走査し、上流成果物を収集する。depth_limit が指定された場合は走査階層を制限する | UC-002 Step 3 / 代替 3a / 代替 4-C |
| AR-008 | RBA | Entity | 有向グラフ | 設定・グラフ定義境界から構築されたノード・エッジの集合（上流走査の基盤） | UC-002 Step 3 |
| AR-009 | RBA | Control | 粒度解決処理 | 走査結果の各上流成果物に対して granularity を適用し、返却本文を確定する。sections フィルタおよび outline 変換も適用する | UC-002 Step 3, 代替 4-A, 4-B |
| AR-010 | RBA | Boundary | 成果物ファイル境界 | 上流成果物の本文・サブノード本文を供給する（不在・欠損も許容） | UC-002 Step 3, 例外 |
| AR-011 | RBA | Entity | 上流成果物一覧 | 走査・フィルタ・粒度解決を経た上流成果物の集合（chain_distance 順・決定論的整列済み）を保持する | UC-002 Step 3, 6 |
| AR-012 | RBA | Control | レイヤーガイドライン解決処理 | 起点のレイヤーに対応するガイドライン文書をレイヤーガイドライン境界から取得し、辞書順で整列する | UC-002 Step 4 |
| AR-013 | RBA | Boundary | レイヤーガイドライン境界 | レイヤーに対応するガイドライン文書を供給する | UC-002 Step 4 |
| AR-014 | RBA | Control | カスタム文書解決処理 | カスタムエッジに基づく追加文書をグラフ定義境界から取得し、辞書順で整列する | UC-002 Step 5 |
| AR-015 | RBA | Control | 結果整形処理 | 6 セクションを決定論的順序で組み立て、コンテキスト解決結果を生成する。返却本文が上限を超える場合はエラーを生成する | UC-002 Step 6 / 例外（大規模返却エラー） |
| AR-016 | RBA | Entity | コンテキスト解決結果 | 6 セクション構成の最終返却データを保持する | UC-002 Step 6 |
| AR-017 | RBA | Boundary | 監査ログ境界 | context_log への書込先（書込失敗もベストエフォート） | UC-002 Step 7 |
| AR-018 | RBA | Control | 監査ログ記録処理 | コンテキスト解決結果の確定後、呼出情報を監査ログ境界へ書き込む。書込失敗は警告として記録しつつ本処理の成功を維持する | UC-002 Step 7 / 例外 |
| AR-019 | RBA | Boundary | コンテキスト解決結果出力窓口 | 解決済みコンテキスト結果をアクターへ返す | UC-002 Step 6 |

全 AR が UC ステップに紐づく（UC ステップに紐づかない AR なし）。SEQA-LGX-002 の時系列メッセージは上記 AR の責務の実行順展開であり、新規 AR を生まない。

なお AR-004（設定ファイル境界）は RBA-LGX-002 に明示的に列挙されているが、SEQA-LGX-002 の基本フローのレーンには設定ファイル境界が登場しない。これは UC-LGX-002 が `.legixy.toml` を事前条件として前提しており、`legixy context` サブコマンドの実行時には設定が既に解決済みであることを示唆する。RBD/SEQD での扱いを確認する（§3 対応表参照）。

## 2. Concrete Responsibilities

RBD-LGX-002（クラス一覧）と SEQD-LGX-002（操作呼び出し）から抽出。

| CR-ID | Source | Class | Operation | Responsibility | Message |
|---|---|---|---|---|---|
| CR-001 | RBD/SEQD | コンテキスト解決コマンド受付窓口 | コンテキスト解決を受け付ける | アクター境界で要求を受理（ファイルパスのコレクション・オプション種別） | Actor→B受付 |
| CR-002 | RBD/SEQD | コンテキスト解決統括処理 | コンテキスト解決を統括する / 部分成功を継続判定する | フロー協調・部分成功継続 | B受付→C統括、C統括→各処理 |
| CR-003 | RBD/SEQD | 起点解決処理 | 起点を解決する / 未解決起点を記録する | ファイルパスからノードへの逆引き・未解決記録 | C統括→C起点→Bグラフ定義→E起点 |
| CR-004 | RBD/SEQD | 設定ファイル境界 | 設定を読み込む / 設定の存在を確認する | 設定供給 | （SEQD に直接登場なし） |
| CR-005 | RBD/SEQD | グラフ定義境界 | ノード情報を読み込む / エッジ情報を読み込む / カスタムエッジ情報を取得する / グラフ定義の存在を確認する | グラフ定義・カスタムエッジ供給 | C起点→Bグラフ定義、Cカスタム→Bグラフ定義 |
| CR-006 | RBD/SEQD | 解決済み起点 | 起点が解決済みか判定する | 起点情報保持・解決状態判定 | C起点→E起点 |
| CR-007 | RBD/SEQD | 上流走査処理 | 上流成果物を走査する / 逆方向に走査する | 有向グラフの逆方向走査・depth_limit 適用 | C統括→C走査→Eグラフ |
| CR-008 | RBD/SEQD | 有向グラフ | ノードを取り出す / 逆方向に隣接ノードを辿る / カスタムエッジを取り出す | グラフ保持・走査・カスタムエッジ取得 | C走査→Eグラフ |
| CR-009 | RBD/SEQD | 粒度解決処理 | 粒度を解決する / セクションフィルタを適用する / アウトライン変換を適用する | 上流成果物本文確定・フィルタ・outline 変換 | C統括→C粒度→B成果物→E上流 |
| CR-010 | RBD/SEQD | 成果物ファイル境界 | 成果物本文を読み込む / サブノード本文を読み込む / 成果物の存在を確認する | 成果物・サブノード本文供給 | C粒度→B成果物 |
| CR-011 | RBD/SEQD | 上流成果物一覧 | 走査距離順に整列する | 上流成果物保持・chain_distance 順整列 | C粒度→E上流 |
| CR-012 | RBD/SEQD | レイヤーガイドライン解決処理 | レイヤーガイドラインを解決する | レイヤー対応ガイドライン取得・辞書順整列 | C統括→Cガイド→Bガイド |
| CR-013 | RBD/SEQD | レイヤーガイドライン境界 | レイヤーに対応するガイドライン文書を取得する | ガイドライン供給 | Cガイド→Bガイド |
| CR-014 | RBD/SEQD | カスタム文書解決処理 | カスタム文書を解決する | カスタムエッジ文書取得・辞書順整列 | C統括→Cカスタム→Bグラフ定義 |
| CR-015 | RBD/SEQD | 結果整形処理 | 結果を整形する / 文字数上限を確認する / エラーを生成する | 6 セクション組み立て・上限検査・エラー生成 | C統括→C整形→E結果→B出力 |
| CR-016 | RBD/SEQD | コンテキスト解決結果 | 文字数を計算する / 終了状態を判定する | 結果保持・文字数計算・終了状態判定 | C整形→E結果、C監査→E結果 |
| CR-017 | RBD/SEQD | 監査ログ境界 | 呼出情報を書き込む | 監査ログへの書込 | C監査→B監査 |
| CR-018 | RBD/SEQD | 監査ログ記録処理 | 監査ログを記録する / 書込失敗を警告として記録する | 監査ログ記録・書込失敗警告 | C統括→C監査→B監査 |
| CR-019 | RBD/SEQD | コンテキスト解決結果出力窓口 | コンテキスト解決結果を渡す / 警告を付記する | 結果渡し・警告付記 | C整形→B出力→Actor |

## 3. Responsibility Mapping

| AR-ID | CR-ID(s) | Relation | Justification | Verdict |
|---|---|---|---|---|
| AR-001 | CR-001 | preserved | 同一 Boundary・境界操作。受付窓口がアクターとの境界を担い操作「コンテキスト解決を受け付ける」で具体化 | GREEN |
| AR-002 | CR-002 | preserved | 同一 Control。統括・部分成功継続の二操作が抽象責務を完全に具体化。SEQD でフロー協調を確認 | GREEN |
| AR-003 | CR-003 | preserved | 同一 Control。「起点を解決する」「未解決起点を記録する」の二操作が逆引き・分別責務を具体化 | GREEN |
| AR-004 | CR-004 | preserved | 同一 Boundary。RBD に設定ファイル境界クラスが存在し操作を識別。SEQD に直接登場しないのは UC で `.legixy.toml` が事前条件として前提されており、context サブコマンド実行時は設定ファイル境界との相互作用が主要フローに現れない設計。RBA §6 Object Discovery にも「グラフ定義境界（Boundary）→ 有向グラフ（Entity）の分離を構造化した」旨の記述があり、設定ファイル境界も同様の可視化扱い。RBD/SEQD の具体クラスに責務は保持されており消失ではない | GREEN |
| AR-005 | CR-005 | preserved | 同一 Boundary。ノード・エッジ・カスタムエッジ情報の 4 操作が抽象責務を具体化 | GREEN |
| AR-006 | CR-006 | preserved | 同一 Entity。「起点が解決済みか判定する」自身データ操作で保持、ファイルパス・成果物識別子・未解決フラグ・未解決起点の記録属性 | GREEN |
| AR-007 | CR-007 | preserved | 同一 Control。「上流成果物を走査する」「逆方向に走査する」の二操作。depth_limit は走査階層制限の引数として具体化。代替 4-C（SEQD §2 代替 4-C）で depth_limit 適用を確認 | GREEN |
| AR-008 | CR-008 | preserved | 同一 Entity。ノード・エッジ・カスタムエッジ属性と走査・辿り・取得操作で保持 | GREEN |
| AR-009 | CR-009 | preserved | 同一 Control。「粒度を解決する」「セクションフィルタを適用する」「アウトライン変換を適用する」の三操作が sections フィルタ・outline 変換責務を具体化。SEQD 代替 4-A/4-B で実行確認 | GREEN |
| AR-010 | CR-010 | preserved | 同一 Boundary。「成果物本文を読み込む」「サブノード本文を読み込む」「成果物の存在を確認する」の三操作 | GREEN |
| AR-011 | CR-011 | preserved | 同一 Entity。「走査距離順に整列する」操作、成果物・整列済みフラグ属性 | GREEN |
| AR-012 | CR-012 | preserved | 同一 Control。「レイヤーガイドラインを解決する」操作が取得・辞書順整列責務を具体化 | GREEN |
| AR-013 | CR-013 | preserved | 同一 Boundary。「レイヤーに対応するガイドライン文書を取得する」操作 | GREEN |
| AR-014 | CR-014 | preserved | 同一 Control。「カスタム文書を解決する」操作がカスタムエッジ文書取得・辞書順整列責務を具体化 | GREEN |
| AR-015 | CR-015 | preserved | 同一 Control。「結果を整形する」「文字数上限を確認する」「エラーを生成する」の三操作が 6 セクション組み立て・大規模返却エラー処理を具体化。SEQD 例外フロー（大規模返却エラー）で実行確認 | GREEN |
| AR-016 | CR-016 | preserved | 同一 Entity。6 セクション属性・文字数合計・部分成功フラグ、「文字数を計算する」「終了状態を判定する」操作 | GREEN |
| AR-017 | CR-017 | preserved | 同一 Boundary。「呼出情報を書き込む」操作・書込先属性 | GREEN |
| AR-018 | CR-018 | preserved | 同一 Control。「監査ログを記録する」「書込失敗を警告として記録する」の二操作がベストエフォート責務を具体化。SEQD 例外フロー（監査ログ書込失敗）で実行確認 | GREEN |
| AR-019 | CR-019 | preserved | 同一 Boundary。「コンテキスト解決結果を渡す」「警告を付記する」の二操作 | GREEN |

19 AR すべて preserved（1:1）。split / merged / shifted / lost / mutated / ambiguous なし。RBD-LGX-002 §4 mapping が新規クラスなしを確認済み（RBA 主語と 1:1）。

**AR-004（設定ファイル境界）の SEQD 不在について補足**: RBD に設定ファイル境界クラスが存在し操作（設定を読み込む・存在を確認する）が識別されている事実から、消失（lost）ではなく「UC の事前条件として前提される設定読込フロー」が context サブコマンドの逐次フローに現れない設計と判断した。legixy CLI では `.legixy.toml` の解決は起動時の共通処理として行われ、個別サブコマンドのシーケンスに毎回現れない可能性が高い。これは同一 chain の UC-LGX-001（グラフ読み込みと検証）も同様の事前条件設計を持つ。責務クラスは具体側に保存されており、shifted / mutated でもない。

## 4. Role Fitness Check（§5.2）

### Boundary
- Finding: 7 Boundary クラス（受付窓口・設定ファイル境界・グラフ定義境界・成果物ファイル境界・レイヤーガイドライン境界・監査ログ境界・結果出力窓口）は境界操作のみ保持。Boundary overreach なし。グラフ定義境界がノード・エッジ・カスタムエッジの 3 種を供給するが、それぞれ異なるデータ種別への読込操作であり単一 Boundary への過剰集中ではない（カスタム文書解決処理がカスタムエッジを、起点解決処理がノード・エッジを参照する構造上自然）。
- Severity: なし / 原因の所在: — / Required action: なし

### Control
- Finding: 8 Control（統括・起点解決・上流走査・粒度解決・レイヤーガイドライン解決・カスタム文書解決・結果整形・監査ログ記録）は調停・処理に留まり、データ保持なし。コンテキスト解決統括処理は 7 処理を協調させるが、データ保持操作を持たず協調ロジックのみ。Service blob 化なし（各処理の責務は明確に分割されている）。Control leakage なし。
- Severity: なし / 原因の所在: — / Required action: なし

### Entity
- Finding: 4 Entity（解決済み起点・有向グラフ・上流成果物一覧・コンテキスト解決結果）は自身のデータ操作のみ。有向グラフの走査操作（逆方向に隣接ノードを辿る）は自身データへの操作で Entity anemia なし。コンテキスト解決結果の「文字数を計算する」「終了状態を判定する」は自身データに対する計算で overreach なし。
- Severity: なし / 原因の所在: — / Required action: なし

## 5. Sequential Execution Check（§5.3）

### Basic Flow
| UC step | SEQA message | SEQD message | Responsibility valid? | Notes |
|---|---|---|---|---|
| Step 1（`context <files>` 実行） | Actor→B受付→C統括 | コンテキスト解決を受け付ける→コンテキスト解決を統括する | Yes | |
| Step 2（ファイルパスから成果物 ID を逆引き） | C統括→C起点→Bグラフ定義→E起点 | 起点を解決する→ノード情報を読み込む→エッジ情報を読み込む→起点が解決済みか判定する | Yes | |
| Step 3（有向グラフを逆方向走査して上流成果物を収集） | C統括→C走査→Eグラフ / C統括→C粒度→B成果物→E上流 | 上流成果物を走査する→逆方向に隣接ノードを辿る / 粒度を解決する→成果物本文を読み込む→走査距離順に整列する | Yes | |
| Step 4（レイヤールールに基づくガイドライン文書を解決） | C統括→Cガイド→Bガイド | レイヤーガイドラインを解決する→レイヤーに対応するガイドライン文書を取得する | Yes | |
| Step 5（カスタムエッジに基づく追加文書を解決） | C統括→Cカスタム→Bグラフ定義 | カスタム文書を解決する→カスタムエッジ情報を取得する | Yes | |
| Step 6（ContextResult として返却） | C統括→C整形→E結果→B出力→Actor | 結果を整形する→文字数を計算する→終了状態を判定する→コンテキスト解決結果を渡す | Yes | |
| Step 7（context_log に監査ログを記録） | C統括→C監査→B監査 | 監査ログを記録する→呼出情報を書き込む | Yes | SEQD では C整形→B出力の後に C統括→C監査の順。監査ログ記録が結果返却後に行われる順序は SEQA/SEQD で一貫 |

### Alternative Flows
| UC step | SEQA message | SEQD message | Responsibility valid? | Notes |
|---|---|---|---|---|
| 代替 2a（ファイルがどのノードにも対応しない） | C起点→E起点（未解決）→C整形→E結果（targets.artifact_id=null） | 起点が解決済みか判定する(未解決)→未解決起点を記録する→結果を整形する（対象成果物識別子未確定） | Yes | |
| 代替 3a（上流成果物が存在しない） | C走査→Eグラフ（上流なし）→E上流（空）→C整形→E結果（upstream 空配列） | 逆方向に隣接ノードを辿る(空)→走査距離順に整列する(空)→結果を整形する | Yes | |
| 代替 4-A（`--outline-only` 指定） | C粒度→B成果物→E上流（見出し階層リスト置換） | 成果物本文を読み込む→アウトライン変換を適用する→走査距離順に整列する | Yes | sections フィルタ後に outline 変換（REQ.18 優先順位）も SEQD §2 代替 4-A の Note で確認 |
| 代替 4-B（`--sections <ids>` 指定） | C粒度→B成果物（granularity=subnode）→E上流（指定 ID 一致のみ） | サブノード本文を読み込む→セクションフィルタを適用する→走査距離順に整列する | Yes | |
| 代替 4-C（`--depth N` 指定） | C走査→Eグラフ（N 階層に制限） | 上流成果物を走査する(depth_limit=N)→逆方向に走査する(走査階層制限) | Yes | |

### Exception Flows
| UC step | SEQA message | SEQD message | Responsibility valid? | Notes |
|---|---|---|---|---|
| 成果物ファイル読込失敗（部分成功継続） | C粒度→B成果物（失敗）→E上流（読込失敗記録・残りを確定）→C整形→E結果 | 成果物の存在を確認する(不在)→成果物本文を読み込む(失敗)→部分成功を継続判定する→走査距離順に整列する（読込失敗記録） | Yes | SEQD では C粒度が C統括の「部分成功を継続判定する」を呼び出す形で継続を確認 |
| 監査ログ書込失敗（ベストエフォート） | C監査→B監査（失敗）→B出力（警告付記） | 呼出情報を書き込む(失敗)→書込失敗を警告として記録する→警告を付記する | Yes | 本処理の成功は維持 |
| 返却本文が上限を超過（大規模返却エラー） | C整形→E結果（文字数超過確認）→B出力（エラー生成） | 文字数を計算する(超過)→文字数上限を確認する→エラーを生成する→コンテキスト解決結果を渡す(エラー内容) | Yes | SEQA §4 例外フローが大規模返却エラーを明示（RBA §6 Object Discovery でも確認） |

全 UC フロー（基本/代替 5 種/例外 3 種）が SEQA / SEQD 上で責務の不整合なく実行可能。

## 6. Mismatches

- **Lost Responsibilities**: None
- **Invented Responsibilities**: None（具体側に抽象側根拠のない責務なし。RBD §4 が新規クラスなしを明記）
- **Shifted Responsibilities**: None
- **Mutated Responsibilities**: None
- **Ambiguous Mappings**: None（AR-004 設定ファイル境界の SEQD 不在は消失でなく設計上の前提化と判断、根拠は §3 補足参照）

## 7. Metrics（監視指標 — 合否は §8 の絶対条件で判定）

| Metric | Value |
|---|---:|
| Total abstract responsibilities | 19 |
| Preserved | 19 |
| Justified split | 0 |
| Justified merge | 0 |
| Lost | 0 |
| Shifted | 0 |
| Mutated | 0 |
| Ambiguous | 0 |
| Preservation rate（監視用） | 100% |
| Invented concrete responsibilities | 0 |
| Total concrete responsibilities | 19 |
| Invention rate（監視用） | 0% |

## 8. 絶対条件ゲート（§7）

- [x] lost = 0
- [x] mutated = 0
- [x] shifted = 0
- [x] ambiguous = 0（解消済）
- [x] 未正当化 invented = 0
- [x] 未正当化 split / merge = 0
- [x] B/C/E 責務違反なし
- [x] UC 基本/代替/例外フローが具体側で実行可能

## 9. Required Changes

- なし（保存失敗なし）

## 10. Verdict（§9 規律）

保存失敗なし（lost/mutated/shifted/ambiguous いずれも 0、invented なし、未正当化 split/merge なし、B/C/E 責務違反なし、UC フロー実行可能）。抽象責務集合（UC 錨着）19 AR が具体責務集合 19 CR へ 1:1 で保存されている。AR-004（設定ファイル境界）の SEQD 不在は消失でなく UC 事前条件の前提化として正当化された。絶対条件ゲート全通過。

<!-- VERDICT:APPROVE -->
