Document ID: GAP-LGX-225

# GAP-LGX-225: UC-LGX-005 が graph.toml 不在・破損の失敗パスをフローに列挙していない

**親 TP**: TP-LGX-015
**観点**: 2.3 EF1（各ステップでの失敗パスが定義されているか）
**ステータス**: closed (2026-06-13)
**起票日**: 2026-06-13
**分類（暫定）**: WEAK

## 1. 観点

UC レベル観点「各ステップでの失敗パスが定義されているか」を UC-LGX-005 の事前条件・基本フロー Step2（BFS 走査のためのグラフロード）にぶつけた。

## 2. 現状の UC / SPEC

UC-LGX-005 の事前条件:
```
- graph.toml と engine.db（embedding、スコア）が存在する
```

UC には代替フロー・例外フローとして graph.toml 不在・破損の失敗パスが一切記述されていない。

- 事前条件「graph.toml が存在する」が崩壊した場合（不在 or TOML 構文破損）の UC フロー記述が存在しない。
- SPEC-LGX-004.REQ.04（graph.toml 破損=exit 1）および SPEC-LGX-005 のグラフロードに関する記述（REQ.07 CLI）で振る舞いは規定済みと考えられるが、UC-005 のフロー記述に反映されていない。
- TP-LGX-011 GAP-LGX-189（UC-LGX-001 の同種ギャップ）と同一パターン。

## 3. 推奨対応（人間裁定）

以下のいずれか:

- **(A) UC へ追記**: 代替フローまたは例外フローに「graph.toml が存在しない場合・破損している場合: ERROR を stderr に報告して exit 1 で終了する（SPEC-LGX-004.REQ.04 参照）」を追加する。
- **(B) drop（委譲容認）**: graph.toml 不在・破損を含む全実行時失敗の終了コード規約は SPEC-LGX-004.REQ.04 へ委譲する設計と認め、UC-005 は graph.toml が存在する前提のフロー記述に限定する。UC は変更しない。

WEAK 候補（SPEC-LGX-004.REQ.04 委譲で解決可）。GAP-LGX-189 と同種のため、同一方針で一括裁定が推奨される。

## 4. 影響範囲

- close されないと TP-LGX-015 が green にならず、UC-005 起点の下流に進めない。
- 振る舞い自体は SPEC-LGX-004.REQ.04 で確定済のため、下流実装の正しさには影響しない（記述完全性の問題）。
- GAP-LGX-189（UC-001 の同種 GAP）の人間裁定結果（A/B）を同一方針で適用できる。

## 5. 解消（2026-06-13）

敵対的精査裁定: **方針B（委譲容認）**。本観点は親 SPEC（対応 TP[SPEC] green 確定済）が回答しており、UC フローへの明示列挙は任意。GREEN-by-delegation として close。詳細: docs/spec-change-proposals/2026-06-13_uc-gap-adversarial-review.md §F。
