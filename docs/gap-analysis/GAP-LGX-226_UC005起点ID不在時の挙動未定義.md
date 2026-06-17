Document ID: GAP-LGX-226

# GAP-LGX-226: UC-LGX-005 が起点 ID 不在時の挙動（空結果 exit 0）をフローに記載していない

**親 TP**: TP-LGX-015
**観点**: 2.3 EF3（起点 `start_ids` に存在しない ID が渡されたときの挙動）
**ステータス**: closed (2026-06-13)
**起票日**: 2026-06-13
**分類（暫定）**: WEAK

## 1. 観点

UC レベル観点「各ステップでの失敗パスが定義されているか（エラー時の状態は不変条件を満たすか）」を UC-LGX-005 基本フロー Step2（BFS 走査起点の解決）にぶつけた。

## 2. 現状の UC / SPEC

UC-LGX-005 基本フロー:
```
1. アクターが `legixy investigate <start_ids> [--drift-threshold <val>]` を実行する
2. システムが起点ノードから有向グラフを逆方向（上流方向）に BFS 走査する
```

- Step2 で起点ノードがグラフに存在しない場合の挙動が UC のフロー記述（代替フロー・例外フロー）に現れていない。
- SPEC-LGX-005.REQ.05「起点ノードがグラフに存在しない場合、空の結果を返す（エラーではない）」で振る舞いは規定済。終了コードは exit 0（NFR-LGX-001 OBS.05 / USE.04、TP-LGX-005 観点 23 で GREEN 確定済）。
- UC の事後条件に終了コードの明示がなく、起点不在時の出力（空 visited / 空 suspicious_nodes / 空 depth_map）も未記述。

## 3. 推奨対応（人間裁定）

以下のいずれか:

- **(A) UC へ追記**: 代替フロー「2a. 起点 ID がグラフに存在しない場合: 空の走査結果（visited=[], suspicious_nodes=[], depth_map={}）を返し、exit 0 で終了する（SPEC-LGX-005.REQ.05 参照）」を追加する。
- **(B) drop（委譲容認）**: 起点不在時の空結果・exit 0 は SPEC-LGX-005.REQ.05 + LGX-COMPAT-001 §4 への委譲で観察可能とし、UC は変更しない。

WEAK 候補（SPEC-LGX-005.REQ.05 + NFR-LGX-001 委譲で解決可）。フロー妥当性は人間レビュー領域。

## 4. 影響範囲

- close されないと TP-LGX-015 が green にならず、UC-005 起点の下流に進めない。
- 振る舞い自体は SPEC-LGX-005.REQ.05 で確定済のため、下流実装の正しさには影響しない（記述完全性の問題）。
- 終了コード（exit 0）の明示化は CI スクリプト等の利用者にとって価値があるため、A（追記）の方が推奨されるが裁定は人間に委ねる。

## 5. 解消（2026-06-13）

敵対的精査裁定: **方針B（委譲容認）**。本観点は親 SPEC（対応 TP[SPEC] green 確定済）が回答しており、UC フローへの明示列挙は任意。GREEN-by-delegation として close。詳細: docs/spec-change-proposals/2026-06-13_uc-gap-adversarial-review.md §F。
