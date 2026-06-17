Document ID: GAP-LGX-291

# GAP-LGX-291: UC-LGX-012 が delete `label:<L>` 解決失敗（不在 label）の exit 1 パスを列挙していない

**親 TP**: TP-LGX-022
**観点**: 2.3 EF1（各ステップでの失敗パスが定義されているか）
**ステータス**: closed (2026-06-13)
**起票日**: 2026-06-13
**分類（暫定）**: GENUINE（不在 label の exit 1 と該当 0 件 snapshot_id の exit 0 が分岐するが、UC フロー記述だけでは判別不能）

## 1. 観点

UC レベル観点「各ステップでの失敗パスが定義されているか」を UC-LGX-012 基本フロー Step6（`snapshot delete <snapshot_id | label:<L>>`）にぶつけた。delete の target は `snapshot_id` と `label:<L>` の 2 形態を取り、解決失敗時の挙動が形態によって異なる。

## 2. 現状の UC / SPEC

- UC-LGX-012 の delete 代替フローは以下 2 つのみを定義する:
  - `6a`: `label:<L>` で同一 label が**複数存在**する場合 → taken_at 最新の 1 件へ決定論的解決して削除（exit 0）
  - `6b`: `snapshot_id` 指定で**該当行 0 件**の場合 → text モード WARNING(stderr) + exit 0、`--json` 時は `{"snapshot_id", "deleted_rows": 0}`
- すなわち UC が列挙する delete の非正常 case は「label 重複（成功解決）」と「snapshot_id の不在（WARNING + exit 0）」のみ。
- 一方、SPEC-LGX-010.REQ.02 delete 節は **「label 解決に失敗した場合は ERROR(stderr) + exit 1」** を明示規定する。さらに REQ.07 は「`delete label:<L>` は label 解決 0 件で **ERROR + exit 1**、`delete <snapshot_id>` は該当 0 行で **WARNING + exit 0**」と DB 不在時も含めてこの非対称を確定している（「名前解決失敗を DB 欠落理由で WARNING に格下げすると project-root 誤り等を覆い隠す」と理由付き）。
- つまり **不在 label（exit 1）と該当 0 件 snapshot_id（exit 0）は意図的に異なる終了コードを返す**が、UC-012 のフロー記述には不在 label の exit 1 パスが現れていない。

振る舞いは親 SPEC（REQ.02 / REQ.07）で確定済だが、UC-012 のフロー記述（失敗パス列挙）には反映されておらず、UC を読むだけでは「label 不在も snapshot_id 不在と同じ exit 0 だろう」と誤読しうる。

## 3. 推奨対応（人間裁定）

以下のいずれか:

- **(A) UC へ追記**: delete 代替フローに `6c`「`label:<L>` 指定で該当 label が存在しない場合（解決 0 件）: ERROR(stderr) + exit 1（SPEC-LGX-010.REQ.02 / REQ.07）」を追加。`snapshot_id` 不在（6b の exit 0）との非対称を明記する。→ UC のフロー完全性が向上し、終了コードの分岐が観察可能化。
- **(B) drop（委譲容認）**: UC は label 重複（6a）と snapshot_id 不在（6b）を代表 case として記述し、不在 label の終了コード規約は SPEC-LGX-010.REQ.02 / REQ.07 へ委譲する設計と認める。→ UC は変更しない。

GENUINE 寄り。6b（exit 0）と挙動が分岐し、SPEC が「名前解決失敗の WARNING 格下げは誤りを覆い隠す」と非対称を意図的に確定しているため、UC フローへの明示反映が完全性に資する。フロー妥当性は人間レビュー領域（ハードルール 11 / 03-spec-level-tdd.md §9）。

## 4. 影響範囲

- close されないと TP-LGX-022 が green にならず、UC-012 起点の下流（RBA 以降）に進めない。
- 振る舞い自体は SPEC-LGX-010.REQ.02 / REQ.07 で確定済のため、下流実装の正しさには影響しない（記述完全性の問題）。ただし不在 label と不在 snapshot_id の exit code 分岐は UC-012 のフロー記述から読み取れないため、(A) 採用が望ましい。

## 5. 解消（2026-06-13）

敵対的精査裁定: **GENUINE**（実 SPEC 照合で確定）。UC 修正で解消（A5: UC-LGX-012 に delete label 解決失敗 → ERROR + exit 1（代替 6c）を追加）。人間承認 2026-06-13（A2/C2/C3 は AskUserQuestion 裁定、推奨案採用）。詳細: docs/spec-change-proposals/2026-06-13_uc-gap-adversarial-review.md §A。
