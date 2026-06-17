Document ID: GAP-LGX-189

# GAP-LGX-189: UC-LGX-001 が設定/graph の破損・パース不能の失敗パスを列挙していない

**親 TP**: TP-LGX-011
**観点**: 2.3 EF1（各ステップでの失敗パスが定義されているか）
**ステータス**: closed (2026-06-13)
**起票日**: 2026-06-13
**分類（暫定）**: WEAK（親 SPEC で解決済。UC への明示反映は任意の可能性）

## 1. 観点

UC レベル観点「各ステップでの失敗パスが定義されているか」を UC-LGX-001 の基本フロー Step2（`.legixy.toml` 読込・解析）/ Step3（`graph.toml` 読込・グラフ構築）にぶつけた。

## 2. 現状の UC / SPEC

- UC-LGX-001 の代替フローは `2a`（`.legixy.toml` が**存在しない**場合）/ `3a`（`graph.toml` が**存在しない**場合）の 2 つのみを定義する。
- いずれも失敗条件は「**不在**」であり、ファイルは存在するが**破損・パース不能**（TOML 構文エラー等）の失敗パスは UC のフロー記述に列挙されていない。
- 一方、SPEC-LGX-004.REQ.04 および TP-LGX-004 観点 E3（2026-06-09 敵対的精査で ALREADY_ANSWERED）で「graph.toml 自体が破損・パース不能なとき=実行時失敗 → exit 1」は**規定済**。慣例仕様 old.source の `load_config_or_exit` / `load_matrix_or_exit` が `process::exit(1)` を実装。

つまり**振る舞いは親 SPEC で確定済**だが、UC-001 のフロー記述（失敗パス列挙）には反映されていない。

## 3. 推奨対応（人間裁定）

以下のいずれか:

- **(A) UC へ追記**: 代替フロー `2a'`/`3a'`「`.legixy.toml`/`graph.toml` が存在するが破損・パース不能の場合、ERROR を報告して exit 1 で終了する」を追加し、SPEC-LGX-004.REQ.04 を参照。→ UC のフロー完全性が向上。
- **(B) drop（委譲容認）**: UC は「不在」を代表失敗パスとして記述し、破損を含む全実行時失敗の終了コード規約は SPEC-LGX-004.REQ.04 へ委譲する設計と認める。→ UC は変更しない。

WEAK 候補（UC フロー記述の粒度方針に依存）。フロー妥当性は人間レビュー領域（ハードルール 11 / 03-spec-level-tdd.md §9）。

## 4. 影響範囲

- close されないと TP-LGX-011 が green にならず、UC-001 起点の下流（RBA 以降）に進めない。
- 振る舞い自体は SPEC-LGX-004 で確定済のため、下流実装の正しさには影響しない（記述完全性の問題）。

## 5. 解消（2026-06-13）

敵対的精査裁定: **方針B（委譲容認）**。本観点は親 SPEC（対応 TP[SPEC] green 確定済）が回答しており、UC フローへの明示列挙は任意。GREEN-by-delegation として close。詳細: docs/spec-change-proposals/2026-06-13_uc-gap-adversarial-review.md §F。
