Document ID: ADR-LGX-007

# ADR-LGX-007: 非有限スコアの扱いと model_version 照合の責務所在

**ステータス**: accepted
**起票日**: 2026-06-10
**承認日**: 2026-06-10
**承認者**: 開発者（提案一括承認 2026-06-10）
**対象**: SPEC-LGX-010 §3 REQ.03 / REQ.09、§4 SCORE-INV-2

## 1. 文脈（Context）

- 背景: GAP-LGX-185（スコアとして NaN/±Inf が生じた場合の挙動が未規定）、GAP-LGX-186（§4 が「SCORE-INV-2 の検証 = 次元不一致 Error のみ」と過大宣言しており、同一次元・別モデルバージョンへの遷移を検出できない）。
- 制約: JSON は NaN/Inf を表現できない。SPEC-LGX-006（生成側）は非ゼロ L2 正規化を保証するが、consumer 側の防御層は別途必要。

## 2. 検討した選択肢（Options）

### 選択肢 A: 用途別の非対称ポリシー + model_version 一次検出（採用）

- 非有限スコア: calibrate/report（集計系）は skip + 集約 Warning、drift（明示対比）は exit 1、`--json` は非有限値を一切出さず統計不能時は `null`。
- model_version 照合: drift の出力契約として SPEC-LGX-010.REQ.03 に所在（SPEC-LGX-006 は生成・bulk API に責務限定）。文字列完全一致（ADR-LGX-003 の複合キー）を一次検出、次元不一致は補完的検出に訂正。

### 選択肢 B: 一律 Error / 一律 skip

- 一律 Error: 集計系が周辺の 1 ペアで全停止し運用に耐えない。一律 skip: 明示指定の drift で壊れた状態を隠す。

## 3. 判断（Decision）

選択肢 A を採用する。

理由:

- 「明示指定の対比は失敗を隠さない / 集計は継続して可視化する」という既確定の原則（次元不一致・現行ファイル欠落の扱い）と同型であり、SPEC 全体の一貫性を保つ。
- 同一次元のままのモデル差替（MiniLM 同次元ファミリ間等）は次元検査をすり抜けるため、model_version 文字列照合を一次検出にしないと SCORE-INV-2 は検証されない。

## 4. 結果（Consequences）

### 期待される効果
- REQ.06（出力決定性）の前提が成立。`--json` consumer のパーサ破壊を防止。SCORE-INV-2 の検証宣言が実態と一致。

### 受け入れる代償
- skip 由来の統計母数減少（集約 Warning で可視化）。

### 残存リスク
- ベースライン側に model_version 未記録の旧データ → マイグレーション側（SPEC-LGX-008）の取り扱いで補完。

## 5. 関連

- closes: GAP-LGX-185, GAP-LGX-186
- 依存: ADR-LGX-003（model_version 複合キー）
