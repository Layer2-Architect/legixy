# legixy CLI / MCP 互換リファレンス

| 項目 | 内容 |
|------|------|
| Document ID | LGX-COMPAT-001 |
| Version | 1.1.0 |
| Status | Reference |
| Date | 2026-06-10 |
| 由来 | `traceability-engine.v3`（v0.4.0-alpha4）実バイナリ・MCP 層・設定ファイルの実測 |

---

## 1. 本文書の位置づけ

legixy は、既存実行ファイル `traceability-engine`（v0.4.0-alpha4、Document ID 由来 DD-TE-007/008）と**実行時引数互換**でなければならない。本文書は再設計時に維持すべき CLI / MCP の契約を、実バイナリ・実装ソースから抽出して固定する。

互換の対象は **(a) サブコマンド名 (b) 位置引数 (c) フラグ名と値 (d) 既定値 (e) 終了コード (f) MCP 3 ツールの CLI マッピング** である。出力本文のフォーマット詳細は SPEC-LGX-003/004/006 が規定する。

> ⚠️ 注意: 旧バイナリの自己記述は「traceability-engine v3 — V-DRS core engine」だが、これは旧称。legixy では名称のみ変更し、**引数体系は本文書の通り維持**する。

---

## 2. 実行バイナリ

- バイナリ名（旧）: `traceability-engine`（Windows: `traceability-engine.exe`）
- ワークスペース構成（旧クレート → legixy 命名）: `te-core/te-graph/te-db/te-ctx/te-check/te-nav/te-embed/te-feedback/te-mig/te-cli` → `legixy-core/...`。CLI 統合バイナリは `te-cli`。
- `--version` 出力: `traceability-engine 0.4.0-alpha4`

---

## 3. グローバルオプション（サブコマンドの前に指定）

| オプション | 型 | 既定 | 説明 |
|-----------|----|------|------|
| `--project-root <PATH>` | path | `.` | プロジェクトルート |
| `--json` | flag | false | JSON 出力モード（全コマンド共通） |
| `--models-dir <PATH>` | path | 設定の `model_dir` | ONNX モデルディレクトリ |
| `-h, --help` | flag | — | ヘルプ |
| `-V, --version` | flag | — | バージョン |

MCP 層は常に `<bin> --project-root <root> <subcommand...>` の形で起動する。

> **グローバル規約（終了コード）**: 使用法誤り（引数パーサ層が検出する構文レベルの誤り）は全サブコマンドで exit 2 を返す（clap 既定動作の契約化）。受理済み引数の値の意味的不正・実行時失敗は exit 1。検証結果に基づく終了コード（check の Error>0 → 1 等）は各サブコマンドの規定に従う。（v1.0.1、SPP-LGX-004 承認 2026-06-07 による追記）

---

## 4. サブコマンドと引数（完全一覧）

凡例: `<positional>` = 位置引数、`[--flag]` = 任意フラグ。

| # | サブコマンド | 位置引数 | フラグ | 備考 |
|---|------------|---------|--------|------|
| 1 | `init` | — | `[--force]` | `.trace-engine.toml` を `.bak` 退避し上書き |
| 2 | `migrate` | — | `--from <PATH>`（必須）, `[--to <PATH>]`, `[--dry-run]`, `[--format markdown\|json]`（既定 markdown） | v0.1.0 → 現行 への移行。`--to` 既定は `--project-root` |
| 3 | `check` | — | `[--formal]` | **終了コード: Error 件数>0 で 1、それ以外 0**（G1 ゲート） |
| 4 | `embed` | — | `[--all]`, `[--node <ID>]`（複数可・--all 排他）, `[--force]` | embedding 再生成。`--node`/`--force` は **SPEC 主導の加算的拡張**（v1.1.0、SPEC-LGX-006.REQ.02 / GAP-LGX-120、人間承認 2026-06-10、ADR 記録）。v3 実バイナリは `--all` のみであり、既存呼出 `embed [--all]` の挙動は不変（後方互換）。`embed --json` の出力スキーマは `{generated, skipped, failed, errors[]}`（SPEC-LGX-006.REQ.02） |
| 5 | `drift` | `<artifact_id>` | `[--against <snapshot:LABEL\|snapshot:ID>]` | 省略時は embeddings 現行行と比較 |
| 6 | `report` | — | — | 全リンク類似度＋候補一覧 |
| 7 | `calibrate` | — | `[--buckets <N>]`（既定 10）, `[--recommend]` | 類似度分布／推奨閾値 |
| 8 | `snapshot` | サブコマンド | `create [--label <L>]` / `list` / `delete <target>` | `delete` の target は `snapshot_id` または `label:<LABEL>` |
| 9 | `refresh-subnodes` | — | `[--dry-run]` \| `[--apply]`（排他、既定 dry-run） | 見出しリネーム時のサブノード ID 連鎖反映。`--apply` 時バックアップ `.refresh-bak.{epoch}` |
| 10 | `context` | `<target_files...>` | `[--command <S>]`, `[--granularity document\|subnode]`, `[--outline-only]`, `[--sections <ids>]`, `[--depth <N>]` | MCP `compile_context` の下位層。granularity 既定 document |
| 11 | `impact` | `<start>` | `[--max-depth <N>]` | 順方向探索 |
| 12 | `investigate` | `<start>` | `[--max-depth <N>]` | 逆方向探索 |
| 13 | `feedback` | — | — | check 結果から Observation 自動生成 |
| 14 | `observe` | `<category> <message>` | `[--severity <S>]`, `[--related-id <ID>]`（複数可）, `[--target-file <PATH>]`（複数可）, `[--missing-doc <ID>]`, `[--source-glob <GLOB>]` | MCP `observe` の下位層。詳細は §5 |
| 15 | `audit` | — | `[--limit <N>]`（1..=50、既定 10） | MCP `get_compile_audit` の下位層 |
| 16 | `analyze` | — | — | pending Observation → Proposal 生成 |
| 17 | `proposals` | — | `[--status <S>]` | Proposal 一覧 |
| 18 | `approve` | `<id>`（i64） | — | Proposal 承認（人間のみ） |
| 19 | `reject` | `<id>`（i64） | `--reason <S>`（必須） | Proposal 却下 |

### 4.1 observe の引数スタイル（重要な互換ポイント）

`observe` の CATEGORY と MESSAGE は**位置引数**である。v0.1.0 の `--category` / `--message` フラグは v3 で**廃止**された。legixy はこの位置引数スタイルを維持する。

```
# 現行（維持する形）
legixy observe manual_note "加算オーバーフロー要確認" --related-id DD-CALC-001
# v0.1.0（廃止済み、受け付けない）
# legixy observe --category compile_miss --message "MSG"
```

- CATEGORY 列挙値: `compile_miss` / `review_correction` / `manual_note`
- `--related-id` / `--target-file` は `num_args = 0..`（0 個以上、繰り返し指定可）

---

## 5. MCP 層（3 ツール）と CLI マッピング

MCP サーバ（TypeScript、`ts-mcp`）は Agent Surface として **3 ツールのみ**公開し（MCP-INV-1）、各ツールは引数を CLI へ忠実転送する（MCP-INV-2）。

| MCP ツール | CLI 変換 | 引数（zod スキーマ） |
|-----------|---------|-------------------|
| `compile_context` | `context <target_files...> [--command][--granularity][--outline-only][--sections][--depth]` | `target_files: string[]`(min1), `command?: string`, `granularity?: "document"\|"subnode"`, `outline_only?: bool`, `sections?: string`(min1), `depth?: int`(min1) |
| `observe` | `observe <category> <message> [--related-id...][--target-file...][--missing-doc][--source-glob]` | `category: "compile_miss"\|"review_correction"\|"manual_note"`, `message: string`, `related_ids?: string[]`, `target_files?: string[]`, `missing_doc?: string`, `source_glob?: string` |
| `get_compile_audit` | `audit [--limit]` | `limit?: int` |

補足:
- `compile_context` の返却には `_meta["anthropic/maxResultSizeChars"] = 500000` を付与（CACHE-INV-4 / LGX-EXT-002）。
- MCP→CLI の引数配列は `["context", ...target_files, "--command", v, ...]` のように、**MCP 入力名（snake_case）→ CLI フラグ（kebab-case）** へ機械変換される（`outline_only`→`--outline-only`、`target_files`→位置引数、`missing_doc`→`--missing-doc` 等）。
- バイナリ解決順: 環境変数 `TRACEABILITY_ENGINE_BIN` → サーバ起動フラグ `--engine-binary` → 既定名 `traceability-engine`。legixy ではバイナリ名を変える場合、この解決経路（env / フラグ）で吸収できる。

---

## 6. 設定ファイル（`.trace-engine.toml`）

`init` が生成し、各コマンドが読む設定ファイル。実測スキーマ:

```toml
[project]
name = "..."

[matrix]                       # graph.toml から生成する matrix ビュー設定
format = "markdown"
file = "docs/traceability/matrix.md"
section = "Traceability Matrix"

[id]
pattern = "{type}-{area}-{seq}"
area = "TE"                    # ID エリアコード
seq_digits = 3

[id.types]                     # typecode → ディレクトリ / 拡張子 / ファイル命名規則
SPEC = { dir = "docs/specs/", ext = ".md", file_pattern = "prefix" }
# UC/RB/SEQ/DD/TS/NFR/VAL/TC/SRC ... 同形式

[id.chain]
order = ["UC", "RB", "SEQ", "DD", "TS", "TC", "SRC"]
independent = ["SPEC", "NFR", "VAL"]

[id.document_id]
pattern = "Document ID:"

[semantic]
enabled = true
model = "paraphrase-multilingual-MiniLM-L12-v2"   # legixy 既定: 日英対応の多言語モデル（旧バイナリ実測値は all-MiniLM-L6-v2）
vector_store = "docs/traceability/vectors/"
similarity_threshold = 0.4
drift_threshold = 0.3
link_candidate_threshold = 0.7

[freshness]
enabled = true
method = "mtime"
```

> 設定ファイル名の方針（決定済み、SPEC-LGX-008.REQ.13）: legixy は **`.legixy.toml` を既定**とし、**`.trace-engine.toml` を旧名フォールバックとして読める**。探索順は `.legixy.toml` → `.trace-engine.toml`。旧名のみ存在時は読込み + 移行 Info、両方存在時は `.legixy.toml` 優先。`init` は常に `.legixy.toml` を生成。スキーマは両ファイルで同一。これにより既存 `traceability-engine` プロジェクトを再 init なしで読める。

---

## 7. legixy 再設計での順守事項（チェックリスト）

- [ ] 19 サブコマンド名・別名を維持（`refresh-subnodes` のハイフン含む）
- [ ] 位置引数の順序・個数を維持（特に `observe <category> <message>`、`drift <artifact_id>`、`context <files...>`）
- [ ] フラグ名（kebab-case）と既定値を維持（`--granularity` 既定 document、`--buckets` 既定 10、`--limit` 既定 10/上限 50 等）
- [ ] グローバル `--project-root` / `--json` / `--models-dir` を全コマンドで受理
- [ ] `check` の終了コード規約（Error>0→1, それ以外 0）を維持
- [ ] `snapshot` / `refresh-subnodes` の排他・既定挙動を維持
- [ ] MCP 3 ツール（`compile_context`/`observe`/`get_compile_audit`）の入力スキーマと CLI 変換を維持
- [ ] `compile_context` 返却の `_meta.maxResultSizeChars=500000` を維持
- [ ] MCP のバイナリ解決（`TRACEABILITY_ENGINE_BIN` / `--engine-binary` / 既定名）を維持
- [ ] 設定ファイル探索: `.legixy.toml` 既定 + `.trace-engine.toml` 旧名フォールバック（SPEC-LGX-008.REQ.13）
- [ ] **加算的拡張の規律**: 凍結契約への引数追加は「既存呼出の挙動を一切変えない加算」に限り、SPEC 改訂 + 人間承認 + ADR 記録を必須とする。適用済み: `embed --node/--force`（v1.1.0、GAP-LGX-120）。既存フラグの意味変更・削除は引き続き禁止（次バージョン SPEC 改訂事項）

---

改訂履歴:
- v1.1.0（2026-06-10）: GAP-LGX-120（人間承認）— §4 #4 embed に `--node <ID>`（複数可・--all 排他）/`--force` を加算的拡張として追記、§7 に加算的拡張の規律を追加。既存呼出の挙動は不変（後方互換）。ADR 記録あり
- v1.0.1（2026-06-07）: SPP-LGX-004 承認 — 終了コードのグローバル規約（使用法誤り exit 2）を契約化

Document End

LGX-COMPAT-001 v1.1.0
