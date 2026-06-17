// ts-mcp MCP server tests (Vitest). Parent chain: SPEC-LGX-009 → DD-LGX-002/008 → TS-LGX-002.
// Canonical graph nodes: SRC-LGX-002 / SRC-LGX-008（ts-mcp は TypeScript 半）。
//
// 構成: 振る舞いは handler ファクトリ + zod schema を直接呼んで検証（SDK 内部に依存しない）。
// MCP-INV-1（3 ツール厳守）は createServer().toolNames + server._registeredTools で検証。
// legixy 新規: _meta["legixy/warnings"]（REQ.03）, タイムアウト（REQ.16）, バイナリ解決（ADR-016）。

import { describe, test, expect, vi } from "vitest";
import { createServer } from "../src/server.js";
import {
  RustEngine,
  RustCliError,
  resolveTimeoutSec,
} from "../src/engine.js";
import type { RunResult } from "../src/types.js";
import { parseArgs, resolveEngineBinary } from "../src/index.js";
import { z } from "zod";
import {
  makeCompileContextHandler,
  compileContextSchema,
  buildCompileContextArgs,
} from "../src/tools/compile-context.js";
import {
  makeObserveHandler,
  observeSchema,
  parseObserveStdout,
  buildObserveArgs,
} from "../src/tools/observe.js";
import {
  makeGetCompileAuditHandler,
  getCompileAuditSchema,
  formatAuditEntry,
} from "../src/tools/get-compile-audit.js";
import type { ContextLogEntry } from "../src/types.js";

// --- test plumbing -----------------------------------------------------------

type Producer = (args: string[]) => RunResult | Promise<RunResult>;

class MockEngine extends RustEngine {
  readonly calls: string[][] = [];
  constructor(private readonly producer: Producer) {
    super("mock-binary", "/mock/project-root");
  }
  override async run(args: string[]): Promise<RunResult> {
    this.calls.push([...args]);
    return this.producer(args);
  }
}

class ThrowingEngine extends RustEngine {
  readonly calls: string[][] = [];
  constructor(private readonly error: Error) {
    super("mock-binary", "/mock/project-root");
  }
  override async run(args: string[]): Promise<RunResult> {
    this.calls.push([...args]);
    throw this.error;
  }
}

const out = (stdout: string, stderr = ""): RunResult => ({ stdout, stderr });

interface CallToolResult {
  content: Array<{ type: "text"; text: string }>;
  _meta?: Record<string, unknown>;
  isError?: boolean;
}

// --- compile_context ---------------------------------------------------------

describe("compile_context", () => {
  test("normal path forwards stdout verbatim and attaches _meta 500000", async () => {
    const stdout =
      "## Targets\n- src/foo.rs -> DD-LGX-001\n\n## Upstream Artifacts\n- [0] SPEC-LGX-003 (SPEC)\n";
    const engine = new MockEngine(() => out(stdout));
    const handler = makeCompileContextHandler(engine);

    const result = (await handler({
      target_files: ["src/foo.rs"],
      command: "implement",
      granularity: "subnode",
    })) as CallToolResult;

    expect(engine.calls).toEqual([
      ["context", "src/foo.rs", "--command", "implement", "--granularity", "subnode"],
    ]);
    expect(result.content[0]).toEqual({ type: "text", text: stdout });
    expect(result._meta).toEqual({ "anthropic/maxResultSizeChars": 500000 });
    expect(result.isError).toBeUndefined();
  });

  test("RustCliError maps to isError result (REQ.07 format)", async () => {
    const engine = new ThrowingEngine(new RustCliError(1, "file not found: x.rs"));
    const handler = makeCompileContextHandler(engine);
    const result = (await handler({ target_files: ["x.rs"] })) as CallToolResult;

    expect(result.isError).toBe(true);
    expect(result.content[0].text).toMatch(
      /Rust CLI failed \(exit 1\): file not found: x\.rs/,
    );
    expect(result._meta).toBeUndefined(); // REQ.13: error 応答に _meta 非付与
  });

  test("Block B args (outline_only/sections/depth) convert snake→kebab (MCP-INV-2)", () => {
    expect(
      buildCompileContextArgs({
        target_files: ["a.md", "b.md"],
        outline_only: true,
        sections: "DD-X-001#abc,DD-X-001#def",
        depth: 2,
      }),
    ).toEqual([
      "context",
      "a.md",
      "b.md",
      "--outline-only",
      "--sections",
      "DD-X-001#abc,DD-X-001#def",
      "--depth",
      "2",
    ]);
    // outline_only=false は付与しない
    expect(buildCompileContextArgs({ target_files: ["a.md"], outline_only: false })).toEqual([
      "context",
      "a.md",
    ]);
  });

  test("body is byte-identical incl. cache-breakpoint marker (MCP-INV-2 / CACHE-INV)", async () => {
    const marker = "<!-- cache-breakpoint: stable-end -->";
    const stdout =
      "## Additional Guidelines\n- extra.md\n\n" + marker + "\n\n## Upstream Artifacts\n- [0] SPEC-LGX-003\n";
    const engine = new MockEngine(() => out(stdout));
    const result = (await makeCompileContextHandler(engine)({
      target_files: ["src/foo.rs"],
    })) as CallToolResult;
    expect(result.content[0].text).toBe(stdout);
    expect(result.content[0].text.split(marker).length - 1).toBe(1);
    expect(Object.keys(result._meta!)).toEqual(["anthropic/maxResultSizeChars"]);
  });
});

// --- observe -----------------------------------------------------------------

describe("observe", () => {
  test("new observation -> Observation #N, no _meta when stderr empty", async () => {
    const engine = new MockEngine(() => out("observation: id=42 skipped=false\n"));
    const result = (await makeObserveHandler(engine)({
      category: "compile_miss",
      message: "test message",
      related_ids: ["DD-LGX-001"],
    })) as CallToolResult;

    expect(engine.calls).toEqual([
      ["observe", "compile_miss", "test message", "--related-id", "DD-LGX-001"],
    ]);
    expect(result.content[0].text).toMatch(/Observation #42/);
    expect(result.content[0].text).toMatch(/category: compile_miss/);
    expect(result._meta).toBeUndefined(); // observe は maxResultSizeChars 非適用
  });

  test("duplicate observation -> already recorded message (MCP-INV-3 delegated)", async () => {
    const engine = new MockEngine(() => out("observation: id=42 skipped=true\n"));
    const result = (await makeObserveHandler(engine)({
      category: "review_correction",
      message: "dup",
    })) as CallToolResult;
    expect(result.content[0].text).toMatch(/observation_id=42/);
  });

  test("target_files/missing_doc/source_glob forwarded verbatim", () => {
    expect(
      buildObserveArgs({
        category: "compile_miss",
        message: "lacks guideline",
        target_files: ["a.md", "b.md"],
        missing_doc: "DD-LGX-001",
        source_glob: "**/*.rs",
      }),
    ).toEqual([
      "observe",
      "compile_miss",
      "lacks guideline",
      "--target-file",
      "a.md",
      "--target-file",
      "b.md",
      "--missing-doc",
      "DD-LGX-001",
      "--source-glob",
      "**/*.rs",
    ]);
  });

  test("parseObserveStdout pins the frozen stdout format (LGX-COMPAT-001 §4.1)", () => {
    expect(parseObserveStdout("observation: id=7 skipped=false")).toEqual({
      id: 7,
      skipped: false,
    });
    expect(parseObserveStdout("observation: id=7 skipped=true\n")).toEqual({
      id: 7,
      skipped: true,
    });
    expect(() => parseObserveStdout("garbage")).toThrow();
  });
});

// --- get_compile_audit -------------------------------------------------------

function entry(id: number, files: string[], command: string | null, at: string): ContextLogEntry {
  return {
    id,
    target_id: files[0] ?? "(none)",
    granularity: "document",
    payload: JSON.stringify({ command, target_files: files }),
    created_at: at,
  };
}

describe("get_compile_audit", () => {
  test("JSON array rendered to Markdown in id-desc order, _meta attached", async () => {
    const entries = [
      entry(3, ["a.md"], "implement", "2026-06-14T10:00:00"),
      entry(2, ["b.md"], null, "2026-06-14T09:00:00"),
      entry(1, ["c.md"], "test", "2026-06-14T08:00:00"),
    ];
    const engine = new MockEngine(() => out(JSON.stringify(entries)));
    const result = (await makeGetCompileAuditHandler(engine)({ limit: 3 })) as CallToolResult;

    expect(engine.calls).toEqual([["audit", "--limit", "3"]]);
    expect(result._meta).toEqual({ "anthropic/maxResultSizeChars": 500000 });
    const text = result.content[0].text;
    expect(text).toContain("### #3 (2026-06-14T10:00:00)");
    expect(text).toContain("- Target: a.md");
    expect(text).toContain("- Command: implement");
    expect(text).toContain("- Command: (none)"); // null payload.command は (none)
    expect(text.indexOf("### #3")).toBeLessThan(text.indexOf("### #1"));
    expect(formatAuditEntry(entries[0])).toBe(
      "### #3 (2026-06-14T10:00:00)\n- Target: a.md\n- Granularity: document\n- Files: a.md\n- Command: implement",
    );
  });

  test("empty table -> explanation message, no --limit when unset", async () => {
    const engine = new MockEngine(() => out("[]\n"));
    const result = (await makeGetCompileAuditHandler(engine)({})) as CallToolResult;
    expect(engine.calls).toEqual([["audit"]]);
    expect(result.content[0].text).toBe("監査ログはありません。");
    expect(result._meta).toEqual({ "anthropic/maxResultSizeChars": 500000 });
  });

  test("invalid JSON -> isError", async () => {
    const engine = new MockEngine(() => out("not json"));
    const result = (await makeGetCompileAuditHandler(engine)({})) as CallToolResult;
    expect(result.isError).toBe(true);
    expect(result.content[0].text).toMatch(/invalid JSON/);
  });
});

// --- zod schema bounds (TS-LGX-002 ケース19 ほか) -----------------------------

describe("zod schema contracts", () => {
  const cc = z.object(compileContextSchema);
  const ga = z.object(getCompileAuditSchema);
  const ob = z.object(observeSchema);

  test("compile_context: granularity enum rejects 'all'", () => {
    expect(cc.safeParse({ target_files: ["a"], granularity: "document" }).success).toBe(true);
    expect(cc.safeParse({ target_files: ["a"], granularity: "subnode" }).success).toBe(true);
    expect(cc.safeParse({ target_files: ["a"], granularity: "all" }).success).toBe(false);
  });

  test("TS-LGX-002 ケース19: depth:0 / 空 target_files / 空 sections は zod reject", () => {
    expect(cc.safeParse({ target_files: ["a"], depth: 0 }).success).toBe(false);
    expect(cc.safeParse({ target_files: [] }).success).toBe(false);
    expect(cc.safeParse({ target_files: ["a"], sections: "" }).success).toBe(false);
    // depth は整数のみ
    expect(cc.safeParse({ target_files: ["a"], depth: 1.5 }).success).toBe(false);
    expect(cc.safeParse({ target_files: ["a"], depth: 1 }).success).toBe(true);
  });

  test("get_compile_audit: limit 1..=50 enforced", () => {
    expect(ga.safeParse({ limit: 1 }).success).toBe(true);
    expect(ga.safeParse({ limit: 50 }).success).toBe(true);
    expect(ga.safeParse({ limit: 0 }).success).toBe(false);
    expect(ga.safeParse({ limit: 51 }).success).toBe(false);
    expect(ga.safeParse({}).success).toBe(true);
  });

  test("observe: category enum + non-empty message", () => {
    expect(ob.safeParse({ category: "manual_note", message: "x" }).success).toBe(true);
    expect(ob.safeParse({ category: "bogus", message: "x" }).success).toBe(false);
    expect(ob.safeParse({ category: "manual_note", message: "" }).success).toBe(false);
  });
});

// --- legixy/warnings (SPEC-LGX-009 REQ.03/13, legixy 新規) --------------------

describe("_meta['legixy/warnings'] forwarding (exit 0 + non-empty stderr)", () => {
  const warn = "[legixy-ctx] audit log write failed: disk full\n";

  test("compile_context: stderr non-empty -> warnings present alongside maxResultSizeChars", async () => {
    const engine = new MockEngine(() => out("## ok\n", warn));
    const result = (await makeCompileContextHandler(engine)({
      target_files: ["a.rs"],
    })) as CallToolResult;
    expect(result._meta).toEqual({
      "anthropic/maxResultSizeChars": 500000,
      "legixy/warnings": warn,
    });
  });

  test("compile_context: stderr empty -> warnings field omitted", async () => {
    const engine = new MockEngine(() => out("## ok\n", "   \n"));
    const result = (await makeCompileContextHandler(engine)({
      target_files: ["a.rs"],
    })) as CallToolResult;
    expect(Object.keys(result._meta!)).toEqual(["anthropic/maxResultSizeChars"]);
  });

  test("observe: stderr non-empty -> warnings present, NO maxResultSizeChars (REQ.13)", async () => {
    const engine = new MockEngine(() => out("observation: id=1 skipped=false\n", warn));
    const result = (await makeObserveHandler(engine)({
      category: "manual_note",
      message: "m",
    })) as CallToolResult;
    expect(result._meta).toEqual({ "legixy/warnings": warn });
  });

  test("get_compile_audit: stderr non-empty -> warnings present", async () => {
    const engine = new MockEngine(() => out("[]\n", warn));
    const result = (await makeGetCompileAuditHandler(engine)({})) as CallToolResult;
    expect(result._meta).toEqual({
      "anthropic/maxResultSizeChars": 500000,
      "legixy/warnings": warn,
    });
  });
});

// --- timeout (SPEC-LGX-009 REQ.16, legixy 新規) -------------------------------

describe("CLI timeout (LGX_MCP_TIMEOUT_SEC)", () => {
  test("timed-out RustCliError -> 'Rust CLI failed (timeout after Ns):' (no partial output)", async () => {
    const engine = new ThrowingEngine(new RustCliError(-1, "timeout after 30s", true, 30));
    const result = (await makeCompileContextHandler(engine)({
      target_files: ["a.rs"],
    })) as CallToolResult;
    expect(result.isError).toBe(true);
    expect(result.content[0].text).toBe("Rust CLI failed (timeout after 30s):");
  });

  test("resolveTimeoutSec: default 30 / 0 disabled / invalid -> 30", () => {
    expect(resolveTimeoutSec(undefined)).toBe(30);
    expect(resolveTimeoutSec("")).toBe(30);
    expect(resolveTimeoutSec("60")).toBe(60);
    expect(resolveTimeoutSec("0")).toBe(0);
    expect(resolveTimeoutSec("abc")).toBe(30);
    expect(resolveTimeoutSec("-5")).toBe(30);
    expect(resolveTimeoutSec("1.5")).toBe(30);
  });
});

// --- binary resolution (ADR-LGX-016) -----------------------------------------

describe("engine binary resolution (ADR-LGX-016)", () => {
  test("precedence: --engine-binary > LGX_BIN > TRACEABILITY_ENGINE_BIN > default 'legixy'", () => {
    const noop = () => {};
    expect(resolveEngineBinary("flagbin", { LGX_BIN: "lgx", TRACEABILITY_ENGINE_BIN: "te" }, noop)).toBe(
      "flagbin",
    );
    expect(resolveEngineBinary(undefined, { LGX_BIN: "lgx", TRACEABILITY_ENGINE_BIN: "te" }, noop)).toBe(
      "lgx",
    );
    expect(resolveEngineBinary(undefined, { TRACEABILITY_ENGINE_BIN: "te" }, noop)).toBe("te");
    expect(resolveEngineBinary(undefined, {}, noop)).toBe("legixy");
  });

  test("old name TRACEABILITY_ENGINE_BIN emits stderr Info (without printing the value)", () => {
    const warn = vi.fn();
    resolveEngineBinary(undefined, { TRACEABILITY_ENGINE_BIN: "/secret/path/te" }, warn);
    expect(warn).toHaveBeenCalledOnce();
    const msg = warn.mock.calls[0][0] as string;
    expect(msg).toMatch(/TRACEABILITY_ENGINE_BIN/);
    expect(msg).not.toContain("/secret/path/te"); // REQ.03: 値はログに出さない
  });

  test("LGX_BIN does not emit Info", () => {
    const warn = vi.fn();
    resolveEngineBinary(undefined, { LGX_BIN: "lgx" }, warn);
    expect(warn).not.toHaveBeenCalled();
  });

  test("parseArgs resolves --project-root and --engine-binary", () => {
    const noop = () => {};
    const parsed = parseArgs(["--project-root", "/p", "--engine-binary", "mybin"], {}, noop);
    expect(parsed.engineBinary).toBe("mybin");
    expect(parsed.projectRoot.endsWith("/p")).toBe(true);
  });
});

// --- MCP-INV-1 / createServer ------------------------------------------------

describe("MCP-INV-1: exactly 3 Agent-Surface tools", () => {
  test("createServer registers exactly compile_context/observe/get_compile_audit", () => {
    const engine = new MockEngine(() => out(""));
    const { server, toolNames } = createServer({
      projectRoot: "/mock",
      engineBinary: "mock",
      engineOverride: engine,
    });

    expect([...toolNames].sort()).toEqual(
      ["compile_context", "get_compile_audit", "observe"].sort(),
    );

    // 実レジストリ側でも 3 ツールのみ（Admin Surface 非公開）を確認。
    const registered = (server as unknown as {
      _registeredTools: Record<string, unknown>;
    })._registeredTools;
    const names = Object.keys(registered).sort();
    expect(names).toEqual(["compile_context", "get_compile_audit", "observe"].sort());
    for (const forbidden of ["approve", "reject", "analyze", "check", "embed", "init", "migrate", "proposals"]) {
      expect(names).not.toContain(forbidden);
    }
  });
});
