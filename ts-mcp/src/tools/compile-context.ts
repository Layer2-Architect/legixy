// ts-mcp tool: compile_context (SRC-LGX-002-TS label; parent DD-LGX-002 §4 / SPEC-LGX-009 REQ.04/13/15).
// Canonical graph node: SRC-LGX-002.
//
// Forward target_files / command / granularity / outline_only / sections / depth to
// `legixy context`, return its Markdown stdout untouched (MCP-INV-2).
// Success: attaches _meta["anthropic/maxResultSizeChars"]=500000 (REQ.13) and, when the
// CLI exited 0 with non-empty stderr, _meta["legixy/warnings"] (REQ.03).

import type { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js";
import { z } from "zod";
import { RustCliError, type RustEngine } from "../engine.js";
import { MAX_RESULT_SIZE_CHARS, formatCliErrorText, withWarnings } from "../meta.js";

export const compileContextSchema = {
  target_files: z
    .array(z.string())
    .min(1)
    .describe("コンテキストを解決するファイルパス"),
  command: z
    .string()
    .optional()
    .describe('作業意図（例: "implement", "test", "refactor"）'),
  // Granularity: legixy-ctx enum (Document | Subnode)。graph|upstream|all は無効値（DD-LGX-002）。
  granularity: z
    .enum(["document", "subnode"])
    .optional()
    .describe(
      "返却粒度（SPEC-LGX-003 REQ.04）。既定: document。subnode 指定でサブノード単位の解決",
    ),
  // Phase 2 Block B 追加引数 — SPEC-LGX-003 REQ.15/16/17（SPEC-LGX-009 REQ.15）
  outline_only: z
    .boolean()
    .optional()
    .describe("見出し構造のみ返却（本文を含まない、SPEC-LGX-003 REQ.15）"),
  sections: z
    .string()
    .min(1)
    .optional()
    .describe(
      '特定のサブノード ID のみ取得（コンマ区切り、SPEC-LGX-003 REQ.16）。例: "DD-X-001#abc,DD-X-001#def"',
    ),
  depth: z
    .number()
    .int()
    .min(1)
    .optional()
    .describe("上流 N 階層まで遡る（SPEC-LGX-003 REQ.17、既定: 無制限）。1 以上の整数"),
};

type CompileContextInput = {
  target_files: string[];
  command?: string;
  granularity?: "document" | "subnode";
  outline_only?: boolean;
  sections?: string;
  depth?: number;
};

/** MCP 入力 → CLI argv（snake_case → kebab-case / 位置引数、MCP-INV-2）。 */
export function buildCompileContextArgs(input: CompileContextInput): string[] {
  const args: string[] = ["context", ...input.target_files];
  if (input.command !== undefined) args.push("--command", input.command);
  if (input.granularity !== undefined) args.push("--granularity", input.granularity);
  if (input.outline_only === true) args.push("--outline-only");
  if (input.sections !== undefined) args.push("--sections", input.sections);
  if (input.depth !== undefined) args.push("--depth", String(input.depth));
  return args;
}

export function makeCompileContextHandler(engine: RustEngine) {
  return async (input: CompileContextInput) => {
    const cliArgs = buildCompileContextArgs(input);
    try {
      const { stdout, stderr } = await engine.run(cliArgs);
      return {
        content: [{ type: "text" as const, text: stdout }],
        _meta: withWarnings(
          { "anthropic/maxResultSizeChars": MAX_RESULT_SIZE_CHARS },
          stderr,
        ),
      };
    } catch (err) {
      if (err instanceof RustCliError) {
        return {
          content: [{ type: "text" as const, text: formatCliErrorText(err) }],
          isError: true,
        };
      }
      throw err;
    }
  };
}

export function registerCompileContext(server: McpServer, engine: RustEngine): void {
  server.tool(
    "compile_context",
    "ファイルパスから参照すべき上流成果物・ガイドラインを解決する。コードの作成・編集前に呼び出すこと。",
    compileContextSchema,
    makeCompileContextHandler(engine),
  );
}
