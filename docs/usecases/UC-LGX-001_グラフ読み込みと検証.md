Document ID: UC-LGX-001

# UC-LGX-001: グラフ読み込みと検証

## 概要

開発者または CI が `legixy check` を実行し、graph.toml で定義された有向グラフの形式的整合性を検証する。

## アクター

- 開発者（CLI 実行者）
- CI システム（自動実行）

## 事前条件

- `.legixy.toml` が存在する
- `graph.toml` が存在する
- graph.toml にノードとエッジが定義されている

## 基本フロー

1. アクターが `legixy check --formal` を実行する
2. システムが `.legixy.toml` を読み込み、設定を解析する
3. システムが `graph.toml` を読み込み、有向グラフをメモリ上に構築する
4. システムが以下の形式検証を実行する:
   a. 各ノードの ID 形式が `{type}-{area}-{seq}` パターンに適合するか
   b. 各ノードの `path` が指すファイルが存在するか
   c. 各ファイルの先頭に `Document ID: {ID}` が記載されているか
   d. チェーン順序に沿って上流成果物が欠落していないか
   e. graph.toml に定義されていないがディレクトリに存在する孤立ファイルがないか
   f. 有向グラフにサイクルが存在しないか（DAG 制約: CTX-INV-4）
   g. SPEC ファイル先頭の `## ID Changelog` または `.legixy.toml` の `[[id_changelog]]` で `redefined` 宣言された ID の引用箇所を下流成果物から列挙する（SPEC-LGX-004.REQ.11、`[id_changelog].enabled = true` のとき）
   h. SPEC 定義表の数値リテラル / キーワードと下流引用文の不整合を検出する（SPEC-LGX-004.REQ.12、`[id_semantic_mismatch].enabled = true` のとき）
   i. SPEC サブノード（定義側）と下流サブノード（引用側）の embedding 類似度を計算し、閾値未満を検出する（SPEC-LGX-004.REQ.13、`[id_semantic_drift].enabled = true` のとき）
5. システムが検証結果を出力する（ERROR / WARNING / INFO / OK）
6. ERROR が 0 件の場合、終了コード 0 を返す

## 代替フロー

- 4a. `--formal` なしで実行された場合、意味的検証（embedding ベース）も追加で実行する（UC-LGX-007 参照）
- 3a. `graph.toml` が存在しない場合、ERROR を報告して終了する
- 2a. `.legixy.toml` が存在しない場合、ERROR を報告して終了する
- 4g-A. SPEC ファイル内に `## ID Changelog` セクションが存在し、`change = redefined` のエントリを含む場合:
  1. システムが Changelog 表を解析し、再定義された ID の集合を取得する
  2. graph 上 chain 下流の各ドキュメントノードを行単位でスキャンし、`| {ID} |` パターンの引用を検出する
  3. 各引用箇所をファイルパス + 行番号 + 引用行先頭で WARNING `IdRedefined` として報告する
  4. アクターは出力リストを基に、引用箇所の妥当性を 1 件ずつ確認・修正する
- 4g-B. `[id_changelog].enabled = false`（デフォルト）の場合、本検査はスキップされる（v0.2.0 と同等の出力）
- 4h-A. `[id_semantic_mismatch].enabled = true` かつ Changelog 宣言が無いまま SPEC 定義の数値が変化した場合、引用文との数値不整合を WARNING または INFO として報告する
- 4i-A. `[id_semantic_drift].enabled = true`（Phase 2 Block F）の場合:
  1. システムは graph 上の全サブノード（is_subnode=1）について、その本文に同一 ID 引用パターンが含まれるサブノードのペアを構築する
  2. SPEC 親に属するサブノードを「定義側」、それ以外を「引用側」として分類する
  3. 各 (定義側, 引用側) ペアで engine.db に格納された embedding の cosine_similarity を計算する
  4. 類似度が閾値（既定 0.75）未満であれば `IdSemanticDrift` WARNING として報告する
  5. 1 ID あたりの比較は `max_pairs_per_id`（既定 50）で打切る
  6. embedding が存在しないサブノードはスキップ（embed --all 未実行は致命扱いしない）

## 事後条件

- 検証結果が標準出力に表示される
- ERROR がある場合は終了コード 1 が返される

## 関連不変条件

- CTX-INV-2: グラフ整合性
- CTX-INV-4: DAG 制約

## 関連要求

- SPEC-LGX-004.REQ.01〜REQ.10: 既存の形式・意味的検証
- SPEC-LGX-004.REQ.11: ID Changelog 宣言検出（IdRedefined）— ISSUE-001 対応
- SPEC-LGX-004.REQ.12: ID 引用整合性検査（IdSemanticMismatch）— ISSUE-001 対応
- SPEC-LGX-004.REQ.13: サブノード単位意味類似度検査（IdSemanticDrift）— Phase 2 Block F、ISSUE-001 機能 C
