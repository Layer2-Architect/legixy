Document ID: GAP-LGX-061

# GAP-LGX-061: 空グラフ（ノード 0 / エッジ 0）に対する check の挙動と終了コードが未定義

**親 TP**: TP-LGX-004
**観点出典**: TP-LGX-004 §2.1 観点 B1
**ステータス**: closed (2026-06-10)
**起票日**: 2026-06-08
**敵対的精査（2026-06-09）**: WEAK_OR_PADDED として維持。終了コードと finding 件数は REQ.04（exit は Error 件数で決定）+ REQ.06 から導出可能（空グラフ = 0 finding = 0 Error = exit 0）。残る「graph 未構築誘導 Info を出すか」のみが未確定だが UX 上の好み問題であり判定に必須ではない。**severity: minor（verification: low-value, 人間判断で drop 可）**。

## 1. 観点

graph.toml にノードが 1 件も登録されていない（または graph.toml が空）の場合、check / check --formal が何を返し、終了コードが何になるかが未定義。空グラフは「全 check Ok」なのか「Warning（graph 未構築の誘導）」なのか。

## 2. 現状の SPEC / UC

SPEC-LGX-004 §3 は各検証カテゴリの動作を REQ.01〜REQ.14 で定義し、REQ.04 で終了コードを Error 件数で規定するが、**検証対象が 0 件のとき**の振る舞いに触れていない。REQ.02 は embeddings テーブルが空のケースは扱う（Info 1 件）が、graph.toml のノード集合が空のケースとは別の境界である。

## 3. 期待される情報

SPEC または UC に追加されるべき記述:

- ノード 0 / エッジ 0 のグラフに対する check の結果（finding 0 件で exit 0 か、graph 未構築を示す Info を 1 件返すか）
- graph.toml が物理的に存在しない場合と、存在するが空の場合の区別（後者は使用法誤りではないため exit 2 ではないことの確認）

## 4. 影響範囲

この GAP がクローズされないと以下に影響:

- UC-LGX-001: 初期化直後（init 直後で成果物未登録）の check 利用時の期待結果が定義できない
- 下流の TS / TC: 空グラフ fixture に対する期待値（finding 件数・exit code）が書けない
- 他の TP / GAP との依存関係: GAP-LGX-065（Ok severity の使用条件）と関連

## 5. 解決経緯（クローズ時に記入）

SPEC-LGX-004 v0.8.0（人間裁定 fix・承認 2026-06-10）: REQ.01 に空グラフ時挙動を追記 — finding 0 件・exit 0 + stderr Info 1 件（graph 未構築誘導）【v3 差分】。物理不在は未初期化（init 誘導）の別経路、存在するが空は exit 2 にしない。

## 6. 関連 ADR

該当なし（architectural 判断は含まない見込み）。
