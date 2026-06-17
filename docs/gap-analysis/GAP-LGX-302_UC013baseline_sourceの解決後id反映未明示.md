Document ID: GAP-LGX-302

# GAP-LGX-302: UC-LGX-013 が baseline_source の「解決後 snapshot_id 反映」をフロー連鎖として明示していない

**親 TP**: TP-LGX-023
**観点**: 2.6 R6（baseline_source 出力値が `--against` の解決後 snapshot_id を反映するか）
**ステータス**: closed (2026-06-13)
**起票日**: 2026-06-13
**分類（暫定）**: WEAK（親 SPEC で解決済。UC への明示反映は任意の可能性）

## 1. 観点

UC レベル観点「成功時の事後条件が観察可能か / 各ステップの事後条件が後続ステップの前提を満たすか」を UC-LGX-013 の Step4（ベースライン選択 = label→snapshot_id 解決）と Step7（`--json` 出力 = `baseline_source` フィールド）の連鎖にぶつけた。

## 2. 現状の UC / SPEC

- UC-LGX-013 Step7 の `--json` 正常スキーマは `{"artifact_id","drift","baseline_available":true,"baseline_source":"embeddings" | "snapshot:<id>"}` であり、snapshot ベースライン時の `baseline_source` 値は **`snapshot:<id>`（解決後 snapshot_id）** を出力契約に持つ。
- 一方 Step4 は `--against snapshot:<L>`（label 入力可能）を「token をまず label として解決」する二段解決で記述する。
- したがって「アクターが `--against snapshot:<LABEL>` で **label を入力**した場合でも、出力 `baseline_source` は label そのものではなく **解決後の snapshot_id**（`snapshot:<id>`）を反映する」という Step4（解決）→ Step7（出力）の連鎖が、UC フローの観察可能なステップとして明示されていない。label 入力時に `baseline_source` が label を返すのか解決後 id を返すのかが、フロー記述から一意に読み取れない。
- 一方 SPEC レベルでは SPEC-LGX-010 REQ.03 の `--json` 正常スキーマ定義が `baseline_source: "embeddings" | "snapshot:<id>"` と明記し、`<id>` = 解決後 snapshot_id である旨を**規定済**（TP-LGX-010 観点 I1/F3 で GREEN）。

つまり**振る舞いは親 SPEC で確定済（baseline_source = 解決後 snapshot_id）**だが、UC-013 のフロー記述（解決 → 出力の連鎖）には反映されていない。二段解決ロジックを UC 本文に持ち込んだ以上、出力がその解決結果を反映する旨を 1 文添えると観察可能性が上がる。

## 3. 推奨対応（人間裁定）

以下のいずれか:

- **(A) UC へ追記**: 基本フロー Step7 に注記「snapshot ベースライン時の `baseline_source` は、`--against` で label を渡した場合も Step4 で**解決後の snapshot_id** を `snapshot:<id>` 形式で反映する（SPEC-LGX-010 REQ.03）」を追加。または Step4 末尾に「解決された snapshot_id は Step7 の `baseline_source` 出力に用いる」を追加。→ 解決 → 出力の連鎖が観察可能化。
- **(B) drop（委譲容認）**: `baseline_source` の値域（`snapshot:<id>` = 解決後 id）は SPEC-LGX-010 REQ.03 の JSON スキーマ定義へ委譲する設計と認め、UC は出力キーの存在のみ記述する。→ UC は変更しない。

WEAK 候補（UC フロー記述の粒度方針に依存）。フロー妥当性は人間レビュー領域。**新規 UC** のため本連鎖は本 TP で初精査。

## 4. 影響範囲

- close されないと TP-LGX-023 が green にならず、UC-013 起点の下流に進めない。
- 振る舞い自体は SPEC-LGX-010 REQ.03 の JSON スキーマで確定済のため、下流実装の正しさには影響しない（記述完全性の問題）。
- GAP-LGX-301（AF2 token 二段解決の最終分岐）と同種（UC が本文に持ち込んだ二段解決ロジックの終端/反映の明示不足）であり、一括裁定が可能。

## 5. 解消（2026-06-13）

敵対的精査裁定: **方針B（委譲容認）**。本観点は親 SPEC（対応 TP[SPEC] green 確定済）が回答しており、UC フローへの明示列挙は任意。GREEN-by-delegation として close。詳細: docs/spec-change-proposals/2026-06-13_uc-gap-adversarial-review.md §F。
