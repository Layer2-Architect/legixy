Document ID: GAP-LGX-196

# GAP-LGX-196: context_log 書込失敗時（REQ.19 本処理優先）が Step7 の失敗パスとして未反映

**親 TP**: TP-LGX-012
**観点**: §2.3 EF2
**ステータス**: closed (2026-06-13)
**起票日**: 2026-06-13
**分類（暫定）**: WEAK

## 1. 観点

SPEC-LGX-003.REQ.19 は「本処理が成功し context_log 書込のみが失敗した場合、結果を返却し exit 0 とし、stderr に Warning 診断を出力する（本処理優先・別トランザクション）」と規定する。UC-LGX-002 の Step7 は「context_log に監査ログを記録する」と記述するだけで、書込失敗時の終端状態（exit 0 継続・stderr Warning・MCP 経由 `_meta["legixy/warnings"]` 通知）が UC フローに反映されていない。Step7 失敗後も処理が「正常終了」することが観察不能。

## 2. 現状の UC / SPEC

- **UC-LGX-002 基本フロー Step7**: 「システムが context_log に監査ログを記録する」（失敗パス未記述）
- **UC-LGX-002 事後条件**: 「context_log に記録が追加される（MCP-INV-4: 監査ログ完全性）」（書込失敗時の代替事後条件なし）
- **SPEC-LGX-003.REQ.19**: 本処理優先・別トランザクション・exit 0 + stderr Warning（MCP 経由は `_meta["legixy/warnings"]` で通知。ベストエフォート）
- **TP-LGX-003 E-03（GAP-LGX-041 closed）**: SPEC レベルでは解消済み

## 3. 推奨対応（人間裁定）

**(A) UC-LGX-002 に代替フロー 7-A を追記する**
「7-A. context_log への書込が失敗した場合（DB ロック・ディスクフル等）、本処理結果の返却は成功扱いとし exit 0 を維持する。stderr に Warning 診断を出力する（SPEC-LGX-003.REQ.19）」として Step7 の失敗パスを明示する。

**(B) drop（委譲容認）**
SPEC-LGX-003.REQ.19 が本処理優先の挙動を確定しており、UC フロー記述に反映せずとも設計上の空白はない。事後条件の MCP-INV-4 も REQ.19 でベストエフォートに確定済のため、UC フロー記述の粒度としては現行で十分とみなす。

## 4. 影響範囲

- UC-LGX-002 代替フロー（追記の場合）または事後条件（注記追加）
- TP-LGX-012 EF2（解消後 GREEN 化）

## 5. 解消（2026-06-13）

敵対的精査裁定: **方針B（委譲容認）**。本観点は親 SPEC（対応 TP[SPEC] green 確定済）が回答しており、UC フローへの明示列挙は任意。GREEN-by-delegation として close。詳細: docs/spec-change-proposals/2026-06-13_uc-gap-adversarial-review.md §F。
