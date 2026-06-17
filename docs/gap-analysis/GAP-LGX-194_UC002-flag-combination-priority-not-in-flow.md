Document ID: GAP-LGX-194

# GAP-LGX-194: Phase 2 代替フローにフラグ組合せ優先順位が未反映

**親 TP**: TP-LGX-012
**観点**: §2.2 AF3
**ステータス**: closed (2026-06-13)
**起票日**: 2026-06-13
**分類（暫定）**: WEAK

## 1. 観点

UC-LGX-002 の代替フロー 4-A（outline-only）/ 4-B（sections）/ 4-C（depth）は各フラグの個別動作を記述するが、SPEC-LGX-003.REQ.18 が規定するフラグ組合せ時の優先順位マトリクス（`--sections` × `--granularity document` で sections 無視 / `--outline-only` × `--sections`（subnode 粒度）で sections 先・outline 後 / `--depth` は直交）がフロー記述に現れない。複数フラグを同時指定した場合の挙動が UC フローから観察できない。

## 2. 現状の UC / SPEC

- **UC-LGX-002 代替フロー 4-A/4-B/4-C**: 各フラグの個別動作のみを記述。フラグ組合せへの言及なし
- **SPEC-LGX-003.REQ.18**: フラグ組合せ優先順位マトリクスを 4 行で確定（outline×document / sections×document / outline×sections / depth 直交）
- **TP-LGX-003 D-08（GREEN 確定済）**: SPEC レベルでは REQ.18 が答えている

## 3. 推奨対応（人間裁定）

**(A) UC-LGX-002 代替フロー に組合せ代替フローを追記する**
例: 4-D として「`--sections` と `--granularity document` を同時指定した場合、`--sections` は無視される」/ 4-E として「`--outline-only` と `--sections`（subnode 粒度）を同時指定した場合、sections フィルタ後に outline 化する」を追記する。

**(B) drop（委譲容認）**
フラグ組合せの優先順位は実装詳細（RBD/DD レベル）であり、UC レベルでは個別フラグの動作を列挙すれば十分とみなす。TP-LGX-003 D-08 が GREEN 確定済の以上、UC フロー記述への反映は不要。

## 4. 影響範囲

- UC-LGX-002 代替フロー（追記の場合）
- TP-LGX-012 AF3（解消後 GREEN 化）

## 5. 解消（2026-06-13）

敵対的精査裁定: **方針B（委譲容認）**。本観点は親 SPEC（対応 TP[SPEC] green 確定済）が回答しており、UC フローへの明示列挙は任意。GREEN-by-delegation として close。詳細: docs/spec-change-proposals/2026-06-13_uc-gap-adversarial-review.md §F。
