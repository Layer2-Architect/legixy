Document ID: GAP-LGX-113

# GAP-LGX-113: Contextual Retrieval の並行性・順序保証と LLM 合成コンテキストの非決定性が未定義

**親 TP**: TP-LGX-006
**観点出典**: TP-LGX-006 §2.4 観点 C-03（＋ §2.9 観点 D-03 を統合）
**ステータス**: closed (2026-06-10)
**起票日**: 2026-06-08
**重要度**: minor (verification: low-value, 人間判断で drop 可) — 敵対的精査パス 2026-06-09: Contextual Retrieval は REQ.06 で既定無効。非決定性（D-03）は有効時のみ顕在化し、既定オフ下では CTX-INV-1 前提が保たれる。並行度・レート制限は実装層。context_hash freshness 寄与の曖昧さのみが SPEC 級だが decision-relevance 低い。WEAK_OR_PADDED として維持。

## 1. 観点

(1) Contextual Retrieval 有効時（REQ.06）、各ノードの embedding 生成前に上流コンテキストを LLM API で合成する。複数ノードの context 合成を並行実行する場合のレート制限・順序保証・タイムアウトの集約（REQ.06.1 は単一呼出のリトライ/タイムアウトのみ規定）が未定義。
(2) LLM 合成 context は**非決定的**（同一入力でも応答が揺れる）。これにより context_hash（REQ.03）が再 embed のたびに変わりうる。content_hash 不変でも context が変われば embedding が変わるため、SCORE-INV-1（content_hash 一致 = fresh）と CTX-INV-1（決定論保証）に矛盾が生じる。freshness 判定が content_hash のみか context_hash も含むかが未定義。

## 2. 現状の SPEC / UC

SPEC-LGX-006 §3 REQ.06 は Contextual Retrieval の有効化と「上流コンテキストを LLM API で合成」を規定、REQ.06.1 は単一 API 呼出の階層的フォールバック（タイムアウト/リトライ/無効化継続）を規定するが、**複数ノードの並行合成・順序・全体のレート制限**は未定義。REQ.03 は context_hash を必須情報に挙げるが、**context_hash と content_hash の freshness 判定への寄与**（drift 判定が content_hash のみか両方か）が未定義。§4 は CTX-INV-1 を REQ.08 経由で「関連」とするが、LLM 合成の非決定性との整合は記述されていない。

## 3. 期待される情報

- 複数ノードの Contextual Retrieval 合成の並行度・レート制限・順序保証
- freshness 判定が content_hash のみか、context_hash も含むか（SCORE-INV-1 の拡張）
- LLM 合成の非決定性が CTX-INV-1（決定論保証）と両立する方法:
  - context をキャッシュして再 embed 時に再合成しない、または
  - context_hash 変化を drift として扱う、または
  - Contextual Retrieval 有効時は CTX-INV-1 の決定性保証を緩める旨の明文化
- 既定無効（REQ.06）であることが CTX-INV-1 の前提を守る根拠であることの明示

## 4. 影響範囲

- CTX-INV-1 / SCORE-INV-1: Contextual Retrieval 有効時の決定性・freshness の意味が変わる
- UC-LGX-007: Contextual Retrieval 有効フローのデータフロー（context の生成・再利用）が定義できない
- 下流 DD / TS: context_hash の利用方法、決定性テスト（有効時の許容範囲）

## 5. 解決経緯（クローズ時に記入）

SPEC-LGX-006 v0.7.0（人間裁定 fix・承認 2026-06-10）: REQ.06 に非決定性との両立を確定 — freshness は content_hash のみ（SCORE-INV-1 不変）、context キャッシュで content_hash 不変なら再合成しない、再生成時の揺れは ADR-LGX-003 に包含、合成は逐次既定、既定無効が CTX-INV-1 前提の一次根拠。ADR-LGX-009。

## 6. 関連 ADR

LLM 合成の非決定性と決定論保証の両立は architectural 判断のため **ADR 候補**。
