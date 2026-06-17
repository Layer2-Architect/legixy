Document ID: GAP-LGX-223

# GAP-LGX-223: UC-LGX-005 代替フロー 3a の出力差分（suspicious_nodes 省略時フォーマット）が未定義

**親 TP**: TP-LGX-015
**観点**: 2.2 AF3（代替フロー 3a の事後条件収束 — 出力フォーマット差分の観察可能性）
**ステータス**: closed (2026-06-13)
**起票日**: 2026-06-13
**分類（暫定）**: GENUINE

## 1. 観点

UC レベル観点「各代替フローも事後条件を持つか（収束先が定義されているか）」を UC-LGX-005 代替フロー 3a にぶつけた。

## 2. 現状の UC / SPEC

UC-LGX-005 代替フロー 3a:
```
- 3a. embedding が未生成の場合、ドリフトスコアなしで走査結果のみ返す
```

UC Step5（基本フロー）の出力:
```
- visited: 走査された全ノード（走査順）
- suspicious_nodes: ドリフト閾値以上のノード（スコア降順）
- depth_map: 各ノードの起点からの距離
```

- 3a で「ドリフトスコアなしで走査結果のみ返す」と記述しているが、このとき `suspicious_nodes` フィールドがどのような状態になるかが不明。
  - 空配列 `[]` か
  - フィールド自体を省略するか（JSON 出力時のスキーマ差異）
  - `null` か
  - 「走査結果のみ」が visited + depth_map の 2 フィールドのみを指すのか
- SPEC-LGX-005.REQ.09 の出力フォーマット定義は「走査結果全般」を規定しているが、embedding 不在ケースにおける suspicious_nodes の省略・空化については言及がない。
- LEGIXY-SPEC-001 §4（investigate エンジン機能）に委譲できるが確認できていない。

## 3. 推奨対応（人間裁定）

以下のいずれか:

- **(A) UC へ追記**: 代替フロー 3a に「このとき suspicious_nodes は空配列（または省略）、visited と depth_map は通常通り返却される」を明示する。併せて SPEC-LGX-005.REQ.09 または LEGIXY-SPEC-001 §4 への参照を追記する。
- **(B) SPEC へ追記 + UC 委譲**: SPEC-LGX-005.REQ.09（または LEGIXY-SPEC-001 §4）に「embedding 不在時の suspicious_nodes の扱い」を規定し、UC は「REQ.09 / §4 へ委譲」とだけ記載する。UC は最小限の変更に留める。

GENUINE 候補（UC フロー記述だけでなく親 SPEC の規定にも明確な答えがない可能性がある）。下流 RBA・DD の実装設計に影響しうる。

## 4. 影響範囲

- close されないと TP-LGX-015 が green にならず、UC-005 起点の下流に進めない。
- 走査結果の出力フォーマット差分が曖昧なまま DD・SRC に進むと、embedding 未生成ケースの出力スキーマが未定義のままコード実装される可能性がある。
- GAP-LGX-224（3a 遷移条件の明示）と連動して一括裁定が推奨される。

## 5. 解消（2026-06-13）

敵対的精査裁定: **方針B（委譲容認）**。本観点は親 SPEC（対応 TP[SPEC] green 確定済）が回答しており、UC フローへの明示列挙は任意。GREEN-by-delegation として close。詳細: docs/spec-change-proposals/2026-06-13_uc-gap-adversarial-review.md §F。
