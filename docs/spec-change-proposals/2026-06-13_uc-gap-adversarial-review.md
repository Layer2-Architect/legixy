# UC レベル GAP 敵対的精査 結果（2026-06-13）

対象: TP[UC] = TP-LGX-011〜023（13 件）が起票した GAP[UC] = **71 件**（GAP-LGX-189/190 + 191〜302 の疎番号 69 件）。
方式: SPEC レベル精査（119→61）と同様、各 GAP を GENUINE / WEAK / REFUTED(OUT_OF_SCOPE/DUPLICATE) に分類。**GENUINE 系の事実主張は実 SPEC / UC / LGX-COMPAT-001 を照合して検証**（サブエージェントは各 UC を個別に見るため GENUINE 過剰検出の傾向あり。memory 既知の「誤認すり抜け」リスクに対応）。

> 注: 本精査は提案。**UC の修正確定は人間レビュー領域**（ハードルール 1・11、03-spec-level-tdd.md §9「UC ドラフトのフロー妥当性レビューは人間」）。本ドキュメントは裁定の基礎資料。

---

## A. 確定 GENUINE — 振る舞い／契約レベルの欠陥（下流の正しさに影響。要 UC 修正）

| # | GAP | UC | 検証済み根拠 | 提案修正 |
|---|---|---|---|---|
| A1 | **GAP-234** | UC-006 | SPEC-005.REQ.05「存在しない起点 → **空の結果を返す（エラーではない）**」(L73-75)。UC-006 代替 2a は「**ERROR を報告する**」= 直接矛盾 | UC-006 2a を「空結果 + exit 0」へ修正 |
| A2 | **GAP-233**（UC-005 にも波及） | UC-006, UC-005 | SPEC-005.REQ.07 `impact <node-id>` / `investigate <node-id>`（**単数**, L88-89）+ LGX-COMPAT-001 #11 `<start>` 単数。UC-006/005 はともに `<start_ids>`（**複数**）+ multi-start セマンティクス | 凍結互換境界（HR7）に従い UC を単数 `<node-id>` へ修正。multi-start が要件なら SPEC/compat 改訂（人間判断） |
| A3 | **GAP-221** | UC-005 | UC-005 事前条件「engine.db が存在する」(L16) が代替 3a「embedding 未生成時はスコアなしで走査結果のみ返す」(L32) + FB-INV-4「engine.db 無しでもグラフ上流は正常返却」と矛盾。investigate の本質依存は graph.toml のみ | engine.db を「任意（drift スコア用。不在時は 3a で degrade）」へ修正 |
| A4 | **GAP-282** | UC-011 | SPEC-010.REQ.05 `calibrate [--buckets <N>] [--recommend]`（L132-147、推奨閾値=パーセンタイル方式・pairs=0 時 INFO）+ compat #7。UC-011 Step1 は `[--buckets N] [--json]` で **`--recommend` と推奨閾値分岐が完全欠落** | UC-011 に `--recommend` 指定の代替フロー（recommended_thresholds 出力 / pairs=0 時 stderr INFO）を追加 |
| A5 | **GAP-291** | UC-012 | SPEC-010「delete: label 解決失敗 → **ERROR + exit 1**」(L83) と「snapshot_id 該当 0 件 → WARNING + exit 0」(L84) の**意図的非対称**。UC-012 は 6a(label 複数)/6b(snapshot_id 0 件 exit 0) のみで **label 不在 → exit 1 が欠落** | UC-012 に代替 6c「delete `label:<L>` 解決失敗（不在）: ERROR + exit 1」を追加 |

## B. 確定 GENUINE — 参照誤り（振る舞い不変。引用修正のみ・低リスク）

| # | GAP | UC | 検証済み根拠 | 提案修正 |
|---|---|---|---|---|
| B1 | **GAP-274** | UC-010 | SPEC-006.REQ.10 の実体は「**モデル更新時の再計算**」(L190)。UC-010 は「REQ.10（**report コマンド**）」(L55) と誤記。report の正準は SPEC-010、API 基盤は SPEC-006.REQ.11 | UC-010 関連 SPEC を「SPEC-010（report 定義）+ SPEC-006.REQ.11（bulk API）」へ訂正 |
| B2 | **GAP-275** ＋ 系統的 | UC-010, **011, 012, 013** | NFR.OBS.06 の実体は「**CheckResult の severity**（4段階, DD-LGX-001 §2.4）」(L145, check 専用)。UC-010/011/012/013 全てが「OBS.06（**ユーザー向け構造化出力**）」と誤記（コピペ伝播） | 4 UC とも OBS.06 → **OBS.02（stdout/stderr 分離）**（+ 必要なら OBS.05 終了コード）へ訂正 |
| B3 | UC-011 参照誤り（サブエージェント未起票） | UC-011 | UC-011 関連 SPEC「SPEC-006 REQ.10（**calibrate コマンド**）」(L56)。REQ.10 ≠ calibrate（=モデル更新時再計算）。calibrate の正準は SPEC-010.REQ.05 | UC-011 関連 SPEC を「SPEC-010.REQ.05（calibrate）+ SPEC-006.REQ.11（bulk API）」へ訂正。B1 と同種 |

> 補足: UC-013 の「SPEC-006 REQ.10（model_version 完全一致判定）」(L66) は **正しい**（REQ.10 は §4 SCORE-INV-2 で model_version 複合キー完全一致を含む, L251）。B1/B3 の誤りは UC-010/011 のみ。

## C. 確定 GENUINE — 構造／責務（設計判断を要する。要人間裁定）

| # | GAP | UC | 検証済み根拠 | 論点 |
|---|---|---|---|---|
| C1 | **GAP-202** | UC-003 | SPEC-002.REQ.12「生成段階ではエラー・Warning を発しない。**検出は check が担う**」(L165) + §4 SUBNODE-INV-3「違反検出は SPEC-004.REQ.14」(L199)。UC-003 Step4「ID 一意性を**検証する**」は生成段階に check の責務を混入 | Step4 を「一意性を担保（明示優先 / 自動同士は縮退）。違反検出は check（SPEC-004.REQ.14）」へ表現修正 |
| C2 | **GAP-269** | UC-009 | UC-009 init Step2 が `docs/traceability/matrix.md（空テンプレート）` を生成 (L31)。matrix.md は **migrate が変換元とする v0.1.0 レガシー入力形式**（SPEC-008.REQ.03, L82-91）。init が新規プロジェクトに旧形式を作る SPEC 根拠なし | matrix.md を init 生成物から除去するか、SPEC-008 に init の matrix.md 生成意図を明記（人間判断） |
| C3 | **GAP-242/244** | UC-007 ⇄ UC-013 | `legixy drift <artifact_id>` が UC-007（L35, 薄い SPEC-006 era 記述）と UC-013（L25, 正準・SPEC-010・代替 1a-6a 完備）の**両方に記述**。重複・UC-007 側は過少仕様 | UC-007 の drift 部を embed 専念へスリム化し UC-013 へ委譲明記、または相互参照で責務分界（人間判断） |

## D. 借料 / 要監視（surface, lean WEAK）

- **GAP-261**（UC-009 migrate ステップ順 vs SPEC-008.REQ.02 確定順序「engine.db コミット先行→平文」L64）: UC は論理列挙か実行順序か曖昧。→ UC に「ステップは論理処理。確定順序は REQ.02」の注記で解消可。
- **GAP-253/256/257**（UC-008）: Observation 4 カテゴリ（chain_integrity/link_candidate/drift/**orphan_file**）に対し analyze の Proposal 変換は 3 種のみ（orphan_file 未対応 → skipped 終端）。FB 変換表の完全性。→ orphan_file→skipped が意図的かを surface。
- **GAP-216**（UC-004 fallback 時 UpstreamArtifact フォーマット）: フィールド構成は DD 所有 → WEAK 寄り。
- **GAP-254**（UC-008 アクター feedback「システム」帰属）/ **GAP-285**（UC-011 exit 境界）: WEAK 寄り。

## E. REFUTED / OUT_OF_SCOPE（サブエージェント過剰検出。敵対的精査で棄却 — 精査が機能した実証）

| GAP | UC | 棄却理由（検証済み） |
|---|---|---|
| **GAP-191** | UC-002 | `--command <S>` は LGX-COMPAT-001 #10（L62）で**凍結済みの正規フラグ**（`compile_context` の `command?`）。「未定義/フロー逸脱」は誤り。意味の SPEC 完備は SPEC-003 レベルで既決（green） |
| **GAP-193** | UC-002 | UC-002 2a（ファイル→ノード逆引き失敗で artifact_id=null）と SPEC-003.REQ.20（起点**ID**未登録は無視して残りで解決, L265）は**別レイヤ**。矛盾なし・相補的。全起点未登録 exit 0 は REQ.20 + UC-002 3a が被覆 |
| **GAP-198** | UC-002 | `--granularity` は UC-004（粒度制御）の主題。SPEC-003 が granularity の対応 UC を UC-004 と規定。UC-002 4-B も参照済。UC-002 への非記述は委譲設計 |

## F. WEAK クラスタ（最大集合・約 40 件）— 単一ポリシー裁定で一括処理

**パターン**: UC が「破損/パース不能の失敗パス」「部分失敗継続」「exit コードの明記」「stdout/stderr 分離」「打切り/skip 時の Info」等を**フロー記述に列挙していない**が、**親 SPEC が既に答えている**（TP[SPEC] で green 確定済）。GAP-LGX-189/190（UC-001）と同型。

該当（代表）: 189,190 / 192,194,195,196,197 / 201,203,204,205 / 211,212,213,214,215,217 / 222,224,225,226 / 231,232,235,236 / 241,243,245 / 252,255 / 262,263,265,266,267,268,270 / 271,272,273 / 281,283,284,286 / 292 / 301,302。

**裁定が必要（A/B いずれかの方針を全 WEAK に一括適用）**:
- **方針 A（UC へ規約注記を追記）**: 各 UC に「実行時失敗（破損・I/O・部分失敗継続・終了コード）の規約は親 SPEC へ委譲。本フローは正常系＋主要分岐を記述」の一文を加え、WEAK を close。UC フロー記述の自己完結性は中程度に留まる。
- **方針 B（委譲容認で drop）**: 親 SPEC が答える観点は UC フロー非記述でも GREEN とみなし、各 GAP を「委譲容認・close」。UC は無変更。SPEC レベル精査の weak-GAP 処理と整合（最小変更）。

> 推奨: **方針 B**（校正＝委譲容認の一貫適用。UC は正常系フローに集中し、エラー規約の単一情報源を SPEC に保つ）。ただし A1-A5/B1-B3/C1-C3 の確定 GENUINE は B 方針でも個別修正する。

---

## 集計（精査後）

| 区分 | 件数 | 処理 |
|---|---|---|
| A 振る舞い/契約 GENUINE | 5 | UC 修正（人間承認） |
| B 参照誤り GENUINE | 3+系統的（OBS.06×4 UC） | 引用訂正（低リスク） |
| C 構造/責務 GENUINE | 3 | 設計裁定（人間） |
| D 借料/WEAK 寄り | ~5 | surface、多くは F へ合流 |
| E REFUTED/OOS | 3 | drop（精査で棄却） |
| F WEAK クラスタ | ~40 | A/B ポリシー一括裁定 |

## 次アクション（人間裁定後）
1. WEAK ポリシー（A/B）確定 → 該当 GAP を close（本文に裁定経緯追記）。
2. A/B/C の GENUINE を UC 修正（提案差分を適用、人間承認）→ 該当 GAP close・対応 TP[UC] 観点を GREEN 化。
3. 全 GAP[UC] closed・全 TP[UC] green を確認 → `bash scripts/trace-check.sh` PASS で UC レベル TDD ループ GREEN 完了 → RBA/SEQA（AI 自律域）へ。
