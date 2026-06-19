# legixy ユーザーマニュアル（日本語）

[English / 英語版](manual.en.md)

> legixy は有向グラフ主体の**トレーサビリティエンジン**。成果物間のリンク（SPEC → … → ソース）を
> 機械検証可能なデータとして保持し、ローカルの凍結埋め込みモデルでリンク間の**意味的逸脱（drift）**を観察する。
>
> バージョン: 0.4.0-alpha4 · バイナリ: `legixy`（Windows は `legixy.exe`）· 19 サブコマンド + 3 ツールの MCP サーバー。

---

## 目次

1. [概念](#1-概念)
2. [インストール](#2-インストール)
3. [埋め込みモデル（ONNX）](#3-埋め込みモデルonnx)
4. [プロジェクトの初期化](#4-プロジェクトの初期化)
5. [設定（`.legixy.toml`）](#5-設定legixytoml)
6. [二層の検証](#6-二層の検証)
7. [コマンドリファレンス](#7-コマンドリファレンス)
8. [グローバルオプションと終了コード](#8-グローバルオプションと終了コード)
9. [JSON 出力](#9-json-出力)
10. [MCP サーバー（Claude Code 統合）](#10-mcp-サーバーclaude-code-統合)
11. [環境変数](#11-環境変数)
12. [トラブルシューティング](#12-トラブルシューティング)
13. [ライセンス](#13-ライセンス)

---

## 1. 概念

- **一次データはグラフ。** 全成果物（仕様・ユースケース・設計・テスト・ソース）は
  `docs/traceability/graph.toml` のノード。エッジは明示的で、人間が読め、Git で差分が追える。
  legixy はグラフを自動で書き換えない。編集するのは人間で、不整合は `check` が事後検出する。
- **二つの層。** *形式層*は決定論的（ID 形式・ファイル存在・連鎖整合性・循環なし）でモデル不要。
  *意味層*は埋め込みで意味の逸脱とリンク候補を観察し、**Warning / Info しか出さない**。
  **報告するのは逸脱であって異常ではない。**
- **計器の凍結。** 埋め込みモデルは意図的に固定する。対象（文書）と計器（モデル）が両方動くと、
  観測した逸脱がどちらに由来するか区別できなくなるため。
- **Admin / Agent サーフェス分離。** 19 コマンドのうち MCP に露出するのは 3 つだけ。
  `approve` / `reject` ほかは CLI 専用＝人間専用。

ID 体系は `{type}-{area}-{seq}`（例 `SPEC-LGX-001`）。Markdown 成果物はファイル名先頭の ID で、
ソースは先頭付近の `// Document ID: SRC-…` コメントで結ばれる。`DD-…#anchor` 形式の**見出し単位
サブノード**で節単位の追跡もできる。

---

## 2. インストール

### 2.1 ビルド済みバイナリ（推奨）

**Linux（x86_64）/ Windows（x86_64）**向けの onnx 有効ビルド済みバイナリが各 GitHub Release に
添付される。install スクリプトはバイナリ**と**埋め込みモデル（§3）の両方を取得するため、意味層が
そのまま動く。

> **Linux の glibc 要件**: ビルド済み Linux バイナリは新しめの ONNX Runtime をリンクするため
> **glibc ≥ 2.39**（Ubuntu 24.04+ / Fedora 39+ / RHEL 10+）が必要。古い distro（RHEL 8/9・Ubuntu 22.04
> 以前）ではソースからビルド（§2.2）。Windows 10/11 はこの考慮は不要。

**Linux / macOS:**

```bash
curl -fsSL https://raw.githubusercontent.com/Layer2-Architect/legixy/main/install.sh | bash -s -- --repo Layer2-Architect/legixy
# clone 済みなら:
bash install.sh --repo Layer2-Architect/legixy [--version vX.Y.Z] [--prefix ~/.local] [--no-model]
```

**Windows（PowerShell）:**

```powershell
$env:LEGIXY_REPO = "Layer2-Architect/legixy"
irm https://raw.githubusercontent.com/Layer2-Architect/legixy/main/install.ps1 | iex
# clone 済みなら:
powershell -ExecutionPolicy Bypass -File install.ps1 -Repo Layer2-Architect/legixy [-Version vX.Y.Z] [-NoModel]
```

install スクリプトは legixy をプレフィックス（`~/.local/share/legixy` または
`%LOCALAPPDATA%\legixy`）に配置し、起動ラッパを PATH に通し、モデルを取得する。macOS のビルド済み
バイナリはまだ無いので、ソースからビルドする（§2.2）。

### 2.2 ソースからビルド

最近の Rust ツールチェーン（edition 2021）が必要。MCP サーバーには Node.js ≥ 20 も要る。

```bash
git clone https://github.com/Layer2-Architect/legixy
cd legixy

# 形式層のみ（モデル不要・ONNX 不要）:
cargo build --release -p legixy-cli
./target/release/legixy --version          # → legixy 0.4.0-alpha4

# 意味層込みのフルビルド（`ort` クレート経由で ONNX Runtime をリンク）:
cargo build --release -p legixy-cli --features onnx
```

CLI + MCP サーバー（+ `LEGIXY_ONNX=1` 時はモデル）をまとめる再現可能な組み立てスクリプトもある:

```bash
LEGIXY_ONNX=1 bash deploy/build-deploy.sh    # 成果物は deploy/ 配下
```

---

## 3. 埋め込みモデル（ONNX）

意味層にはローカルモデル **`paraphrase-multilingual-MiniLM-L12-v2`**（日本語を含む多言語・
384 次元・mean pooling）が要る。2 ファイルからなる:

```
<model-dir>/paraphrase-multilingual-MiniLM-L12-v2/
├── model.onnx        （約 500 MB）
└── tokenizer.json
```

モデルはリポジトリにもリリースアーカイブにも含めない（サイズ + 別ライセンス）。入手方法は 3 通り:

1. **install スクリプト**（自動取得）— §2.1 参照。
2. **取得スクリプト:** `bash scripts/fetch-model.sh` → `models/…` に取得。
3. **手動:** `tokenizer.json`（リポジトリ直下）と `onnx/model.onnx` を
   <https://huggingface.co/sentence-transformers/paraphrase-multilingual-MiniLM-L12-v2> から取得。
   上流に `onnx/model.onnx` が無ければ書き出す:
   ```bash
   pip install "optimum[onnxruntime]" sentence-transformers
   optimum-cli export onnx -m sentence-transformers/paraphrase-multilingual-MiniLM-L12-v2 \
     --task feature-extraction <model-dir>/paraphrase-multilingual-MiniLM-L12-v2
   ```

**モデルの探索順**（最初に見つかったものを使用）:
`--models-dir <PATH>` → 環境変数 `LGX_MODELS_DIR` → 設定の `model_dir` →
`<project-root>/models/paraphrase-multilingual-MiniLM-L12-v2`。

その上で設定の `[semantic] enabled = true` で意味層を有効化する。

---

## 4. プロジェクトの初期化

```bash
legixy init                       # .legixy.toml / docs/traceability/graph.toml /
                                  # .legixy/engine.db と文書ディレクトリを生成
```

`docs/traceability/graph.toml` に成果物とリンクを登録:

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

その後:

```bash
legixy check --formal             # 決定論的チェック（CI 向け）
legixy embed --all                # embedding 生成（モデル要）
legixy check                      # 形式層 + 意味層
```

---

## 5. 設定（`.legixy.toml`）

`init` が `.legixy.toml` を生成する。旧名 `.trace-engine.toml` は fallback として読まれる
（探索順 `.legixy.toml` → `.trace-engine.toml`。旧名のみ存在時は読込み + 移行を促す info 出力）。
スキーマ（抜粋）:

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

`[id.chain]` が legixy を方法論非依存にする要 — 独自の typecode / 順序に置き換えられる
（例 `["REQ","DES","IMPL","TEST"]`）。閾値はプロジェクト固有。既定を信じず `legixy calibrate` で校正する。

---

## 6. 二層の検証

| 層 | コマンド | カテゴリ | モデル |
|---|---|---|---|
| **形式層**（決定論） | `legixy check --formal` | ID 形式・ファイル存在・連鎖整合性・鮮度(mtime)・循環なし(DAG)・孤児ファイル・サブノード ID 形式 ほか（代表例。`check --formal` はサブノード一意性/親/DAG・ID 再定義・未解決エッジ等も発行する） | 不要 |
| **意味層**（埋め込み） | `legixy check` | SemanticSimilarity（リンク間が閾値未満→Warning）・LinkCandidate（非リンクが閾値超過→Info）・Drift（content_hash 不一致→Warning） | 要 |

`check` の終了コード: **Error 件数 > 0 → 1、それ以外 0**（G1 ゲート）。意味層の所見は Warning/Info で、
それ自体ではゲートを落とさない。

---

## 7. コマンドリファレンス

呼び出し形式: `legixy [グローバルオプション] <サブコマンド> [引数]`。位置引数は `<山括弧>`、
任意フラグは `[角括弧]`。

| # | コマンド | 位置引数 | フラグ | 備考 |
|---|---------|---------|--------|------|
| 1 | `init` | — | `[--force]` | 既存 `.legixy.toml` を `.bak` 退避し上書き |
| 2 | `migrate` | — | `--from <PATH>`（必須）, `[--to <PATH>]`, `[--dry-run]`, `[--format markdown\|json]` | v0.1.0 プロジェクトの移行。`--to` 既定は `--project-root` |
| 3 | `check` | — | `[--formal]` | `--formal` で形式層のみ、無指定で形式層 + 意味層。Error>0 で exit 1 |
| 4 | `embed` | — | `[--all]`, `[--node <ID>]`（複数可・`--all` と排他）, `[--force]` | embedding の(再)生成。JSON: `{generated, skipped, failed, errors[]}` |
| 5 | `drift` | `<artifact_id>` | `[--against snapshot:<LABEL>\|snapshot:<ID>]` | `--against` 省略時は現行 embedding と比較 |
| 6 | `report` | — | — | 全リンク類似度 + リンク候補一覧 |
| 7 | `calibrate` | — | `[--buckets <N>]`（既定 10）, `[--recommend]` | 類似度分布 / 推奨閾値 |
| 8 | `snapshot` | `create`\|`list`\|`delete` | `create [--label <L>]`, `delete <target>` | `delete` の target は snapshot id または `label:<LABEL>` |
| 9 | `refresh-subnodes` | — | `[--dry-run]` \| `[--apply]`（排他・既定 dry-run） | 見出しリネームをサブノード ID に連鎖反映。`--apply` 時 `.refresh-bak.{epoch}` にバックアップ |
| 10 | `context` | `<target_files...>` | `[--command <S>]`, `[--granularity document\|subnode]`, `[--outline-only]`, `[--sections <ids>]`, `[--depth <N>]` | MCP `compile_context` の下位層。granularity 既定 `document` |
| 11 | `impact` | `<start>` | `[--max-depth <N>]` | 順方向（下流）探索 |
| 12 | `investigate` | `<start>` | `[--max-depth <N>]` | 逆方向（上流）探索 |
| 13 | `feedback` | — | — | `check` 結果から Observation を自動生成 |
| 14 | `observe` | `<category> <message>` | `[--severity <S>]`, `[--related-id <ID>]`*, `[--target-file <PATH>]`*, `[--missing-doc <ID>]`, `[--source-glob <GLOB>]` | MCP `observe` の下位層。category ∈ `compile_miss`\|`review_correction`\|`manual_note`。(* 複数可) |
| 15 | `audit` | — | `[--limit <N>]`（1–50・既定 10） | MCP `get_compile_audit` の下位層 |
| 16 | `analyze` | — | — | pending Observation → Proposal を生成 |
| 17 | `proposals` | — | `[--status pending\|approved\|rejected]` | Proposal 一覧 |
| 18 | `approve` | `<id>` | — | Proposal 承認（人間のみ） |
| 19 | `reject` | `<id>` | `--reason <S>`（必須） | Proposal 却下 |

`observe` の注意: `<category>` と `<message>` は**位置引数**（旧 `--category` / `--message` フラグは
廃止）。例:

```bash
legixy observe manual_note "オーバーフロー要確認" --related-id DD-CALC-001
```

フィードバックループ（`observe` → `analyze` → `proposals` → `approve`/`reject`）と監査ログは
`.legixy/engine.db`（SQLite, WAL、自動生成）を使う。

---

## 8. グローバルオプションと終了コード

グローバルオプションはサブコマンドの**前**に置く:

| オプション | 既定 | 意味 |
|---|---|---|
| `--project-root <PATH>` | `.` | プロジェクトルート |
| `--json` | off | JSON 出力（全コマンド） |
| `--models-dir <PATH>` | 設定の `model_dir` | ONNX モデルディレクトリ |
| `-h, --help` / `-V, --version` | — | ヘルプ / バージョン |

終了コード: **0** 成功 · **1** 実行時失敗または `check` の Error>0 · **2** 使用法誤り（引数解析）。
ログは stderr、結果は stdout に出る。

---

## 9. JSON 出力

`--json` はグローバルフラグで、サブコマンドの前に置く:

```bash
legixy --json check --formal
legixy --json embed --all          # {generated, skipped, failed, errors[]}
legixy --json audit --limit 20
```

スクリプト連携・CI ゲートに使う（終了コードを見て、stdout を parse する）。

---

## 10. MCP サーバー（Claude Code 統合）

MCP サーバー（`legixy-mcp`、TypeScript / Node.js ≥ 20）は**3 ツールだけ**を公開し、各呼び出しを
CLI に忠実転送する:

- **`compile_context`** ― 対象ファイルパスを渡すと上流成果物（UC / DD / SPEC …）を遡って Markdown で返す。*コードを書く前のガイダンス。*
- **`observe`** ― エージェントが気づいた欠落・矛盾を `engine.db` に記録（重複排除）。
- **`get_compile_audit`** ― `compile_context` の呼出履歴を返す。*何を参照して書いたか*が申告でなくログで残る。

`approve` も `check` も `init` も MCP には**無い**。サーバーはステートレスで、書込みは常に `legixy`
バイナリの spawn 経由、出力は無加工で転送する。

`.mcp.json` で接続する（install 済みの起動ラッパが `LGX_BIN` を設定してくれる）:

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

あるいはサーバーを直接起動し、`LGX_BIN` でバイナリを指す:

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

`compile_context` の返却には `_meta["anthropic/maxResultSizeChars"] = 500000` が付く。

---

## 11. 環境変数

| 変数 | 利用元 | 意味 |
|---|---|---|
| `LGX_MODELS_DIR` | CLI | ONNX モデルディレクトリ（設定より優先・`--models-dir` の下位） |
| `LGX_BIN` | MCP サーバー | サーバーが spawn する `legixy` バイナリのパス |
| `LEGIXY_ONNX=1` | `deploy/build-deploy.sh` | `onnx` feature 付きビルド + モデル配置 |

---

## 12. トラブルシューティング

- **「ONNX モデルが見つかりません …」** ― 意味層がモデルを解決できない。install スクリプト、または
  `bash scripts/fetch-model.sh` を実行するか、`--models-dir` を渡す / `LGX_MODELS_DIR` を設定する。
  形式層（`check --formal`）はモデル不要。
- **onnx 無しビルドで `embed` が exit 1** ― 既定ビルドは ONNX を含まない。`--features onnx` で
  再ビルドする（または `LEGIXY_ONNX=1 bash deploy/build-deploy.sh`）。
- **起動時の共有ライブラリエラー（Linux）** ― ONNX Runtime をロードできる必要がある。`bin/legixy` を
  直接でなく、同梱の `legixy` 起動ラッパ（`bin/` に `LD_LIBRARY_PATH` を通す）を使う。
- **`onnxruntime.dll` が見つからない（Windows）** ― `legixy.exe` と `onnxruntime.dll` を同じ
  ディレクトリに置く（install スクリプトがそうする。そのディレクトリが PATH に入る）。
- **MCP サーバーがバイナリを見つけられない** ― `LGX_BIN` に `legixy` の絶対パスを設定するか、
  `legixy-mcp` 起動ラッパを使う。
- **意味層の所見がノイズに見える** ― 閾値は普遍定数ではない。`legixy calibrate --recommend` で
  プロジェクトの実分布から校正する。意味保存/意味破壊編集への計器応答を再現可能に測る手順は
  [`CALIBRATION.md`](../../CALIBRATION.md) を参照。

---

## 13. ライセンス

Apache-2.0。`LICENSE` と `NOTICE` を参照。ONNX Runtime（MIT）と埋め込みモデル（Apache-2.0,
sentence-transformers）は別途取得され、それぞれのライセンスに従う。
