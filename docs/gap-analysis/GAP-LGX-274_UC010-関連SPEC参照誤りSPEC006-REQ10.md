Document ID: GAP-LGX-274

# GAP-LGX-274: UC-LGX-010 の「関連 SPEC」が SPEC-LGX-006 REQ.10 を誤参照している

**親 TP**: TP-LGX-020
**観点**: 2.6 R2（UC 関連 SPEC 参照の正確性）
**ステータス**: closed (2026-06-13)
**起票日**: 2026-06-13
**分類（暫定）**: GENUINE（下流参照誤りに連鎖する可能性がある誤参照）

## 1. 観点

UC レベル観点として UC-LGX-010 の「関連 SPEC / NFR」節の参照正確性を確認した。

## 2. 現状の UC / SPEC

UC-LGX-010 の末尾「関連 SPEC / NFR」に以下の記載がある:

```
- SPEC-LGX-006 REQ.10（report コマンド）, REQ.11（bulk similarity API）
```

しかし：

- **SPEC-LGX-006 REQ.10** は「モデル更新時の再計算」（model_version 複合キー生成・完全一致判定・全再生成 `embed --all --force` 等）の規定であり、`report` コマンドとは直接関係しない。
- **`report` コマンドの定義** は **SPEC-LGX-010 REQ.04**（トレーサビリティ健全性監査: links / candidates / summary の出力要求・空ストア挙動・スキップ可視化）に所在する。
- SPEC-LGX-006 REQ.11（bulk similarity API: `compute_edge_scores` / `compute_link_candidates` / 決定論的ロード等）は report の計算基盤として UC 基本フロー Step3 の関数名引用（`te_embed::compute_edge_scores` / `te_embed::compute_link_candidates`）に対応しており、こちらの参照は正確。

UC の「関連 SPEC」が「SPEC-LGX-006 REQ.10（report コマンド）」と括弧注釈を誤ったことで、下流成果物（RBA/SEQA 等）が report の主要根拠として SPEC-LGX-006 REQ.10 を誤参照するリスクがある。

## 3. 推奨対応（人間裁定）

以下のいずれか:

- **(A) UC へ修正（推奨）**: 「関連 SPEC / NFR」を以下に修正する:
  ```
  - SPEC-LGX-010 REQ.04（report コマンド）, SPEC-LGX-006 REQ.11（bulk similarity API）
  ```
  SPEC-LGX-006 REQ.10 は不要であれば削除、または「モデル更新時の再計算（embed --all --force 連携）」として別途参照する場合は括弧注釈を訂正する。
- **(B) drop**: 誤参照だが下流 RBA/DD の作成時に実際の SPEC を読んで確認するため影響軽微と判断し、UC 本文は変更しない。

GENUINE 候補（括弧注釈の誤りは根拠連鎖の断絶を引き起こす。RBA が SPEC-LGX-006 REQ.10 を report の根拠として引用した場合、モデル更新系の要求が report フローに混入する恐れがある）。

## 4. 影響範囲

- UC の根拠参照誤りは下流 RBA/SEQA/DD での SPEC 引用誤りに連鎖する可能性がある。
- report コマンドの振る舞い自体は SPEC-LGX-010 REQ.04 で正確に定義されているため、UC 参照を修正しても機能要件は変わらない（修正コストは低い）。
- close されないと TP-LGX-020 が green にならず、UC-010 起点の下流（RBA 以降）に進めない。

## 5. 解消（2026-06-13）

敵対的精査裁定: **GENUINE**（実 SPEC 照合で確定）。UC 修正で解消（B1: UC-LGX-010 関連 SPEC を SPEC-LGX-010.REQ.04（report）+ SPEC-LGX-006.REQ.11 へ訂正）。人間承認 2026-06-13（A2/C2/C3 は AskUserQuestion 裁定、推奨案採用）。詳細: docs/spec-change-proposals/2026-06-13_uc-gap-adversarial-review.md §B。
