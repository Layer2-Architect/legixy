Document ID: GAP-LGX-122

# GAP-LGX-122: related_id の重複・不在ノード・形式検証の扱いが未定義

**親 TP**: TP-LGX-007
**観点出典**: TP-LGX-007 §2.1 観点 B2 / §2.7 観点 I2
**ステータス**: closed (2026-06-10)
**起票日**: 2026-06-08

## 1. 観点

`observe` の `--related-id`（num_args=0..）に、0 個 / 多数 / 重複した ID / 存在しないノード ID / ID 形式（`{type}-LGX-NNN`）を満たさない文字列が渡された場合の挙動が定義されていない。特に重複排除キー `(category, related_ids)` の正準化（昇順ソート後 JSON 化）における重複 ID の正規化規則が曖昧。

## 2. 現状の SPEC / UC

SPEC-LGX-007 REQ.11 は重複排除キーを「related_ids は昇順ソート後に JSON 文字列化して比較」と定義するが、**重複 ID の dedup（distinct 化）有無**を明示していない。LGX-COMPAT-001 §4.1 は `--related-id` が 0 個以上であることのみ規定。REQ.08 は related_node_id フィールドにサブノード ID も格納可能とするが、ID 形式検証や実在検証の有無に触れていない。

## 3. 期待される情報

SPEC-LGX-007 REQ.01 / REQ.11 に以下を追加すべき:

- related_id の ID 形式検証の有無と、形式不正時の終了コード（exit 2 か受理して保存か）
- 実在しないノード ID を許容するか（observation は「見落とし」記録のため未登録 ID を許容する可能性が高いが、明文化が必要）
- 重複 ID が渡された場合に正準キー生成前に distinct 化するか（重複排除キーの安定性に直結）

## 4. 影響範囲

この GAP がクローズされないと以下に影響:

- UC-LGX-008: related_id バリデーションステップが具体化できない
- 下流の DD / TS: 正準キー生成関数の入力前処理（sort + distinct）の仕様が確定しない → MCP-INV-3 重複排除テストの期待値がぶれる
- semantic_key の機械保証手段（旧 GAP-LGX-131）は FB-INV-5 + REQ.09 で不変条件確定済み・実装手段は DD 領分として敵対的精査で削除済み。related_ids の distinct 正規化は本 GAP で確定させ、その正準化ロジックを semantic_key 生成と共有すること

## 5. 解決経緯（クローズ時に記入）

SPEC-LGX-007 v0.4.1（人間承認 2026-06-10）: REQ.01 に related_id の無検証受理方針（未登録対象への気づき記録用途）、REQ.11 正準化に distinct 化ステップを追加（semantic_key 生成と共有）。凍結比較セマンティクス不変。

## 6. 関連 ADR

該当なし。
