Document ID: GAP-LGX-135

# GAP-LGX-135: proposal / observation の保持・retention・GC ポリシーが未定義

**親 TP**: TP-LGX-007
**観点出典**: TP-LGX-007 §2.8 観点 L3 / L4
**ステータス**: closed (2026-06-10)
**起票日**: 2026-06-08
**重大度（2026-06-09 敵対的精査パス）**: minor（verification: low-value, 人間判断で drop 可）。retention/GC は単独開発者向けキャッシュ DB の長期肥大化という運用論点で、UC-LGX-008 の主フロー成立を妨げない。ADR 候補。
**統合（2026-06-09 敵対的精査パス）**: 旧 GAP-LGX-136（observation の無限蓄積上限・GC）を本 GAP に統合。proposal 終端状態の retention と observation 解決済みの GC は同一の「engine.db 長期肥大化 + 保持ポリシー」論点であり、解決は単一の retention 方針定義で足りるため。

## 1. 観点

approved / rejected 済みの proposal、および解決済み observation（REQ.11 で同一キー再観測可のため単調増加しうる）をいつまで保持するか、削除・パージ手段があるかが定義されていない。無限蓄積した場合の proposals/observations 一覧の肥大化への対処も不明。

## 2. 現状の SPEC / UC

SPEC-LGX-007 REQ.04 は proposals 一覧の status フィルタを定義するが、終端状態 proposal の retention（保持期間・削除コマンド・自動パージ）に触れていない。proposals テーブル（REQ.09）・observations テーブル（REQ.08）も保持ポリシーを規定しない。REQ.11 は解決済み observation の再観測可を許すが上限管理は未定義。

## 3. 期待される情報

SPEC-LGX-007 REQ.08 / REQ.09 に以下を追加すべき:

- 終端状態 proposal・解決済み observation を永続保持するか（監査証跡として）、パージ手段を提供するか
- パージする場合の対象（rejected のみ / 一定期間経過後）と監査証跡保護との両立
- (旧 136) 解決済み observation のパージが重複排除キー再利用に与える影響（削除後は同一キーが「新規」扱いとなる挙動の明示）

## 4. 影響範囲

この GAP がクローズされないと以下に影響:

- 下流の DD: proposals / observations テーブルの retention/GC 設計が決まらない
- 監査証跡完全性とパージの両立
- 長期運用時の engine.db 肥大化（STATE-INV-1 で再生成可能だがユーザ生成データは復元不能）

## 5. 解決経緯（クローズ時に記入）

SPEC-LGX-007 v0.5.0（人間裁定 2026-06-10: 永続保持）: REQ.08 に保持ポリシーを確定 — observation/proposal は監査証跡として永続保持、自動パージなし・パージコマンド非提供（提供は次バージョン SPEC 改訂事項）、手動 SQL は運用責任域、削除後の dedup キー再利用挙動を注記。ADR-LGX-005 と整合（旧 GAP-136 統合分も解消）。

## 6. 関連 ADR

retention ポリシーは運用方針判断のため ADR 起票を検討。
