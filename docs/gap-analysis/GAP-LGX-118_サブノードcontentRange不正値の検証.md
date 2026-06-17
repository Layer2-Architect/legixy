Document ID: GAP-LGX-118

# GAP-LGX-118: サブノード content_range が不正値の場合の入力検証が未定義

**親 TP**: TP-LGX-006
**観点出典**: TP-LGX-006 §2.6 観点 I-02
**ステータス**: closed (2026-06-10)
**起票日**: 2026-06-08
**重要度**: minor (verification: low-value, 人間判断で drop 可) — 敵対的精査パス 2026-06-09: content_range の生成は LGX-EXT-001 §4.5.1 の責務（信頼境界内の producer）。UTF-8 境界 panic 防止は DD/実装防御であり NFR SEC.03/SEC.04（不正入力で非クラッシュ）が既にカバー。WEAK_OR_PADDED として維持。

## 1. 観点

REQ.09/REQ.12 はサブノード embedding 入力を `content[range.0..range.1]` とするが、**content_range が不正な場合の検証**が未定義: (a) `range.0 > range.1`（逆転）、(b) range がファイル長を超える（範囲外）、(c) `range.0 == range.1`（空 range）、(d) byte 境界がマルチバイト文字の途中を指す（UTF-8 文字境界違反でスライス panic しうる）。Rust の byte slice は範囲外/文字境界違反で panic するため、検証なしでは embed クラッシュの原因になる。

## 2. 現状の SPEC / UC

SPEC-LGX-006 §3 REQ.09 は「`content[range.0..range.1]`（親見出し・上位章は含めない）」、REQ.12 は content_range 部分のみから content_hash 計算とするが、**range の妥当性検証**（逆転・範囲外・空・UTF-8 境界）を規定していない。content_range の生成は LGX-EXT-001 §4.5.1（見出し抽出）の責務だが、embed 側が信頼して使う前の防御的検証の有無が SPEC-006 で未確定。空 range は GAP-LGX-101（空テキスト）と重なる。

## 3. 期待される情報

- content_range の妥当性検証方針（逆転・範囲外・UTF-8 文字境界）と違反時の挙動（該当サブノードのみ Error 計上 + 継続、REQ.09 の部分失敗トレランス適用）
- UTF-8 文字境界をまたぐ byte range のスライス panic 防止（char_indices ベースの安全な切り出し）
- 空 range の扱い（GAP-LGX-101 へ委譲か独自定義か）
- content_range が信頼境界（LGX-EXT-001 のサブノード抽出が常に妥当 range を生成する保証）か、防御的検証が必要かの明示

## 4. 影響範囲

- NFR-LGX-001.SEC.04（悪意ある入力で OOM/スタックオーバーフローしない）/ SEC.03（不正入力でクラッシュしない）: 不正 range によるパニック防止
- UC-LGX-007: サブノード embedding の例外フロー
- 下流 DD / TS: range 検証ロジックと境界テスト（逆転 / 範囲外 / マルチバイト境界）
- GAP-LGX-101（空テキスト = 空 range）と連携

## 5. 解決経緯（クローズ時に記入）

SPEC-LGX-006 v0.7.0（人間裁定 fix・承認 2026-06-10）: REQ.09 に content_range 防御的検証（逆転・範囲外・UTF-8 境界違反は当該サブノードのみ Error + 継続、panic 禁止 = SEC.03/04、安全切り出しは DD）を確定。空 range は GAP-101 の skip 経路。

## 6. 関連 ADR

該当なし（入力検証方針）。
