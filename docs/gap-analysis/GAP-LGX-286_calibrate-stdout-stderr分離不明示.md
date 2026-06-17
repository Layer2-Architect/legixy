Document ID: GAP-LGX-286

# GAP-LGX-286: calibrate の UC フローに stdout/stderr 分離が不明示

**親 TP**: TP-LGX-021
**観点**: §2.5 DF2 stdout/stderr 分離の不明示
**ステータス**: closed (2026-06-13)
**起票日**: 2026-06-13
**分類（暫定）**: WEAK

## 1. 観点

UC-LGX-011 の基本フロー Step5/6 はヒストグラムや JSON 出力の内容を記述しているが、出力先（stdout / stderr）を明示していない。また代替フロー 2a の INFO メッセージも出力先が不明確。SPEC-LGX-010 REQ.01【v3 差分】は「INFO/WARNING/ERROR は stderr、結果は stdout」を規定しており、v3 からの変更点として重要な契約であるにもかかわらず UC フローに反映されていない。

## 2. 現状の UC / SPEC

**UC-LGX-011 の記述:**
- Step5（text モード）: `ASCII ヒストグラム + 最小/最大/平均 + 現閾値一覧` — 出力先の記述なし
- Step6（--json モード）: `{"pairs": N, "min", "max", "mean", ...}` — 出力先の記述なし
- 代替フロー 2a: `INFO: ベクトルストアが空です。embed --all を実行してください を出力して exit 0` — 出力先の記述なし
- 事後条件: `標準出力にヒストグラム + 閾値が出力される` — Step5/6 については「標準出力」と一応記述あり。ただし 2a の INFO が stderr であること、および代替フローの診断メッセージが stderr 出力であることが不明

**SPEC-LGX-010 REQ.01 の規定（【v3 差分】）:**
- `診断メッセージの出力先: INFO / WARNING / ERROR は stderr に出力する`
- `v3 差分: v3 は report / calibrate の text モード INFO を stdout に出力していた（println）。NFR 整合とパイプ出力の汚染防止のため stderr に統一する`
- 参照: TP-LGX-010 L1（GREEN「REQ.01、NFR.OBS.02」で確立済み）

事後条件「標準出力にヒストグラム + 閾値が出力される」は Step5/6 のメイン出力については stdout を示唆するが、2a の INFO が stderr であることは UC から読み取れない。

## 3. 推奨対応（人間裁定）

**(A) UC へ追記案**: フロー記述を以下のように明示化する:
- Step5: `ASCII ヒストグラム + 最小/最大/平均 + 現閾値一覧を標準出力（stdout）に出力する`
- Step6: `JSON 構造体を標準出力（stdout）に出力する`
- 代替フロー 2a: `INFO メッセージを標準エラー出力（stderr）に出力して exit 0`
- 事後条件（既存）: `標準出力にヒストグラム + 閾値が出力される` はそのまま維持

**(B) drop（委譲容認）案**: stdout/stderr 分離は SPEC-LGX-010 REQ.01 が所有する横断的規約であり、UC の事後条件「標準出力にヒストグラム + 閾値が出力される」でメイン出力 = stdout は明示済み。診断の stderr 分離は SPEC 委譲として UC フローへの逐一明示は不要と判断する。

## 4. 影響範囲

- UC-LGX-011 §基本フロー Step5/6・§代替フロー 2a・§事後条件（追記案 A の場合）
- パイプ利用（`legixy calibrate --json | jq '.distribution'` 等）: stdout/stderr 分離が明示されていないと UC を見た実装者が v3 の誤ったパターン（stdout への INFO 混入）を踏襲するリスクがある
- v3 差分の伝達: 本ギャップが close しないと v3 差分（stdout INFO → stderr INFO）の変更理由が UC レベルで失われる

## 5. 解消（2026-06-13）

敵対的精査裁定: **方針B（委譲容認）**。本観点は親 SPEC（対応 TP[SPEC] green 確定済）が回答しており、UC フローへの明示列挙は任意。GREEN-by-delegation として close。詳細: docs/spec-change-proposals/2026-06-13_uc-gap-adversarial-review.md §F。
