Document ID: GAP-LGX-198

# GAP-LGX-198: `--granularity` パラメータが UC-LGX-002 フローに未記述

**親 TP**: TP-LGX-012
**観点**: §2.6 R1
**ステータス**: closed (2026-06-13)
**起票日**: 2026-06-13
**分類（暫定）**: GENUINE

## 1. 観点

SPEC-LGX-003.REQ.01/REQ.03 は `--granularity`（document/subnode の 2 値、既定 document）を compile_context の中核オプションとして規定する。しかし UC-LGX-002 の基本フロー・代替フローに `--granularity` の記述が一切現れない。UC-LGX-004 が粒度別フローを担当するとしても、UC-LGX-002 の関連要求欄には UC-LGX-004 への委譲の記述がなく、SPEC の重要パラメータがフローで具体化されていない。

## 2. 現状の UC / SPEC

- **UC-LGX-002 基本フロー・代替フロー**: `--granularity` への言及なし
- **UC-LGX-002 関連要求**: SPEC-LGX-003.REQ.01〜09 / REQ.15 / REQ.16 / REQ.17 を列挙しているが REQ.03（`granularity` パラメータ定義）が含まれているため SPEC 参照はあるが、フロー上で具体化していない
- **SPEC-LGX-003 ヘッダ**: 「対応 UC: UC-LGX-002, UC-LGX-004」と記載。REQ.03 の検証方法は「UC-LGX-004 粒度別テスト」
- **SPEC-LGX-003.REQ.03**: `granularity = document`（既定）/ `subnode` の 2 値、document が v0.1.0 互換

## 3. 推奨対応（人間裁定）

**(A) UC-LGX-002 基本フロー Step1 または代替フローに `--granularity` を追記する**
UC-LGX-002 Step1 を「`legixy context <files> [--granularity <document|subnode>] [--command <intent>]`」に修正し、代替フローとして「`--granularity subnode` 指定時はサブノード粒度で上流を返す（詳細は UC-LGX-004 へ委譲）」を追記する。これにより REQ.01/REQ.03 のフロー具体化が観察可能になる。

**(B) drop（委譲容認）**
SPEC-LGX-003 の検証方法が「UC-LGX-004 粒度別テスト」であることを根拠に、UC-LGX-002 は `granularity` の基本的な受け付けのみを担い詳細は UC-LGX-004 が具体化すると整理する。UC-LGX-002 の関連要求に UC-LGX-004 への委譲注記を追加することで整合を確認する。

## 4. 影響範囲

- UC-LGX-002 基本フロー Step1 または代替フロー（追記の場合）
- UC-LGX-002 関連要求欄（UC-LGX-004 委譲注記追加の場合）
- TP-LGX-012 R1（解消後 GREEN 化）

## 5. 解消（2026-06-13）

敵対的精査裁定: **REFUTED / OUT_OF_SCOPE**（棄却）。実 SPEC / LGX-COMPAT-001 照合により本 GAP の前提が成立しないことを確認した（サブエージェントの過剰検出）。詳細: docs/spec-change-proposals/2026-06-13_uc-gap-adversarial-review.md §E。
