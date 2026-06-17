Document ID: PAI-MCP-001
**対象成果物**: ts-mcp（MCP サーバ。`deploy/ts-mcp/dist` + 起動ラッパ `deploy/legixy-mcp`）
**対象契約 / SPEC**: CTR-MCP-001、SPEC-LGX-009（MCP サーバ）、LGX-EXT-002（Result Persistence / maxResultSizeChars）、LGX-COMPAT-001（MCP 3 ツール）
**実施主体**: 外部検査スイート `legixy.test`。Author（実装）セッションと独立な主体が・黒箱で・実環境で実施（DevProc_V4.1 §13 §3）。
**実施環境**: Node v22 / 実 legixy バイナリ spawn（E2E）/ `legixy.test/fixtures/*` / Linux

<!-- metadata（chain 外 independent。親への参照。CLAUDE.md ハードルール 3）
  parent_contract: CTR-MCP-001
  parent_specs: SPEC-LGX-009, LGX-EXT-002, LGX-COMPAT-001
  harness: legixy.test（独立黒箱スイート）
  area: MCP（配送軸、independent）
  release_gate: PAI-CLI-001 §3（共通手順を参照）
-->

# PAI-MCP-001: MCP 完成品契約適合検査

> 完成品適合検査（`docs/DevProc_V4/13-product-acceptance-inspection.md`）。**作者と独立に・黒箱で・実環境で**、
> 完成品（ts-mcp MCP サーバ）が SPEC の意図と凍結境界契約（CTR-MCP-001 / MCP-INV）を満たすかを検査する。
> ケースは SPEC-REQ / CTR から独立に作成し、判定は外部観測（MCP プロトコル応答・`_meta`・転送忠実性）のみ。

## 0. 検査ハーネスと独立性

- **ハーネス**: `legixy.test`（`external-test-plan.md` の MCP 節 + `external-test-subnode-cache.md` の EXT-CACHE-META/DET）。
  加えて in-repo の `ts-mcp/tests/e2e.test.ts`（TC-MCP-001、実バイナリ spawn）が決定論回帰を担うが、**PAI は独立黒箱として別運用**。
- **独立性（§3）**: 内部実装・in-repo TC を参照せず、MCP クライアント観点の外部観測のみで判定。

## 1. 検査計画（SPEC-REQ / CTR 表面 → PAI ケース 全数 mapping）

| 領域（SPEC-REQ / CTR） | PAI ケース群 | 黒箱操作 | SPEC 意図の観測 |
|---|---|---|---|
| SPEC-LGX-009.REQ.02 / MCP-INV-1（Agent Surface 限定） | tool 一覧検査 | MCP `list_tools` | 公開ツールは compile_context / observe / get_compile_audit の **3 つのみ**（feedback/analyze/approve/reject 非露出） |
| MCP-INV-2 忠実転送 | EXT-SUB-GRAN-007 等 | `compile_context {granularity:"subnode", outline_only:true}` | snake_case→kebab-case・位置引数で CLI と同等出力 |
| LGX-EXT-002 §4 / CACHE-INV-4 | EXT-CACHE-META-001 | `compile_context` 応答 `_meta` | `_meta["anthropic/maxResultSizeChars"] = 500000` |
| SPEC-LGX-009.REQ.03/13 warnings | — | exit0+非空 stderr のツール呼出 | `_meta["legixy/warnings"]` に stderr 転送（空なら省略） |
| LGX-EXT-002 §3 決定論レイアウト | EXT-CACHE-DET-001/002/003 | `compile_context` 同入力 3 回 | バイト単位同一（プレフィックスキャッシュ前提） |
| MCP-INV-4 監査ログ | EXT-CACHE-META-003 | `compile_context` 後 `get_compile_audit` | 当該呼出が監査に反映 |
| SPEC-LGX-007.REQ.01 observe | — | `observe` ツール | 凍結 stdout `observation: id=N skipped=bool` を parse・記録 |
| CTR-MCP-001 エラー転送 | — | exit≠0 のツール呼出 | `Rust CLI failed (exit N): <stderr>` で isError |

## 2. 結果

| サイクル | 日付 | 検出 | 状態 |
|---|---|---|---|
| 各サイクル | 2026-06-14〜16 | MCP 3 ツール・`_meta=500000`・決定論・監査反映 = 維持。回帰なし | PASS |

**現状判定（2026-06-16）**: 要改善 **0**。MCP 3 ツール・`_meta` 契約・決定論レイアウト・監査ログを実環境（実バイナリ spawn）で確認。
in-repo TC-MCP-001（e2e.test.ts 4 件 + integration 26 件 = 30）も GREEN。

## 3. サマリ & リリースゲート

要改善 **0**。リリース前 PAI ゲート手順は **PAI-CLI-001 §3 を共通適用**する（凍結→独立黒箱実行→要改善0で通過→失敗時は症状+構造の二段修復）。
独立性保持のため `scripts/trace-check.sh` には畳まない（in-repo TC-MCP-001 は必要だが不十分、本 PAI が独立補完）。

## 4. 未カバー契約項目

- 漏れなし。`EXT-CACHE-META-002`（500000 超過時の挙動）は大規模上流 fixture が必要なため運用時に追検査（パニックしないことのみ最低保証）。

## 5. 申し送り

- 決定論回帰は `ts-mcp/tests/e2e.test.ts`（TC-MCP-001）へ昇格済み。PAI（独立黒箱）は廃止せず両輪運用。

## 6. プロセスの穴

R-1〜R-7 と同根の所見（presence ではなく SPEC outcome を検証すべき）は `docs/decisions/2026-06-16_pai-process-hole.md` に集約。

## 改訂履歴

| 日付 | 版 | 変更 |
|---|---|---|
| 2026-06-16 | 1.0.0 | 初版。ts-mcp 完成品 MCP サーフェスの PAI を正式化。MCP 3 ツール・`_meta`・決定論・監査の検査計画と現状全 PASS を記録。 |
