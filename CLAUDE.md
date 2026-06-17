# CLAUDE.md (Author モード)

このプロジェクト **legixy** は **品質偏向防止を第一目的とする AI コンパイラ運用フレームワーク**(DevProc_V4.1)を採用している。

**プロセスの詳細は `docs/DevProc_V4/README.md` を参照すること。** 本ファイルは AI が常時遵守すべき最低限の規律のみを記載する。

> **モード分離**: このファイルは Author モード（生成・実装作業）向け。AI レビュアを起動する場合は `docs/DevProc_V4/bootstrap/CLAUDE-reviewer.md.template` を CLAUDE.md として配置する。両モードを 1 セッションで兼務しない（判定の独立性を守るため）。

> **第一目的**: 確率論的変換器(AI)を SPEC, UC からのソフトウェアコンパイラとして採用する開発プロセスは、品質最大化ではなく **品質偏向防止** を第一目的とする(`docs/DevProc_V4/00-philosophy.md` §1)。Author は自身の出力に過信せず、Reviewer 層と AT に検証を委ねる前提で動く。

## プロジェクト概要

legixy は、有向グラフを主体とするトレーサビリティエンジン（Rust CLI + TypeScript MCP サーバ）。旧称 traceability-engine v3 を OSS 公開向けにリブランドしたもの。

- 基盤仕様: `docs/legixy_foundational_spec.md`（LEGIXY-SPEC-001）
- 機能拡張: `docs/legixy_subnode_spec_v0.2.1.md`（LGX-EXT-001）, `docs/legixy_cache_spec_v0_1_0.md`（LGX-EXT-002）
- **CLI/MCP 互換契約**: `docs/legixy_cli_compat_reference.md`（LGX-COMPAT-001）

> ⚠️ **互換制約**: legixy は既存実行ファイル `traceability-engine`（v0.4.0-alpha4）と**実行時引数互換**でなければならない。19 サブコマンド・位置引数・フラグ・既定値・終了コード・MCP 3 ツールを維持する。設計・実装時は必ず LGX-COMPAT-001 を参照すること。

## チェーン順序（概念）

```
Raw SPEC → [前段ループ: QSET ⇄ SPP ⇄ FCR] → Accepted SPEC
         → TP[SPEC] → GAP[SPEC] → UC → TP[UC] → GAP[UC]
         → RBA → SEQA → RBD → SEQD → DD
         → TS → TC[RED] → SRC → TC[GREEN]
```

**ICONIX 二段化**: RB/SEQ は抽象側(RBA/SEQA、ドメインレベル)と具体側(RBD/SEQD、クラス図レベル)に分離。両者とも言語非依存。言語固有要素(関数名、引数型、戻り型、`async fn`, `Result<T,E>` 等)は DD で初出。

独立成果物: `QSET`, `SPP`, `FCR`（前段ループ）、`TP`, `GAP`（仕様レベル TDD）、`AT`（受け入れテスト）, `NFR`（非機能要件）, `ADR`（設計判断記録）, `VAL`（横断的妥当性確認）, `RPC`（責務保存率検査）

**実装上の注**: `.trace-engine.toml` の `[id.chain] order` は `UC → RBA → SEQA → RBD → SEQD → DD → TS → TC → SRC`、`SPEC / QSET / SPP / FCR / TP / GAP / AT / NFR / ADR / VAL / RPC` は `independent`。前段ループと SPEC レベル TDD ループは本文 metadata と `scripts/trace-check.sh` の grep ゲートで検証する。

## ハードルール（常時適用）

1. **SPEC の変更は人間承認が必要。** AI は提案する、人間が決定する。
2. **GAP がクローズしないうちに次フェーズへ進まない。** GAP[SPEC] open のうちは UC 着手禁止。GAP[UC] open のうちは RBA 着手禁止。`bash scripts/trace-check.sh` がこれを機械検証する。
3. **すべての成果物は親への参照を持つ。** chain 内成果物は `traceability-engine check --formal` で、chain 外成果物は本文 metadata + `scripts/trace-check.sh` で検証する。
4. **新しい成果物タイプは `.trace-engine.toml` 更新が先。** チェーンに無いタイプを勝手に作らない。新タイプ追加時は (a) typecode を `.trace-engine.toml` に追加、(b) ID を `{type}-LGX-NNN` 形式で命名、(c) ファイル先頭に `Document ID:` 行を必置、(d) `docs/traceability/graph.toml` の `[[nodes]]` に登録。
5. **AT は終端ではなく独立した検証チャネル。** 暗黙知・ドメイン慣行・前提の不一致専用（`docs/DevProc_V4/00-philosophy.md` §2.4）。
6. **仕様書とテストコードは実装着手後に変更しない。** 実装がテストに合わせる。ただし `/defect-fix`・`/spec-change` を経由する上流修正は対象外。
7. **境界 API の契約は DD 段階で凍結する。** 凍結後の変更は次バージョンの SPEC 改訂として扱う。**legixy の場合、LGX-COMPAT-001 が規定する CLI/MCP 引数は既に凍結済みの境界契約とみなす。**
8. **テストが通らない実装はマージしない。**
9. **SPEC は前段ループで FCR.frontend_status = ACCEPTED に到達していなければ TP[SPEC] / UC 着手禁止。** `bash scripts/trace-check.sh` が機械検証する。スキップ時は ADR で記録（`docs/DevProc_V4/03a-frontend-pass.md` §11）。
10. **ICONIX 二段化レイヤ汚染禁止 + 三者整合性検証必須。** 抽象側 RBA/SEQA にはドメイン語彙のみ、具体側 RBD/SEQD には操作名とクラス図表記まで。言語固有要素は DD でのみ。`scripts/trace-check.sh` の [5/5] が grep で検出。詳細は `docs/DevProc_V4/04-iconix-layer.md`。
11. **人間関与は SPEC と UC に限定する。** RBA 以降は AI 自律実行 + AI Reviewer 層 + AT で品質保証する。例外: 境界 API 凍結対象リスト承認（ハードルール 7）。詳細は `docs/DevProc_V4/guides/ai-collaboration.md` §1。

## 新規 SPEC 受け取り時の起動条件

新規 SPEC を受け取ったら、最初にするのは**前段ループの起動**（QSET 発行）。TP[SPEC] や UC の生成に直接着手してはならない。手順は `docs/DevProc_V4/03a-frontend-pass.md` を参照。

## 出力規律

- 下流成果物（RBA/SEQA/RBD/SEQD/DD/TS/TC/SRC）を生成するとき、上流成果物の typecode-id を必ず引用元として記載する。
- SPEC/UC レベルの変更を勝手に行わない（人間の判断領域）。
- 不明点は人間に問い、推測で埋めない。
- 前段ループの QSET / SPP / FCR を勝手に書き換えない。各反復は新規 ID で発行し履歴を残す。
- **CLI/MCP の引数を変更・追加する設計は LGX-COMPAT-001 の互換チェックリストに違反しないこと。**

## 検証コマンド

フェーズ完了を主張する前に必ず実行:

```bash
bash scripts/trace-check.sh
#   1. traceability-engine check --formal（第 1 層: ID 形式・chain 整合・OrphanFile・IdRedefined・DAG）
#   2. SPEC レベル TDD ゲート（red TP / open GAP の grep 検査）
#   3. 前段ループゲート（各 SPEC の最新 FCR の frontend_status 検査）
#   4. レイヤ汚染検査（RBD/SEQD への言語固有要素混入検出）
```

第 2 層（semantic、ONNX 必須）は `models/paraphrase-multilingual-MiniLM-L12-v2/` 配置 + `.trace-engine.toml` の `[semantic] enabled = true` 後に `traceability-engine check`。

詳細は `docs/DevProc_V4/06-trace-engine.md`、`docs/DevProc_V4/manual/traceability-engine.v3/manual.md`。

## プロジェクト固有の補足

- area コード: `LGX`
- 採用言語: Rust（CLI 本体） + TypeScript（MCP サーバ `ts-mcp`）
- 採用テストフレームワーク: cargo test + proptest（Rust）、Vitest + fast-check（TS MCP）
- 設定ファイル: `.trace-engine.toml`（開発ツールの実バイナリが読む名。legixy-the-tool 既定の `.legixy.toml` とは別レイヤ。LGX-COMPAT-001 §6 参照）
- 観点ナレッジベース: `docs/perspectives/core-perspectives.md`, `docs/perspectives/ux-perspectives.md`
- ゲートスキップ記録: `docs/decisions/gate-skips.md`
- 前段スキップ記録: `docs/adr/`（`ADR-LGX-NNN_frontend-pass-skip-<SPEC-ID>.md`）
- ONNX モデル: `models/paraphrase-multilingual-MiniLM-L12-v2/`（第 2 層 semantic を有効化する場合のみ。配置済み）
- 基盤・互換リファレンス: `docs/legixy_foundational_spec.md`, `docs/legixy_cli_compat_reference.md`

## 主要ドキュメントの参照表

| 作業内容 | 参照先 |
|---|---|
| プロセス全体像 | `docs/DevProc_V4/01-overview.md` |
| 成果物タイプと ID | `docs/DevProc_V4/02-typecodes.md` |
| **前段ループ（Raw SPEC → Accepted SPEC）** | `docs/DevProc_V4/03a-frontend-pass.md` |
| SPEC, UC, TP, GAP の作業 | `docs/DevProc_V4/03-spec-level-tdd.md` |
| **RBA/SEQA/RBD/SEQD/DD の作業（ICONIX 二段化）** | `docs/DevProc_V4/04-iconix-layer.md` |
| TS, TC, SRC の作業 | `docs/DevProc_V4/05-test-and-impl.md` |
| traceability-engine 操作 | `docs/DevProc_V4/06-trace-engine.md` |
| AT, NFR の作業 | `docs/DevProc_V4/07-at-and-nfr.md` |
| ゲート判定 | `docs/DevProc_V4/08-gates.md` |
| **AI レビュア層の運用** | `docs/DevProc_V4/review-guidelines/README.md` |
| **Reviewer モード CLAUDE.md** | `docs/DevProc_V4/bootstrap/CLAUDE-reviewer.md.template` |
| 各成果物の雛形 | `docs/DevProc_V4/templates/` |
| 観点ナレッジベース | `docs/perspectives/` |
| 言語別ガイド | `docs/DevProc_V4/guides/language-stacks/rust.md`, `.../typescript.md` |
| legixy CLI/MCP 互換契約 | `docs/legixy_cli_compat_reference.md` |
