Document ID: GAP-LGX-214

# GAP-LGX-214: UC-LGX-004 フラグ組合せ指定時の適用順序がフロー記述に現れていない

**親 TP**: TP-LGX-014
**観点**: §2.2 AF3（フラグ適用順序の非記述）
**ステータス**: closed (2026-06-13)
**起票日**: 2026-06-13
**分類（暫定）**: WEAK

## 1. 観点

UC-LGX-004 の代替フロー 4-A（outline-only）・4-B（sections フィルタ）・4-C（depth_limit）・4-D（subnode 展開）が列挙されているが、これらを組合せ指定した場合の適用順序（SPEC-LGX-003.REQ.18 が規定する「sections フィルタ → outline 化」等のフラグ組合せ優先順位マトリクス）が UC フローに観察可能な形で示されていない。

## 2. 現状の UC / SPEC

- **UC-LGX-004 代替フロー**: 4-A〜4-D が独立した代替フローとして列挙されるが、同時指定時の相互関係が記述されていない。
- **SPEC-LGX-003.REQ.18**: フラグ組合せマトリクスを規定。`--outline-only` × `--sections`（subnode 粒度）では「sections フィルタが先、outline 化が後」、`--depth` は直交。
- **問題点**: UC フローを読むだけでは、例えば `--granularity subnode --sections X --outline-only` の同時指定時に 4-B（sections）→ 4-A（outline）の順で適用されることが判断できない。

## 3. 推奨対応（人間裁定）

**(A) UC への追記案**

代替フロー末尾に「フラグ組合せ時の適用優先順位: sections フィルタ（4-B）→ outline 化（4-A）→ depth 制限（4-C）は直交（SPEC-LGX-003.REQ.18 へ委譲）」という 1 行の注釈を追加する。

**(B) drop（委譲容認）案**

SPEC-LGX-003.REQ.18 のフラグ組合せマトリクスに委譲し、UC フローの修正は不要と裁定する。UC は「各フラグの個別動作」を示す役割であり、組合せ優先順位の詳細は SPEC レベルの規定に属する。

## 4. 影響範囲

- UC-LGX-004 §代替フロー（追記案の場合のみ）
- 下流への影響: DD レベルでのフラグ適用ロジック実装に影響するが SPEC-LGX-003.REQ.18 が既に規定しているため GENUINE ではない

## 5. 解消（2026-06-13）

敵対的精査裁定: **方針B（委譲容認）**。本観点は親 SPEC（対応 TP[SPEC] green 確定済）が回答しており、UC フローへの明示列挙は任意。GREEN-by-delegation として close。詳細: docs/spec-change-proposals/2026-06-13_uc-gap-adversarial-review.md §F。
