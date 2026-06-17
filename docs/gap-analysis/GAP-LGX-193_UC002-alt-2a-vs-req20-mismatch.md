Document ID: GAP-LGX-193

# GAP-LGX-193: 代替フロー 2a「artifact_id=null」と SPEC REQ.20「無視して残りで解決」の挙動矛盾

**親 TP**: TP-LGX-012
**観点**: §2.2 AF1
**ステータス**: closed (2026-06-13)
**起票日**: 2026-06-13
**分類（暫定）**: GENUINE

## 1. 観点

UC-LGX-002 代替フロー 2a は「ファイルがどのノードにも対応しない場合、targets の artifact_id を null として返す」と記述する。しかし SPEC-LGX-003.REQ.20 は「指定された起点のうち graph.toml に未登録のものは無視し、残りの起点で解決して exit 0」と規定しており、「null を返す」という記述と「無視して残りで解決する」という挙動は意味が異なる可能性がある。特に全起点が未登録の場合（「空の upstream で exit 0」）と部分未登録の場合（「残りで解決」）の分岐が UC フローで観察できない。

## 2. 現状の UC / SPEC

- **UC-LGX-002 代替フロー 2a**: 「ファイルがどのノードにも対応しない場合、targets の artifact_id を null として返す」
- **SPEC-LGX-003.REQ.20 (1)**: 「指定された起点のうち graph.toml に未登録のものは**無視し、残りの起点で解決して exit 0**。全起点が未登録の場合は空の upstream で exit 0。未解決の起点は Target Node Metadata セクションおよび stderr Info 診断に記録する」
- **TP-LGX-003 I-01（GAP-LGX-043 closed）**: SPEC レベルでは解消済み

差異の焦点:
1. UC「null として返す」 vs SPEC「無視して残りで解決する（null フィールドを返すのではなく存在しないものとして処理）」
2. 全起点未登録 vs 部分未登録の分岐が UC 2a に存在しない（1 ケースしか記述されていない）
3. stderr Info 診断の出力が UC フローに現れない

## 3. 推奨対応（人間裁定）

**(A) UC-LGX-002 代替フロー 2a を REQ.20 に合わせて修正・分割する**
- 2a-1: 一部ファイルが未登録の場合 → 未登録を無視し残りで解決・stderr Info 診断
- 2a-2: 全ファイルが未登録の場合 → 空 upstream で exit 0・stderr Info 診断
- 「artifact_id=null」の記述を REQ.20 の「無視して残りで解決」に整合させる（null フィールドとして返す実装かどうかは DD レベルの判断）

**(B) drop（委譲容認）**
「artifact_id=null」が「当該 target の artifact_id が未解決であることを Target Node Metadata で記録する」の UC 抽象表現であり、REQ.20 の「未解決を Target Node Metadata に記録」と同義と解釈する。この解釈が成立するなら SPEC への委譲で整合。

## 4. 影響範囲

- UC-LGX-002 代替フロー 2a（修正・分割の場合）
- TP-LGX-012 AF1（解消後 GREEN 化）

## 5. 解消（2026-06-13）

敵対的精査裁定: **REFUTED / OUT_OF_SCOPE**（棄却）。実 SPEC / LGX-COMPAT-001 照合により本 GAP の前提が成立しないことを確認した（サブエージェントの過剰検出）。詳細: docs/spec-change-proposals/2026-06-13_uc-gap-adversarial-review.md §E。
