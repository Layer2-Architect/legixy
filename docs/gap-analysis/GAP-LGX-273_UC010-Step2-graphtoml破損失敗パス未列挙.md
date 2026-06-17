Document ID: GAP-LGX-273

# GAP-LGX-273: UC-LGX-010 が Step2 の graph.toml 不在・破損の失敗パスを列挙していない

**親 TP**: TP-LGX-020
**観点**: 2.3 EF1（各ステップでの失敗パスが定義されているか）
**ステータス**: closed (2026-06-13)
**起票日**: 2026-06-13
**分類（暫定）**: WEAK（実装は失敗するが UC フロー記述への明示が任意か必須か裁定待ち）

## 1. 観点

UC レベル観点「各ステップでの失敗パスが定義されているか」を UC-LGX-010 基本フロー Step2（graph.toml パース + embeddings ロード）にぶつけた。Step2 の graph.toml 不在・破損・パース不能の失敗パスが代替フローに列挙されていない。

## 2. 現状の UC / SPEC

- UC-LGX-010 の代替フローは `2a`（embeddings テーブルが空）/ `3a`（compute 失敗）の 2 件のみ。
- Step2 の graph.toml を入力とするパース処理の失敗パス（ファイル不在・TOML 破損・パース不能）は列挙されていない。
- SPEC-LGX-010 REQ.04 は report の主要出力・スキップ・空ストア挙動を規定するが、graph.toml 入力の失敗パスを明示していない。REQ.07「graph.toml に書き込まない」は非破壊性の規定であり、入力としての失敗は別論点。
- UC-LGX-001（check コマンド）の同種ギャップは GAP-LGX-189 として既に起票済（WEAK）。

振る舞い自体は report コマンドの実装が graph.toml パース失敗時に exit 1 で終了することが v3 実測から予期されるが、SPEC-LGX-010 REQ.04 に明示的な規定がない。SPEC-LGX-001 の全般的なエラー処理方針（anyhow + exit 1）から推論は可能。

## 3. 推奨対応（人間裁定）

以下のいずれか:

- **(A) UC へ追記**: 代替フロー `2a'`「graph.toml が存在しない、または破損・パース不能の場合: anyhow エラーコンテキスト付きで stderr に ERROR を出力し exit 1 で終了する」を追加し、SPEC-LGX-010 REQ.04 または SPEC-LGX-001 のエラー処理方針を参照。→ UC のフロー完全性が向上。
- **(B) drop（委譲容認）**: graph.toml パース失敗を含む全実行時失敗の終了コード規約は SPEC-LGX-010 REQ.01 の「実行エラー（exit 1）」へ委譲する設計と認める。代替フロー 3a「compute 失敗 + exit 1」に graph.toml パース失敗を包含できる。→ UC は変更しない。

WEAK 候補（GAP-LGX-189 と同種。UC フロー記述の粒度方針に依存。一括裁定が可能）。

## 4. 影響範囲

- close されないと TP-LGX-020 が green にならず、UC-010 起点の下流（RBA 以降）に進めない。
- 振る舞いは SPEC-LGX-010 REQ.01 の実行エラー定義から推論可能なため、下流実装の正しさには影響しない（記述完全性の問題）。
- GAP-LGX-189（UC-LGX-001 / check コマンド）と同種であり、一括裁定方針が可能。

## 5. 解消（2026-06-13）

敵対的精査裁定: **方針B（委譲容認）**。本観点は親 SPEC（対応 TP[SPEC] green 確定済）が回答しており、UC フローへの明示列挙は任意。GREEN-by-delegation として close。詳細: docs/spec-change-proposals/2026-06-13_uc-gap-adversarial-review.md §F。
