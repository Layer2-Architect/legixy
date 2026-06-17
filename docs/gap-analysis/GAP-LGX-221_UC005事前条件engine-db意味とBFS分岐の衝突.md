Document ID: GAP-LGX-221

# GAP-LGX-221: UC-LGX-005 の事前条件「engine.db が存在する」と代替フロー 3a の分岐条件が衝突する

**親 TP**: TP-LGX-015
**観点**: 2.1 BF2（事前条件とステップ前提の整合）
**ステータス**: closed (2026-06-13)
**起票日**: 2026-06-13
**分類（暫定）**: GENUINE

## 1. 観点

UC レベル観点「各ステップの前提条件が先行ステップの事後条件を満たすか / アクター入力の検証タイミング」を UC-LGX-005 の事前条件とフロー分岐にぶつけた。

## 2. 現状の UC / SPEC

UC-LGX-005 の事前条件:
```
- graph.toml と engine.db（embedding、スコア）が存在する
```

代替フロー 3a:
```
- 3a. embedding が未生成の場合、ドリフトスコアなしで走査結果のみ返す
```

- 事前条件は「engine.db が存在する」と規定しているが、代替フロー 3a は「embedding が未生成の場合」を有効な代替パスとして扱っている。
- engine.db が存在するにもかかわらず embedding が未生成（または一部のみ生成）という状態が起こりうる場合、事前条件「engine.db が存在する」を満たしつつ 3a に落ちる矛盾が生じる。
- 一方、engine.db 自体が不在の場合は事前条件が崩壊（事前条件エラー扱いか、3a へのパスか）という別の問題も生じる。
- SPEC-LGX-005 や LEGIXY-SPEC-001 §4 に engine.db の「存在」と「embedding 生成済み」の区別を規定する箇所が存在するか確認できていない。

## 3. 推奨対応（人間裁定）

以下のいずれか:

- **(A) UC 事前条件を分割追記**: 事前条件を「graph.toml が存在する（必須）」と「engine.db が存在しかつ embedding が生成済みである（任意、未充足時は 3a）」に分割し、代替フロー 3a の発火条件（engine.db 不在 OR embedding 未生成）を明示する。
- **(B) drop（委譲容認）**: 事前条件「engine.db が存在する」は「engine.db が存在する場合の最良パス前提」であり、不在または embedding 未生成の場合は 3a で自動分岐するという設計として認め、LEGIXY-SPEC-001 §4 / SPEC-LGX-006 への委譲と明示する。UC は変更しない。

GENUINE 候補（UC フロー記述の事前条件定義と分岐条件が意味的に衝突している可能性）。

## 4. 影響範囲

- close されないと TP-LGX-015 が green にならず、UC-005 起点の下流（RBA 以降）に進めない。
- 振る舞いが SPEC-006 / LEGIXY-SPEC-001 §4 で確定していれば実装への影響は限定的だが、UC 記述の意味不整合は下流 RBA のアクター責務分担に誤りを生む可能性がある。
- GAP-LGX-224（3a 遷移条件の明示）と連動して一括裁定が推奨される。

## 5. 解消（2026-06-13）

敵対的精査裁定: **GENUINE**（実 SPEC 照合で確定）。UC 修正で解消（A3: UC-LGX-005 事前条件で engine.db を任意化（不在時 3a で degrade、FB-INV-4））。人間承認 2026-06-13（A2/C2/C3 は AskUserQuestion 裁定、推奨案採用）。詳細: docs/spec-change-proposals/2026-06-13_uc-gap-adversarial-review.md §A。
