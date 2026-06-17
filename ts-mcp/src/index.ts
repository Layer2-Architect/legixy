// Document ID: SRC-MCP-001
// ts-mcp entrypoint (配送軸 area=MCP、DevProc_V4.1 §12。CTR-MCP-001 系。parent DD-LGX-002 §4 / SPEC-LGX-009 REQ.08).
// Canonical graph nodes: SRC-LGX-002 / SRC-LGX-008.
//
// Parses --project-root / --engine-binary, resolves the engine binary (ADR-LGX-016),
// wires up StdioServerTransport, installs graceful shutdown handlers.

import { StdioServerTransport } from "@modelcontextprotocol/sdk/server/stdio.js";
import * as path from "node:path";
import { fileURLToPath } from "node:url";
import { createServer } from "./server.js";

export interface ParsedArgs {
  projectRoot: string;
  engineBinary: string;
}

/**
 * Resolve the Rust CLI binary path/name (ADR-LGX-016 §23, SPEC-LGX-009 REQ.08).
 * 解決順: `--engine-binary` フラグ > `LGX_BIN`（正準）> `TRACEABILITY_ENGINE_BIN`（旧名・stderr Info 案内）
 *        > 既定名 `legixy`。
 * 旧名使用時の Info は新名を案内するが、環境変数の値は出力しない（REQ.03）。
 * バイナリ存在確認は行わない（fail-fast せず、ツール呼出時に REQ.08 の isError 応答で診断）。
 */
export function resolveEngineBinary(
  flagValue: string | undefined,
  env: NodeJS.ProcessEnv = process.env,
  warn: (msg: string) => void = (m) => console.error(m),
): string {
  if (flagValue) return flagValue;
  if (env.LGX_BIN) return env.LGX_BIN;
  if (env.TRACEABILITY_ENGINE_BIN) {
    warn(
      "[legixy-mcp] Info: 環境変数 TRACEABILITY_ENGINE_BIN は旧名です。正準名 LGX_BIN への移行を推奨します。",
    );
    return env.TRACEABILITY_ENGINE_BIN;
  }
  return "legixy";
}

export function parseArgs(
  argv: string[],
  env: NodeJS.ProcessEnv = process.env,
  warn: (msg: string) => void = (m) => console.error(m),
): ParsedArgs {
  let projectRoot = ".";
  let flagBinary: string | undefined;

  for (let i = 0; i < argv.length; i++) {
    const flag = argv[i];
    const value = argv[i + 1];
    if (flag === "--project-root" && value) {
      projectRoot = value;
      i++;
    } else if (flag === "--engine-binary" && value) {
      flagBinary = value;
      i++;
    }
  }

  return {
    projectRoot: path.resolve(projectRoot),
    engineBinary: resolveEngineBinary(flagBinary, env, warn),
  };
}

export function main(): void {
  const { projectRoot, engineBinary } = parseArgs(process.argv.slice(2));

  const { server, cleanup } = createServer({ projectRoot, engineBinary });
  const transport = new StdioServerTransport();

  const shutdown = (): void => {
    cleanup();
    process.exit(0);
  };
  process.on("SIGTERM", shutdown);
  process.on("SIGINT", shutdown);

  server.connect(transport).catch((err: unknown) => {
    console.error("MCP server failed to start:", err);
    cleanup();
    process.exit(1);
  });
}

// ESM entrypoint guard: run main() only when executed directly (not when imported by tests).
const invoked = process.argv[1];
if (invoked && fileURLToPath(import.meta.url) === path.resolve(invoked)) {
  main();
}
