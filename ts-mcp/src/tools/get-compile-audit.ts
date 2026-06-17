// ts-mcp tool: get_compile_audit (SRC-LGX-008-TS label; parent DD-LGX-008 §3.2 / SPEC-LGX-009 REQ.13).
// Canonical graph node: SRC-LGX-008.
//
// Forward limit to `legixy audit`, format the JSON array into Markdown, attach _meta
// (REQ.13 maxResultSizeChars + REQ.03 legixy/warnings on exit 0 + non-empty stderr).

import type { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js";
import { z } from "zod";
import { RustCliError, type RustEngine } from "../engine.js";
import { MAX_RESULT_SIZE_CHARS, formatCliErrorText, withWarnings } from "../meta.js";
import type { ContextLogEntry } from "../types.js";

/** Render a single ContextLogEntry into a Markdown section（legixy context_log schema）。 */
export function formatAuditEntry(e: ContextLogEntry): string {
  let command = "(none)";
  let files: string[] = [];
  try {
    const p = JSON.parse(e.payload) as {
      command?: string | null;
      target_files?: string[];
    };
    if (p.command) command = p.command;
    if (Array.isArray(p.target_files)) files = p.target_files;
  } catch {
    // payload が非 JSON でも描画は継続（raw は target_id/granularity を表示）。
  }
  return (
    `### #${e.id} (${e.created_at})\n` +
    `- Target: ${e.target_id}\n` +
    `- Granularity: ${e.granularity ?? "(none)"}\n` +
    `- Files: ${files.length > 0 ? files.join(", ") : "(none)"}\n` +
    `- Command: ${command}`
  );
}

export const getCompileAuditSchema = {
  limit: z
    .number()
    .int()
    .min(1)
    .max(50)
    .optional()
    .describe("取得件数（既定: 10）"),
};

type GetCompileAuditInput = { limit?: number };

export function buildAuditArgs(input: GetCompileAuditInput): string[] {
  const args: string[] = ["audit"];
  if (typeof input.limit === "number") args.push("--limit", String(input.limit));
  return args;
}

export function makeGetCompileAuditHandler(engine: RustEngine) {
  return async (input: GetCompileAuditInput) => {
    const cliArgs = buildAuditArgs(input);

    let stdout: string;
    let stderr: string;
    try {
      const r = await engine.run(cliArgs);
      stdout = r.stdout;
      stderr = r.stderr;
    } catch (err) {
      if (err instanceof RustCliError) {
        return {
          content: [{ type: "text" as const, text: formatCliErrorText(err) }],
          isError: true,
        };
      }
      throw err;
    }

    let entries: ContextLogEntry[];
    try {
      entries = JSON.parse(stdout) as ContextLogEntry[];
    } catch (parseErr) {
      return {
        content: [
          { type: "text" as const, text: `Rust CLI returned invalid JSON: ${String(parseErr)}` },
        ],
        isError: true,
      };
    }

    const meta = withWarnings(
      { "anthropic/maxResultSizeChars": MAX_RESULT_SIZE_CHARS },
      stderr,
    );

    if (entries.length === 0) {
      return {
        content: [{ type: "text" as const, text: "監査ログはありません。" }],
        _meta: meta,
      };
    }

    const text = entries.map(formatAuditEntry).join("\n\n");
    return { content: [{ type: "text" as const, text }], _meta: meta };
  };
}

export function registerGetCompileAudit(server: McpServer, engine: RustEngine): void {
  server.tool(
    "get_compile_audit",
    "過去のコンテキスト解決結果を参照する。「前回何が返されたか」を確認したいとき。",
    getCompileAuditSchema,
    makeGetCompileAuditHandler(engine),
  );
}
