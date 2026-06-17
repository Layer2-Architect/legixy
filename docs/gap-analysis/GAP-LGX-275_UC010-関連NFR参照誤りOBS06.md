Document ID: GAP-LGX-275

# GAP-LGX-275: UC-LGX-010 の「関連 NFR」が check 専用の NFR-LGX-001.OBS.06 を誤参照している

**親 TP**: TP-LGX-020
**観点**: 2.6 R3（UC 関連 NFR 参照の正確性）
**ステータス**: closed (2026-06-13)
**起票日**: 2026-06-13
**分類（暫定）**: GENUINE（report に severity 概念を混入させる誤参照）

## 1. 観点

UC レベル観点として UC-LGX-010 の「関連 SPEC / NFR」節の NFR 参照正確性を確認した。

## 2. 現状の UC / SPEC

UC-LGX-010 の末尾「関連 SPEC / NFR」に以下の記載がある:

```
- NFR-LGX-001.OBS.06（ユーザー向け構造化出力）
```

しかし：

- **NFR-LGX-001.OBS.06** の実際の定義は「CheckResult の severity: Ok / Info / Warning / Error の 4 段階（DD-LGX-001 §2.4）」であり、`check` コマンドの検証結果（CheckResult）の severity 分類に関する要件である。
- **`report` コマンドは severity 概念を持たない**（SPEC-LGX-010 REQ.04: 「閾値判定を行わない計測レポート。report は severity 概念を持たない」）。これは check/report の責務非重複として SPEC に明示されている。
- UC-LGX-010 概要にも「閾値判定を行わない」と記述されており、UC 本文の意図と「関連 NFR」の OBS.06 参照が矛盾している。

NFR-LGX-001.OBS.06 の誤参照により、下流成果物（RBA/DD 等）が report の出力として severity（Ok/Info/Warning/Error）を混入させる設計誤りを引き起こす可能性がある。

正しい NFR 参照（report コマンドに直接適用）:

- **NFR-LGX-001.OBS.02**（出力先: ログは stderr、結果は stdout）
- **NFR-LGX-001.OBS.05**（エラーコード: 0=OK, 1=Error, 2=使用法誤り）

## 3. 推奨対応（人間裁定）

以下のいずれか:

- **(A) UC へ修正（推奨）**: 「関連 SPEC / NFR」の NFR 行を以下に修正する:
  ```
  - NFR-LGX-001.OBS.02（stdout/stderr 分離）, NFR-LGX-001.OBS.05（終了コード）
  ```
  OBS.06 は削除する（report には適用不要）。なお OBS.06 が「ユーザー向け構造化出力」の意図であれば NFR-LGX-001.OBS.06 は不適切であり、該当する構造化出力要件は SPEC-LGX-010 REQ.04 の JSON スキーマ定義または NFR-LGX-001.OBS.03（`--log-format=json`）を参照すること。
- **(B) drop**: 誤参照だが UC 概要の「閾値判定を行わない」記述が明確に severity 非適用を示しており、下流開発者は UC を読んで判断できると判断し、UC 本文は変更しない。

GENUINE 候補（severity 4 段階は check 専用の型定義。report が severity を持つという誤解は DD の型設計に混入する恐れがある）。

## 4. 影響範囲

- UC の NFR 誤参照が下流 RBA/DD での型設計・出力設計に影響する可能性がある。具体的には report の出力型に CheckResult（severity 付き）を混用する設計誤り。
- report の出力定義自体は SPEC-LGX-010 REQ.04 で正確に記述されているため、NFR 参照を修正しても機能要件は変わらない（修正コストは低い）。
- GAP-LGX-274 と同節（関連 SPEC/NFR 節）の問題であり、一括修正が可能。
- close されないと TP-LGX-020 が green にならず、UC-010 起点の下流に進めない。

## 5. 解消（2026-06-13）

敵対的精査裁定: **GENUINE**（実 SPEC 照合で確定）。UC 修正で解消（B2: UC-LGX-010/011/012/013 の NFR.OBS.06 誤参照を OBS.02 + OBS.05 へ訂正）。人間承認 2026-06-13（A2/C2/C3 は AskUserQuestion 裁定、推奨案採用）。詳細: docs/spec-change-proposals/2026-06-13_uc-gap-adversarial-review.md §B。
