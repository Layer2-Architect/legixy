Document ID: GAP-LGX-158

# GAP-LGX-158: matrix.md / `[id.chain]` の抽出規則と不正入力時の挙動が未定義

**親 TP**: TP-LGX-008
**観点出典**: TP-LGX-008 §2.7 観点 I-1, I-2
**ステータス**: closed (2026-06-10)
**起票日**: 2026-06-08

## 1. 観点

REQ.03 は「matrix.md の各成果物 ID をノードとして抽出」「`[id.chain]` の順序定義に基づき chain エッジを生成」とするが、(1) matrix.md からの **ID 抽出規則**（どの節・どの表記・どのパターンを ID とみなすか）、(2) matrix.md の節構造が想定と異なる場合の挙動、(3) `[id.chain]` の順序定義が **欠落/不正** な v0.1.0 入力時のエッジ生成挙動が定義されていない。

## 2. 現状の SPEC / UC

SPEC-LGX-008 §3 REQ.03 で **抽出ルールの概要** に触れているが、**抽出の具体的構文規則・節構造の前提・不正/欠落入力時の扱い** は未定義。LGX-COMPAT-001 §6 は config スキーマを示すが、`[id.chain]` 欠落時の migrate 挙動は規定外。

## 3. 期待される情報

SPEC または UC に追加されるべき記述:

- matrix.md からの ID 抽出規則（対象節 `[matrix]` の `section` 設定との関係、ID パターンは `[id]` の `pattern` に従うか）
- matrix.md が想定構造でない（節が無い・複数ある・表形式が崩れている）場合の挙動（Error / 警告して継続 / 抽出 0 件）
- `[id.chain]` の `order` が欠落/不正な場合の挙動（chain エッジ 0 本で続行か Error か）

## 4. 影響範囲

この GAP がクローズされないと以下に影響:

- UC-LGX-009: matrix → graph 変換フローの入力検証ステップが定義できない
- 下流 TS / TC: 不正/欠損 matrix 入力テストの期待値が書けない
- 他の GAP との依存: GAP-LGX-144（破損検出）, GAP-LGX-141（空入力）

## 5. 解決経緯（クローズ時に記入）

SPEC-LGX-008 v0.7.0（人間裁定 fix・承認 2026-06-10）: REQ.03 に抽出規則を確定 — [matrix] section + [id] pattern 準拠（細目 DD）、構造崩れで抽出 0 件は空入力（GAP-141）として正常終了 + Info、[id.chain] order 欠落/不正は破損（REQ.03a）として Error（暗黙 0 本続行の禁止）。

## 6. 関連 ADR

該当時に起票。
