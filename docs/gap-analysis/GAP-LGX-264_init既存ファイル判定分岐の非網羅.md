Document ID: GAP-LGX-264

# GAP-LGX-264: UC init 代替フロー 2a の既存ファイル判定が `.legixy.toml` のみに限定

**親 TP**: TP-LGX-019
**観点**: §2.2 AF1
**ステータス**: closed (2026-06-13)
**起票日**: 2026-06-13
**分類（暫定）**: GENUINE

## 1. 観点

UC-LGX-009 の init 代替フロー 2a は「`.legixy.toml` が既に存在する場合」のみを記述するが、SPEC-LGX-008.REQ.07 が規定する「既存」判定対象は複数ファイルにわたる。UC 分岐の非網羅を問う。

## 2. 現状の UC / SPEC

UC-LGX-009 の代替フロー:
- **2a.** init で `.legixy.toml` が既に存在する場合、ERROR を報告する

SPEC-LGX-008.REQ.07「既存ファイルがある場合はエラー（`--force` で上書き）」および「既存判定対象は legixy 管理生成物のみ」（GAP-LGX-143 解消 2026-06-10 対応）で、判定対象ファイルは以下の 4 種:

1. `.legixy.toml`
2. `.trace-engine.toml`（旧名）
3. `docs/traceability/graph.toml`
4. `.legixy/engine.db`

UC の 2a は `.legixy.toml` のみを例示しており、`.trace-engine.toml` 既存（旧プロジェクトへの後付け init シナリオ）や `graph.toml` 既存・`engine.db` 既存のケースが代替フローとして現れていない。

特に `.trace-engine.toml` 既存ケースは、v0.1.0 プロジェクトに legixy を後付けする際の実際的シナリオであり、REQ.13 設定ファイル探索順とも交差する重要な分岐。

## 3. 推奨対応（人間裁定）

**(A) UC へ追記案:**
代替フロー 2a を一般化する:
- **2a.** init で legixy 管理生成物（`.legixy.toml` / `.trace-engine.toml` / `docs/traceability/graph.toml` / `.legixy/engine.db`）のいずれかが既に存在する場合、ERROR を報告する（`--force` で上書き可）

**(B) drop（委譲容認）案:**
SPEC-LGX-008.REQ.07（判定対象の詳細）と TP-LGX-008 E-5/B-5 へ委譲確定とする。UC は代表例（`.legixy.toml`）を示せば十分であり、判定対象の詳細は SPEC/DD レベルで定義。

## 4. 影響範囲

- 下流 RBA: init ロバストネス図での既存ファイル判定パス
- 下流 TS: init 既存ファイルテストの各 case（`.trace-engine.toml` 既存を含む）
- REQ.13 設定ファイル探索順との交差（`.trace-engine.toml` 既存時の init 挙動）

## 5. 解消（2026-06-13）

敵対的精査裁定: **方針B（委譲容認）**。本観点は親 SPEC（対応 TP[SPEC] green 確定済）が回答しており、UC フローへの明示列挙は任意。GREEN-by-delegation として close。詳細: docs/spec-change-proposals/2026-06-13_uc-gap-adversarial-review.md §F。
