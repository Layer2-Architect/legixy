Document ID: UC-LGX-012

# UC-LGX-012: ベースライン凍結管理

## 概要

運用者が embedding ベースラインのライフサイクル（凍結・一覧・削除）を管理する。`snapshot create` で embeddings ストアの現行全行を凍結し、後の standalone ドリフト対比（UC-LGX-013 の `--against snapshot:...`）の基準点とする。マイルストーン到達時・ONNX モデル切替前の「この時点の意味空間」の保全が主目的。

## アクター

- 運用者 / 設定管理者（マイルストーン毎のベースライン凍結と世代管理）
- 設計者（ONNX モデル切替前の旧モデルベースライン保全）
- QA リード（リリース基準点の凍結・不要世代の廃棄判断）

## 事前条件

- プロジェクトが legixy 形式で初期化済（`init` + `migrate` 完了）
- （意味のある凍結を行う場合）`embed --all` 実行済で embeddings ストアに現行行が存在する

## 基本フロー

1. アクターが `legixy snapshot create --label <L>` を実行する（マイルストーン名等を label に付与。`--label` は省略可）
2. システムが embeddings ストアの現行全行を**単一トランザクション**でスナップショット領域へ複製する
3. システムが一意な snapshot_id（`snap-` プレフィクス。内部形式は不透明トークン）を発行し、text / `--json` で返す
4. アクターが `legixy snapshot list` で凍結済ベースラインを確認する（snapshot_id / label / node_count / taken_at を taken_at 降順で一覧）
5. （後続利用）アクターが UC-LGX-013 のドリフト対比で `--against snapshot:<L>` の基準点として参照する
6. 不要になったベースラインをアクターが `legixy snapshot delete <snapshot_id | label:<L>>` で削除する
7. exit 0 で終了

## 代替フロー

- 1a. サブコマンド省略（`legixy snapshot` のみ）: 使用法誤りとして exit 2（LGX-COMPAT-001 §7 排他・既定挙動の維持）
- 2a. 空ストアで create: WARNING（stderr）+ exit 0。複製行 0 件のため**スナップショットは永続化されず**、返却された snapshot_id は以後の list に現れない。`--json` 時は `{"snapshot_id", "label", "node_count": 0, "warning"}`
- 4a. list 0 件: 案内メッセージ（text）/ 空配列（json）+ exit 0
- 6a. delete `label:<L>` で同一 label が複数存在: taken_at 最新の 1 件へ決定論的に解決して削除（同時刻タイブレークは DD 確定。drift の `--against snapshot:<L>` と同一規則）
- 6b. delete snapshot_id 指定で該当行 0 件: text モードは WARNING（stderr）+ exit 0、`--json` 時は `{"snapshot_id", "deleted_rows": 0}` のみを返し WARNING は出力しない
- 6c. delete `label:<L>` で該当 label が存在しない（解決失敗）: ERROR（stderr）+ exit 1（snapshot_id 不在の 6b exit 0 との非対称は意図的 — label 誤り・project-root 誤りを WARNING で覆い隠さない。SPEC-LGX-010.REQ.02）

## 復旧フロー

- **誤削除**: スナップショットの復元手段は提供されない（delete は不可逆）。誤って削除した場合は現行 embeddings ストアから `snapshot create` で新規凍結し直す。ただし削除時点のベースライン内容は失われ、過去時点の再現はできない。運用上は delete 前に `snapshot list` で対象（snapshot_id / label / taken_at）を確認する
- **空ストア create の非永続の見逃し**: 凍結済と誤認しても `snapshot list` に現れないことで検出できる（2a の WARNING / `node_count: 0` が一次通知、list 不在が二次検出）。`embed --all` 実行後に再 create する

## 事後条件

- 成功時（create）: スナップショット領域に現行行の複製と snapshot_id が永続化される（空ストア時 2a を除く）
- 成功時（delete）: 該当スナップショット行が除去される
- embeddings ストアの現行行・graph.toml・成果物ファイルは不変（snapshot は embeddings 本体に触れない。`snapshot list` は読取専用）
- engine.db 不在時に DB ファイルを新規作成しない（空ストア相当の挙動で正常終了）

## 関連不変条件

- SCORE-INV-1: snapshot は content_hash / model_version を含む行を複製し、ベースラインの同一性情報を保持する（SPEC-LGX-010 §4）
- STATE-INV-1（engine.db 永続化）

## 関連 SPEC / NFR

- SPEC-LGX-010 REQ.02（snapshot — ベースライン凍結管理）, REQ.06（list の安定出力。create の snapshot_id / taken_at は決定性対象外）, REQ.07（ストレージ境界・DB 不在時非作成）, REQ.01（共通規約・exit 分類）
- LGX-COMPAT-001 §4 #8（snapshot の凍結済引数契約）
- NFR-LGX-001.OBS.02（出力先 = ログ stderr / 結果 stdout）, OBS.05（終了コード）
