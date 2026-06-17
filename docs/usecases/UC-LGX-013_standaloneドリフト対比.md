Document ID: UC-LGX-013

# UC-LGX-013: standalone ドリフト対比

## 概要

運用者が特定成果物の**現行ファイル内容から生成した embedding** とベースラインとの乖離を `drift = 1.0 − cosine 類似度`（値域 [0.0, 2.0]）で定量対比する。ベースラインは embeddings ストアの現行保存行（既定）または UC-LGX-012 で凍結したスナップショットから選択する。

`check` 内の Drift（検証層が content_hash 変化を Warning 報告する機能。SPEC-LGX-004 REQ.02 / SPEC-LGX-006 REQ.05）とは**別物**であり、本 UC は「運用層が特定成果物をベースラインと定量対比する」機能を扱う。

## アクター

- 設計者（大規模リファクタ・仕様改訂後の特定成果物の意味乖離確認）
- 運用者（ONNX モデル切替・再 embed 後のベースライン対比検証）
- QA リード（マイルストーン凍結ベースラインとの定点対比）

## 事前条件

- プロジェクトが legixy 形式で初期化済、対象成果物が graph.toml に登録済かつ現行ファイルが存在する
- ONNX モデルが解決可能（`--models-dir` フラグ ＞ 環境変数 `LGX_MODELS_DIR` ＞ 環境変数 `TE_MODELS_DIR`（旧名フォールバック）＞ 設定ファイル の順。4 コマンド中 drift のみの実行時依存）
- ベースラインが存在する（`embed --all` 実行済、または UC-LGX-012 で凍結済のスナップショット）

## 基本フロー

1. アクターが `legixy drift <artifact_id> [--against snapshot:<token>] [--json]` を実行する
2. システムが ONNX モデルを解決順序に従って解決する
3. システムが対象成果物の現行ファイル内容を読み込み、embedding を生成する
4. システムがベースラインを選択する（`--against` 省略時 = embeddings ストアの現行保存行。`snapshot:<token>` 指定時 = token をまず label として解決し、解決できなければ snapshot_id とみなす。`snapshot:label:<LABEL>` の明示判別形式も label として解決）
5. システムがベースラインと現行 embedding の次元数一致・model_version 完全一致（SPEC-LGX-006.REQ.10 の判定）を検査する
6. システムが drift = 1.0 − cosine 類似度を算出する
7. text または `--json`（`{"artifact_id", "drift", "baseline_available": true, "baseline_source": "embeddings" | "snapshot:<id>"}`）で出力し exit 0 で終了する

## 代替フロー

- 1a. `snapshot:` プレフィクスを欠く `--against` 値: 実行エラーとして exit 1（アプリ層判定のため exit 2 ではない）
- 2a. モデル解決失敗・モデル読込失敗: 試行内容を stderr に通知して exit 1
- 2b. 旧名 `TE_MODELS_DIR` で解決された場合: stderr に Info で新名 `LGX_MODELS_DIR` を案内し処理続行（両変数同時設定時は `LGX_MODELS_DIR` 優先）
- 3a. `<artifact_id>` が graph.toml に存在しない: ERROR（stderr）+ exit 1
- 3b. graph.toml に存在するが現行ファイルが欠落: ERROR（stderr）+ exit 1（graph.toml が存在を主張するファイルが消えた**壊れた状態**を隠さない。4a の exit 0 との非対称は意図的）
- 4a. ベースライン不在（未 embed のノード、スナップショットに当該行なし）: 正常なライフサイクル状態として INFO（stderr）+ exit 0。`--json` 時は `{"artifact_id", "drift": null, "baseline_available": false}` を stdout に返し、INFO は stderr に併出する
- 4b. `snapshot:<L>` の label が同一複数: taken_at 最新の 1 件へ決定論的に解決（UC-LGX-012 の delete と同一規則）
- 5a. 次元数不一致: 実行エラー exit 1（check 内類似度計算の「集約 Warning + skip」と異なり、明示指定の対比は失敗を隠さない）
- 5b. model_version 不一致（次元は一致）: 実行エラー exit 1（GAP-LGX-186 対応 — SCORE-INV-2 違反状態。同一次元のまま別バージョンへ遷移したケースは次元検査をすり抜けるため、model_version 文字列照合が一次検出）
- 6a. 非有限スコア（NaN/±Inf）発生: exit 1（SPEC-LGX-010 REQ.09）

## 事後条件

- engine.db・graph.toml・成果物ファイルは不変（読取専用。engine.db 不在時も DB ファイルを新規作成しない）
- 成功時: stdout に drift 値が出力される（同一入力 = graph.toml + embeddings ストア + 設定 + 現行ファイル内容 に対して決定論的）

## 典型的な判断材料

- drift ≈ 0.0: ベースラインから意味的変化なし（編集が表層的）
- drift が `drift_threshold`（UC-LGX-011 でキャリブレーション）を超過: 意味乖離が大きい → 下流成果物の追従修正・再 embed・連鎖レビューを検討
- `baseline_available: false`: ベースライン未整備のライフサイクル状態 → `embed --all` または `snapshot create`（UC-LGX-012）の実施
- 5b の model_version 不一致 exit 1: ベースラインが旧モデル由来 → 旧モデルスナップショットとの対比は不能。新モデルで `embed --all` + 再凍結

## 関連不変条件

- SCORE-INV-2（モデルバージョン一致）: model_version 完全一致照合が一次検出、次元検査は補完（SPEC-LGX-010 §4 / GAP-LGX-186）
- SCORE-INV-1 / 出力の決定性（SPEC-LGX-010 REQ.06）

## 関連 SPEC / NFR

- SPEC-LGX-010 REQ.03（drift — standalone ドリフト対比）, REQ.06（決定性）, REQ.07（読取専用・DB 不在時非作成）, REQ.09（非有限スコア）
- SPEC-LGX-006 REQ.10（model_version 完全一致判定）
- LGX-COMPAT-001 §3（`--models-dir` グローバルオプション）, §4 #5（drift の凍結済引数契約）
- NFR-LGX-001.OBS.02（出力先 = ログ stderr / 結果 stdout）, OBS.05（終了コード）
