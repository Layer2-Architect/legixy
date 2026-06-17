Document ID: TS-MCP-001

# TS-MCP-001: legixy MCP（ts-mcp）契約適合テスト仕様

> 配送軸のテスト仕様（DLV-MCP-001 の子、TC-MCP-001 の親）。CTR-MCP-001 の 3 ツール契約を Vitest の
> 検証ケースへ翻訳する。実バイナリ E2E は `tests/e2e.test.ts`、契約・転送・_meta は `tests/integration.test.ts`。

**親 DLV**: DLV-MCP-001
**対象契約**: CTR-MCP-001
**実装テスト**: TC-MCP-001（`ts-mcp/tests/e2e.test.ts`、実 `legixy` バイナリ spawn）

## 1. 検証方針

- **実バイナリ E2E**（`e2e.test.ts`）: 実 legixy を spawn し compile_context/observe/get_compile_audit を駆動。
- **契約・転送**（`integration.test.ts`）: 3 ツール限定（MCP-INV-1）、snake→kebab（MCP-INV-2）、zod min1、
  `_meta` 規約、タイムアウト、バイナリ解決。

## 2. ケース群（CTR-MCP-001 §3 と対応）

| 契約項目 | 検証ケース | 現状 |
|---|---|---|
| 3 ツールのみ（MCP-INV-1） | `MCP-INV-1` ケース | GREEN |
| CLI 変換（snake→kebab、位置引数） | buildArgs ケース群 | GREEN |
| zod min1 / 値域 | schema contracts ケース | GREEN |
| `_meta.maxResultSizeChars=500000` | _meta ケース + e2e | GREEN |
| `_meta.legixy/warnings` | warnings ケース群 | GREEN |
| バイナリ解決（ADR-016） | resolution ケース | GREEN |
| 実バイナリ E2E（3ツール） | e2e.test.ts（4） | GREEN |

## 3. 申し送り

MCP 転送層は適合済み。ただし get_compile_audit の audit 内容は CLI 側 context_log に依存し、CLI の
BUG-003/004 解消で内容適合が向上する。CLI 側 RED は CTR-CLI-001 / TS-CLI-001 が追跡する。
