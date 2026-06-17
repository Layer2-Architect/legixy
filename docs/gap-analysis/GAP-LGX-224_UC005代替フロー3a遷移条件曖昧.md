Document ID: GAP-LGX-224

# GAP-LGX-224: UC-LGX-005 代替フロー 3a の遷移条件「embedding が未生成の場合」が曖昧

**親 TP**: TP-LGX-015
**観点**: 2.2 AF4（代替フロー 3a への遷移条件の明示）
**ステータス**: closed (2026-06-13)
**起票日**: 2026-06-13
**分類（暫定）**: WEAK

## 1. 観点

UC レベル観点「代替フローへの遷移条件が明示されているか」を UC-LGX-005 代替フロー 3a にぶつけた。

## 2. 現状の UC / SPEC

UC-LGX-005 代替フロー 3a:
```
- 3a. embedding が未生成の場合、ドリフトスコアなしで走査結果のみ返す
```

- 「embedding が未生成の場合」という発火条件が曖昧。以下の複数の解釈が成立しうる:
  1. engine.db ファイル自体が存在しない
  2. engine.db は存在するが、起点ノードの embedding が未生成
  3. engine.db は存在するが、グラフ上のすべてのノードの embedding が未生成（embed --all 未実行）
  4. engine.db は存在するが、ドリフトスコアの計算に必要な embedding が一部でも欠如
- 解釈によって「3a に遷移するかどうか」の判定タイミングと対象が異なる。
- SPEC-LGX-006 や LEGIXY-SPEC-001 §4 に委譲できる可能性があるが、確認できていない。

## 3. 推奨対応（人間裁定）

以下のいずれか:

- **(A) UC へ追記**: 代替フロー 3a の遷移条件を「engine.db が不在、または engine.db は存在するが起点ノード（または走査対象ノード）の embedding が未生成の場合」と明示する。SPEC-LGX-006 または LEGIXY-SPEC-001 §4 の定義を参照する形が望ましい。
- **(B) drop（委譲容認）**: 「embedding が未生成」の定義（engine.db 不在を含む全ケース）は SPEC-LGX-006 / LEGIXY-SPEC-001 §4 へ委譲する設計と認める。UC の記述粒度は現行のまま。

WEAK 候補（SPEC-LGX-006 / LEGIXY-SPEC-001 §4 に定義があれば委譲容認可能）。
GAP-LGX-221（事前条件の意味定義）と合わせて一括裁定が推奨される。

## 4. 影響範囲

- close されないと TP-LGX-015 が green にならず、UC-005 起点の下流に進めない。
- 振る舞いが SPEC-006 で確定している場合は下流実装への影響は限定的（記述完全性の問題）。
- GAP-LGX-221（事前条件）/ GAP-LGX-223（出力差分）と同根であり一括裁定が効率的。

## 5. 解消（2026-06-13）

敵対的精査裁定: **方針B（委譲容認）**。本観点は親 SPEC（対応 TP[SPEC] green 確定済）が回答しており、UC フローへの明示列挙は任意。GREEN-by-delegation として close。詳細: docs/spec-change-proposals/2026-06-13_uc-gap-adversarial-review.md §F。
