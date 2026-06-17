# legixy 実行環境（deploy/）

legixy CLI（Rust）と MCP サーバ（TypeScript, ts-mcp）の実行環境を組み立てる。
本ディレクトリの成果物は `build-deploy.sh` で**再現生成**する（`bin/` 等はコミットしない）。

## 組み立て

```bash
bash deploy/build-deploy.sh
```

- `cargo build --release -p legixy-cli` → `deploy/bin/legixy`（実行ファイル名は `legixy`）
- ts-mcp を `npm ci && npm run build` → `deploy/ts-mcp/dist` + ランタイム依存（`@modelcontextprotocol/sdk`, `zod`）

前提: Rust（cargo, edition 2021）/ Node.js ≥ 20 / npm。

### embed / 意味層を使う場合（日本語対応 ONNX）

```bash
LEGIXY_ONNX=1 bash deploy/build-deploy.sh
```

- onnx feature 有効の `legixy` を release ビルドし、**日本語対応の多言語モデル
  `paraphrase-multilingual-MiniLM-L12-v2`（384 次元 mean pooling）を `deploy/models/` へ hardlink 配置**する
  （リポジトリ `models/` を底本、470MB の重複なし）。
- 起動ラッパ `deploy/legixy` が同梱モデルを検出して `LGX_MODELS_DIR` を自動設定するため、
  `deploy/legixy --project-root <repo> embed --all` がそのまま実 ONNX 推論で動く（環境変数の手設定不要）。

## 構成

```
deploy/
├── build-deploy.sh    # 再現生成スクリプト
├── legixy             # CLI 起動ラッパ（→ bin/legixy）
├── legixy-mcp         # MCP サーバ起動ラッパ（LGX_BIN を同梱 legixy に設定して ts-mcp を起動）
├── bin/legixy         # release バイナリ（生成物）
├── ts-mcp/            # MCP サーバ（dist + 本番依存）（生成物）
├── config/            # .trace-engine.toml サンプル（生成物）
├── models/            # ONNX モデル配置先の案内（embed/意味層用）
└── README.md
```

## 使い方（現状で動作する範囲）

```bash
# 検証（G1 ゲート）。<repo> は graph.toml を持つプロジェクトルート。
deploy/legixy --project-root <repo> check --formal     # 形式層のみ
deploy/legixy --project-root <repo> check              # 意味層込み（embeddings 未生成なら Info）

# グラフ走査
deploy/legixy --project-root <repo> impact <node-id> [--max-depth N]       # 順方向
deploy/legixy --project-root <repo> investigate <node-id> [--max-depth N]  # 逆方向

# コンテキスト解決（MCP compile_context の下位層。上流成果物を文脈として合成）
deploy/legixy --project-root <repo> context <files...> [--command S] [--granularity document|subnode] \
    [--outline-only] [--sections IDS] [--depth N]

# フィードバックループ（engine.db = .legixy/engine.db を自動作成。ADR-LGX-015）
deploy/legixy --project-root <repo> observe <category> <message> [--related-id ID]...  # 気づき記録
deploy/legixy --project-root <repo> feedback     # check 結果から自動 Observation
deploy/legixy --project-root <repo> analyze      # pending Observation → Proposal
deploy/legixy --project-root <repo> proposals [--status pending|approved|rejected]
deploy/legixy --project-root <repo> approve <id> # Proposal 承認（人間）
deploy/legixy --project-root <repo> reject <id> --reason <S>
deploy/legixy --project-root <repo> audit [--limit N]   # コンテキスト解決履歴（JSON）

deploy/legixy --help    # 全 19 サブコマンドのサーフェス（LGX-COMPAT-001 §3）
```

終了コード（LGX-COMPAT-001 §3 / SPEC-LGX-004.REQ.04）: `0`=成功、`1`=実行時失敗 or check error>0、`2`=使用法誤り。

## ⚠️ 現状（増分 1）と残作業

本実行環境は**増分 4** であり、配線状況は以下のとおり:

| サブコマンド | 状態 |
|---|---|
| `check` / `impact` / `investigate` | ✅ 配線済み（増分1、純グラフ系） |
| `observe` / `feedback` / `analyze` / `proposals` / `approve` / `reject` / `audit` | ✅ 配線済み（増分2、engine.db） |
| `context` | ✅ 配線済み（増分3、ctx ContextCompiler。監査ログを context_log に実書込） |
| `report` / `calibrate` / `snapshot` / `drift` | ✅ 配線済み（増分4、engine.db embeddings。ONNX 不要で動作） |
| `embed` | ✅ 配線済み（増分4）。**実推論には onnx feature ビルド + ONNX モデルが必要**。既定ビルドでは exit 1（モデル/onnx 未対応）を返す |
| `init` / `migrate` | ✅ 配線済み（増分5、legixy-mig。init=プロジェクト初期化、migrate=v0.1.0 移行） |
| `refresh-subnodes` | ✅ 配線済み（増分6、ADR-LGX-023。見出しリネーム→subnode ID 連鎖反映を engine.db embeddings で照合。--apply 時 engine.db を `.refresh-bak.{epoch}` へバックアップ） |

**★ 全 19 サブコマンド配線完了（19/19）。**

残りの未配線サブコマンドはサーフェス宣言済み（clap パース・exit 2 まで契約準拠）。実行すると exit 1 で「未配線」を報告する。

- **engine.db**: `.legixy/engine.db`（正準、ADR-LGX-015）。**書込は常に正準パス**、読取専用コマンドは正準不在時のみ `.trace-engine/engine.db` を読取フォールバック（v3 相互運用）。context は `context_log` に監査ログを書き `audit` が読む。embed は embeddings/snapshots テーブルを使う。
- **★ MCP サーバ（ts-mcp）E2E 完成**: 3 ツール `compile_context` / `observe` / `get_compile_audit` がすべて実 `legixy` バイナリを spawn して動作（実バイナリ E2E は `ts-mcp/tests/e2e.test.ts` で検証）。MCP クライアントからは起動ラッパ `deploy/legixy-mcp` を使う。
- **embed（実 ONNX）**: 既定 deploy は onnx 無効。embed を動かすには `LEGIXY_ONNX=1 bash deploy/build-deploy.sh` で onnx feature ビルドし、ONNX モデルを `deploy/models/` か `LGX_MODELS_DIR` に配置する（`deploy/models/README.txt` 参照）。report/calibrate/snapshot/drift は onnx 不要。

**全 19 サブコマンドが配線完了**: check / impact / investigate / context / observe / feedback / analyze / proposals / approve / reject / audit / embed / drift / report / calibrate / snapshot / init / migrate / refresh-subnodes。MCP サーバ 3 ツールも実バイナリで E2E 動作。`embed` / `refresh-subnodes` の実推論部は onnx feature ビルド + ONNX モデルを要する（`LEGIXY_ONNX=1 bash deploy/build-deploy.sh`）。
