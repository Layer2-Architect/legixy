Document ID: GAP-LGX-139

# GAP-LGX-139: context_log 記録失敗時に compile_context 本体が失敗するかが未定義

**親 TP**: TP-LGX-007
**観点出典**: TP-LGX-007 §2.9 観点 O6
**ステータス**: closed (2026-06-10)
**起票日**: 2026-06-08

## 1. 観点

compile_context の呼出し記録（context_log への書き込み）が失敗した場合に、compile_context 本体の結果返却が失敗するのか（記録は必須）、本体は成功させて記録のみ best-effort で諦めるのかが定義されていない。MCP-INV-4（全呼出し記録）と FB-INV-4（DB 不在でも上流は返る）が緊張関係にある。

## 2. 現状の SPEC / UC

SPEC-LGX-007 REQ.06 は「compile_context の全呼出しは context_log に記録される」、MCP-INV-4 は「全呼び出しが記録される」と完全性を求める一方、FB-INV-4 / SPEC-LGX-003 は「DB 不在でもグラフ上流は正常に返される」とする。DB は存在するが context_log の INSERT がロック/破損で失敗した中間ケースで、記録完全性（MCP-INV-4）と可用性（FB-INV-4）のどちらを優先するかが未定義。

## 3. 期待される情報

SPEC-LGX-007 REQ.06 に以下を追加すべき:

- context_log 記録が失敗した場合、compile_context 本体を成功させて返すか（可用性優先・記録 best-effort）、記録失敗を以て呼出し全体を exit 1 にするか（完全性優先）
- MCP-INV-4 の「全呼出し記録」が DB 利用可能時のみの保証であることの明示（FB-INV-4 との優先順位）

## 4. 影響範囲

この GAP がクローズされないと以下に影響:

- UC-LGX-008 / UC（compile_context 側）: 記録失敗時の例外フローが具体化できない
- 下流の DD: context_log 書き込みのトランザクション結合（本体と同一 tx か分離か）が設計できない
- MCP-INV-4 と FB-INV-4 の優先順位が下流まで未解決のまま伝播
- 他の TP / GAP との依存関係: GAP-LGX-126（DB 破損時挙動）と同根

## 5. 解決経緯（クローズ時に記入）

SPEC-LGX-007 v0.4.5（人間承認 2026-06-10）: REQ.06 に context_log INSERT 失敗時の可用性優先（本体成功・記録ベストエフォート・stderr Warning。正準は SPEC-LGX-003.REQ.19）を確定。126 との方針分岐を明記。§4 MCP-INV-4 行更新。ADR-LGX-004。

## 6. 関連 ADR

監査完全性 vs 可用性のトレードオフは architectural 判断のため ADR 起票を推奨。
