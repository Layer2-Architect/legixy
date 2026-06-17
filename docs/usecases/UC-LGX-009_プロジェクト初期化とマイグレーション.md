Document ID: UC-LGX-009

# UC-LGX-009: プロジェクト初期化とマイグレーション

## 概要

新規プロジェクトの初期化（`init`）と、v0.1.0 マトリクス形式からの移行（`migrate`）を提供する。

## アクター

- 開発者（CLI）

## 事前条件

### init
- カレントディレクトリに `.legixy.toml` が存在しない

### migrate
- v0.1.0 プロジェクトが存在する（`.legixy.toml` + matrix.md + feedback.db）

## 基本フロー

### init

1. アクターが `legixy init` を実行する
2. システムが以下を生成する:
   - `.legixy.toml`（legixy テンプレート、`[graph]` セクション含む）
   - `docs/traceability/graph.toml`（サンプルノード/エッジ付き）
   - 各成果物タイプのディレクトリ
   - `.legixy/` ディレクトリ（.gitignore 付き）

### migrate

1. アクターが `legixy migrate --from <v01_project_root>` を実行する
2. システムが v0.1.0 プロジェクトを読み込む:
   a. `.legixy.toml` を解析
   b. matrix.md（または matrix.json）をパース
3. マトリクスの各行から graph.toml のノードとエッジを生成する:
   - 各 Present な成果物 → `[[nodes]]`
   - チェーン内の隣接ペア → `[[edges]]` (kind = "chain")
4. v0.1.0 の `.legixy.toml` を legixy 形式に変換する
5. feedback.db を engine.db に移行する（テーブル構造をコピー）
6. vectors.bin があれば embeddings テーブルにインポートする
7. 移行レポートを出力する

## 代替フロー

- 2a. init で `.legixy.toml` が既に存在する場合、ERROR を報告する
- 2b. migrate で v0.1.0 プロジェクトが見つからない場合、ERROR を報告する

## 事後条件

- init: 有効な legixy プロジェクト構造が作成される
- migrate: v0.1.0 の全データが legixy 形式に変換される
