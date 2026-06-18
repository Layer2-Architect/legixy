# legixy — User Manual (English)

[日本語版 / Japanese](manual.ja.md)

> legixy is a directed-graph **traceability engine**: it keeps the links between your
> artifacts (SPEC → … → source) as machine-verifiable data and observes **semantic drift**
> between linked artifacts using a local, frozen embedding model.
>
> Version: 0.4.0-alpha4 · Binary: `legixy` (`legixy.exe` on Windows) · 19 subcommands + a 3-tool MCP server.

---

## Table of contents

1. [Concepts](#1-concepts)
2. [Installation](#2-installation)
3. [The embedding model (ONNX)](#3-the-embedding-model-onnx)
4. [Project setup](#4-project-setup)
5. [Configuration (`.legixy.toml`)](#5-configuration-legixytoml)
6. [Two-layer verification](#6-two-layer-verification)
7. [Command reference](#7-command-reference)
8. [Global options & exit codes](#8-global-options--exit-codes)
9. [JSON output](#9-json-output)
10. [MCP server (Claude Code integration)](#10-mcp-server-claude-code-integration)
11. [Environment variables](#11-environment-variables)
12. [Troubleshooting](#12-troubleshooting)
13. [License](#13-license)

---

## 1. Concepts

- **Graph as primary data.** Every artifact (spec, use case, design, test, source) is a node in
  `docs/traceability/graph.toml`. Edges are explicit, human-readable, Git-diffable. legixy never
  rewrites the graph; you edit it, and `check` detects inconsistencies after the fact.
- **Two layers.** The *formal* layer is deterministic (ID format, file existence, chain integrity,
  acyclicity) and needs no model. The *semantic* layer uses embeddings to observe meaning drift and
  link candidates — and only ever emits Warning/Info. **Deviation is reported, not judged.**
- **Frozen instrument.** The embedding model is held constant on purpose: if both the subject (your
  documents) and the instrument (the model) move, an observed deviation is no longer attributable.
- **Admin vs. Agent surface.** Of the 19 CLI commands, only 3 are exposed over MCP. `approve` /
  `reject` and the rest are CLI-only — i.e. human-only.

IDs follow `{type}-{area}-{seq}` (e.g. `SPEC-LGX-001`). Markdown artifacts bind by the ID prefix in
the filename; source files bind by a `// Document ID: SRC-…` comment near the top. Heading-level
**subnodes** (`DD-…#anchor`) enable section-granularity tracing.

---

## 2. Installation

### 2.1 Prebuilt binaries (recommended)

Prebuilt, onnx-enabled binaries for **Linux (x86_64)** and **Windows (x86_64)** are attached to each
GitHub Release. The install scripts download the binary **and** the embedding model (see §3), so the
semantic layer works out of the box.

**Linux / macOS:**

```bash
curl -fsSL https://raw.githubusercontent.com/Layer2-Architect/legixy/main/install.sh | bash -s -- --repo Layer2-Architect/legixy
# or, after cloning:
bash install.sh --repo Layer2-Architect/legixy [--version vX.Y.Z] [--prefix ~/.local] [--no-model]
```

**Windows (PowerShell):**

```powershell
$env:LEGIXY_REPO = "Layer2-Architect/legixy"
irm https://raw.githubusercontent.com/Layer2-Architect/legixy/main/install.ps1 | iex
# or, after cloning:
powershell -ExecutionPolicy Bypass -File install.ps1 -Repo Layer2-Architect/legixy [-Version vX.Y.Z] [-NoModel]
```

The installer places legixy under a prefix (`~/.local/share/legixy`, or `%LOCALAPPDATA%\legixy`),
puts the launcher on your PATH, and downloads the model. macOS has no prebuilt binary yet — build
from source (§2.2).

### 2.2 Build from source

Requires a recent Rust toolchain (edition 2021) and, for the MCP server, Node.js ≥ 20.

```bash
git clone https://github.com/Layer2-Architect/legixy
cd legixy

# Formal layer only (no model, no ONNX):
cargo build --release -p legixy-cli
./target/release/legixy --version          # → legixy 0.4.0-alpha4

# Full build with the semantic layer (links ONNX Runtime via the `ort` crate):
cargo build --release -p legixy-cli --features onnx
```

There is also a reproducible assembly script that bundles the CLI + MCP server (+ model when
`LEGIXY_ONNX=1`):

```bash
LEGIXY_ONNX=1 bash deploy/build-deploy.sh    # output under deploy/
```

---

## 3. The embedding model (ONNX)

The semantic layer needs a local model: **`paraphrase-multilingual-MiniLM-L12-v2`** (multilingual,
including Japanese; 384-dim, mean pooling). It consists of two files:

```
<model-dir>/paraphrase-multilingual-MiniLM-L12-v2/
├── model.onnx        (~500 MB)
└── tokenizer.json
```

The model is **not** shipped in the repository or the release archives (size + separate license).
You obtain it one of three ways:

1. **Install script** (does this automatically) — see §2.1.
2. **Fetch script:** `bash scripts/fetch-model.sh` → downloads into `models/…`.
3. **Manual:** download `tokenizer.json` (repo root) and `onnx/model.onnx` from
   <https://huggingface.co/sentence-transformers/paraphrase-multilingual-MiniLM-L12-v2>. If
   `onnx/model.onnx` is absent upstream, export it:
   ```bash
   pip install "optimum[onnxruntime]" sentence-transformers
   optimum-cli export onnx -m sentence-transformers/paraphrase-multilingual-MiniLM-L12-v2 \
     --task feature-extraction <model-dir>/paraphrase-multilingual-MiniLM-L12-v2
   ```

**How legixy finds the model** (first match wins):
`--models-dir <PATH>` → `LGX_MODELS_DIR` env → config `model_dir` → `<project-root>/models/paraphrase-multilingual-MiniLM-L12-v2`.

Then enable the semantic layer in config: `[semantic] enabled = true`.

---

## 4. Project setup

```bash
legixy init                       # generate .legixy.toml, docs/traceability/graph.toml,
                                  # .legixy/engine.db and the documentation directories
```

Register artifacts and links in `docs/traceability/graph.toml`:

```toml
[[nodes]]
id   = "SPEC-PR-001"
type = "SPEC"
path = "docs/specs/SPEC-PR-001_login.md"

[[nodes]]
id   = "UC-PR-001"
type = "UC"
path = "docs/usecases/UC-PR-001_login.md"

[[edges]]
from = "SPEC-PR-001"
to   = "UC-PR-001"
kind = "chain"
```

Then:

```bash
legixy check --formal             # deterministic checks (CI-friendly)
legixy embed --all                # generate embeddings (needs the model)
legixy check                      # formal + semantic
```

---

## 5. Configuration (`.legixy.toml`)

`init` generates `.legixy.toml`. The legacy name `.trace-engine.toml` is read as a fallback
(search order: `.legixy.toml` → `.trace-engine.toml`; if only the legacy name exists, it is read and
an info message suggests migrating). Schema (excerpt):

```toml
[project]
name = "my-project"

[id]
pattern = "{type}-{area}-{seq}"
area = "LGX"
seq_digits = 3

[id.chain]
order = ["UC", "RBA", "SEQA", "RBD", "SEQD", "DD", "TS", "TC", "SRC"]
independent = ["SPEC", "NFR", "VAL"]

[semantic]
enabled = true
model = "paraphrase-multilingual-MiniLM-L12-v2"
similarity_threshold   = 0.4
drift_threshold        = 0.3
link_candidate_threshold = 0.7

[freshness]
enabled = true
method = "mtime"
```

`[id.chain]` is what makes legixy methodology-agnostic — replace it with your own typecodes/order
(e.g. `["REQ","DES","IMPL","TEST"]`). Thresholds are project-specific; calibrate with
`legixy calibrate` rather than trusting defaults.

---

## 6. Two-layer verification

| Layer | Command | Categories | Needs model |
|---|---|---|---|
| **Formal** (deterministic) | `legixy check --formal` | ID format, file existence, chain integrity, acyclicity (DAG), orphan files, subnode ID format | No |
| **Semantic** (embedding) | `legixy check` | SemanticSimilarity (linked pair below threshold → Warning), LinkCandidate (unlinked pair above threshold → Info), Drift (content_hash mismatch → Warning) | Yes |

`check` exit code: **Error count > 0 → 1, otherwise 0** (the G1 gate). Semantic findings are
Warning/Info and do not by themselves fail the gate.

---

## 7. Command reference

Invocation: `legixy [GLOBAL OPTIONS] <subcommand> [ARGS]`. Positional args are `<angle>`; optional
flags are `[--bracketed]`.

| # | Command | Positional | Flags | Notes |
|---|---------|-----------|-------|-------|
| 1 | `init` | — | `[--force]` | Backs up an existing `.legixy.toml` to `.bak` and overwrites |
| 2 | `migrate` | — | `--from <PATH>` (required), `[--to <PATH>]`, `[--dry-run]`, `[--format markdown\|json]` | Migrate a v0.1.0 project. `--to` defaults to `--project-root` |
| 3 | `check` | — | `[--formal]` | Formal only with `--formal`; otherwise formal + semantic. Exit 1 if Error > 0 |
| 4 | `embed` | — | `[--all]`, `[--node <ID>]` (repeatable, exclusive with `--all`), `[--force]` | (Re)generate embeddings. JSON: `{generated, skipped, failed, errors[]}` |
| 5 | `drift` | `<artifact_id>` | `[--against snapshot:<LABEL>\|snapshot:<ID>]` | Without `--against`, compares to current embeddings |
| 6 | `report` | — | — | All link similarities + link candidates |
| 7 | `calibrate` | — | `[--buckets <N>]` (default 10), `[--recommend]` | Similarity distribution / recommended thresholds |
| 8 | `snapshot` | `create`\|`list`\|`delete` | `create [--label <L>]`, `delete <target>` | `delete` target is a snapshot id or `label:<LABEL>` |
| 9 | `refresh-subnodes` | — | `[--dry-run]` \| `[--apply]` (exclusive, default dry-run) | Propagate heading renames to subnode IDs. `--apply` backs up to `.refresh-bak.{epoch}` |
| 10 | `context` | `<target_files...>` | `[--command <S>]`, `[--granularity document\|subnode]`, `[--outline-only]`, `[--sections <ids>]`, `[--depth <N>]` | Underlying layer of MCP `compile_context`. granularity default `document` |
| 11 | `impact` | `<start>` | `[--max-depth <N>]` | Forward (downstream) traversal |
| 12 | `investigate` | `<start>` | `[--max-depth <N>]` | Backward (upstream) traversal |
| 13 | `feedback` | — | — | Generate Observations automatically from `check` results |
| 14 | `observe` | `<category> <message>` | `[--severity <S>]`, `[--related-id <ID>]`*, `[--target-file <PATH>]`*, `[--missing-doc <ID>]`, `[--source-glob <GLOB>]` | Underlying layer of MCP `observe`. category ∈ `compile_miss`\|`review_correction`\|`manual_note`. (* repeatable) |
| 15 | `audit` | — | `[--limit <N>]` (1–50, default 10) | Underlying layer of MCP `get_compile_audit` |
| 16 | `analyze` | — | — | Turn pending Observations into Proposals |
| 17 | `proposals` | — | `[--status pending\|approved\|rejected]` | List Proposals |
| 18 | `approve` | `<id>` | — | Approve a Proposal (human only) |
| 19 | `reject` | `<id>` | `--reason <S>` (required) | Reject a Proposal |

Note on `observe`: `<category>` and `<message>` are **positional** (the old `--category` /
`--message` flags were removed). Example:

```bash
legixy observe manual_note "overflow needs review" --related-id DD-CALC-001
```

The feedback loop (`observe` → `analyze` → `proposals` → `approve`/`reject`) and the audit log use
`.legixy/engine.db` (SQLite, WAL), created automatically.

---

## 8. Global options & exit codes

Global options come **before** the subcommand:

| Option | Default | Meaning |
|---|---|---|
| `--project-root <PATH>` | `.` | Project root |
| `--json` | off | JSON output (all commands) |
| `--models-dir <PATH>` | config `model_dir` | ONNX model directory |
| `-h, --help` / `-V, --version` | — | Help / version |

Exit codes: **0** success · **1** runtime failure or `check` Error > 0 · **2** usage error (argument
parsing). Logs go to stderr; results go to stdout.

---

## 9. JSON output

`--json` is a global flag placed before the subcommand:

```bash
legixy --json check --formal
legixy --json embed --all          # {generated, skipped, failed, errors[]}
legixy --json audit --limit 20
```

Use it for scripting and CI gates (check the exit code; parse stdout).

---

## 10. MCP server (Claude Code integration)

The MCP server (`legixy-mcp`, TypeScript / Node.js ≥ 20) exposes exactly **three tools** and forwards
each call faithfully to the CLI:

- **`compile_context`** — given target file paths, walks upstream and returns the relevant artifacts
  (UC / DD / SPEC …) as Markdown context. *Guidance before writing code.*
- **`observe`** — records a gap/contradiction the agent noticed into `engine.db` (deduplicated).
- **`get_compile_audit`** — returns the `compile_context` call history, so what the agent referenced
  is logged, not self-reported.

`approve`, `check`, `init` are **not** available over MCP. The server is stateless; all writes go
through a spawned `legixy` binary and output is forwarded unmodified.

Wire it via `.mcp.json` (the installed launcher sets `LGX_BIN` for you):

```json
{
  "mcpServers": {
    "legixy": {
      "command": "/path/to/legixy/legixy-mcp",
      "args": ["--project-root", "/path/to/your/project"]
    }
  }
}
```

Or run the server directly, pointing `LGX_BIN` at the binary:

```json
{
  "mcpServers": {
    "legixy": {
      "command": "node",
      "args": ["/path/to/legixy/ts-mcp/dist/index.js", "--project-root", "/path/to/project"],
      "env": { "LGX_BIN": "/path/to/legixy/bin/legixy" }
    }
  }
}
```

`compile_context` responses carry `_meta["anthropic/maxResultSizeChars"] = 500000`.

---

## 11. Environment variables

| Variable | Used by | Meaning |
|---|---|---|
| `LGX_MODELS_DIR` | CLI | ONNX model directory (overrides config; below `--models-dir`) |
| `LGX_BIN` | MCP server | Path to the `legixy` binary the server spawns |
| `LEGIXY_ONNX=1` | `deploy/build-deploy.sh` | Build with the `onnx` feature and stage the model |

---

## 12. Troubleshooting

- **"ONNX model not found …"** — the semantic layer can't locate the model. Run the install script,
  or `bash scripts/fetch-model.sh`, or pass `--models-dir`/set `LGX_MODELS_DIR`. The formal layer
  (`check --formal`) does not need the model.
- **`embed` exits 1 on a build without onnx** — the default build excludes ONNX. Rebuild with
  `--features onnx` (or `LEGIXY_ONNX=1 bash deploy/build-deploy.sh`).
- **Shared-library error at startup (Linux)** — the ONNX Runtime library must be loadable. Use the
  bundled `legixy` launcher (it sets `LD_LIBRARY_PATH` to the bundled `bin/`), not `bin/legixy`
  directly.
- **`onnxruntime.dll` not found (Windows)** — keep `legixy.exe` and `onnxruntime.dll` in the same
  directory (the installer does this; that directory is what gets added to PATH).
- **MCP server can't find the binary** — set `LGX_BIN` to the absolute path of `legixy`, or use the
  `legixy-mcp` launcher.
- **Semantic findings look noisy** — thresholds are not universal constants. Run `legixy calibrate
  --recommend` against your project's real distribution.

---

## 13. License

Apache-2.0. See `LICENSE` and `NOTICE`. The ONNX Runtime (MIT) and the embedding model (Apache-2.0,
from sentence-transformers) are obtained separately and retain their own licenses.
