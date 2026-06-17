Document ID: GAP-LGX-121

# GAP-LGX-121: observe の message の長さ・Unicode 境界方針が未定義

**親 TP**: TP-LGX-007
**観点出典**: TP-LGX-007 §2.1 観点 B1
**ステータス**: closed (2026-06-10)
**起票日**: 2026-06-08
**重大度（2026-06-09 敵対的精査パス）**: minor（verification: low-value, 人間判断で drop 可）。message は重複排除キー非包含（REQ.11）の自由テキストであり、長さ/Unicode 方針の欠落は主フロー成立を妨げない。境界値テスト期待値の確定のための情報補完に留まる。

## 1. 観点

`observe <category> <message>` の `message`（位置引数）について、空文字列の許容、最大長、改行・Unicode（BiDi 制御文字、ZWJ、normalization）の取り扱いが定義されていない。

## 2. 現状の SPEC / UC

SPEC-LGX-007 REQ.01 は `message` を「気づきのテキスト（必須・位置引数）」とのみ記述。LGX-COMPAT-001 §4.1 / §5 も `message: string` とのみ規定し、長さ上限・空文字許容・正規化方針に触れていない。REQ.11 では message が重複排除キーに含まれないことのみ明記されている。

## 3. 期待される情報

SPEC-LGX-007 REQ.01 に以下を追加すべき:

- message の最小・最大長（空文字列を許容するか、許容する場合の意味）
- 改行・制御文字を許容するか、保存時にサニタイズ/正規化するか
- 過大入力時の挙動（切り捨て / exit 2 / exit 1 のいずれか）

## 4. 影響範囲

この GAP がクローズされないと以下に影響:

- UC-LGX-008: observe フローの入力検証ステップが具体化できない
- 下流の TS / TC: message の境界値テスト（0 / 1 / max / max+1、Unicode 境界）の期待値が決まらない
- 機密情報混入（旧 GAP-LGX-138）は NFR-LGX-001 SEC.05（observations テーブルダンプ検査を検証方法に明記）で既出のため敵対的精査で削除済み。message のサニタイズ層は SEC.05 と整合させること

## 5. 解決経緯（クローズ時に記入）

SPEC-LGX-007 v0.5.0（人間裁定 fix・承認 2026-06-10）: REQ.01 に message 境界を確定 — 空/空白のみは CLI exit 1（値の意味的不正）・MCP は zod trim 後 min(1)、最大長なし（SQLite TEXT）、改行・Unicode は無加工保存（忠実記録、SEC.05 はダンプ検査で担保）。

## 6. 関連 ADR

入力長上限が性能/DB スキーマに影響する場合は ADR 起票を検討。
