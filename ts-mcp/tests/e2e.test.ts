// Document ID: TC-MCP-001
// 配送軸 TC[DLV]（area=MCP。親 TS-MCP-001 / 契約 CTR-MCP-001、DevProc_V4.1 §12）。
// MCP E2E テスト: 実 legixy バイナリを spawn し、3 ツール（compile_context / observe /
// get_compile_audit）を RustEngine + handler 経由で駆動する。mock を使わない真の end-to-end。
//
// 前提: legixy バイナリがビルド済み（../target/{release,debug}/legixy か LEGIXY_BIN）。
// 不在ならスキップ（CI で cargo build 未実施でも grün）。

import { describe, test, expect, beforeAll } from "vitest";
import * as fs from "node:fs";
import * as os from "node:os";
import * as path from "node:path";
import { RustEngine } from "../src/engine.js";
import { makeCompileContextHandler } from "../src/tools/compile-context.js";
import { makeObserveHandler } from "../src/tools/observe.js";
import { makeGetCompileAuditHandler } from "../src/tools/get-compile-audit.js";

interface CallToolResult {
  content: Array<{ type: "text"; text: string }>;
  _meta?: Record<string, unknown>;
  isError?: boolean;
}

function findBin(): string | null {
  const env = process.env.LEGIXY_BIN;
  if (env && fs.existsSync(env)) return env;
  for (const rel of ["../target/release/legixy", "../target/debug/legixy"]) {
    const p = path.resolve(process.cwd(), rel);
    if (fs.existsSync(p)) return p;
  }
  return null;
}

const BIN = findBin();

const GRAPH = `
[[nodes]]
id = "UC-LGX-001"
type = "UC"
path = "uc.md"

[[nodes]]
id = "SPEC-LGX-001"
type = "SPEC"
path = "spec.md"

[[edges]]
from = "SPEC-LGX-001"
to = "UC-LGX-001"
kind = "chain"
`;

describe.skipIf(!BIN)("MCP E2E (real legixy binary)", () => {
  let projectRoot: string;
  let engine: RustEngine;

  beforeAll(() => {
    projectRoot = fs.mkdtempSync(path.join(os.tmpdir(), "legixy-e2e-"));
    const gdir = path.join(projectRoot, "docs/traceability");
    fs.mkdirSync(gdir, { recursive: true });
    fs.writeFileSync(path.join(gdir, "graph.toml"), GRAPH);
    fs.writeFileSync(path.join(projectRoot, "uc.md"), "# UC body\n");
    fs.writeFileSync(path.join(projectRoot, "spec.md"), "# SPEC body\n");
    engine = new RustEngine(BIN as string, projectRoot);
  });

  test("compile_context spawns `legixy context` and returns markdown + _meta", async () => {
    const result = (await makeCompileContextHandler(engine)({
      target_files: ["uc.md"],
      command: "test",
    })) as CallToolResult;

    expect(result.isError).toBeUndefined();
    expect(result._meta?.["anthropic/maxResultSizeChars"]).toBe(500000);
    const text = result.content[0].text;
    // 6 セクション枠 + キャッシュブレーク点マーカ（REQ.10/REL.09）。
    expect(text).toContain("Upstream Artifacts");
    expect(text).toContain("cache-breakpoint: stable-end");
  });

  test("observe spawns `legixy observe` and records (frozen stdout parsed)", async () => {
    const result = (await makeObserveHandler(engine)({
      category: "manual_note",
      message: "E2E メモ",
      related_ids: ["UC-LGX-001"],
    })) as CallToolResult;

    expect(result.isError).toBeUndefined();
    expect(result.content[0].text).toMatch(/Observation #\d+/);
  });

  test("get_compile_audit reflects the prior compile_context call (context_log populated)", async () => {
    // 直前の compile_context が context_log を書き込んでいるため、監査に現れる。
    const result = (await makeGetCompileAuditHandler(engine)({})) as CallToolResult;

    expect(result.isError).toBeUndefined();
    expect(result._meta?.["anthropic/maxResultSizeChars"]).toBe(500000);
    const text = result.content[0].text;
    expect(text).toContain("Target: UC-LGX-001");
    expect(text).toContain("Files: uc.md");
    expect(text).toContain("Command: test");
  });

  test("error path: compile_context on missing graph forwards isError (exit 1)", async () => {
    const badRoot = fs.mkdtempSync(path.join(os.tmpdir(), "legixy-e2e-bad-"));
    const badEngine = new RustEngine(BIN as string, badRoot);
    const result = (await makeCompileContextHandler(badEngine)({
      target_files: ["x.md"],
    })) as CallToolResult;
    expect(result.isError).toBe(true);
    expect(result.content[0].text).toMatch(/Rust CLI failed \(exit 1\)/);
  });
});

if (!BIN) {
  // バイナリ不在を可視化（スキップ理由）。
  test("MCP E2E skipped: legixy binary not found (run cargo build)", () => {
    expect(true).toBe(true);
  });
}
