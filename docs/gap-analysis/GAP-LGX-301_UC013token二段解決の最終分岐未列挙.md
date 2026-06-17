Document ID: GAP-LGX-301

# GAP-LGX-301: UC-LGX-013 が `--against` token 二段解決の「最終解決失敗」分岐をフローに列挙していない

**親 TP**: TP-LGX-023
**観点**: 2.2 AF2（代替フローの分岐網羅: token 二段解決の最終分岐）
**ステータス**: closed (2026-06-13)
**起票日**: 2026-06-13
**分類（暫定）**: WEAK（親 SPEC で解決済。UC への明示反映は任意の可能性）

## 1. 観点

UC レベル観点「分岐条件が網羅されているか（境界値・列挙の全 case）」を UC-LGX-013 の基本フロー Step4（ベースライン選択）にぶつけた。Step4 は `--against snapshot:<token>` を「token をまず label として解決し、解決できなければ snapshot_id とみなす」という**二段解決ロジック**で記述しており、この二段解決には「label にも snapshot_id にも一致しない（最終的に解決失敗）」という終端分岐が論理的に存在する。

## 2. 現状の UC / SPEC

- UC-LGX-013 Step4 は二段解決（token → label → snapshot_id フォールバック）を明記する。
- 代替フロー 4a は「ベースライン不在（未 embed のノード、スナップショットに**当該行**なし）」を正常ライフサイクル状態として **INFO + exit 0** で扱う。これは「snapshot は解決できたがその中に対象 artifact の行が無い」ケースである。
- しかし「指定 token が label にも snapshot_id にも一致せず、**いかなる snapshot にも解決しない**」最終分岐（二段解決の終端）が、4a の「当該行なし」に含まれるのか、それとも別の不正参照（exit 1）として扱われるのかが、UC のフロー記述から一意に読み取れない。
- 一方 SPEC レベルでは TP-LGX-010 観点 E9（2026-06-09 敵対的精査で削除＝ALREADY_ANSWERED）が「baseline-absent exit 0 の既存規則（『スナップショットに当該行なし』を exit 0 で扱う）が、token が既存 snapshot に解決しない最終分岐を自然に包含する。`snapshot:` プレフィクス欠如の gross-typo は別途 exit 1 で reject 済」として **GREEN 委譲済**。SPEC-LGX-010 REQ.03 はこの分岐を baseline-absent 経路に吸収する設計。

つまり**振る舞いは親 SPEC で確定済（最終解決失敗 = baseline-absent exit 0）**だが、UC-013 のフロー記述（二段解決の終端列挙）には反映されていない。二段解決ロジックを UC 本文に持ち込んだ以上、その分岐の終端を明示するのが連鎖整合上は望ましい。

## 3. 推奨対応（人間裁定）

以下のいずれか:

- **(A) UC へ追記**: 代替フロー `4c`「`--against snapshot:<token>` が label にも snapshot_id にも解決しない場合: ベースライン不在として扱い INFO（stderr）+ exit 0（`--json` 時は `{"artifact_id","drift":null,"baseline_available":false}`）。`snapshot:` プレフィクス欠如（1a）の gross-typo reject とは区別する（SPEC-LGX-010 REQ.03）」を追加。→ UC のフロー完全性が向上し、二段解決の終端が観察可能化。
- **(B) drop（委譲容認）**: UC は二段解決の起点と代表ケース（4a 当該行なし）を記述し、token 完全解決失敗の最終分岐の終了コード規約は SPEC-LGX-010 REQ.03（baseline-absent exit 0 の包含規則）へ委譲する設計と認める。→ UC は変更しない。

WEAK 候補（UC フロー記述の粒度方針に依存）。フロー妥当性は人間レビュー領域（ハードルール 11 / 03-spec-level-tdd.md §9）。**新規 UC** のため二段解決ロジックの終端明示は本 TP で初精査されており、裁定の価値が高い。

## 4. 影響範囲

- close されないと TP-LGX-023 が green にならず、UC-013 起点の下流（RBA 以降）に進めない。
- 振る舞い自体は SPEC-LGX-010 REQ.03（baseline-absent exit 0 包含）で確定済のため、下流実装の正しさには影響しない（記述完全性の問題）。
- GAP-LGX-302（R6 baseline_source の解決後 id 反映）と同種（UC が本文に持ち込んだ二段解決ロジックの終端/反映の明示不足）であり、一括裁定が可能。

## 5. 解消（2026-06-13）

敵対的精査裁定: **方針B（委譲容認）**。本観点は親 SPEC（対応 TP[SPEC] green 確定済）が回答しており、UC フローへの明示列挙は任意。GREEN-by-delegation として close。詳細: docs/spec-change-proposals/2026-06-13_uc-gap-adversarial-review.md §F。
