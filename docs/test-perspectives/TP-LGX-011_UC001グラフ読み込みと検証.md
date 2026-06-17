Document ID: TP-LGX-011

# TP-LGX-011: UC-LGX-001「グラフ読み込みと検証」観点（UC レベル）

> TP は **テストケース** ではなく **観点リスト**。UC レベル TP は「ユースケースのフロー記述に問いかける質問のリスト」として書く。SPEC レベル TP（TP-LGX-004）が「仕様が答えるか」を問うのに対し、UC レベル TP は「フローが先行成果物（親 SPEC）を観察可能なステップへ忠実かつ完全に具体化しているか」を問う。

**親**: UC-LGX-001
**ステータス**: green
**最終更新**: 2026-06-13

## 1. 対象スコープ

この TP は UC-LGX-001「グラフ読み込みと検証」の全フロー（基本フロー Step 1〜6、代替フロー 2a/3a/4a/4g-A/4g-B/4h-A/4i-A、事後条件）に UC レベル観点をぶつける。

- 対象: UC-LGX-001 全節（概要・アクター・事前条件・基本フロー・代替フロー・事後条件・関連不変条件・関連要求）
- 親 SPEC: SPEC-LGX-004（検証）REQ.01〜REQ.13
- 関連 SPEC §: SPEC-LGX-002（グラフ基盤: ID 形式 `{type}-{area}-{seq}` / DAG 制約 CTX-INV-4）、設定ファイル探索順（`.legixy.toml` → `.trace-engine.toml`）は SPEC-LGX-008.REQ.13 / LGX-COMPAT-001 §6、終了コード凍結契約は LGX-COMPAT-001 §3 / §4 #3
- 委譲方針: check の検証セマンティクス（severity・finding カテゴリ・閾値・冪等性・部分失敗継続の**規定そのもの**）は TP-LGX-004（green 確定済）が所有する。本 TP はそれらを再検証せず、「UC-001 のフロー記述が SPEC-004 の規定を観察可能なステップとして正しく具体化しているか」のみを問う。

## 2. 観点リスト

### 2.1 基本フロー（ステップ連鎖の整合）

- [ ] 観点 BF1: 各ステップの事後条件が後続ステップの前提を満たすか（Step1 コマンド受理 → Step2 設定解析 → Step3 グラフ構築 → Step4 検証 → Step5 出力 → Step6 exit の連鎖整合）
- [ ] 観点 BF2: アクター入力の検証タイミングの段階区分（段階1 構文=引数パース / 段階2 形式=graph.toml 形式検証 / 段階3 意味=embedding）が UC で観察可能か。`check --formal`（段階1+2）と `check`（段階1+2+3）の差が観察可能か
- [ ] 観点 BF3: 成功時事後条件（検証結果 stdout 出力・ERROR 0 件時 exit 0・ERROR 有時 exit 1）が外部観察可能で、後続 UC（UC-008 フィードバック / UC-010 監査）の前提として参照可能か
- [ ] 観点 BF4: Step4 の 9 サブ検証（a〜i）の実行モデル（独立・全件実行・部分失敗継続）が UC レベルで観察可能か

### 2.2 代替フロー（分岐網羅）

- [ ] 観点 AF1: 分岐の網羅性。明示分岐（2a `.legixy.toml` 不在 / 3a `graph.toml` 不在 / 4a `--formal` 無 / 4g-A Changelog 有 / 4g-B Changelog OFF / 4h-A mismatch ON / 4i-A drift ON）が opt-in 検査（4g/4h/4i）の ON/OFF 両 case を被覆しているか
- [ ] 観点 AF2: 各代替フローの事後条件収束。前提崩壊系（2a/3a「ERROR 報告して終了」）の終了コードが基本フローの「ERROR 時 exit 1」へ収束するか、それとも別コード（exit 2 使用法誤り）か
- [ ] 観点 AF3: 代替フローへの遷移条件が明示されているか（4g-A=「change=redefined エントリを含む場合」、4i-A=「enabled=true（Phase 2 Block F）」など発火条件の明示）

### 2.3 例外フロー（失敗パス）

- [ ] 観点 EF1: 各ステップでの失敗パスが定義されているか。特に Step2（設定）/ Step3（graph）の**破損・パース不能**（不在ではない）失敗パスが代替/例外フローに列挙されているか
- [ ] 観点 EF2: エラー時の状態が不変条件を満たすか（check は読み取り専用 → グラフ/DB を変更しない → 中間状態破壊が原理的に発生しない、ことが UC 前提として担保されているか）
- [ ] 観点 EF3: エラー時のユーザ通知（finding の severity 区分 ERROR/WARNING/INFO/OK・メッセージ）が定義されているか
- [ ] 観点 EF4: 部分成功時の扱い。Step4 で一部ファイル読込失敗時に他チェックを継続し、失敗を finding として報告する挙動が UC フローに反映されているか

### 2.4 アクター遷移と権限

- [ ] 観点 AT1: アクター（開発者 / CI システム）の権限・状態が一貫しているか。check が読み取り専用であり両アクターが同一権限で実行可能であることが UC で一貫しているか
- [ ] 観点 AT2: 責任境界。システムは検出のみを行い、4g-A.4「アクターが引用箇所を 1 件ずつ確認・修正」のような是正アクションはアクター責務であることの分担が明示されているか
- [ ] 観点 AT3: check 実行中に対象ファイル / graph.toml / engine.db が外部更新（並行アクセス）された場合の整合性前提が UC レベルで成立しているか

### 2.5 データフロー

- [ ] 観点 DF1: 入出力データの型・制約。入力（graph.toml + `.legixy.toml` + 各成果物ファイル本文）→ 出力（CheckReport=stdout / ログ=stderr）の分離が UC で観察可能か
- [ ] 観点 DF2: embedding データのライフタイム。4i-A の engine.db 内 embedding 参照と「embed --all 未実行（embedding 不在）は致命扱いしない」がデータフローとして整合しているか
- [ ] 観点 DF3: エラー時のデータ解放保証（読み取り専用・メモリ上グラフのみで永続リソースの確保解放を伴わないことが UC 前提と整合するか）

### 2.6 領域固有観点（トレーサビリティエンジン / 検証 UC）

- [ ] 観点 R1: `check --formal`（形式層のみ）と無印 `check`（意味層追加）の責務差が UC で観察可能か（4a が UC-LGX-007 へ委譲する境界の妥当性）
- [ ] 観点 R2: ID 形式検証（4a）・ファイル存在（4b）・Document ID 一致（4c）・チェーン整合（4d）・孤立ファイル（4e）・DAG（4f）の各形式検査が SPEC-LGX-002 / SPEC-LGX-004 の検証カテゴリへ漏れなく対応しているか
- [ ] 観点 R3: opt-in 検査（4g/4h/4i）の既定 OFF（後方互換）が UC の代替フローで観察可能か（4g-B が既定スキップを明示）
- [ ] 観点 R4: 終了コード契約（0/1/2）が UC 事後条件と LGX-COMPAT-001 §3 / §4 #3 の凍結契約で一致するか
- [ ] 観点 R5: Step5 の出力（ERROR/WARNING/INFO/OK の 4 段階）が SPEC-LGX-004.REQ.03 の severity 定義と一致するか

## 3. RED / GREEN 判定

| 観点 | 判定 | 親 SPEC / UC §で回答（委譲先） | 関連 GAP |
|---|---|---|---|
| 2.1 BF1 ステップ連鎖整合 | GREEN | 基本フロー Step1〜6 が事前条件（`.legixy.toml`/`graph.toml` 存在）→ Step2/3 で読込 → Step4 検証 → Step5/6 出力・exit と連鎖。各事後条件が後続前提を満たす | — |
| 2.1 BF2 段階区分の観察可能性 | GREEN | 4a（`--formal` 無で意味検証追加）+ 基本フロー Step4（形式検証）で段階1+2／段階3 の差が観察可能。引数構文層は LGX-COMPAT-001 §3 へ委譲 | — |
| 2.1 BF3 成功時事後条件の観察可能性 | GREEN | 事後条件「検証結果が stdout 表示」「ERROR 時 exit 1」+ Step6「ERROR 0 件で exit 0」。外部観察可能・後続 UC 前提として参照可 | — |
| 2.1 BF4 9 サブ検証の実行モデル | GREEN | Step4 a〜i を列挙。独立全件実行は SPEC-LGX-004.REQ.05（部分失敗継続）へ委譲。ただし「部分失敗継続」自体の UC フロー反映は EF4 で別途評価 | （→ EF4） |
| 2.2 AF1 分岐網羅（opt-in ON/OFF） | GREEN | 4g-A（Changelog 有）/ 4g-B（既定 OFF）/ 4h-A（mismatch ON）/ 4i-A（drift ON）。4h/4i の既定 OFF は SPEC-LGX-004.REQ.12/REQ.13（既定 false）+ 4g-B の OFF 記述パターンへ委譲 | — |
| 2.2 AF2 前提崩壊系の exit コード収束 | GREEN | 2a/3a「ERROR 報告して終了」は事後条件「ERROR 時 exit 1」へ収束。ファイル不在は実行時失敗 → exit 1（SPEC-LGX-004.REQ.04 + TP-LGX-004 E3 で確立、exit 2 は構文層誤りに限定）+ LGX-COMPAT-001 §3 | — |
| 2.2 AF3 遷移条件の明示 | GREEN | 4g-A「change=redefined エントリを含む場合」・4h-A「enabled=true かつ Changelog 宣言が無いまま数値変化」・4i-A「enabled=true（Phase 2 Block F）」と発火条件を明示 | — |
| 2.3 EF1 破損・パース不能の失敗パス | RED | UC-001 の代替フローは 2a/3a とも「**不在**」のみを扱い、`.legixy.toml`/`graph.toml` の**破損・パース不能**の失敗パスを列挙していない。SPEC-LGX-004.REQ.04/TP-LGX-004 E3 で「graph.toml 破損=exit 1」は規定済だが、UC のフロー記述（失敗パス列挙）に反映されていない。【WEAK: SPEC-004 委譲で解決可。UC への明示列挙は任意】 | GAP-LGX-189 |
| 2.3 EF2 エラー時状態の不変条件保持 | GREEN | check は読み取り専用（グラフ/DB を変更しない）。LEGIXY-SPEC-001 §10.2 FB-INV-4「DB 不在時も上流は正常返却」+ SPEC-LGX-004 が check を判定専用と規定。状態破壊が原理的に発生しない | — |
| 2.3 EF3 エラー通知（severity 区分） | GREEN | Step5「ERROR / WARNING / INFO / OK」を出力と明示。severity 定義は SPEC-LGX-004.REQ.03、finding message 情報量は TP-LGX-004 O3 へ委譲 | — |
| 2.3 EF4 部分失敗継続のフロー反映 | RED | UC-001 Step4 は a〜i の実行を列挙するが、「一部ファイル読込失敗時に他チェックを継続し失敗を finding として報告する」挙動（SPEC-LGX-004.REQ.05）がフロー記述に現れていない。事後条件（部分 CheckReport + 読込失敗 ERROR finding）が観察可能化されていない。【WEAK: SPEC-004.REQ.05 委譲で解決可】 | GAP-LGX-190 |
| 2.4 AT1 アクター権限の一貫性 | GREEN | 開発者 / CI とも読み取り専用 check を同一権限で実行。権限差は本質的に存在せず UC 記述と一貫 | — |
| 2.4 AT2 責任境界（検出 vs 是正） | GREEN | 4g-A.4「アクターが引用箇所を 1 件ずつ確認・修正」でシステム=検出 / アクター=是正の分担を明示 | — |
| 2.4 AT3 実行中外部更新の整合性前提 | GREEN | 並行アクセス/ロックは NFR-LGX-001.REL.07（SQLite busy_timeout）+ REL.08 の射程（TP-LGX-004 C1/C2 で確立済）。check 読み取り専用前提で UC レベル整合 | — |
| 2.5 DF1 入出力データ型・stdout/stderr 分離 | GREEN | 入力（graph.toml + `.legixy.toml` + 成果物本文）→ 出力（CheckReport=stdout / ログ=stderr）。分離は SPEC-LGX-004.REQ.08（TP-LGX-004 O1）へ委譲、UC Step5 が CheckReport を stdout 出力と整合 | — |
| 2.5 DF2 embedding ライフタイム | GREEN | 4i-A.3（engine.db の embedding 参照）+ 4i-A.6「embedding 不在サブノードはスキップ、embed --all 未実行は致命扱いしない」でデータフロー整合 | — |
| 2.5 DF3 エラー時データ解放 | GREEN | 読み取り専用・メモリ上グラフのみで永続リソース確保解放を伴わない。データ解放保証は N/A 相当（言語固有のリソース管理は DD へ委譲） | — |
| 2.6 R1 formal/意味層の責務境界 | GREEN | 4a が無印 check の意味検証を UC-LGX-007 へ委譲。境界は SPEC-LGX-004.REQ.02 / TP-LGX-004 R4/R5 で確立 | — |
| 2.6 R2 形式検査カテゴリ対応 | GREEN | Step4 a（ID 形式）/ b（ファイル存在）/ c（Document ID）/ d（チェーン整合）/ e（孤立ファイル）/ f（DAG=CTX-INV-4）が SPEC-LGX-004.REQ.01 + SPEC-LGX-002 の検証カテゴリに対応 | — |
| 2.6 R3 opt-in 既定 OFF の観察可能性 | GREEN | 4g-B「`[id_changelog].enabled = false`（デフォルト）でスキップ（v0.2.0 同等出力）」で既定 OFF を明示。4h/4i の既定 false は SPEC-LGX-004.REQ.12/13 へ委譲 | — |
| 2.6 R4 終了コード契約一致 | GREEN | 事後条件 exit 0/1 + AF2 の exit 2 境界が LGX-COMPAT-001 §3/§4 #3 の凍結契約（0/1/2）と一致（TP-LGX-004 V1/V2 で確立） | — |
| 2.6 R5 severity 4 段階一致 | GREEN | Step5「ERROR / WARNING / INFO / OK」が SPEC-LGX-004.REQ.03 の 4 段階定義と一致 | — |

集計: **全 22 観点 / GREEN 20 / RED 2**（RED は EF1 / EF4、いずれも WEAK 候補）

## 4. ステータスの決定

RED 観点が 2 件（EF1 / GAP-LGX-189、EF4 / GAP-LGX-190）残存するため、本 TP のステータスは `**ステータス**: red`。

- いずれも WEAK 候補（親 SPEC-LGX-004 が既に答えており、UC フロー記述への明示反映が任意か必須かの裁定が必要）。
- 敵対的精査パスで GENUINE / WEAK / OUT_OF_SCOPE を確定し、WEAK 確定分は人間裁定（UC フローへの追記 or drop）を経て close。全観点 GREEN 化後に本 TP を green へ更新する。

## 5. 観点ナレッジベース参照

この TP の生成時に参照したナレッジベース章節:

- `docs/perspectives/core-perspectives.md` §UC レベル観点（基本フロー / 代替フロー / 例外フロー / アクター遷移と権限 / データフロー）
- `docs/perspectives/core-perspectives.md` §汎用観点（境界値=空グラフ・終了コード境界 / エラーハンドリング / 状態遷移）
- `docs/perspectives/ux-perspectives.md` §エラー・例外の UX（finding 出力の可読性 EF3 に適用）
- 親 SPEC: SPEC-LGX-004.REQ.01〜REQ.13、SPEC-LGX-002（ID 形式 / DAG）
- 委譲先 TP: TP-LGX-004（検証 SPEC レベル観点、green 確定済）
- LGX-COMPAT-001 §3 / §4 #3（終了コード凍結契約）

UX 層観点（Undo/フォーカス/タッチ等）は CLI 検証コマンドには本質的に N/A のため、エラー UX（finding 出力の可読性）以外はスキップした。

## 6. 履歴

| 日付 | 変更内容 |
|---|---|
| 2026-06-13 | 初版作成。UC レベル観点 22 件（GREEN 20 / RED 2）。GAP-LGX-189（EF1 破損失敗パス）/ GAP-LGX-190（EF4 部分失敗継続フロー反映）を起票 |

## 7. 解消（2026-06-13、敵対的精査裁定後）

本 TP が起票した GAP[UC] は全て closed。内訳: **WEAK=方針B（委譲容認）** / **REFUTED=棄却** / **GENUINE=UC 修正で解消**（A/B/C、人間承認 2026-06-13）。§3 表の判定列は初版（起票時）の draft 判定を保持する（精査の履歴として温存）。全 RED 観点は上記裁定で解消したため本 TP は **green**。各 GAP の最終状態は当該 GAP ファイル（§5）と docs/spec-change-proposals/2026-06-13_uc-gap-adversarial-review.md を参照。
