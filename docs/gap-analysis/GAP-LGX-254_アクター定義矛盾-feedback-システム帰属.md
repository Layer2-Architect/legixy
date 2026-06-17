Document ID: GAP-LGX-254

# GAP-LGX-254: UC アクター定義で feedback コマンドが「システム」帰属だが SPEC は「人間のみ CLI 実行」と規定

**親 TP**: TP-LGX-018
**観点**: §2.4 AT1「アクター定義と人間のみ CLI 実行の整合」
**ステータス**: closed (2026-06-13)
**起票日**: 2026-06-13
**分類（暫定）**: GENUINE

## 1. 観点

UC-LGX-008 §アクターは `feedback` コマンドを「システム（自動 Observation 生成: `feedback` コマンド）」に帰属させている。しかし SPEC-LGX-007.REQ.02 は「**人間のみが CLI で実行する**」と明記する。「自動 Observation 生成」という UC 記述と「人間のみが実行する」という SPEC 要求が矛盾しており、後続 RBA でのアクター責務割り当てが根拠を持てない。

## 2. 現状の UC / SPEC

**UC-LGX-008 §アクター:**
```
- システム（自動 Observation 生成: `feedback` コマンド）
- Claude Code（手動 Observation 記録: `observe` コマンド、MCP 経由）
- 人間（Proposal の承認・却下: `approve` / `reject` コマンド）
```

**SPEC-LGX-007.REQ.02:**
```
`legixy feedback` は check の結果や embedding から未対応の observation を生成する。
人間のみが CLI で実行する。
```

**SPEC-LGX-007 §5 Surface 分離マトリクス（参考）:**
```
| feedback 生成 | `feedback` | - |  ← Admin Surface（CLI）、Agent Surface 非対応
```

Admin Surface の CLI コマンドは「人間が実行する」コマンドとして定義されている（REQ.02/03/05 が共通して「人間のみが CLI で実行する」と宣言）。`feedback` が「システム（自動）」帰属であれば MCP 経由での Agent 実行または CI スクリプトによる自動実行を示唆するが、SPEC はそれを禁止している。

考えられる解釈:
1. **誤記説**: UC で「システム」と書いたが「人間がシステム機能（feedback コマンド）を起動する」を意図しており、アクターは「人間」が正しい
2. **意図的な「自動実行可能」説**: feedback は人間だけでなく CI スクリプト等が自動実行することを意図しており、SPEC の「人間のみ」制約が過剰に厳しいと考えた

## 3. 推奨対応（人間裁定）

### (A) UC アクター定義を修正（誤記説を採用）

```
- 人間（Observation 生成: `feedback` コマンド、Proposal の承認・却下: `approve` / `reject` コマンド）
```

または「Admin Surface」としてまとめ:

```
- 人間（Admin Surface: `feedback` / `analyze` / `proposals` / `approve` / `reject` コマンドを CLI で実行）
- Claude Code（Agent Surface: `observe` コマンド MCP 経由）
```

### (B) SPEC-LGX-007.REQ.02 を修正（自動実行解禁）

CI スクリプト等による自動実行を認める場合、REQ.02 の「人間のみが CLI で実行する」制約を緩和し UC の「システム（自動）」帰属を正当化する。ただしこれは SPEC 変更であり `/spec-change` プロセスを経る必要がある。

## 4. 影響範囲

- RBA/SEQA: feedback コマンドのアクター（人間 vs システム）が確定しないと責務割り当て図が描けない
- UC-LGX-008 §アクター: 修正の場合は本 UC ファイルの更新（人間裁定後に Author が実施）
- SPEC-LGX-007: 案(B) 採用の場合は spec-change イベント発生

## 5. 解消（2026-06-13）

敵対的精査裁定: **方針B（委譲容認）**。本観点は親 SPEC（対応 TP[SPEC] green 確定済）が回答しており、UC フローへの明示列挙は任意。GREEN-by-delegation として close。詳細: docs/spec-change-proposals/2026-06-13_uc-gap-adversarial-review.md §F。
