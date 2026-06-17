// ts-mcp server (parent DD-LGX-002 §4 / DD-LGX-008 §4.2 / SPEC-LGX-009 REQ.02).
// Canonical graph nodes: SRC-LGX-002 / SRC-LGX-008.
//
// createServer(): build the McpServer and register exactly 3 tools (MCP-INV-1, REQ.02).
// DI: engineOverride lets tests inject a fake RustEngine without spawning the real CLI.

import { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js";
import { RustEngine } from "./engine.js";
import { registerCompileContext } from "./tools/compile-context.js";
import { registerObserve } from "./tools/observe.js";
import { registerGetCompileAudit } from "./tools/get-compile-audit.js";

export interface ServerOptions {
  projectRoot: string;
  engineBinary: string;
  /** Test-only hook: replace the default RustEngine with a mock. */
  engineOverride?: RustEngine;
}

export interface CreateServerResult {
  server: McpServer;
  /** Tool names registered on this server (MCP-INV-1 検証用)。 */
  toolNames: string[];
  /** Called on SIGTERM / SIGINT. legixy MCP holds no persistent state, so this is a no-op. */
  cleanup: () => void;
}

export function createServer(options: ServerOptions): CreateServerResult {
  const server = new McpServer({
    name: "legixy-mcp",
    version: "0.4.0-alpha4",
  });

  const engine =
    options.engineOverride ??
    new RustEngine(options.engineBinary, options.projectRoot);

  // MCP-INV-1: exactly these 3 tools, nothing else (no Admin Surface).
  registerCompileContext(server, engine);
  registerObserve(server, engine);
  registerGetCompileAudit(server, engine);

  const cleanup = (): void => {
    // legixy MCP is stateless (STATE-INV-1); no DB handles or caches to release.
  };

  return {
    server,
    toolNames: ["compile_context", "observe", "get_compile_audit"],
    cleanup,
  };
}
