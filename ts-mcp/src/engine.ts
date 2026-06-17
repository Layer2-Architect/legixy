// ts-mcp transport engine (SRC-LGX-002-TS label; parent DD-LGX-002 §4 / SPEC-LGX-009 REQ.05/07/16).
// Canonical graph node: SRC-LGX-002 (Rust+TS 統合 SRC).
//
// RustEngine: thin spawn wrapper around the legixy Rust CLI.
// - Per-call, short-lived child process (SPEC-LGX-009 REQ.05, STATE-INV-1)
// - Faithful transport: returns stdout AND stderr verbatim (MCP-INV-2)
//   so the tool layer can attach `_meta["legixy/warnings"]` on exit 0 (REQ.03/13).
// - Non-zero exit carries exit code + stderr (REQ.07).
// - Configurable timeout via LGX_MCP_TIMEOUT_SEC (default 30s, 0 = disabled; REQ.16).

import { execFile } from "node:child_process";
import { promisify } from "node:util";
import type { RunResult } from "./types.js";

const execFileAsync = promisify(execFile);

const MAX_STDOUT_BYTES = 10 * 1024 * 1024;
const DEFAULT_TIMEOUT_SEC = 30;

/**
 * Resolve the CLI subprocess timeout in seconds from LGX_MCP_TIMEOUT_SEC.
 * - unset / invalid → DEFAULT_TIMEOUT_SEC (30)
 * - 0 → disabled (no timeout, v3-compatible behaviour)
 * (SPEC-LGX-009 REQ.16, 人間裁定 2026-06-10【v3 差分】)
 */
export function resolveTimeoutSec(
  raw: string | undefined = process.env.LGX_MCP_TIMEOUT_SEC,
): number {
  if (raw === undefined || raw.trim() === "") return DEFAULT_TIMEOUT_SEC;
  const n = Number(raw);
  if (!Number.isFinite(n) || n < 0 || !Number.isInteger(n)) {
    return DEFAULT_TIMEOUT_SEC;
  }
  return n;
}

export class RustEngine {
  private readonly timeoutSec: number;

  constructor(
    private readonly binPath: string,
    private readonly projectRoot: string,
    timeoutSec: number = resolveTimeoutSec(),
  ) {
    this.timeoutSec = timeoutSec;
  }

  /**
   * Invoke the Rust CLI with the given argv tail, passing `--project-root`
   * implicitly (LGX-COMPAT-001 §5). Returns raw stdout and stderr so callers can
   * forward stdout unmodified (MCP-INV-2) and surface stderr Warnings (REQ.03).
   */
  async run(args: string[]): Promise<RunResult> {
    try {
      const { stdout, stderr } = await execFileAsync(
        this.binPath,
        ["--project-root", this.projectRoot, ...args],
        {
          maxBuffer: MAX_STDOUT_BYTES,
          // Node: timeout 0 = no timeout.
          timeout: this.timeoutSec === 0 ? 0 : this.timeoutSec * 1000,
          encoding: "utf8",
        },
      );
      return { stdout, stderr };
    } catch (err) {
      throw toRustCliError(err, this.timeoutSec);
    }
  }
}

export class RustCliError extends Error {
  constructor(
    public readonly exitCode: number,
    message: string,
    /** True when the child was killed by the timeout (REQ.16). */
    public readonly timedOut: boolean = false,
    /** Timeout budget in seconds (only meaningful when timedOut). */
    public readonly timeoutSec: number = 0,
  ) {
    super(message);
    this.name = "RustCliError";
  }
}

function toRustCliError(err: unknown, timeoutSec: number): RustCliError {
  // execFile's rejection shape (Node): { code, signal, killed, stdout, stderr, cmd, ... }
  // - `code` is a number for normal non-zero exit, or a string code like "ENOENT" /
  //   "ETIMEDOUT" when the process itself couldn't run or was killed.
  const anyErr = err as {
    code?: number | string;
    signal?: string;
    killed?: boolean;
    stderr?: string | Buffer;
    stdout?: string | Buffer;
    message?: string;
  };

  // Timeout: execFile kills the child (killed === true) when the timeout elapses.
  // Per REQ.16, partial stdout/stderr is NOT forwarded (all-or-nothing).
  if (anyErr?.killed === true) {
    return new RustCliError(
      -1,
      `timeout after ${timeoutSec}s`,
      true,
      timeoutSec,
    );
  }

  const rawCode = anyErr?.code;
  // Binary resolution failure (ENOENT) / other spawn failures → exit -1 (REQ.08).
  const exitCode = typeof rawCode === "number" ? rawCode : -1;

  const stderr = decodeStream(anyErr?.stderr);
  const stdout = decodeStream(anyErr?.stdout);

  // Prefer stderr (Rust anyhow errors print there), fall back to stdout then Node's message.
  const fallback =
    typeof rawCode === "string"
      ? `${rawCode}: ${anyErr?.message ?? ""}`
      : anyErr?.message ?? "unknown error";
  const message =
    stderr.trim() || stdout.trim() || fallback.trim() || "unknown error";

  return new RustCliError(exitCode, message);
}

function decodeStream(value: string | Buffer | undefined): string {
  if (value === undefined) return "";
  return typeof value === "string" ? value : value.toString("utf8");
}
