Document ID: UC-LGX-011

# UC-LGX-011: 閾値キャリブレーション

## 概要

プロジェクト固有の類似度分布に基づき、`.legixy.toml` の 3 閾値（`similarity_threshold` / `drift_threshold` / `link_candidate_threshold`）を調整する根拠を得る。`calibrate` コマンドで全ペア類似度の**ヒストグラム**と**統計サマリ**、および現在の閾値設定を一覧する。

## アクター

- プロジェクトマネージャー / 設定管理者（プロジェクト立ち上げ時のチューニング）
- 設計者（ONNX モデル切り替え時の再キャリブレーション）
- QA リード（false positive / false negative トレードオフ判断）

## 事前条件

- プロジェクトが legixy 形式で初期化済
- `embed --all` 実行済で embeddings テーブルに N 件（N ≥ 2）のエントリがある
- `.legixy.toml` に `semantic.*_threshold` が設定されている

## 基本フロー

1. アクターが `legixy calibrate [--buckets N] [--recommend] [--json]` を実行する
2. システムが embeddings テーブルから全件をロードする
3. システムが全ペア類似度を算出する（`te_embed::compute_all_pair_scores`、O(N²)）
4. 指定バケット数（既定 10、`--buckets` で調整可）のヒストグラムを生成する（`te_embed::histogram`）
5. text モード: ASCII ヒストグラム + 最小/最大/平均 + 現閾値一覧
6. `--json` モード: `{"pairs": N, "min", "max", "mean", "distribution": [...], "thresholds": {...}}`
7. exit 0 で終了

## 代替フロー

- 2a. embeddings が空の場合: `INFO: ベクトルストアが空です。embed --all を実行してください` を出力して exit 0
- 1a. `--buckets 0` 指定時: エラーメッセージ + exit 1
- 3a. 全ペア算出失敗時: anyhow エラーコンテキスト付きで exit 1
- 1b. `--recommend` 指定時: `recommended_thresholds`（パーセンタイル方式）を追加出力する（SPEC-LGX-010.REQ.05）
- 3b. `--recommend` 指定かつペア数 0（空ストア / ノード 1 件 / 全ペア次元不一致 skip）: stderr に INFO「ペア数 0 のため推奨値は算出されません」を出力（`--json` の stdout は汚さない）

## 事後条件

- 標準出力にヒストグラム + 閾値が出力される
- engine.db は不変（読取のみ）
- アクターが出力を根拠に `.legixy.toml` の閾値を編集する判断を得る（閾値変更自体は別手順）

## 典型的な判断材料

- 分布の中央が 0.4–0.6 付近 → `similarity_threshold = 0.40`（既定）で多くのリンクが閾値超過
- 分布が右寄り（0.8–0.95 に集中） → `link_candidate_threshold = 0.9` 程度まで引き上げないと大量の候補でノイズ化
- 分布が左寄り（0.2–0.4） → モデル選択が不適切の可能性、再キャリブレーションが必要

## 関連不変条件

- SCORE-INV-1（決定性保証）: 同一 embeddings → 同一ヒストグラム
- STATE-INV-1（engine.db 永続化）: 読取のみ

## 関連 SPEC / NFR

- SPEC-LGX-010 REQ.05（calibrate — 閾値キャリブレーション）, SPEC-LGX-006 REQ.11（bulk similarity API = calibrate の算出基盤）
- NFR-LGX-001.OBS.02（出力先 = ログ stderr / 結果 stdout）, OBS.05（終了コード）
