Document ID: UC-LGX-007

# UC-LGX-007: embedding 生成とドリフト検出

## 概要

成果物の embedding を ONNX モデルで生成し、engine.db に格納する。既存 embedding との比較でドリフト（意味的乖離）を検出する。

## アクター

- 開発者（CLI）
- CI システム

## 事前条件

- ONNX モデル（model.onnx + tokenizer.json）が model_dir に配置されている
- engine.db が存在する（なければ自動作成）

## 基本フロー

### embedding 生成（`embed` コマンド）

1. アクターが `legixy embed [--all] [--subnodes]` を実行する
2. システムが graph.toml から全ノードを取得する
3. 各ノードについて:
   a. ファイル内容のハッシュ（SHA-256）を計算する
   b. engine.db の既存ハッシュと比較し、変更がなければスキップする（SCORE-INV-1）
   c. 変更がある場合、前処理（EmbeddingPreprocessor）を適用する
   d. ONNX モデルで embedding を生成する（トークン化 → 推論 → Mean Pooling → L2 正規化）
   e. engine.db の embeddings テーブルに格納する
4. `--subnodes` が指定された場合、サブノードの embedding も生成する

### ドリフト検出（`drift` コマンド）

`legixy drift <artifact_id>` による standalone ドリフト対比は **UC-LGX-013（standalone ドリフト対比）** が正準として扱う（ベースライン選択・`--against`・終了コード・代替フローは UC-013 が規定）。本 UC は embedding 生成（`embed`）に専念し、drift は UC-013 を参照する。

## 代替フロー

- 2a. ONNX モデルが存在しない場合、ERROR を報告する
- 3b. `--all` の場合、ハッシュ比較をスキップして全ノードを再生成する

## 事後条件

- engine.db の embeddings テーブルが更新される
- モデルバージョンが記録される（SCORE-INV-2）

## 関連不変条件

- SCORE-INV-1: ハッシュ一致保証
- SCORE-INV-2: モデルバージョン一致
