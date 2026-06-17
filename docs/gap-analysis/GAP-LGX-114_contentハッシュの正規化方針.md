Document ID: GAP-LGX-114

# GAP-LGX-114: content_hash 算出時のテキスト正規化（改行/BOM/Unicode）方針が未定義

**親 TP**: TP-LGX-006
**観点出典**: TP-LGX-006 §2.5 観点 P-03
**ステータス**: closed (2026-06-10)
**起票日**: 2026-06-08

## 1. 観点

REQ.03/REQ.05 は content_hash を SHA-256 とするが、**ハッシュ対象テキストの正規化**が未定義。同一論理内容が改行コード差（CRLF ↔ LF）・BOM 有無・Unicode 正規化形（NFC/NFD）の違いで異なる SHA-256 を生む。これにより drift（REQ.05）が偽 stale（内容不変だがハッシュ差で「古い」と誤報）または偽 fresh になる。NFR-LGX-001.COMPAT.07（UTF-8 固定、BOM 受容）/ COMPAT.08（LF/CRLF 受容、出力 LF）は IO 方針だが、**ハッシュ計算前の正規化**が SPEC-006 で確定していない。

## 2. 現状の SPEC / UC

SPEC-LGX-006 §3 REQ.03 は「元コンテンツのハッシュ（SHA-256）」、REQ.05 は「content_hash が変化した場合 drift」とするが、**ハッシュ入力の正規化手順**（改行統一・BOM 除去・Unicode 正規化・末尾空白）を規定していない。§4 SCORE-INV-1（ハッシュ一致 = fresh）の判定がこの正規化に依存するため、正規化方針が未確定だと SCORE-INV-1 の意味が環境依存になる。REQ.12 はサブノードの content_hash を content_range 部分のみから計算するとするが、その部分テキストの正規化も同様に未定義。

## 3. 期待される情報

- content_hash 計算前の正規化手順:
  - 改行コードの統一（LF 統一が NFR COMPAT.08 と整合）
  - BOM の除去（NFR COMPAT.07）
  - Unicode 正規化形（NFC 推奨か無正規化か）
  - 末尾空白・末尾改行の扱い
- サブノード content_range 切り出し後の同一正規化適用
- SCORE-INV-1 の「ハッシュ一致」が正規化後テキストに対する一致であることの明示

## 4. 影響範囲

- SCORE-INV-1: freshness 判定の決定性・環境非依存性の根拠
- drift 検出（REQ.05）/ `detect_drift`（REQ.11）: 偽 stale/偽 fresh の防止
- Windows（CRLF）↔ Linux（LF）間でのリポジトリ共有時の drift 一貫性（NFR COMPAT.01/02 の Step1/Step2 両対応）
- 下流 DD / TS: ハッシュ正規化ロジックとクロスプラットフォームテスト

## 5. 解決経緯（クローズ時に記入）

SPEC-LGX-006 v0.6.0（人間承認 2026-06-10）: REQ.03/REQ.12 に content_hash 正規化手順（BOM 除去→CRLF/CR→LF→NFC→末尾正規化→SHA-256）を確定し SCORE-INV-1 を環境非依存化。末尾正規化の厳密挙動は DD 委譲。ADR-LGX-003。

## 6. 関連 ADR

正規化方針はクロスプラットフォーム整合に関わるため ADR 候補。
