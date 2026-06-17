Document ID: CTR-MCP-001

# CTR-MCP-001: legixy MCP 境界契約（配送サーフェス = ts-mcp サーバ）

> 配送軸（DevProc_V4.1 §12）のチェーン根。MCP サーバ（`ts-mcp`）が Agent Surface として公開する
> 3 ツールの入力スキーマ・CLI 変換・`_meta` 規約を**凍結**する。正準根拠は LGX-COMPAT-001 §5 + SPEC-LGX-009。

**サーフェス種別**: MCP サーバ（TypeScript、`ts-mcp`）
**area**: MCP
**凍結状態**: frozen（HR7）
**正準根拠**: LGX-COMPAT-001 §5（MCP 3 ツールと CLI マッピング）, SPEC-LGX-009
**dispatch 設計**: DLV-MCP-001
**適合テスト**: TC-MCP-001（`ts-mcp/tests/e2e.test.ts`、実 legixy バイナリ spawn の E2E）

## 1. 公開サーフェス（3 ツールのみ、MCP-INV-1）

| MCP ツール | CLI 変換 | 入力（zod） |
|---|---|---|
| `compile_context` | `context <files...> [--command][--granularity][--outline-only][--sections][--depth]` | target_files:string[](min1), command?, granularity?(document\|subnode), outline_only?, sections?(min1), depth?(int≥1) |
| `observe` | `observe <category> <message> [--related-id...][--target-file...][--missing-doc][--source-glob]` | category(3値), message, related_ids?, target_files?, missing_doc?, source_glob? |
| `get_compile_audit` | `audit [--limit]` | limit?(int 1..=50) |

## 2. 不変条件・規約

- MCP-INV-1: 公開は 3 ツールのみ（Admin Surface 非公開）。
- MCP-INV-2: CLI へ忠実転送（snake_case→kebab-case、位置引数）。
- `_meta["anthropic/maxResultSizeChars"]=500000`（compile_context/get_compile_audit、REQ.13）。
- `_meta["legixy/warnings"]`（exit 0 + 非空 stderr、全 3 ツール、REQ.03）。
- バイナリ解決: `--engine-binary` > `LGX_BIN` > `TRACEABILITY_ENGINE_BIN`（旧名）> 既定（ADR-LGX-016）。
- タイムアウト `LGX_MCP_TIMEOUT_SEC`（既定 30s / 0=無効、REQ.16）。

## 3. 適合チェックリスト（→ TC-MCP-001 に mapping、P-3）

| # | 契約項目（LGX-COMPAT-001 §7） | TC-MCP ケース | 状態 |
|---|---|---|---|
| 7 | MCP 3 ツールの入力スキーマ・CLI 変換 | integration.test.ts（3ツール/zod/snake→kebab）+ e2e | ✅ GREEN |
| 8 | compile_context `_meta.maxResultSizeChars=500000` | `_meta` ケース + e2e | ✅ GREEN |
| 9 | MCP バイナリ解決（env/フラグ/既定名） | `engine binary resolution` ケース | ✅ GREEN |

> 注: MCP サーフェスは Vitest（integration 26 + e2e 4）で実バイナリ E2E まで検証済み。CLI サーフェス
> （CTR-CLI-001）の RED 項目（BUG 群）が解消されると、MCP get_compile_audit の audit 内容適合も向上する。

## 4. 凍結変更の扱い

凍結後の契約変更は次バージョンの契約改訂（HR7・人間承認）。下流（DLV-MCP→TS→TC→SRC）へ再構築。
