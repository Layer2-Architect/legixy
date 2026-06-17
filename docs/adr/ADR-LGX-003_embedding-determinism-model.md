Document ID: ADR-LGX-003

# ADR-LGX-003: embedding 決定論モデル（順序決定性・正規化ハッシュ・複合 model_version）

**ステータス**: accepted
**起票日**: 2026-06-10
**承認日**: 2026-06-10
**承認者**: 開発者（提案一括承認 2026-06-10）
**対象**: SPEC-LGX-006 §3 REQ.03 / REQ.04 / REQ.10 / REQ.12、§4 CTX-INV-1 / SCORE-INV-1 / SCORE-INV-2

## 1. 文脈（Context）

- 背景: GAP-LGX-104（ゼロベクトル cosine と浮動小数点推論の数値安定性）、GAP-LGX-114（content_hash の正規化方針）、GAP-LGX-115（model_version の生成と変化判定）。3 件はいずれも「embedding 系の決定論をどの層で・どこまで保証するか」という同一の設計判断に帰着するため 1 ADR に統合する。
- 制約: ONNX 推論は環境（CPU/BLAS 実装）によりビット単位の再現性を保証できない。クロスプラットフォーム（Windows/Linux）で改行・BOM・Unicode 正規化形が揺れる。

## 2. 検討した選択肢（Options）

### 選択肢 A: 三層の決定論モデル（採用）

1. **値の決定論は放棄し順序の決定論のみ保証**: CTX-INV-1/SCORE-INV は走査・出力順序の決定性のみを対象とし、推論値の微小差は drift_threshold が吸収する。ゼロベクトル cosine は NaN/Inf を返さず skip + 集約 Warning（標準経路）/ Error（standalone drift）。
2. **入力の決定論は正規化で確保**: content_hash は BOM 除去 → CRLF/CR→LF → NFC → 末尾正規化後の UTF-8 への SHA-256（SCORE-INV-1 を環境非依存化）。
3. **モデル同一性は複合キーで確保**: model_version = モデル名 + ONNX ファイル内容ハッシュ + 前処理プロファイル + 出力次元。判定は文字列完全一致（SCORE-INV-2 の決定論化）。

### 選択肢 B: ビット単位の完全再現性を要求

- 欠点: ONNX Runtime のスレッド数・SIMD・BLAS 差で実現不能、もしくは性能を大幅に犠牲にする。検証不能な過大宣言になる。

## 3. 判断（Decision）

選択肢 A を採用する。

理由:

- 保証可能なもの（順序・入力・モデル同一性）と保証不能なもの（推論値のビット再現）を明確に分離し、過大宣言（検証不能な不変条件）を SPEC から排除する。
- 偽 stale / 偽 fresh の主要因（改行・BOM・正規化形・モデル無言差替）を正規化と複合キーで構造的に潰す。

## 4. 結果（Consequences）

### 期待される効果
- クロスプラットフォームで drift 判定が安定。モデルファイル差替の無言すり抜けを防止。

### 受け入れる代償
- 微小な推論値差は検出対象外（drift_threshold 未満は同一とみなす）。

### 残存リスク
- 末尾正規化の厳密挙動・hex 桁数は DD 委譲 — DD 凍結時に fixture テストで pin する。

## 5. 関連

- closes: GAP-LGX-104, GAP-LGX-114, GAP-LGX-115
- 関連: GAP-LGX-116（旧モデル移行、別管理）、SPEC-LGX-010.REQ.09（非有限値の consumer 防御 = ADR-LGX-007）
