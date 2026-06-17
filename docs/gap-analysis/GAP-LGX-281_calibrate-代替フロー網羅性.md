Document ID: GAP-LGX-281

# GAP-LGX-281: calibrate 代替フロー網羅性（--recommend / 次元不一致スキップの未記載）

**親 TP**: TP-LGX-021
**観点**: §2.2 AF1 代替フロー網羅性
**ステータス**: closed (2026-06-13)
**起票日**: 2026-06-13
**分類（暫定）**: WEAK

## 1. 観点

UC-LGX-011 の代替フローが SPEC-LGX-010 REQ.05 の全境界ケースを被覆しているか。具体的には `--recommend` フラグの使用パターンおよび次元不一致スキップ（部分成功継続）が代替フローに列挙されているかを問う。

## 2. 現状の UC / SPEC

UC-LGX-011 の代替フローは以下の 3 件のみ:
- 2a. embeddings が空の場合: INFO 出力して exit 0
- 1a. `--buckets 0` 指定時: エラーメッセージ + exit 1
- 3a. 全ペア算出失敗時: anyhow エラーコンテキスト付きで exit 1

SPEC-LGX-010 REQ.05 は以下をさらに規定する:
- `--recommend` 指定時: `recommended_thresholds` 追加出力（p25 / 1.0−p90 / p75）
- `--recommend` 指定かつ pairs=0: stderr に INFO 1 件（「ペア数 0 のため推奨値は算出されません」）+ 非出力
- 次元不一致ペア: スキップ + 集約 Warning 1 件（stderr）【v3 差分】

これらのうち `--recommend` と次元不一致スキップは UC の基本フロー・代替フローに記述されていない。ただし SPEC-LGX-010 REQ.05 および TP-LGX-010 B7（pairs=0 + --recommend）/ TP-LGX-010 E10（集約 Warning）が既に回答済みである。

## 3. 推奨対応（人間裁定）

**(A) UC へ追記案**: 以下の代替フローを UC-LGX-011 に追加する:
- `1b. --recommend` 指定時: 推奨閾値を追加出力（pairs=0 の場合は INFO + 非出力 + exit 0）
- `3b. 次元不一致ペアが存在する場合: 当該ペアをスキップし集約 Warning 1 件（stderr）を出力して継続、exit 0`

**(B) drop（委譲容認）案**: `--recommend` の出力増分および次元不一致スキップは SPEC-LGX-010 REQ.05 が所有するエンジン挙動であり、UC は主要な分岐（空ストア / バリデーション失敗 / 全体失敗）を記述すれば足りる。部分的なスキップ継続は SPEC-010 委譲として UC フロー記述は不要と判断する。

## 4. 影響範囲

- UC-LGX-011 §代替フロー（追記案 A の場合）
- 下流成果物: RBA 以降に `--recommend` フローおよび次元不一致スキップの挙動が引き継がれる
- 関連 GAP: GAP-LGX-282（--recommend 詳細）/ GAP-LGX-283（次元不一致スキップ詳細）と連動

## 5. 解消（2026-06-13）

敵対的精査裁定: **方針B（委譲容認）**。本観点は親 SPEC（対応 TP[SPEC] green 確定済）が回答しており、UC フローへの明示列挙は任意。GREEN-by-delegation として close。詳細: docs/spec-change-proposals/2026-06-13_uc-gap-adversarial-review.md §F。
