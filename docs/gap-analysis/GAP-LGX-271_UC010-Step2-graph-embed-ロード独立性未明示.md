Document ID: GAP-LGX-271

# GAP-LGX-271: UC-LGX-010 が Step2 の graph.toml パースと embeddings ロードの独立性を明示していない

**親 TP**: TP-LGX-020
**観点**: 2.1 BF2（Step2 の部分失敗の独立性）
**ステータス**: closed (2026-06-13)
**起票日**: 2026-06-13
**分類（暫定）**: WEAK（読み取り専用性から原理的に独立。UC フロー記述への明示は任意の可能性）

## 1. 観点

UC レベル観点「各ステップの事後条件が後続ステップの前提を満たすか」を UC-LGX-010 基本フロー Step2 にぶつけた。Step2 は「graph.toml をパースし、embeddings テーブルから全件をロードする」の 2 操作を単一ステップに束ねているが、両操作の独立性（一方の失敗が他方に干渉しないか）がフロー記述に現れていない。

## 2. 現状の UC / SPEC

- UC-LGX-010 Step2「システムが graph.toml をパースし、embeddings テーブルから全件をロードする」は 2 操作を単一ステップとして記述する。
- 代替フロー 3a は Step3 の `compute_edge_scores` / `compute_link_candidates` の失敗を扱う。Step2 の個別失敗（graph.toml パース失敗 / embeddings ロード失敗）の独立性は代替フローに現れていない。
- SPEC-LGX-010 REQ.07 は「report は読取専用。engine.db/graph.toml を変更しない」と規定し、STATE-INV-1 が書込みなしを確立する。読み取り専用性から両操作は原理的に干渉しない。
- ただし、graph.toml パース失敗と「embeddings テーブルへのアクセス継続可否」の関係は SPEC-LGX-010 REQ.04 に明示がない。

## 3. 推奨対応（人間裁定）

以下のいずれか:

- **(A) UC へ追記**: Step2 に注記「graph.toml のパースと embeddings テーブルのロードは独立操作。graph.toml パース失敗時は Step3 が実行不能（3a へ）、embeddings ロード失敗時は空ストア相当（2a へ）として扱う」を追加し、SPEC-LGX-010 REQ.04/REQ.07 を参照。→ ステップ連鎖の明確化。
- **(B) drop（委譲容認）**: Step2 の 2 操作独立性は SPEC-LGX-010 REQ.07（読み取り専用性）と STATE-INV-1 から自明であり、UC フロー記述への明示は不要と認める。graph.toml パース失敗は実質 Step3 への入力欠如として 3a に包含される設計と判断。→ UC は変更しない。

WEAK 候補（親 SPEC の読み取り専用性から原理的に担保。フロー記述の粒度方針に依存）。

## 4. 影響範囲

- close されないと TP-LGX-020 が green にならず、UC-010 起点の下流（RBA 以降）に進めない。
- 振る舞い自体は SPEC-LGX-010 REQ.07 + STATE-INV-1 で担保されているため、下流実装の正しさには影響しない（記述明確性の問題）。

## 5. 解消（2026-06-13）

敵対的精査裁定: **方針B（委譲容認）**。本観点は親 SPEC（対応 TP[SPEC] green 確定済）が回答しており、UC フローへの明示列挙は任意。GREEN-by-delegation として close。詳細: docs/spec-change-proposals/2026-06-13_uc-gap-adversarial-review.md §F。
