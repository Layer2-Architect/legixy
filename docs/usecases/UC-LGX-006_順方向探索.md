Document ID: UC-LGX-006

# UC-LGX-006: 順方向探索（impact）

## 概要

上流成果物（SPEC や UC）を変更した際に、影響を受ける下流成果物を特定する。

## アクター

- 開発者（CLI）
- Linear Agent コンテナ（影響評価時）

## 事前条件

- graph.toml が存在する
- 起点となる成果物 ID が指定される

## 基本フロー

1. アクターが `legixy impact <node-id> [--max-depth <n>]` を実行する
2. システムが起点ノードから有向グラフを順方向（下流方向）に BFS 走査する
3. `--max-depth` が指定されている場合、その深度で走査を打ち切る
4. 結果を以下の形式で返却する:
   - visited: 影響を受ける全ノード（走査順）
   - depth_map: 各ノードの起点からの距離

## 代替フロー

- 1a. `--max-depth` 未指定の場合、全深度を走査する
- 2a. 起点ノードが graph.toml に存在しない場合、空の結果（visited 空・depth_map 空）を返して exit 0（SPEC-LGX-005.REQ.05: 存在しない起点はエラーではない）

## 事後条件

- 走査結果が標準出力に表示される
- グラフの状態は変更されない（読み取り専用操作）

## 関連仕様

- LEGIXY-SPEC-001 §4: impact（エンジン機能）
- LEGIXY-SPEC-001 §5: 双方向探索
