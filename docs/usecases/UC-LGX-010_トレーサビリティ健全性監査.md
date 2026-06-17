Document ID: UC-LGX-010

# UC-LGX-010: トレーサビリティ健全性監査

## 概要

運用者がプロジェクト全体のトレーサビリティ健全性を俯瞰する。`report` コマンドで以下 2 種を一覧する:

1. graph.toml 既存の全エッジ（Chain / Custom / ParentChild）の類似度スコア
2. エッジ未定義だが類似度閾値を超える**リンク漏れ候補**（LinkCandidate）

日常開発を止めずに、週次 / マイルストーン / PR レビュー時の監査点検を支援する。

## アクター

- プロジェクトリード / QA リード（週次健全性監査）
- 設計者 / 実装者（新規成果物追加直後のリンク漏れ発見）
- CI システム（PR 毎に `--json` 出力を上流ツールへ渡して差分監視）

## 事前条件

- プロジェクトが legixy 形式で初期化済（`init` + `migrate` 完了）
- `embed --all` 実行済で engine.db の embeddings テーブルに成果物分のエントリがある
- `.legixy.toml` の `semantic.link_candidate_threshold` が設定されている（既定 0.7）

## 基本フロー

1. アクターが `legixy report [--json]` を実行する
2. システムが graph.toml をパースし、embeddings テーブルから全件をロードする
3. システムが以下を算出する:
   a. 全エッジの cosine 類似度（`te_embed::compute_edge_scores`）
   b. 非エッジペアで類似度 ≥ `link_candidate_threshold` のもの（`te_embed::compute_link_candidates`）
4. text モード: 人間可読な階層表示（`=== Traceability Report: All Links ===` + 各行類似度 + `=== Link Candidates ===` + 候補一覧 + 統計サマリ）
5. `--json` モード: `{"links": [...], "candidates": [...], "summary": {...}}` の構造化 JSON
6. exit 0 で終了

## 代替フロー

- 2a. embeddings テーブルが空の場合: `INFO: ベクトルストアが空です。embed --all を実行してください` を出力して exit 0 で終了
- 3a. `compute_edge_scores` / `compute_link_candidates` が失敗した場合: anyhow エラーコンテキスト付きで exit 1

## 事後条件

- 標準出力に報告が出力される（text or JSON）
- engine.db / graph.toml は不変（読取のみ）
- アクターがレビュー結果を基に `observe` で観察事項を記録する選択肢がある（UC-LGX-008 フィードバックループへ連携）

## 関連不変条件

- SCORE-INV-1（決定性保証）: 同一入力 → 同一出力（bulk API の決定論的走査順 + cosine_similarity の決定性で担保）
- STATE-INV-1（engine.db 永続化）: 読取のみ、書込みなし

## 関連 SPEC / NFR

- SPEC-LGX-010 REQ.04（report — トレーサビリティ健全性監査）, SPEC-LGX-006 REQ.11（bulk similarity API = report の算出基盤）
- NFR-LGX-001.OBS.02（出力先 = ログ stderr / 結果 stdout）, OBS.05（終了コード）
