// ts-mcp shared MCP response helpers (parent SPEC-LGX-009 REQ.03/07/13/16).
// Canonical graph nodes: SRC-LGX-002 / SRC-LGX-008.

import { RustCliError } from "./engine.js";

/** REQ.13 / CACHE-INV-3: compile_context / get_compile_audit 返却サイズ上限宣言（Unicode コードポイント）。 */
export const MAX_RESULT_SIZE_CHARS = 500_000;

/**
 * Format a RustCliError into the isError message body.
 * - 通常失敗: `Rust CLI failed (exit N): <stderr>`（REQ.07、v3 実在形式）
 * - タイムアウト: `Rust CLI failed (timeout after Ns):`（REQ.16、部分出力は転送しない）
 */
export function formatCliErrorText(err: RustCliError): string {
  return err.timedOut
    ? `Rust CLI failed (timeout after ${err.timeoutSec}s):`
    : `Rust CLI failed (exit ${err.exitCode}): ${err.message}`;
}

/**
 * exit 0 で stderr が非空なら `_meta["legixy/warnings"]` に stderr 本文を格納する（REQ.03）。
 * stderr が空の場合はフィールド自体を省略する（空文字列は格納しない）。
 * 渡された meta オブジェクトを破壊的に更新して返す。
 */
export function withWarnings(
  meta: Record<string, unknown>,
  stderr: string,
): Record<string, unknown> {
  if (stderr && stderr.trim() !== "") {
    meta["legixy/warnings"] = stderr;
  }
  return meta;
}
