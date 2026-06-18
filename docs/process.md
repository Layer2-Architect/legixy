# Development process — SPEC Compiling Pipeline (SCP)

**English** | [日本語](#日本語)

legixy is developed using the **SPEC Compiling Pipeline (SCP)** — a quality-bias-prevention
framework for operating an LLM as a software compiler from SPEC and use cases. SCP was called
**"DevProc"** during legixy's development and was renamed for public release.

SCP is published as a **separate repository**; it is intentionally not vendored here:

- **https://github.com/Layer2-Architect/SPEC-compiling-pipeline** — pin to a release tag (e.g. `v1.0.0`).

The process references in [`../CLAUDE.md`](../CLAUDE.md) and `.claude/` point into that
repository's `ja/` tree (the Japanese edition is the source of truth; an English edition is in
progress under `en/`).

> Process links are pinned to the SCP release tag `v1.0.0`; change that tag if you adopt a different release.

## Note on historical references

Some of legixy's own development artifacts (`docs/adr/`, `docs/gap-analysis/`,
`docs/specs-supplement/`, …) cite "**DevProc_V4.1**" or "**DevProc_V2**". These are former names of
SCP, kept as the historical record of how legixy was built.

---

# 日本語

legixy は **SPEC Compiling Pipeline (SCP)**（LLM を SPEC・ユースケースからのソフトウェアコンパイラとして
運用する品質偏向防止フレームワーク）に従って開発している。SCP は legixy 開発当時 **「DevProc」** と呼ばれて
おり、公開にあたり改称した。

SCP は**別リポジトリ**で公開しており、本リポジトリには意図的に同梱しない:

- **https://github.com/Layer2-Architect/SPEC-compiling-pipeline** — リリースタグ（例 `v1.0.0`）に固定して参照。

[`../CLAUDE.md`](../CLAUDE.md) や `.claude/` 内のプロセス参照は、そのリポジトリの `ja/` 配下（日本語版が
正本。英語版は `en/` で進行中）を指す。プロセス参照は SCP リリースタグ `v1.0.0` に固定している（採用タグを
変える場合は各リンクの `v1.0.0` を置換）。

**歴史的注記**: legixy 自身の開発成果物（`docs/adr/`・`docs/gap-analysis/`・`docs/specs-supplement/` 等）に
残る「DevProc_V4.1 / DevProc_V2」表記は SCP の旧称で、当時の記録としてそのまま保持している。
