Document ID: GAP-LGX-129

# GAP-LGX-129: observation の状態集合と遷移（pending/analyzing/解決済み）の完全定義が欠落

**親 TP**: TP-LGX-007
**観点出典**: TP-LGX-007 §2.3 観点 S4
**ステータス**: closed (2026-06-10)
**起票日**: 2026-06-08

## 1. 観点

observation のライフサイクル状態の完全な集合と遷移規則が定義されていない。REQ.11 では `status IN ('pending', 'analyzing')` と「解決済み」が断片的に現れるが、状態の入口/出口、誰が遷移を起こすか（feedback / analyze / approve）が明示されていない。

## 2. 現状の SPEC / UC

SPEC-LGX-007 REQ.11 は重複排除の適用範囲として `status IN ('pending', 'analyzing')` を、また「解決済み observation は同一キーで再観測可能」と記述。REQ.03 で analyze が observations を集約・分析するが、analyze が observation の status を `pending → analyzing` に遷移させるのか、proposal 承認（approve）が observation を「解決済み」に遷移させるのか、状態遷移の主体と契機が未定義。状態の列挙値（pending / analyzing / 解決済み = どんな値か）も未確定。

## 3. 期待される情報

SPEC-LGX-007 REQ.08 / REQ.11 に以下を追加すべき:

- observation の status 列挙値の完全集合（pending / analyzing / resolved 等）
- 各遷移の契機（observe で pending 生成 → analyze で analyzing → ? で解決済み）と遷移を起こすコマンド
- 「解決済み」へ遷移する条件（対応 proposal の approve か、別経路か）

## 4. 影響範囲

この GAP がクローズされないと以下に影響:

- UC-LGX-008: observation lifecycle の状態遷移図が確定しない
- 下流の DD: observations テーブルの status 列の値域と遷移制約が設計できない
- 重複排除の適用範囲（GAP-LGX-122, REQ.11）が状態集合に依存するため整合が取れない

## 5. 解決経緯（クローズ時に記入）

SPEC-LGX-007 v0.4.4（人間承認 2026-06-10）: REQ.08 に observation 状態モデル（pending/analyzing/resolved、resolved 終端不可逆）を確定し REQ.11 dedup 適用範囲を状態モデルへ接続。127+129 の §3.1 統合再編は ADR 付き将来課題。

## 6. 関連 ADR

該当なし。
