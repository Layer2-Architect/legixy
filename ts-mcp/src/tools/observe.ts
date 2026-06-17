// ts-mcp tool: observe (SRC-LGX-008-TS label; parent DD-LGX-008 §3.2 / SPEC-LGX-009 REQ.03).
// Canonical graph node: SRC-LGX-008.
//
// Forward an observation to `legixy observe <category> <message> ...`.
// Duplicate detection is owned by legixy-feedback (MCP-INV-3); this wrapper reflects the
// `skipped` flag from CLI stdout. observe gets _meta["legixy/warnings"] on exit 0 + non-empty
// stderr (REQ.03), but NOT _meta["anthropic/maxResultSizeChars"] (REQ.13 適用対象外).

import type { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js";
import { z } from "zod";
import { RustCliError, type RustEngine } from "../engine.js";
import { formatCliErrorText, withWarnings } from "../meta.js";
import type { ObserveStdoutParsed } from "../types.js";

/**
 * Parse the single-line stdout emitted by `legixy observe` (LGX-COMPAT-001 §4.1 凍結形式):
 *   "observation: id=<N> skipped=<true|false>"
 */
export function parseObserveStdout(stdout: string): ObserveStdoutParsed {
  const match = /^observation:\s*id=(\d+)\s+skipped=(true|false)/.exec(stdout.trim());
  if (!match) {
    throw new Error(`Unexpected observe stdout: ${stdout}`);
  }
  return {
    id: Number(match[1]),
    skipped: match[2] === "true",
  };
}

export const observeSchema = {
  category: z
    .enum(["compile_miss", "review_correction", "manual_note"])
    .describe("報告カテゴリ"),
  message: z.string().min(1).describe("報告内容"),
  related_ids: z.array(z.string()).optional().describe("関連する成果物 ID"),
  target_files: z.array(z.string()).optional().describe("関連するファイルパス"),
  missing_doc: z.string().optional().describe("不足していたドキュメントのパス"),
  source_glob: z.string().optional().describe("カスタムエッジの source_glob を明示指定"),
};

type ObserveInput = {
  category: "compile_miss" | "review_correction" | "manual_note";
  message: string;
  related_ids?: string[];
  target_files?: string[];
  missing_doc?: string;
  source_glob?: string;
};

/** MCP 入力 → CLI argv（位置引数 category/message + 繰り返しフラグ、MCP-INV-2）。 */
export function buildObserveArgs(input: ObserveInput): string[] {
  const args: string[] = ["observe", input.category, input.message];
  if (input.related_ids) {
    for (const id of input.related_ids) args.push("--related-id", id);
  }
  if (input.target_files) {
    for (const p of input.target_files) args.push("--target-file", p);
  }
  if (input.missing_doc !== undefined) args.push("--missing-doc", input.missing_doc);
  if (input.source_glob !== undefined) args.push("--source-glob", input.source_glob);
  return args;
}

export function makeObserveHandler(engine: RustEngine) {
  return async (input: ObserveInput) => {
    const cliArgs = buildObserveArgs(input);
    try {
      const { stdout, stderr } = await engine.run(cliArgs);
      const parsed = parseObserveStdout(stdout);
      const text = parsed.skipped
        ? `既に記録済み（observation_id=${parsed.id}）`
        : `Observation #${parsed.id} を記録（category: ${input.category}）`;
      const meta = withWarnings({}, stderr);
      const res: {
        content: Array<{ type: "text"; text: string }>;
        _meta?: Record<string, unknown>;
      } = { content: [{ type: "text", text }] };
      // observe は maxResultSizeChars 非適用。warnings のみ、存在する場合だけ付与（REQ.13）。
      if (Object.keys(meta).length > 0) res._meta = meta;
      return res;
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

export function registerObserve(server: McpServer, engine: RustEngine): void {
  server.tool(
    "observe",
    "ガイドライン不足やレビュー修正を報告する。コード作成後のセルフレビューで不足を発見したら呼び出すこと。",
    observeSchema,
    makeObserveHandler(engine),
  );
}
