Document ID: GAP-LGX-222

# GAP-LGX-222: UC-LGX-005 の基本フロー Step1 に `--max-depth` 引数が記載されていない

**親 TP**: TP-LGX-015
**観点**: 2.2 AF1（分岐の網羅性 — `--max-depth` 未指定代替フロー欠落）
**ステータス**: closed (2026-06-13)
**起票日**: 2026-06-13
**分類（暫定）**: WEAK

## 1. 観点

UC レベル観点「分岐条件が網羅されているか（境界値・列挙の全 case）」を UC-LGX-005 の引数定義にぶつけた。

## 2. 現状の UC / SPEC

UC-LGX-005 基本フロー Step1:
```
1. アクターが `legixy investigate <start_ids> [--drift-threshold <val>]` を実行する
```

- `--max-depth` オプションが Step1 のコマンド記述に現れていない。
- 代替フロー 1a は `--drift-threshold` の省略パスを定義しているが、`--max-depth` 省略パスに相当する代替フローは存在しない。
- SPEC-LGX-005.REQ.04 は「max_depth 省略時は無制限とし、impact/investigate の両コマンドに適用」と明記している（REQ.07 + REQ.04 を合わせると `investigate` も `--max-depth` を受け付ける）。
- LGX-COMPAT-001 §4 #12 が `investigate <start>` の互換引数を凍結しており、`[--max-depth N]` はそこで規定されている可能性がある（未確認）。
- 一方、`--drift-threshold` は investigate 固有の引数として 1a で明示しているため、`--max-depth` も同様に UC で扱う一貫性が問われる。

## 3. 推奨対応（人間裁定）

以下のいずれか:

- **(A) UC へ追記**: Step1 のコマンド行を `legixy investigate <start_ids> [--drift-threshold <val>] [--max-depth N]` に修正し、`--max-depth` 省略時（省略=無制限）を代替フロー 1b または注記として追記する。→ SPEC-LGX-005.REQ.04 との対応が観察可能化。
- **(B) drop（委譲容認）**: `--max-depth` の引数定義と省略時挙動は SPEC-LGX-005.REQ.04 / LGX-COMPAT-001 §4 #12 へ委譲する設計と認める。investigate UC は drift 固有の引数（`--drift-threshold`）のみを記載すれば十分とする。→ UC は変更しない。

WEAK 候補（SPEC-LGX-005.REQ.04 + LGX-COMPAT-001 への委譲で解決可）。フロー妥当性は人間レビュー領域。

## 4. 影響範囲

- close されないと TP-LGX-015 が green にならず、UC-005 起点の下流に進めない。
- `--max-depth` の実装挙動は SPEC-LGX-005.REQ.04 で確定済のため、下流実装への影響はない（記述完全性の問題）。

## 5. 解消（2026-06-13）

敵対的精査裁定: **方針B（委譲容認）**。本観点は親 SPEC（対応 TP[SPEC] green 確定済）が回答しており、UC フローへの明示列挙は任意。GREEN-by-delegation として close。詳細: docs/spec-change-proposals/2026-06-13_uc-gap-adversarial-review.md §F。
