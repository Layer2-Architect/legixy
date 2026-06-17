Document ID: ADR-LGX-009

# ADR-LGX-009: Contextual Retrieval の非決定性と決定論保証の両立（context キャッシュ + content_hash 限定 freshness）

**ステータス**: accepted
**起票日**: 2026-06-10
**承認日**: 2026-06-10
**承認者**: 開発者（weak GAP fix 一括承認 2026-06-10）
**対象**: SPEC-LGX-006 §3 REQ.06 / REQ.06.1 / REQ.03、SCORE-INV-1、CTX-INV-1

## 1. 文脈（Context）

- 背景: GAP-LGX-113。Contextual Retrieval 有効時、embedding 生成前に LLM API で上流コンテキストを合成するが、LLM 応答は同一入力でも揺れる（非決定的）。context が再合成のたびに変われば context_hash・embedding が変わり、SCORE-INV-1（content_hash 一致 = fresh）と CTX-INV-1（決定論保証）に緊張が生じる。
- 制約: LLM 出力の決定論化は不可能。機能は既定無効（REQ.06）。

## 2. 検討した選択肢（Options）

### 選択肢 A: context キャッシュ + freshness は content_hash のみ（採用）

- freshness 判定に context_hash を寄与させない（SCORE-INV-1 不変）。合成済み context をキャッシュし、content_hash 不変なら再合成しない。再生成時（stale/--force）の context 揺れは「embedding 値の揺れ」として ADR-LGX-003（値のビット再現は対象外）に包含。合成は逐次実行を既定。

### 選択肢 B: context_hash も freshness に含める

- 欠点: LLM の揺れがそのまま stale 判定の揺れになり、再 embed ループが収束しない（同一 content でも毎回 stale になり得る）。

### 選択肢 C: 有効時は決定論保証を放棄と明記するだけ

- 欠点: fresh 判定まで揺れることを許すと SCORE-INV-1 の意味が機能ごと崩れる。キャッシュで防げる崩れを放置する理由がない。

## 3. 判断（Decision）

選択肢 A を採用する。

理由:

- 非決定性の影響範囲を「再生成時の embedding 値」のみ:に封じ込め、不変条件（SCORE-INV-1 の判定決定性・CTX-INV-1 の順序決定性）を無傷で保つ。
- ADR-LGX-003 の決定論モデル（順序のみ保証・値は drift_threshold 吸収）と完全に同型で、SPEC 全体の一貫性を保つ。

## 4. 結果（Consequences）

### 期待される効果
- Contextual Retrieval 有効時も fresh/stale 判定が決定論的。API 呼出回数も削減（キャッシュ再利用）。

### 受け入れる代償
- content 不変でも上流文書が変わった場合に context が古くなり得る（上流変化は上流側ノードの stale 化で間接検出される）。

### 残存リスク
- キャッシュの保存場所・無効化条件の詳細は DD — context キャッシュの整合テストで担保。

## 5. 関連

- closes: GAP-LGX-113
- 依存: ADR-LGX-003（embedding 決定論モデル）
