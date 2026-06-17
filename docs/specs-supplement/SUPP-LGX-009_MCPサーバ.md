Document ID: SUPP-LGX-009

# SUPP-LGX-009: SPEC-LGX-009（MCP サーバ）実装補完情報

| 項目 | 内容 |
|------|------|
| Document ID | SUPP-LGX-009 |
| 対象 SPEC | SPEC-LGX-009: MCP サーバ（Version 0.6.0, 2026-06-10 Approved） |
| Status | AI 生成・非正準・人間査読待ち |
| Date | 2026-06-12 |
| 調査ソース | legixy.old.p1/docs/ および traceability-engine.v3.chg_to_lexigy/（ts-mcp/, deploy/mcp/, claudedocs/, old.source/） |

> **本文書は SPEC 本文の変更ではなく実装のための補完情報（参考資料）である。SPEC 変更には人間承認が必要（SPEC-LGX-001 §7.1）。**
> [補完] = 旧文書・旧実装から根拠付きで確認できた内容。[要決定] = 情報が見つからない・矛盾がある・人間の判断が必要な論点。

---

## §1 未解決参照（SPEC が参照するが新リポジトリに存在しない文書）

新リポジトリ `legixy` には `docs/specs/`（SPEC 10 件）と `docs/specs-supplement/`、`docs/DevPorc/`（DevProc vendored）のみが存在する。SPEC-LGX-009 が参照する以下の文書は新リポジトリに**存在しない**が、すべて旧リポジトリで所在を確認した。

| # | 参照 ID / 文書 | SPEC 内の参照箇所 | 何のために必要か | 確認済み所在（旧リポジトリ） |
|---|---------------|------------------|----------------|---------------------------|
| 1 | LEGIXY-SPEC-001 | ヘッダ「親文書」、§2、REQ.01/02/03/05 | MCP-INV-1〜4・STATE-INV の正準定義（§10）、実装言語方針（§4/§8） | `legixy.old.p1/docs/legixy_foundational_spec.md` |
| 2 | LGX-EXT-001 | ヘッダ「親文書」、§2、REQ.02/03/04/09/15 | 「新ツール追加禁止・引数転送のみで拡張」の根拠（§6.1〜6.4） | `legixy.old.p1/docs/legixy_subnode_spec_v0.2.1.md` |
| 3 | LGX-EXT-002 | REQ.13/14（§4.1〜4.5, §5.2, §8.5〜8.6 引用） | `_meta` 付与仕様、CACHE-INV-4、サイズ超過エラー形式（§4.3）、責務分担（§8.5）、observe 非適用判断（§8.6） | `legixy.old.p1/docs/legixy_cache_spec_v0_1_0.md` |
| 4 | LGX-COMPAT-001 | REQ.07「凍結対象 (a)〜(f)」、REQ.16 | 凍結契約の定義、CLI 全サブコマンド・引数一覧（§4）、MCP↔CLI マッピング（§5）、バイナリ解決順 | `legixy.old.p1/docs/legixy_cli_compat_reference.md` |
| 5 | NFR-LGX-001 | ヘッダ「対応 NFR」、REQ.03/06/07/11/12/13/14/16 | PERF.03 正準値（§3.2 / §13）、COMPAT.06/10/11/12、SEC.05、USE.02 | `legixy.old.p1/docs/nfr/NFR-LGX-001_非機能要件.md` |
| 6 | UC-LGX-002 / 004 / 008 | ヘッダ「対応 UC」 | MCP 経由のコンテキスト解決・粒度制御・フィードバックの利用シナリオ | `legixy.old.p1/docs/usecases/UC-LGX-002_コンテキスト解決.md`、`UC-LGX-004_粒度制御付きコンテキスト解決.md`、`UC-LGX-008_フィードバックループ.md` |
| 7 | GAP-LGX-162 / 168 / 169 / 170 / 171 | REQ.03/06/08/13/16 | v0.5.2/0.6.0 改訂の背景（エラー時 _meta、初期化失敗、タイムアウト、ロギング、PERF 値整合） | `legixy.old.p1/docs/gap-analysis/GAP-LGX-162_エラー応答へのmeta付与可否.md` ほか同ディレクトリ |
| 8 | QSET-LGX-009 / QSET-LGX-011 | REQ.07/13（回答引用） | maxResultSizeChars 単位確定（Q1）・exit code 区別（Q2）・反復 2 訂正の経緯 | `legixy.old.p1/docs/frontend-pass/questionnaires/QSET-LGX-009_MCPサーバ.md`、`QSET-LGX-011_MCPサーバ-反復2.md` |
| 9 | SPP-LGX-009 / SPP-LGX-011 | 変更履歴 v0.5.0/0.5.1 | 承認済みパッチの本文（REQ 改訂の正確な差分） | `legixy.old.p1/docs/spec-patches/SPP-LGX-009_MCPサーバ.md`、`SPP-LGX-011_MCPサーバ-反復2.md` |
| 10 | ADR-LGX-010 | REQ.16 根拠 | タイムアウト導入の決定記録（既定 30 秒・`LGX_MCP_TIMEOUT_SEC`・0 で無効・SIGTERM→5 秒→SIGKILL・部分出力非転送） | `legixy.old.p1/docs/adr/ADR-LGX-010_mcp-child-process-timeout.md` |
| 11 | workflow_2026-04-28_v0.4.0-ga-roadmap.md | REQ.15 根拠（§2.2 MCP-SYNC） | Block B 引数の MCP 同期作業の経緯 | `traceability-engine.v3.chg_to_lexigy/claudedocs/workflow_2026-04-28_v0.4.0-ga-roadmap.md` |
| 12 | 前世代 `old.source/TypeScriptMCP/` | §2「慣例仕様として参照」 | v0.1.0 世代 MCP 実装（慣例仕様） | `traceability-engine.v3.chg_to_lexigy/old.source/TypeScriptMCP/` |
| 13 | `ts-mcp/tests/integration.test.ts`、`ts-mcp/tests/tools/compile-context.test.ts` | REQ.04/15 検証方法 | 引数ラウンドトリップ（T-MCP-CC-003）・Block B 11 テストの参照実体 | `traceability-engine.v3.chg_to_lexigy/ts-mcp/tests/integration.test.ts`（569 行、T-MCP-CC-003 は :154）、`tests/tools/compile-context.test.ts`（289 行） |

**新リポジトリ内で解決できる参照（未解決ではない）:** SPEC-LGX-001 §7.1（変更ポリシー）、SPEC-LGX-003 REQ.10/13/15〜17、SPEC-LGX-004 REQ.04（終了コード）は `legixy/docs/specs/` に存在する。QSET/SPP/FCR/GAP 等の DevProc 用語の定義は `docs/DevPorc/`（vendored DevProc）で参照可能。

---

## §2 実装に必要だが SPEC 内で未規定の事項

### 2.1 ツール入力スキーマの全体像 [補完]

SPEC-LGX-009 は granularity（REQ.04）と Block B 3 引数（REQ.15）のみ規定し、**基本引数（target_files / command、observe の全引数、get_compile_audit の limit）のスキーマは未規定**。以下が v3 実装の確定値（根拠: LGX-COMPAT-001 §5 の表 + `ts-mcp/src/tools/*.ts`）。

**compile_context**（`ts-mcp/src/tools/compile-context.ts`）:

| MCP 引数 | zod 制約 | CLI 変換 |
|----------|---------|---------|
| `target_files` | `string[]`、min 1、**必須** | `context` の位置引数 |
| `command` | `string?` | `--command <S>` |
| `granularity` | `enum["document","subnode"]?`（`graph`/`upstream`/`all` は**不正値**） | `--granularity <v>` |
| `outline_only` | `boolean?`（`true` の時のみフラグ付与） | `--outline-only` |
| `sections` | `string?` min 1 | `--sections <v>`（分割せず渡す） |
| `depth` | `int?` ≥ 1 | `--depth <N>`（`String(n)`） |

**observe**（`ts-mcp/src/tools/observe.ts`）:

| MCP 引数 | zod 制約 | CLI 変換 |
|----------|---------|---------|
| `category` | `enum["compile_miss","review_correction","manual_note"]`、必須 | 位置引数 1（**フラグではない** — LGX-COMPAT-001 §4.1 の重要互換ポイント） |
| `message` | `string`、必須 | 位置引数 2 |
| `related_ids` | `string[]?` | 要素ごとに `--related-id <ID>` を繰り返し |
| `target_files` | `string[]?` | 要素ごとに `--target-file <PATH>` を繰り返し |
| `missing_doc` | `string?` | `--missing-doc <v>` |
| `source_glob` | `string?` | `--source-glob <v>` |

注: CLI 側には `--severity` フラグが存在する（LGX-COMPAT-001 §4 #14）が、MCP の observe ツールは公開していない（v3 実測）。

**get_compile_audit**（`ts-mcp/src/tools/get-compile-audit.ts`）:

| MCP 引数 | zod 制約 | CLI 変換 |
|----------|---------|---------|
| `limit` | `int?`、**1..=50**、CLI 側既定 10 | `audit --limit <N>` |

### 2.2 CLI 呼出形式と作業ディレクトリ [補完]

SPEC は「引数として転送」とのみ規定。実形式は **`<bin> --project-root <abs_root> <subcommand...>`** で、`--project-root` は全呼出しに暗黙前置される（根拠: LGX-COMPAT-001 §3「MCP 層は常に `<bin> --project-root <root> <subcommand...>` の形で起動する」、`ts-mcp/src/engine.ts` `runText()`）。project-root は起動時に `path.resolve()` で絶対化される（`ts-mcp/src/index.ts`）。子プロセスの cwd 変更は行わない。

### 2.3 バイナリ解決とサーバ起動引数 [補完] + [要決定]

- [補完] 解決順序: **環境変数 → 起動フラグ `--engine-binary` → 既定名（PATH 探索）**（根拠: LGX-COMPAT-001 §5 補足「`TRACEABILITY_ENGINE_BIN` → `--engine-binary` → 既定名 `traceability-engine`」、`ts-mcp/src/index.ts` parseArgs）。起動引数は `--project-root <PATH>`（既定 `.`）と `--engine-binary <PATH>` の 2 つのみ。
- [要決定] **環境変数名・既定バイナリ名の正準化**。三者が不一致:
  - SPEC-LGX-009 REQ.08 本文: `TRACEABILITY_ENGINE_BIN`
  - 旧実装（chg_to_lexigy 版 `index.ts`）: `LEXIGY_BIN`（既定名 `lexigy`）
  - SPEC-LGX-009 REQ.12: 既定バイナリ名は `legixy` / `legixy.exe`
  - 選択肢: (a) `LEGIXY_BIN` に統一し旧名を alias として併読 (b) `TRACEABILITY_ENGINE_BIN` を維持（REQ.08 文言通り） (c) 両方受理（優先順位を定義）。環境変数名は凍結対象 (a)〜(f) 外だが、配備手順（`.mcp.json` の `env` 設定）に影響するため人間決定が必要。

### 2.4 observe の応答本文形式 [補完]

SPEC 未規定。v3 は CLI の単行 stdout `observation: id=<N> skipped=<true|false>`（正規表現 `/^observation:\s*id=(\d+)\s+skipped=(true|false)/`）をパースし、以下の日本語メッセージへ整形する（根拠: `ts-mcp/src/tools/observe.ts` `parseObserveStdout()` と handler）:
- `skipped=false` → `Observation #<id> を記録（category: <category>）`
- `skipped=true` → `既に記録済み（observation_id=<id>）`

これは厳密には「忠実転送」の例外だが、v3 実在挙動として T-MCP-OB-001/002 で pin されている。_meta は付与しない（REQ.13 で正準化済み）。

### 2.5 get_compile_audit の応答本文形式 [補完]（一部 [要決定]）

SPEC 未規定。v3 は CLI `audit` の JSON 配列（`ContextLogEntry[]`、`ts-mcp/src/types.ts` 参照）を Markdown に整形する（根拠: `ts-mcp/src/tools/get-compile-audit.ts` `formatAuditEntry()`）:

```
### #<id> (<created_at>)
- Files: <input_files をパースしてコンマ結合>
- Command: <input_command または "(none)">
```

- 空配列時は本文 `監査ログはありません。`（_meta は付与する。T-MCP-GA-002 で pin）。
- CLI stdout が JSON としてパース不能な場合は `Rust CLI returned invalid JSON: <err>` の `isError: true` 応答（同ファイル）。これも SPEC 未規定の v3 挙動。
- [要決定] この整形は `resolved_targets` / `upstream_artifacts` / `custom_documents` / `layer_documents` を**表示しない**（情報省略）。MCP-INV-2「フィルタリング・省略なし」との緊張がある。選択肢: (a) v3 形式をそのまま正準化（互換優先、T-MCP-GA-001 が pin 済み） (b) 全フィールド表示へ拡張（INV-2 厳格解釈、ただし v3 から観測可能差分が生じる）。v3 互換の観点では (a) が無難だが SPEC への明文化（人間承認）が望ましい。

### 2.6 エラーメッセージ組立規則 [補完]

REQ.07 の `Rust CLI failed (exit N): <stderr 本文>` の `<stderr 本文>` 組立は以下（根拠: `ts-mcp/src/engine.ts` `toRustCliError()`）:
1. stderr（trim 後、非空なら採用 — Rust anyhow エラーは stderr に出る）
2. → stdout（trim 後）
3. → Node の err.code が文字列（`ENOENT`/`ETIMEDOUT` 等）なら `"<code>: <err.message>"`、それ以外は `err.message`
4. → 全て空なら `unknown error`

exit code は `err.code` が数値ならその値、文字列コード・シグナル死は **-1**（REQ.08 の `exit -1` 正準化と一致）。全 3 ツールが同一の catch パターンで `RustCliError` → `isError: true` 応答に変換し、`RustCliError` 以外の例外は再 throw する。

### 2.7 stdout バッファ上限 [補完]

SPEC 未規定。v3 は `execFile` の `maxBuffer = 10 MiB`（`MAX_STDOUT_BYTES = 10 * 1024 * 1024`、`ts-mcp/src/engine.ts`）。500,000 コードポイント上限（CACHE-INV-3、判定は Rust 側）の最大 UTF-8 長 ≈ 2 MB を十分に上回るため整合する。超過時は Node が子プロセスを kill し、エラー経路（exit -1 系）に入る。

### 2.8 REQ.16 タイムアウトの実装方式 [要決定]

REQ.16（v0.6.0 新設、【v3 差分】）を満たす実装は**どの参照実装にも存在しない**。論点:
- 旧実装 `engine.ts` には `CLI_TIMEOUT_MS = 30_000` のハードコード（`execFile` の `timeout` オプション）が既に存在するが、(1) `LGX_MCP_TIMEOUT_SEC` 環境変数非対応、(2) `0` での無効化非対応、(3) `execFile` の timeout は killSignal（既定 SIGTERM）一発のみで **SIGTERM→猶予 5 秒→SIGKILL の二段階エスカレーションを実現できない**、(4) タイムアウト時は ETIMEDOUT → `exit -1` 扱いとなり、REQ.16 の専用形式 `Rust CLI failed (timeout after {N}s):` を生成しない。
- 従って `child_process.spawn` + 手動タイマ（SIGTERM 送出 → 5 秒後 SIGKILL → `close` イベントで回収確認）への書き換えが必要。Windows（Step 1）では SIGTERM/SIGKILL は Node が `TerminateProcess` 相当へマップするため猶予二段階が実質一段になる点の扱い（許容するか、Windows では即 kill とみなすか）も決定対象。
- 部分出力非転送（全か無か）は spawn でバッファした stdout/stderr をタイムアウト経路で破棄することで実現する。
- なお SPEC/ADR-LGX-010 の「v3 MCP 層にタイムアウト機構は存在しない（無限待ち）」という記述は、参照した `traceability-engine.v3.chg_to_lexigy/ts-mcp/src/engine.ts` の 30 秒ハードコードと**矛盾する**（同リポジトリはリブランド作業中の変更を含む可能性がある）。どちらを「v3 基準」とするかの事実確認も人間に委ねる。

### 2.9 MCP SDK・依存・サーバメタデータ [補完] + [要決定]

- [補完] 依存: `@modelcontextprotocol/sdk ^1.0.0`、`zod ^3.24.0`、Node `>=20.0.0`（engines）。devDeps: typescript ^5.7、vitest ^2.0、@types/node ^22（根拠: `ts-mcp/package.json`）。transport は `StdioServerTransport`、ツール登録は `McpServer#tool(name, description, zodShape, handler)` 形式（`ts-mcp/src/server.ts`）。テスト用 DI として `ServerOptions.engineOverride`（mock RustEngine 注入）を持つ。
- [補完] graceful shutdown: SIGTERM/SIGINT で cleanup（ステートレスのため no-op）して exit 0。`server.connect()` 失敗時は stderr にメッセージを出し exit 1（`ts-mcp/src/index.ts`）。
- [要決定] **サーバ名・パッケージ名**: v3 は `McpServer({ name: "traceability-mcp", version: "0.2.0" })`、npm パッケージ名 `traceability-mcp`、`.mcp.json` のサーバキー `traceability`。legixy で `legixy-mcp` 等へ改名するか。MCP 応答の serverInfo は凍結対象 (a)〜(f) 外だが、既存ユーザの `.mcp.json` 設定互換に影響する。

### 2.10 ツール description 文言 [補完]

SPEC 未規定だが Agent の挙動（呼出しタイミング）に影響する。v3 の値（`ts-mcp/src/tools/*.ts`）:
- `compile_context`: 「ファイルパスから参照すべき上流成果物・ガイドラインを解決する。コードの作成・編集前に呼び出すこと。」
- `observe`: 「ガイドライン不足やレビュー修正を報告する。コード作成後のセルフレビューで不足を発見したら呼び出すこと。」
- `get_compile_audit`: 「過去のコンテキスト解決結果を参照する。『前回何が返されたか』を確認したいとき。」

### 2.11 observe stdout パース失敗時の挙動 [要決定]

`parseObserveStdout()` は形式不一致時に通常の `Error` を throw し、handler は `RustCliError` 以外を再 throw するため、**MCP プロトコルレベルのエラー**（isError 応答ではない）になる（`ts-mcp/src/tools/observe.ts`）。SPEC はこのケースを規定していない。選択肢: (a) v3 挙動を踏襲（CLI 出力契約違反は内部エラー扱い） (b) `isError: true` に包んで Agent が読める形にする。

### 2.12 診断ログの水準 [要決定]

REQ.03 は「stderr への最小限の出力に留める（有無・水準の詳細は DD）」とし DD へ委譲。v3 実装は起動失敗時の 1 行（`MCP server failed to start:`）以外ほぼ無ログ。legixy DD で (a) 完全無ログ踏襲 (b) 呼出トレース（ツール名・所要時間のみ、引数値なし）追加 — を決定する必要がある。いずれでも環境変数値の出力禁止（REQ.03）は遵守。

### 2.13 検証テストの参照実体 [補完]

REQ.04/15 の検証方法が参照するテスト一覧（`ts-mcp/tests/integration.test.ts`）: T-MCP-CC-001〜005（compile_context: 忠実転送+_meta、isError 変換、granularity 列挙、_meta 単一キー+バイト同一、キャッシュブレーク点マーカ保全）、T-MCP-OB-001〜003（observe）、T-MCP-GA-001〜003（audit）、T-MCP-INV-001/002（3 ツール限定、無損失転送）、T-STR/T-PERF。Block B 境界条件 11 テストは `ts-mcp/tests/tools/compile-context.test.ts`。

---

## §3 用語・前提の補完

| 用語 | 定義 | 根拠 |
|------|------|------|
| MCP-INV-1 | Agent Surface 限定 — MCP は compile_context, observe, get_compile_audit の 3 ツールのみ | legixy_foundational_spec.md §10.4（:251） |
| MCP-INV-2 | 忠実な転送 — Rust CLI 出力のフィルタリング・省略なし | 同 §10.4（:252） |
| MCP-INV-3 | Observation 重複排除 — MCP 経由でも重複排除が機能する（実装 owner は feedback 層 = SPEC-LGX-007.REQ.11） | 同 §10.4（:253） |
| MCP-INV-4 | 監査ログ完全性 — compile_context の全呼び出しが記録される | 同 §10.4（:254） |
| STATE-INV-1 | ステートレス性 — 永続的な独自状態を持たず graph.toml（Git 管理）と engine.db（再生成可能キャッシュ）のみ扱う | 同 §10.5（:260） |
| CACHE-INV-3 | 大規模返却時のエラー伝達 — 500,000 文字超過は切り捨てず明示エラー。エラー本文形式は `Error: compile_context result exceeds 500,000 characters. / Current size: <N> characters. / Suggested action: ...` | legixy_cache_spec_v0_1_0.md §4.3, §5.2 |
| CACHE-INV-4 | メタデータ付与の忠実性 — MCP サーバが設定する `_meta` は Rust CLI の出力本文に影響しない | 同 §5.2（:373-375） |
| 凍結対象 (a)〜(f) | (a) サブコマンド名 (b) 位置引数 (c) フラグ名と値 (d) 既定値 (e) 終了コード (f) MCP 3 ツールの CLI マッピング。引数追加は「既存呼出の挙動を一切変えない加算」のみ可（SPEC 改訂+人間承認+ADR 必須） | legixy_cli_compat_reference.md §1（:17）, §7（:164） |
| 終了コードの 3 値 | exit 0 = 成功 / exit 1 = 検証 Error・実行時失敗（成果物を修正すべき）/ exit 2 = 使用法誤り（clap 構文エラー、呼出し側の引数を修正すべき）。全サブコマンド共通規約 | 新リポジトリ SPEC-LGX-004 REQ.04、LGX-COMPAT-001 §3 グローバル規約 |
| legixy-ctx 等の crate 名 | 旧 crate 写像: te-core/te-graph/te-db/te-ctx/te-check/te-nav/te-embed/te-feedback/te-mig/te-cli → legixy-*。名称は**例示**で DD 凍結事項（SPEC-LGX-001 REQ.03）。旧実装リポジトリの実体は `lx-*`（lx-ctx 等） | LGX-COMPAT-001 §2、`traceability-engine.v3.chg_to_lexigy/crates/` |
| Step 1 / Step 2 | Step 1 = Windows 11 native（.exe 配布、開発者が Node.js LTS を導入）/ Step 2 = Ubuntu 24.04 Docker（MCP+CLI+Node 一体 image, x86_64） | NFR-LGX-001.COMPAT.11（:131） |
| PERF.03 正準値 | Step 1 < 300 ms【E-04 で緩和】/ Step 2 < 200 ms（サブノード 100 件、最大粒度指定なし、MCP 経由ベンチ）。注意: NFR §13 暫定表 :237 に「< 200 ms」の内部 drift が残存（SPEC-LGX-009 v0.5.2 変更履歴が NFR 側改訂として別途提起済み） | NFR-LGX-001 §3.2（:81）, §13（:237, :253-256） |
| MCP Result Persistence | Claude Code v2.1.91+ の機能。`_meta["anthropic/maxResultSizeChars"]` を解釈し大きなツール結果をディスク永続化する。旧バージョンでは未知メタデータとして無視（動作影響なし） | legixy_cache_spec_v0_1_0.md §4.1, §4.5、NFR-LGX-001.COMPAT.12 |
| maxResultSizeChars の単位 | Unicode コードポイント数（Rust `.chars().count()` 相当）。SPEC-LGX-003 REQ.13 の「500,000 文字」と同一概念・同一単位。判定とエラー生成は Rust CLI 側、MCP は `_meta` 宣言のみ | QSET-LGX-009 Q1 回答（v3 実測 `te-ctx/src/section_formatter.rs:129-144`）、cache spec §8.5 |
| 前段ループ反復 / QSET / SPP / FCR | DevProc_V4.1 frontend-pass の工程用語（質問票 QSET → 回答 → パッチ SPP → 検査 FCR）。定義は新リポジトリ `docs/DevPorc/03a-frontend-pass.md` 参照 | `legixy/docs/DevPorc/`（vendored） |
| thin forwarder | MCP 層の設計方針 — ビジネスロジックを持たず引数転送と応答変換のみを担う（REQ.09「引数転送ロジックの拡張のみ」の言い換え） | ADR-LGX-010 §1 |
| observe stdout 契約 | CLI `observe` の stdout は単行 `observation: id=<N> skipped=<true|false>` | `ts-mcp/src/types.ts` ObserveStdoutParsed、`src/tools/observe.ts` |
| ContextLogEntry | CLI `audit --json` 出力の 1 エントリ。`input_files` 等 5 フィールドは SQLite TEXT 内 JSON 文字列の二重シリアライズ形（受領側で必要時パース） | `ts-mcp/src/types.ts` |

---

## §4 旧実装からの参考情報

### 4.1 TypeScript MCP サーバ（直接の移植元）

`traceability-engine.v3.chg_to_lexigy/ts-mcp/`

| ファイル | 行数 | 役割 |
|---------|------|------|
| `src/index.ts` | 59 | エントリポイント。引数パース（--project-root / --engine-binary）、env からのバイナリ解決、stdio transport 接続、SIGTERM/SIGINT shutdown |
| `src/server.ts` | 47 | `createServer()` — McpServer 生成と 3 ツール登録（MCP-INV-1）、テスト用 engineOverride DI |
| `src/engine.ts` | 83 | `RustEngine.runText()`（execFile spawn、--project-root 前置、maxBuffer 10 MiB、timeout 30 s ハードコード）、`RustCliError`（exitCode 保持、:67 相当の -1 フォールバック）、`toRustCliError()` |
| `src/types.ts` | 70 | ContextLogEntry / ObserveStdoutParsed / ContextResult（v0.1.0 互換参考） |
| `src/tools/compile-context.ts` | 108 | 6 引数 zod スキーマ、argv 組立順序、成功時 `_meta` 付与（:88-91 相当）、catch で `Rust CLI failed (exit N):`（:98 相当） |
| `src/tools/observe.ts` | 106 | 6 引数、stdout パース、日本語確認メッセージ、_meta 非付与 |
| `src/tools/get-compile-audit.ts` | 98 | limit 1..=50、JSON→Markdown 整形、空ログ文言、invalid JSON エラー |
| `tests/integration.test.ts` | 569 | T-MCP-CC/OB/GA/INV/STR/PERF 系テスト（§2.13 参照） |
| `tests/tools/compile-context.test.ts` | 289 | Block B 境界条件 11 テスト（REQ.15 検証方法の参照実体） |
| `package.json` | — | 依存・engines（§2.9 参照）。パッケージ名 `traceability-mcp` v0.2.0 |

ソース内 Document ID は `DD-LX-008`（旧 DD。新リポジトリには DD 文書自体が無い — DD 段階で再作成対象）。

### 4.2 配備物・設定例

`traceability-engine.v3.chg_to_lexigy/deploy/mcp/` — ビルド済み dist + README。README に `.mcp.json` の実設定例（`command: node`, `args: [dist/index.js, --project-root, <path>]`, `env: { LEXIGY_BIN: <path> }`）、`npm install --omit=dev` 手順、「better-sqlite3 は含まない（engine.db 直接アクセス禁止 = Rust CLI spawn 経由）」の設計判断記録あり。`deploy/INSTALL.md` / `deploy/manual.md` も配備手順の参考。

### 4.3 Rust CLI 側の対向実装（引数受け口）

`traceability-engine.v3.chg_to_lexigy/crates/` の `lx-cli`（context / observe / audit サブコマンド定義、clap）、`lx-ctx`（granularity・outline-only・sections・depth・サイズ判定 CACHE-INV-3）、`lx-feedback`（observe 重複排除 MCP-INV-3、ContextLogEntry / audit）。MCP の引数転送先契約を確認する際はここを参照。

### 4.4 前世代（v0.1.0）実装

`traceability-engine.v3.chg_to_lexigy/old.source/TypeScriptMCP/` — SPEC §2 が「慣例仕様」として挙げる v0.1.0 世代。REQ.01（言語継承）・REQ.09（v0.1.0 からの更新最小化）の判定基準。

### 4.5 経緯文書

- `traceability-engine.v3.chg_to_lexigy/claudedocs/workflow_2026-04-28_v0.4.0-ga-roadmap.md`（REQ.15 の根拠 §2.2 MCP-SYNC）
- `legixy.old.p1/docs/` 配下: `specs/SPEC-LGX-009_MCPサーバ.md`（同一版の旧所在）、`spec-patches/SPP-LGX-009, -011`、`frontend-pass/questionnaires/QSET-LGX-009, -011`、`frontend-pass/check-results/FCR-LGX-009, -011`、`gap-analysis/GAP-LGX-162/168/169/170/171`、`adr/ADR-LGX-010`

---

*生成: 2026-06-12、Claude Code（仕様分析エージェント）。本文書の内容は人間査読まで非正準。*
