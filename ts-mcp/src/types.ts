// ts-mcp transport types (SRC-LGX-008-TS label; parent DD-LGX-008 §2.4 / DD-LGX-002 §4).
// Canonical graph nodes: SRC-LGX-002 / SRC-LGX-008 (Rust+TS 統合 SRC; ts-mcp は TypeScript 半).
// Shared TypeScript types. Mirror the Rust CLI JSON outputs consumed by ts-mcp.

/**
 * Output of `legixy audit [--limit N]` — a JSON array of ContextLogEntry.
 * Mirrors `legixy-feedback::audit::ContextLogEntry` / `context_log` テーブル（DD-LGX-008 schema）。
 *
 * `payload` は `legixy context`（legixy-ctx AuditLogger）が書いた JSON 文字列
 * （`{command, target_files, resolved_targets, upstream_count, unresolved}`）。downstream は
 * 必要時にパースする（doubly-serialized）。
 */
export interface ContextLogEntry {
  id: number;
  target_id: string;
  granularity: string | null;
  payload: string;
  created_at: string;
}

/**
 * Parsed form of the single-line `observe` stdout (LGX-COMPAT-001 §4.1 凍結形式):
 *   "observation: id=<N> skipped=<true|false>"
 */
export interface ObserveStdoutParsed {
  id: number;
  skipped: boolean;
}

/**
 * Result of a Rust CLI invocation. Both streams are captured so the MCP layer can
 * forward stderr Warning lines via `_meta["legixy/warnings"]` on exit 0
 * (SPEC-LGX-009 REQ.03 / REQ.13, ADR-LGX-004 可観測性保証).
 */
export interface RunResult {
  stdout: string;
  stderr: string;
}
