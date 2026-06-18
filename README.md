# legixy

**English** | [日本語](#日本語)

> A directed-graph **traceability engine** that keeps the links between your artifacts (SPEC → … → source) as machine-verifiable data, and watches for **semantic drift** — built for AI-assisted development.

[![License: Apache-2.0](https://img.shields.io/badge/License-Apache_2.0-blue.svg)](#license)
![status](https://img.shields.io/badge/status-0.4.0--alpha4-orange)

legixy registers every artifact (specs, use cases, design, tests, source) as a node in a directed graph, verifies the graph **deterministically** (ID format, file existence, chain integrity, acyclicity, …), and **observes** semantic deviation between linked artifacts with a frozen local embedding model.

It **reports deviation, not anomaly.** It does not judge, does not edit your files, and never rewrites the graph on its own. It narrows down what a human should review — approval and rejection always stay with you.

---

## Why

When AI agents edit documents over many turns, content degrades in a way that **passes diff review**: not by deletion, but by *plausible rewrites where only the meaning moves*. Tests guard code; nothing guards the meaning of SPEC / UC / DD natural-language artifacts. legixy adds that missing verification loop:

- **diff / tests / hashes / Git** detect *that something changed* — not *that meaning moved*, and not *where to look*.
- legixy fixes the structure as a graph and measures meaning movement as **Drift** (1 − cosine similarity vs. the previous embedding) and **SemanticSimilarity** (decoupling of graph-adjacent artifacts).

The deep rationale (the "stochastic compiler" view, the empirical threat model, the OSS-publishing philosophy) lives in the linked articles under [Background](#background).

---

## Highlights

- **Single Rust binary** (`legixy`, 19 subcommands — human-facing) **+ TypeScript MCP server** (3 tools — for Claude Code).
- **No runtime network dependency.** Embeddings run on a local frozen ONNX model.
- **Two-layer verification**: deterministic *formal* layer (CI-friendly, no model) + *semantic* layer (embedding-based, multilingual incl. Japanese).
- **Human-in-the-loop by construction**: `approve` / `reject` are CLI-only — never exposed to the agent (Admin vs. Agent surface separation).
- **Process-independent**: ICONIX is only the default template. Typecodes and chain order are configuration; the engine reads text, IDs, chains, and embeddings — not your methodology.
- **Publishes its own provenance**: the OSS release ships not just code but legixy's *entire* artifact set (SPEC → SRC) and the directed graph that links them.

---

## Install

Two ways: **prebuilt binaries** (recommended) or **build from source**. Full guide:
**[docs/manual/manual.en.md](docs/manual/manual.en.md)**.

### Prebuilt binaries (Linux / Windows, x86_64)

Each GitHub Release attaches onnx-enabled binaries. The install scripts also download the embedding
model, so the semantic layer works out of the box (use `--no-model` / `-NoModel` to skip it).

```bash
# Linux / macOS
curl -fsSL https://raw.githubusercontent.com/Layer2-Architect/legixy/main/install.sh | bash -s -- --repo Layer2-Architect/legixy
```

```powershell
# Windows (PowerShell)
$env:LEGIXY_REPO="Layer2-Architect/legixy"; irm https://raw.githubusercontent.com/Layer2-Architect/legixy/main/install.ps1 | iex
```

### Build from source

Requires a recent Rust toolchain (and Node.js ≥ 20 for the MCP server).

```bash
git clone https://github.com/Layer2-Architect/legixy
cd legixy
cargo build --release -p legixy-cli              # formal layer only
cargo build --release -p legixy-cli --features onnx   # + semantic layer (links ONNX Runtime)
./target/release/legixy --version
# → legixy 0.4.0-alpha4
```

Single binary (ONNX runtime, tokenizers, SQLite bundled). No Python dependency.

The semantic layer (layer 2) additionally needs a local model. The install script fetches it; or run
`bash scripts/fetch-model.sh`; or place `model.onnx` + `tokenizer.json` of
**`paraphrase-multilingual-MiniLM-L12-v2`** (multilingual, including Japanese; 384-dim) under
`models/paraphrase-multilingual-MiniLM-L12-v2/`, or point `--models-dir` / `LGX_MODELS_DIR` at it.

## Quick start (10 minutes)

```bash
# 1. Initialize a project (generates .legixy.toml, docs/traceability/graph.toml, .legixy/engine.db, doc dirs)
legixy init

# 2. Register artifacts in docs/traceability/graph.toml (see below), then:

# 3a. Formal layer only — fast, deterministic, CI-friendly
legixy check --formal

# 3b. Formal + semantic layer (needs the ONNX model)
legixy embed --all          # generate embeddings
legixy check

# 4. Walk the graph
legixy investigate SRC-PR-001          # upstream: what does this implementation derive from?
legixy impact      SPEC-PR-001 --max-depth 5   # downstream: how far does this spec change reach?

# JSON output for scripting (--json is a global flag, before the subcommand)
legixy --json check --formal
```

Registering artifacts (`docs/traceability/graph.toml`):

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

IDs follow `{type}-{area}-{seq}`. Markdown artifacts bind by the ID prefix in the filename; source files bind by a
`// Document ID: SRC-PR-001` comment near the top. Heading-level **subnodes** (`DD-PR-001#anchor`) let you trace at
section granularity, not just whole documents.

---

## Architecture

```
project/
├── .legixy.toml                    # config (.trace-engine.toml is read as a legacy fallback)
├── docs/traceability/graph.toml    # the graph = primary, human-readable, Git-diffable data
├── .legixy/engine.db               # SQLite (WAL): observations, proposals, embeddings, audit log
└── models/paraphrase-multilingual-MiniLM-L12-v2/   # local ONNX model (semantic layer)

 Rust CLI (human, 19 commands)        MCP server (Claude Code, 3 tools only)
 init/check/embed/drift/report        compile_context
 calibrate/snapshot/refresh-subnodes  observe
 context/impact/investigate           get_compile_audit
 feedback/observe/audit/analyze       (spawns the Rust CLI and forwards faithfully)
 proposals/approve/reject/migrate
```

- **The graph (`graph.toml`) is the primary data.** It is never auto-updated; updates are explicit human/agent edits, and inconsistencies are detected after the fact by `check --formal`. The audit trail does not silently mutate.
- **The embedding model is frozen.** Keeping the *instrument* stationary is the point: if both the subject (AI output) and the instrument drift, an observed deviation is no longer attributable. Model-version equality is an invariant.
- **Admin vs. Agent surface separation.** Of the 19 CLI commands, only `compile_context` / `observe` / `get_compile_audit` are exposed over MCP. `approve` / `reject` and 13 others are CLI-only — i.e. human-only.

### Two-layer verification

| Layer | Command | Categories | Needs model |
|---|---|---|---|
| **Formal** (deterministic) | `check --formal` | ID format, file existence, chain integrity, acyclicity (DAG), orphan files, subnode ID format | No |
| **Semantic** (embedding) | `check` | SemanticSimilarity (linked pair below threshold → Warning), LinkCandidate (unlinked pair above threshold → Info), Drift (content_hash mismatch → Warning) | Yes |

Semantic findings are **Warning / Info only** — deviation is not anomaly. Thresholds are not universal constants;
calibrate them against your project's real distribution with `legixy calibrate`.

---

## Claude Code / MCP integration

Wire the MCP server (`legixy-mcp`, TypeScript / Node.js) via `.mcp.json`. Claude Code then sees three tools:

- **`compile_context`** — given target file paths, walks upstream and returns the relevant artifacts (UC / DD / SPEC …) as Markdown context. *Guidance before writing code.*
- **`observe`** — records a gap/contradiction the agent noticed into `engine.db` (deduplicated).
- **`get_compile_audit`** — returns the `compile_context` call history, so *what the agent referenced* is logged, not self-reported.

That is all. `approve`, `check`, `init` do not exist over MCP. The server is stateless; all writes go through a spawned Rust CLI and output is forwarded unmodified.

---

## Process independence

The engine is methodology-agnostic. Override `[id.chain]` (and `[id.types.*]`) in the config:

```toml
# Waterfall / requirement-driven
[id.chain]
order = ["REQ", "DES", "IMPL", "TEST"]
independent = ["ARCH"]

# Agile / user-story
[id.chain]
order = ["US", "AC", "FR", "CODE"]
independent = ["EPIC", "PERSONA"]
```

More than that, legixy is not even tied to software. Its only requirements are: (1) a defined workflow, (2) AI
creating/editing documents, (3) documents with upstream→downstream semantic links. Contract→spec→deliverable,
application→design→validation-record, research-plan→protocol→report all qualify.

---

## What the OSS release publishes

Beyond source and binaries, the release includes legixy's **entire artifact set from SPEC to SRC**, plus the
**directed-graph data (`graph.toml`)** that links them.

| | Conventional OSS | legixy |
|---|---|---|
| Unit of publication | source (+ build artifacts) | SPEC → UC → design → tests → SRC |
| Relations between artifacts | implicit (dirs & naming) | **explicit directed graph** (machine-readable) |
| Form of audit | read the code | **walk the graph, follow the derivation** |
| Question answered | what does this code do? | **why is it this way?** (trace to the governing spec) |

So you don't have to read a testimonial — you can `legixy investigate SRC-…` the repository itself and follow any
implementation back to its spec.

---

## Limitations & non-goals

- **Does not judge correctness.** Detection of deviation only; whether it is a problem is out of scope.
- **Does not auto-update the graph.** File add/remove/rename is followed by manual `graph.toml` edits; `check` detects drift after the fact.
- **The semantic layer is an instrument, not a judge.** Thresholds are project-specific baselines; do not take severity at face value before calibrating. Confidence is asymmetric: the **formal layer is validated in practice; the semantic layer's response characteristics are still n=1** — independent validation is welcome (see the articles).
- **Overkill for small, short-lived projects.** Declaring and maintaining the graph has a cost that pays off only for long-lived, large, or high-stakes work.
- legixy is **one block** of a larger process (machine verification + measurement), not a whole methodology.

---

## Background

- **Tool guide** (Zenn): 信じないものを、追跡する ― SCP の計測基盤 legixy 紹介 *(link TBD)*
- **Origin & epistemology** (Zenn): 「意味はトレースできない」とAIは言った *(link TBD)*
- **The development process (SCP)**: 品質「最大化」をやめる ― AIを確率論的コンパイラとして扱う開発プロセス *(link TBD)*

> Note: the articles describe an earlier state (binary `traceability-engine` v0.2.0, 17 commands, `all-MiniLM-L6-v2`,
> `.trace-engine.toml`). The current binary is **`legixy` v0.4.0-alpha4** (19 commands, multilingual
> `paraphrase-multilingual-MiniLM-L12-v2`, default config `.legixy.toml`).

## License

Apache-2.0.

---
---

# 日本語

[English](#legixy) | **日本語**

> 成果物間のリンク（SPEC → … → ソース）を機械検証可能なデータとして保持し、**意味的な逸脱（Drift）**を観察する、
> 有向グラフ主体の**トレーサビリティエンジン**。AI 支援開発のために作られている。

legixy は全成果物（仕様・ユースケース・設計・テスト・ソース）を有向グラフのノードとして登録し、**決定論的に**検証し
（ID 形式・ファイル存在・連鎖整合性・循環なし …）、凍結したローカル埋め込みモデルでリンク間の**意味的逸脱**を観察する。

報告するのは**逸脱（deviation）であって異常（anomaly）ではない。** 判断しない。ファイルを編集しない。グラフを勝手に
書き換えない。レビュー対象を絞り込むだけで、承認・却下は常に人間が行う。

---

## なぜ要るか

AI エージェントが文書を何ターンも編集すると、内容は **diff レビューを通過する形**で劣化する ― 削除ではなく、
*形式・長さ・流暢さを保ったまま意味だけが移動するもっともらしい書き換え*で。テストはコードを守るが、SPEC / UC / DD
という自然言語成果物の意味は何も守っていない。legixy はその欠けた検証ループを足す:

- **diff / テスト / hash / Git** は「変わったこと」を捕らえるが、「意味が移動したこと」も「どこを見るべきか」も教えない。
- legixy は構造を有向グラフとして固定し、意味の移動量を **Drift**（前回 embedding との 1 − コサイン類似度）と
  **SemanticSimilarity**（グラフ隣接成果物の意味的剥離）として測る。

設計の深い根拠（「確率論的コンパイラ」という視座、実測に基づく脅威モデル、OSS 公開の思想）は[背景](#背景)のリンク記事にある。

---

## 特徴

- **Rust 単一バイナリ**（`legixy`、19 サブコマンド、人間用）**＋ TypeScript MCP サーバー**（3 ツール、Claude Code 用）。
- **ランタイムのネットワーク依存なし。** 埋め込みはローカルの凍結 ONNX モデルで動く。
- **二層検証**: 決定論的な *形式層*（モデル不要、CI に置ける）＋ *意味層*（埋め込みベース、日本語を含む多言語）。
- **人間関与を構造で担保**: `approve` / `reject` は CLI 専用 ― エージェントには露出しない（Admin / Agent サーフェス分離）。
- **プロセス非依存**: ICONIX は既定テンプレートにすぎない。typecode と連鎖順序は設定値で、エンジンが見るのはテキスト・
  ID・連鎖・embedding だけ。
- **自身の来歴を公開**: OSS リリースはコードだけでなく、legixy 自身の *全*成果物（SPEC → SRC）と、それらを結ぶ有向グラフを同梱する。

---

## インストール

**ビルド済みバイナリ**（推奨）または**ソースからビルド**の 2 通り。詳細は
**[docs/manual/manual.ja.md](docs/manual/manual.ja.md)**。

### ビルド済みバイナリ（Linux / Windows・x86_64）

各 GitHub Release に onnx 有効バイナリが添付される。install スクリプトは埋め込みモデルも取得するため、
意味層がそのまま動く（`--no-model` / `-NoModel` で省略可）。

```bash
# Linux / macOS
curl -fsSL https://raw.githubusercontent.com/Layer2-Architect/legixy/main/install.sh | bash -s -- --repo Layer2-Architect/legixy
```

```powershell
# Windows (PowerShell)
$env:LEGIXY_REPO="Layer2-Architect/legixy"; irm https://raw.githubusercontent.com/Layer2-Architect/legixy/main/install.ps1 | iex
```

### ソースからビルド

最近の Rust ツールチェーンが必要（MCP サーバーには Node.js ≥ 20 も）。

```bash
git clone https://github.com/Layer2-Architect/legixy
cd legixy
cargo build --release -p legixy-cli              # 形式層のみ
cargo build --release -p legixy-cli --features onnx   # + 意味層（ONNX Runtime をリンク）
./target/release/legixy --version
# → legixy 0.4.0-alpha4
```

単一バイナリ（ONNX runtime・tokenizers・SQLite 同梱）。Python 依存なし。

意味層（第2層）を使う場合のみ、ローカルモデルが追加で要る。install スクリプトが取得する。または
`bash scripts/fetch-model.sh`、あるいは **`paraphrase-multilingual-MiniLM-L12-v2`**
（日本語を含む多言語・384 次元）の `model.onnx` + `tokenizer.json` を
`models/paraphrase-multilingual-MiniLM-L12-v2/` に配置するか、`--models-dir` / `LGX_MODELS_DIR` で指す。

## 10 分で試す

```bash
# 1. 初期化（.legixy.toml / docs/traceability/graph.toml / .legixy/engine.db / 成果物ディレクトリを生成）
legixy init

# 2. docs/traceability/graph.toml に成果物を登録（下記）。その後:

# 3a. 形式層のみ ― 高速・決定論的・CI 向け
legixy check --formal

# 3b. 形式層 + 意味層（ONNX モデル要）
legixy embed --all          # embedding 生成
legixy check

# 4. グラフを歩く
legixy investigate SRC-PR-001          # 上流遡及: この実装は何に根拠を持つか
legixy impact      SPEC-PR-001 --max-depth 5   # 下流影響: この仕様変更はどこまで波及するか

# スクリプト連携（--json はグローバルフラグ、サブコマンドの前）
legixy --json check --formal
```

成果物の登録（`docs/traceability/graph.toml`）:

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

ID 体系は `{type}-{area}-{seq}`。Markdown 成果物はファイル名先頭の ID で、ソースコードは先頭付近の
`// Document ID: SRC-PR-001` コメントで、ファイルと ID が結ばれる。`DD-PR-001#anchor` 形式の**見出し単位サブノード**で、
ドキュメント単位でなく節単位の追跡もできる。

---

## アーキテクチャ

```
project/
├── .legixy.toml                    # 設定（旧名 .trace-engine.toml も fallback として読む）
├── docs/traceability/graph.toml    # グラフ = 一次データ。人間が読め、Git で差分が追える
├── .legixy/engine.db               # SQLite(WAL): 観察・提案・embedding・監査ログ
└── models/paraphrase-multilingual-MiniLM-L12-v2/   # ローカル ONNX モデル（意味層）

 Rust CLI（人間用・19 コマンド）        MCP サーバー（Claude Code 用・3 ツールのみ）
 init/check/embed/drift/report         compile_context
 calibrate/snapshot/refresh-subnodes   observe
 context/impact/investigate            get_compile_audit
 feedback/observe/audit/analyze        （内部で Rust CLI を spawn し忠実転送）
 proposals/approve/reject/migrate
```

- **一次データはグラフ（`graph.toml`）。** 自動更新されない。更新は人間/エージェントの明示編集で、不整合は
  `check --formal` が事後検出する。証跡が暗黙に書き換わらない。
- **埋め込みモデルは凍結。** 計器を止めることが要点 ― 対象（AI 出力）と計器（モデル）が両方動くと、観測した逸脱が
  どちらのものか区別できなくなる。モデルバージョン一致は不変条件。
- **Admin / Agent サーフェス分離。** 19 コマンドのうち MCP に露出するのは `compile_context` / `observe` /
  `get_compile_audit` の 3 つだけ。`approve` / `reject` 等 14 は CLI 専用＝人間専用。

### 二層の検証

| 層 | コマンド | カテゴリ | モデル |
|---|---|---|---|
| **形式層**（決定論） | `check --formal` | ID 形式・ファイル存在・連鎖整合性・循環なし(DAG)・孤児ファイル・サブノード ID 形式 | 不要 |
| **意味層**（埋め込み） | `check` | SemanticSimilarity（リンク間が閾値未満→Warning）・LinkCandidate（非リンクが閾値超過→Info）・Drift（content_hash 不一致→Warning） | 要 |

意味層の出力は **Warning / Info 止まり** ― 逸脱は異常ではない。閾値は普遍定数ではなく、`legixy calibrate` で
プロジェクトの実分布から校正する。

---

## Claude Code / MCP 統合

MCP サーバー（`legixy-mcp`、TypeScript / Node.js）を `.mcp.json` で繋ぐと、Claude Code から 3 ツールが見える:

- **`compile_context`** ― 対象ファイルパスを渡すと上流成果物（UC / DD / SPEC …）を遡って Markdown で返す。*コードを書く前のガイダンス。*
- **`observe`** ― エージェントが気づいた欠落・矛盾を `engine.db` に記録（重複排除）。
- **`get_compile_audit`** ― `compile_context` の呼出履歴を返す。*何を参照して書いたか*が申告でなくログで残る。

これだけ。`approve` も `check` も `init` も MCP には無い。サーバーはステートレスで、書込みは常に Rust CLI の spawn 経由、出力は無加工で忠実転送。

---

## プロセス非依存

エンジンは方法論非依存。設定の `[id.chain]`（および `[id.types.*]`）を書き換える:

```toml
# Waterfall / 要求駆動
[id.chain]
order = ["REQ", "DES", "IMPL", "TEST"]
independent = ["ARCH"]

# Agile / User Story
[id.chain]
order = ["US", "AC", "FR", "CODE"]
independent = ["EPIC", "PERSONA"]
```

さらに legixy はソフトウェアにも縛られない。適用条件は (1) 定義された作業フロー、(2) AI が文書を作成・編集する、
(3) 文書が上流→下流の意味的連結を持つ ― の三つだけ。契約→仕様書→納品物、申請→設計→検証記録、研究計画→
プロトコル→報告書、いずれも当てはまる。

---

## 何を公開するか

ソースと実行ファイルに加えて、legixy 自身の **SPEC から SRC に至る全成果物**と、それらを結ぶ**有向グラフのデータ
（`graph.toml`）**を公開する。

| | 従来の OSS | legixy |
|---|---|---|
| 公開の単位 | ソース（+ ビルド成果物） | SPEC → UC → 設計 → テスト → SRC の全成果物 |
| 成果物間の関係 | 暗黙（ディレクトリ・命名から推測） | **有向グラフとして明示**（機械可読） |
| 監査の形 | コードを読む | **グラフを走査し、導出を追う** |
| 答えられる問い | このコードは何をするか | **なぜこうなのか**（根拠の仕様まで遡れる） |

体験談を読む必要はない ― リポジトリ自身を `legixy investigate SRC-…` で走査し、任意の実装を根拠の仕様まで遡れる。

---

## 限界・非目標

- **正しさを判定しない。** 逸脱の検出までで、それが問題かは答えない。
- **グラフを自動更新しない。** ファイルの追加・削除・リネームは `graph.toml` の手動編集で追従し、`check` が事後検出する。
- **意味層は計器であって審判ではない。** 閾値はプロジェクト固有のベースライン。校正前に severity を真に受けない。
  確信は非対称 ― **形式層は実地で有効を確認済み、意味層の応答特性はまだ n=1**。独立検証を歓迎する（記事参照）。
- **小規模・短命なプロジェクトには過剰装備。** グラフの宣言・維持にはコストがあり、長期・大規模・破局コストの大きい領域で初めてペイする。
- legixy はより大きなプロセスの**一ブロック**（機械検証 + 計測）であって、方法論の全体ではない。

---

## 背景

- **道具編**（Zenn）: 信じないものを、追跡する ― SCP の計測基盤 legixy 紹介 *(リンク未定)*
- **来歴編**（Zenn）: 「意味はトレースできない」とAIは言った *(リンク未定)*
- **開発プロセス（SCP）**: 品質「最大化」をやめる ― AIを確率論的コンパイラとして扱う開発プロセス *(リンク未定)*

> 注: 記事は旧状態（バイナリ `traceability-engine` v0.2.0、17 コマンド、`all-MiniLM-L6-v2`、`.trace-engine.toml`）を
> 記述している。現行は **`legixy` v0.4.0-alpha4**（19 コマンド、多言語 `paraphrase-multilingual-MiniLM-L12-v2`、
> 既定設定 `.legixy.toml`）。

## ライセンス

Apache-2.0。
