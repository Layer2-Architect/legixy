Document ID: DLV-MCP-001

# DLV-MCP-001: legixy MCP（ts-mcp）dispatch 設計

> 配送軸の設計層（CTR-MCP-001 の子）。3 ツールの zod schema + handler を CLI サブプロセスへ転送する設計。

**親 CTR**: CTR-MCP-001
**area**: MCP
**サーフェス source（SRC-MCP-001 anchor）**: `ts-mcp/src/index.ts`（+ server/engine/tools/*）

## 1. dispatch マッピング（MCP ツール → CLI サブプロセス）

| ツール | handler（ts-mcp） | CLI argv | 補足 |
|---|---|---|---|
| compile_context | `tools/compile-context.ts` | `context <files...> [--command][--granularity][--outline-only][--sections][--depth]` | snake→kebab、_meta maxResultSizeChars+warnings |
| observe | `tools/observe.ts` | `observe <category> <message> [...]` | stdout `observation: id=N skipped=bool` を parse |
| get_compile_audit | `tools/get-compile-audit.ts` | `audit [--limit]` | JSON→Markdown 整形 |

## 2. 共通処理（engine.ts / server.ts / index.ts）

- バイナリ解決（ADR-LGX-016）: `--engine-binary` > `LGX_BIN` > `TRACEABILITY_ENGINE_BIN`（旧名・stderr Info）> 既定 `legixy`。
- タイムアウト `LGX_MCP_TIMEOUT_SEC`（既定30/0=無効）。`Rust CLI failed (exit N): <stderr>` / `(timeout after Ns):`。
- `_meta["legixy/warnings"]`（exit0+非空 stderr、全3ツール）。`_meta["anthropic/maxResultSizeChars"]=500000`（cc/audit）。
- MCP-INV-1（3ツールのみ）/ MCP-INV-2（忠実転送）。

## 3. 適合状況（TC-MCP-001）

Vitest（integration 26 + e2e 4、実 legixy バイナリ spawn）で GREEN。CLI 側 BUG（特に BUG-003 設定・BUG-004 drift）の
解消で audit/context の内容適合が向上する（MCP 自体の転送層は適合済み）。

## 4. 非対象

CLI サブプロセスの振る舞いは CTR-CLI-001 / DLV-CLI-001 の責務。MCP は転送・スキーマ・_meta のみ。
