Document ID: GAP-LGX-284

# GAP-LGX-284: calibrate 非有限スコア（NaN/±Inf）の失敗パスが UC に未記述

**親 TP**: TP-LGX-021
**観点**: §2.3 EF2 非有限スコアの失敗パス欠落
**ステータス**: closed (2026-06-13)
**起票日**: 2026-06-13
**分類（暫定）**: WEAK

## 1. 観点

SPEC-LGX-010 REQ.09 は calibrate での NaN/±Inf ペアを「skip + 集約 Warning 1 件」として規定する。これは次元不一致スキップ（EF1 / GAP-LGX-283）と同一経路だが独立した規定であり、UC-LGX-011 のフロー記述に現れていない。

## 2. 現状の UC / SPEC

**UC-LGX-011 の記述:**
- 全ペア算出（Step3）・ヒストグラム生成（Step4）・出力（Step5/6）: 非有限スコアへの言及なし
- 代替フロー 3a: 「全ペア算出失敗時」のみ扱い、NaN/Inf の部分スキップ継続パスなし

**SPEC-LGX-010 REQ.09 の規定（GAP-LGX-185 対応 v0.2.1）:**
- calibrate / report: 非有限スコアのペアは「対比・統計に算入しない（skip + 集約 Warning 1 件。次元不一致 skip と同経路）」
- `--json` 出力は非有限値を一切含まない（JSON 仕様上も NaN/Inf は表現不能）。統計が算出不能な場合は該当フィールドを null とする
- 参照: TP-LGX-010 B9（GAP-LGX-185 による 2026-06-10 GREEN 化）が確立済み

これは SPEC-LGX-010 REQ.09 が既に回答済みの挙動である。

## 3. 推奨対応（人間裁定）

**(A) UC へ追記案**: 代替フローに以下を追加する:
- `3c. 非有限スコア（NaN/±Inf）が発生した場合: 当該ペアをスキップし集約 Warning 1 件（stderr）を出力して処理を継続する（exit 0）。--json 出力に非有限値を含まない`

**(B) drop（委譲容認）案**: 非有限スコアスキップは SPEC-LGX-010 REQ.09 が所有するエンジン挙動であり、GAP-LGX-283（次元不一致スキップ）と同一性質の WEAK。UC への明示は不要と判断する。本 GAP の裁定は GAP-LGX-283 と同一方針を適用する。

## 4. 影響範囲

- UC-LGX-011 §代替フロー（追記案 A の場合）
- GAP-LGX-283（次元不一致スキップ未記述）: 同性質。両 GAP の裁定方針を揃えること
- GAP-LGX-281（代替フロー網羅性）: 本 GAP close により GAP-281 の一部も解消される

## 5. 解消（2026-06-13）

敵対的精査裁定: **方針B（委譲容認）**。本観点は親 SPEC（対応 TP[SPEC] green 確定済）が回答しており、UC フローへの明示列挙は任意。GREEN-by-delegation として close。詳細: docs/spec-change-proposals/2026-06-13_uc-gap-adversarial-review.md §F。
