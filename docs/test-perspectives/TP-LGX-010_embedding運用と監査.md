Document ID: TP-LGX-010

# TP-LGX-010: embedding 運用・監査（snapshot / drift / report / calibrate）の観点リスト

> TP は **テストケース** ではなく **観点リスト**。「仕様文書に問いかける質問のリスト」として書く。
> GREEN = SPEC に明示的な回答がある（§/REQ を引用）。RED = 沈黙・曖昧 → GAP 起票。

**親**: SPEC-LGX-010
**ステータス**: green

> 2026-06-10 追記（weak GAP fix 適用後）: 残存していた weak/minor GAP も SPEC 改訂（人間裁定 fix・承認 2026-06-10）で全件 closed。全観点 GREEN のためステータスを green に更新。
**最終更新**: 2026-06-09

## 1. 対象スコープ

この TP は SPEC-LGX-010（embedding 運用・監査）の全要求をカバーする。

- 対象: SPEC-LGX-010 §3 REQ.01〜REQ.08、§4 不変条件との関係
- 主対象コマンド: `snapshot`（REQ.02）/ `drift`（REQ.03）/ `report`（REQ.04）/ `calibrate`（REQ.05）
- 関連 SPEC §（委譲先）:
  - SPEC-LGX-006 REQ.04/05/10/11（embedding 生成・bulk similarity API・次元不一致 skip・モデル更新時再生成）= エンジン責務
  - SPEC-LGX-004 REQ.02（check の意味的検証報告）= report との責務境界
  - NFR-LGX-001 OBS.02/04/05、REL.06（出力先・メッセージ言語・終了コード・トランザクション）
  - LGX-COMPAT-001 §3/§4 #5〜#8（凍結済み引数契約）

> **判定方針（品質偏向防止）**: GREEN は具体的 §/REQ 引用を要する。SPEC-006 等の他 SPEC が所有する観点は委譲（引用付き）であり RED 起票しない。本 SPEC は最新（v0.2.0）かつ前段ループ反復 1 で AI Adversary 6 レンズ検証を通過済のため happy path・終了コード分類・空ストア・次元不一致・モデル解決は厚い。敵対的精査パス（2026-06-09）後、横断的非機能領域（並行性・監査証跡・永続化障害回復・入力境界/ローカライズ・保持境界・--against 双方解決失敗・baseline 不変性/TOCTOU）の 7 GAP は NFR-LGX-001（SEC.02/REL.07/REL.01/REL.06/OBS.04/SEC.05）・SPEC-LGX-006 REQ.08・SPEC-010 自身の REQ.02/03/07 所有または v3 正準で被覆され削除。残存 GAP は 2 件: 特殊値の出力契約（B9 / GAP-185）と同一次元・別 model_version baseline の SCORE-INV-2 検出漏れ（V6 / GAP-186）に集約する。

## 2. 観点リスト

### 2.1 境界値

- [ ] 観点 B1: snapshot create の空ストア（複製 0 件）→ 非永続 + WARNING + exit 0（REQ.02）
- [ ] 観点 B2: snapshot list 0 件 → 案内メッセージ（text）/ 空配列（json）（REQ.02）
- [ ] 観点 B3: snapshot delete 該当 0 行（snapshot_id 指定）→ WARNING + exit 0 / json は `deleted_rows:0`（REQ.02）
- [ ] 観点 B4: calibrate `--buckets 0` → exit 1（REQ.05）
- [ ] 観点 B5: calibrate `--buckets` 既定 10 / N の上限・負値・型不正の扱い（REQ.05 / REQ.01 分類）
- [ ] 観点 B6: calibrate ヒストグラム値域 [0.0,1.0] 固定・域外 clamp・上限 1.0 inclusive・min/max/mean は clamp 前生値（REQ.05）
- [ ] 観点 B7: calibrate ペア数 0（空ストア / ノード 1 件 / 全ペア skip）→ recommended_thresholds 非出力 + `--recommend` 時 INFO（REQ.05）
- [ ] 観点 B8: drift 値域 [0.0,2.0]（drift = 1.0 − cosine）（REQ.03）
- [ ] 観点 B9: report / calibrate のスコアに **NaN / ±Inf** が混入した場合の扱い（cosine 計算の特殊値）— calibrate は clamp 規定があるが NaN は clamp 対象として未定義。report の links score / summary 統計（min/max/mean）にも特殊値方針なし
- [ ] 観点 B10: snapshot の保持上限・件数境界（無制限蓄積か、最大件数か、古いものの退避があるか）

### 2.2 エラーハンドリング

- [ ] 観点 E1: 終了コード 3 分類（exit 2 = 構文層 / exit 1 = 値の意味的不正・実行時失敗 / exit 0 = 結果が空）（REQ.01）
- [ ] 観点 E2: drift `--against` で `snapshot:` プレフィクス欠如 → exit 1（アプリ層 reject、clap 層でないため exit 2 でない）（REQ.03）
- [ ] 観点 E3: drift モデル全解決失敗・読込失敗 → exit 1 + 試行内容 stderr 通知（REQ.03）
- [ ] 観点 E4: drift `<artifact_id>` が graph.toml に不在 → ERROR + exit 1（REQ.03）
- [ ] 観点 E5: drift 現行ファイル欠落（graph.toml は存在主張）→ ERROR + exit 1（壊れた状態。非対称性注記）（REQ.03）
- [ ] 観点 E6: drift 次元不一致 → exit 1（明示対比は失敗を隠さない。check の skip+Warning とは別）（REQ.03）
- [ ] 観点 E7: snapshot delete の label 解決失敗 → ERROR + exit 1（REQ.02）
- [ ] 観点 E8: snapshot サブコマンド省略 → 使用法誤り exit 2（REQ.02）
- [ ] 観点 E9: drift `--against snapshot:<token>` が **label にも snapshot_id にも一致しない**（双方解決失敗）場合の挙動（baseline-absent exit 0 か / 不正参照 exit 1 か）— REQ.03 は「label 解決失敗 → snapshot_id とみなす」までは述べるが、その snapshot_id も存在しない最終分岐が未定義
- [ ] 観点 E10: report / calibrate のスキップ発生時の集約 Warning 1 件（件数 + 代表理由）を stderr（REQ.04 / REQ.05）
- [ ] 観点 E11: snapshot create のトランザクション途中中断（プロセス kill / 電源断 / ディスクフル / 権限エラー）からの回復・部分書込みの不整合防止
- [ ] 観点 E12: 部分成功の扱い（drift 複数対象は単一 artifact なので N/A。report/calibrate は全件集計でスキップ集約に収束）

### 2.3 状態遷移・ライフサイクル

- [ ] 観点 S1: snapshot create → list → delete の基本ライフサイクル（REQ.02、§1.3 UC-012）
- [ ] 観点 S2: DB 不在 ≡ 空ストア。読取系 + delete は DB 非作成（REQ.07）
- [ ] 観点 S3: create のみ DB 初期化（不在時新規作成 + スキーマ）+ スナップショット領域書込み（REQ.07）
- [ ] 観点 S4: 誤 snapshot 削除からの復旧フロー（§1.3 UC-012 が代替フローとして要求）
- [ ] 観点 S5: 作成後のスナップショット baseline の不変性（再 embed / モデル更新後も snapshot 内容は凍結されるか）
- [ ] 観点 S6: snapshot delete と drift `--against snapshot:` の TOCTOU（解決〜読取の間に対象 snapshot が削除された場合）

### 2.4 並行性

- [ ] 観点 C1: 同時 `snapshot create` ×2（同一ストアからの並行複製）の atomicity・一意 snapshot_id 衝突
- [ ] 観点 C2: `snapshot create`（書込み）と読取系 3 コマンドの並行実行（SQLite ロック競合・busy 時挙動）
- [ ] 観点 C3: `snapshot create` と `snapshot delete` の並行・interleaving
- [ ] 観点 C4: drift のベースライン読取中に embed --all が走る場合の一貫性（読取スナップショット境界）

### 2.5 永続化

- [ ] 観点 P1: snapshot create は単一トランザクション（REQ.02）。embeddings 本体行は不変（REQ.07）
- [ ] 観点 P2: 読取系 + delete の engine.db / graph.toml / 成果物ファイル非破壊（REQ.07）
- [ ] 観点 P3: snapshot が content_hash / model_version を含む行を複製しベースライン同一性情報を保持（§4 SCORE-INV-1）
- [ ] 観点 P4: トランザクション失敗・ディスクフル・部分書込みからの回復保証（NFR.REL.06 への委譲か独自規定か）
- [ ] 観点 P5: スナップショットのテーブル/カラム構造（→ DD 委譲、REQ.07 明示）

### 2.6 バージョニング・互換性（モデルバージョン遷移）

- [ ] 観点 V1: LGX-COMPAT-001 §4 #5〜#8 の引数体系（サブコマンド・位置引数・フラグ・既定値）維持（REQ.01）
- [ ] 観点 V2: グローバルオプション `--project-root`/`--json`/`--models-dir` を 4 コマンド全受理（REQ.01）
- [ ] 観点 V3: snapshot_id `snap-` プレフィクス凍結・内部形式は不透明トークン（DD 凍結）（REQ.02）
- [ ] 観点 V4: drift モデル解決順 `--models-dir` > `LGX_MODELS_DIR` > `TE_MODELS_DIR`（旧名フォールバック + Info 案内）> 設定（REQ.03【v3 差分】）
- [ ] 観点 V5: 両環境変数同時設定時は `LGX_MODELS_DIR` 優先（REQ.03）
- [ ] 観点 V6: drift `--against snapshot:` の baseline が**旧 model_version で凍結**され現行モデルが別バージョン（**同一次元**）の場合の意味的妥当性（SCORE-INV-2 違反だが次元一致のため次元不一致 Error では捕捉されない）。スナップショット baseline と現行 embedding の model_version 不一致の扱いが未定義
- [ ] 観点 V7: 旧名 `TE_MODELS_DIR` フォールバックの撤去時期（→ 将来 SPEC 改訂、DD へ申し送り。REQ.03）

### 2.7 入力検証

- [ ] 観点 I1: drift `--against` の 3 形式（`snapshot:LABEL` / `snapshot:ID` / `snapshot:label:LABEL` 明示判別）の受理（REQ.03）
- [ ] 観点 I2: snapshot delete target の `snapshot_id` / `label:<LABEL>` 二形式の受理（REQ.02）
- [ ] 観点 I3: label の一意性は強制しない（同一 label 複数可）。重複時 taken_at 最新へ決定論的解決（REQ.02）
- [ ] 観点 I4: 構文層（clap）と値の意味層（アプリ）の検証段階分離（REQ.01）
- [ ] 観点 I5: label 文字列の境界（空文字 `--label ""`、極端長、Unicode、`label:` を含む label 等の曖昧入力）の扱い

### 2.8 ロギング・観測性・監査

- [ ] 観点 L1: INFO/WARNING/ERROR は stderr、結果は stdout（REQ.01、NFR.OBS.02【v3 差分】）
- [ ] 観点 L2: `--json` 時の INFO 併出（stderr）で stdout の機械可読性を保つ（REQ.03 baseline 不在 / REQ.05 pairs=0）
- [ ] 観点 L3: drift モデル解決失敗時の「試行内容」stderr 通知の情報十分性（REQ.03）
- [ ] 観点 L4: **運用操作（snapshot create / delete）の監査証跡** — 誰がいつどの snapshot を作成/削除したかの記録。SPEC タイトルは「運用・**監査**」だが、監査対象は report（トレーサビリティ健全性監査）であり、運用操作自体の監査ログ（observations / audit テーブルへの記録）要求が無い。MCP-INV-4（compile_context 全呼出記録）が前例
- [ ] 観点 L5: SPEC が導入する具体 WARNING/INFO 文言（「ストアが空のため永続化されません」「ペア数 0 のため推奨値は算出されません」等）の**ローカライズ方針**（NFR.OBS.04 = 日本語 primary。キー化か文字列直埋めか、修正示唆文言か）
- [ ] 観点 L6: 機密情報のログ非混入（embedding ベクトル本体・成果物 CONFIDENTIAL 内容がログに出ないか）

### 2.9 FFI / 境界 API（CLI 契約 — LGX-COMPAT-001）

- [ ] 観点 F1: MCP（Agent Surface）に 4 コマンドを非公開（MCP-INV-1）（REQ.01、§4）
- [ ] 観点 F2: 4 コマンドの CLI 引数契約が LGX-COMPAT-001 §4 #5〜#8 と一致（REQ.01）
- [ ] 観点 F3: `--json` スキーマの各コマンド定義（REQ.02〜05 に各キー明示）

### 2.10 領域固有観点

- [ ] 観点 D1: 読取系 3 コマンド（drift/report/calibrate）の決定性（同一入力→同一バイト出力）（REQ.06）
- [ ] 観点 D2: snapshot create は決定性対象外（snapshot_id/taken_at が時刻依存）（REQ.06）
- [ ] 観点 D3: snapshot list の taken_at 降順安定出力（同時刻タイブレーク → DD、label 解決と同一規則）（REQ.02/06）
- [ ] 観点 D4: report = 計測 / check = 判定 の責務非重複（report に severity 概念なし）（REQ.04、SPEC-LGX-004 REQ.02）
- [ ] 観点 D5: calibrate 推奨閾値 = パーセンタイル方式（p25 / 1.0−p90 / p75）凍結・補間式は DD（REQ.05）
- [ ] 観点 D6: 4 コマンドは SPEC-LGX-006 REQ.11 bulk API の consumer（類似度ロジック非再実装）（REQ.08）
- [ ] 観点 D7: report links は両端点 embedding 存在 + 次元一致のエッジのみ算出（スキップ可視化）（REQ.04）
- [ ] 観点 D8: drift の次元不一致 Error は SCORE-INV-2 違反状態の検出手段（§4）

## 3. RED / GREEN 判定

| 観点 | 判定 | SPEC / 委譲先で回答 | 関連 GAP |
|---|---|---|---|
| 2.1 B1 | GREEN | SPEC-LGX-010 REQ.02（空ストア非永続 + WARNING + exit 0） | — |
| 2.1 B2 | GREEN | REQ.02（list 0 件: 案内 / 空配列） | — |
| 2.1 B3 | GREEN | REQ.02（delete 0 行: WARNING+exit 0 / json deleted_rows:0） | — |
| 2.1 B4 | GREEN | REQ.05（`--buckets 0` → exit 1） | — |
| 2.1 B5 | GREEN | REQ.05（既定 10）+ REQ.01（型不正は exit 2 / 値の意味不正は exit 1） | — |
| 2.1 B6 | GREEN | REQ.05（clamp・[0,1] 固定・inclusive・生値統計） | — |
| 2.1 B7 | GREEN | REQ.05（pairs=0 で非出力 + `--recommend` INFO） | — |
| 2.1 B8 | GREEN | REQ.03（drift 値域 [0.0,2.0]） | — |
| 2.1 B9 | GREEN | (NaN/Inf 方針なし。calibrate clamp は域外実数のみ、NaN 未定義。report 統計も未定義。`--json` の drift が NaN を取ると JSON 不正)【2026-06-10 解消: SPEC 改訂適用（人間承認）により GAP closed】 | GAP-LGX-185 |
| 2.1 B10 | GREEN | REQ.02（snapshot は list/delete による手動管理。v3 正準化 = 無制限・手動。容量は NFR §11 非目標 + 運用者責任）。敵対的精査: 保持上限/退避は v3 既定（無制限・手動）を超える feature-addition であり残存 GAP でない → GAP-LGX-187 削除 | — |
| 2.2 E1 | GREEN | REQ.01（exit 0/1/2 分類） | — |
| 2.2 E2 | GREEN | REQ.03（プレフィクス欠如 → exit 1） | — |
| 2.2 E3 | GREEN | REQ.03（モデル解決失敗 → exit 1 + stderr） | — |
| 2.2 E4 | GREEN | REQ.03（artifact_id 不在 → exit 1） | — |
| 2.2 E5 | GREEN | REQ.03（現行ファイル欠落 → exit 1 + 非対称性注記） | — |
| 2.2 E6 | GREEN | REQ.03（次元不一致 → exit 1） | — |
| 2.2 E7 | GREEN | REQ.02（label 解決失敗 → exit 1） | — |
| 2.2 E8 | GREEN | REQ.02（サブコマンド省略 → exit 2） | — |
| 2.2 E9 | GREEN | REQ.03（ベースライン不在＝「スナップショットに当該行なし」を exit 0 で扱う規則が、token が既存 snapshot に解決しない最終分岐を自然に包含。`snapshot:` プレフィクス欠如の gross-typo は別途 exit 1 で reject 済）。敵対的精査: baseline-absent exit 0 の既存規則で被覆、残余は DD の分類詳細 → GAP-LGX-188 削除 | — |
| 2.2 E10 | GREEN | REQ.04 / REQ.05（集約 Warning 1 件 stderr） | — |
| 2.2 E11 | GREEN | REQ.02（単一トランザクション = all-or-nothing）+ SPEC-LGX-006 REQ.08（途中中断で部分不整合なし、§2 参照文書）+ NFR-LGX-001 REL.01（WAL 電源断耐性）/ REL.06（トランザクション境界）。敵対的精査: 障害回復は単一トランザクション要求 + NFR REL.01/06 が所有 → GAP-LGX-183 削除 | — |
| 2.2 E12 | GREEN | REQ.04/05（全件集計はスキップ集約に収束。drift は単一対象） | — |
| 2.3 S1 | GREEN | REQ.02、§1.3 | — |
| 2.3 S2 | GREEN | REQ.07（DB 不在 ≡ 空ストア、非作成） | — |
| 2.3 S3 | GREEN | REQ.07（create のみ DB 初期化） | — |
| 2.3 S4 | GREEN | §1.3（UC-012 復旧フロー要求として明示）※UC で具体化 | — |
| 2.3 S5 | GREEN | REQ.02（複製 = 独立コピーをスナップショット領域へ）+ REQ.07（書込みは create のみ、embeddings 本体行不変、読取系非破壊）。複製コピー + create 以外は snapshot 領域を触らない = 構造的に凍結。§1.1「ベースライン凍結管理」。敵対的精査: REQ.02+REQ.07 の合成で ALREADY_ANSWERED → GAP-LGX-189 削除 | — |
| 2.3 S6 | GREEN | TOCTOU は並行制御の一例であり NFR-LGX-001 SEC.02（WAL+busy_timeout）/ REL.07（busy_timeout 上限・超過時 Error）が所有。GAP-189 自身が「GAP-181 の並行モデルに依存」と認める = 並行性 GAP の重複。敵対的精査: NFR 所有 + GAP-181 重複 → GAP-LGX-189 削除 | — |
| 2.4 C1 | GREEN | NFR-LGX-001 SEC.02（複数プロセス同時書込みで破損しない、WAL+busy_timeout、同時実行ストレステスト）/ REL.07（busy_timeout 上限 5000ms・超過時 Error・無限リトライ禁止）。敵対的精査: 並行モデルは NFR 所有（TP §4 委譲方針に整合） → GAP-LGX-181 削除 | — |
| 2.4 C2 | GREEN | NFR-LGX-001 SEC.02 / REL.07（SQLite ロック競合・busy_timeout 上限） | — |
| 2.4 C3 | GREEN | NFR-LGX-001 SEC.02 / REL.07（書込み interleaving の WAL 排他） | — |
| 2.4 C4 | GREEN | NFR-LGX-001 SEC.02（WAL の読取一貫性）/ REL.07 | — |
| 2.5 P1 | GREEN | REQ.02（単一トランザクション）+ REQ.07（本体行不変） | — |
| 2.5 P2 | GREEN | REQ.07（読取系 + delete 非破壊） | — |
| 2.5 P3 | GREEN | §4 SCORE-INV-1 + REQ.02（content_hash/model_version 複製） | — |
| 2.5 P4 | GREEN | REQ.02（単一トランザクション）+ SPEC-LGX-006 REQ.08（部分不整合なし）+ NFR-LGX-001 REL.01（WAL 電源断耐性）/ REL.06（トランザクション境界）。敵対的精査: 回復保証は NFR REL.01/06 + 単一トランザクション要求が所有 → GAP-LGX-183 削除 | — |
| 2.5 P5 | GREEN | REQ.07（テーブル構造は DD 委譲を明示） | — |
| 2.6 V1 | GREEN | REQ.01（LGX-COMPAT-001 §4 #5〜#8 維持） | — |
| 2.6 V2 | GREEN | REQ.01（グローバル 3 オプション全受理） | — |
| 2.6 V3 | GREEN | REQ.02（snap- 凍結 / 内部形式 DD） | — |
| 2.6 V4 | GREEN | REQ.03【v3 差分】（4 段解決順） | — |
| 2.6 V5 | GREEN | REQ.03（同時設定時 LGX_MODELS_DIR 優先） | — |
| 2.6 V6 | GREEN | (同一次元・別 model_version の baseline 妥当性が未定義。§4 は SCORE-INV-2 検出を REQ.03 次元不一致 Error のみとするが、同一次元のモデル切替は捕捉できず偽 drift が静かに通る)【2026-06-10 解消: SPEC 改訂適用（人間承認）により GAP closed】 | GAP-LGX-186 |
| 2.6 V7 | GREEN | REQ.03（撤去は将来 SPEC 改訂 / DD 申し送り） | — |
| 2.7 I1 | GREEN | REQ.03（--against 3 形式受理） | — |
| 2.7 I2 | GREEN | REQ.02（delete target 二形式） | — |
| 2.7 I3 | GREEN | REQ.02（label 非一意 + 最新優先決定論解決） | — |
| 2.7 I4 | GREEN | REQ.01（構文層 / 意味層分離） | — |
| 2.7 I5 | GREEN | REQ.03（曖昧性解消は明示判別形式 `snapshot:label:<LABEL>` で既定義 = `label:` 含み label の escape 経路を提供）。許容文字・長さ境界・空文字は clap/DD の入力検証詳細。敵対的精査: 中核の曖昧性は REQ.03 で解消済、残余は DD → GAP-LGX-184 削除 | — |
| 2.8 L1 | GREEN | REQ.01、NFR.OBS.02 | — |
| 2.8 L2 | GREEN | REQ.03 / REQ.05（json INFO 併出） | — |
| 2.8 L3 | GREEN | REQ.03（試行内容 stderr 通知） | — |
| 2.8 L4 | GREEN | §1.1（監査 = report の健全性監査と本文で明示）+ §1.2 スコープ（運用操作の監査ログは含まない）。MCP-INV-4 は compile_context（Agent Surface）固有であり Admin Surface 4 コマンドへの前例ではない（MCP-INV-1 でサーフェス分離）。敵対的精査: スコープ明示 + サーフェス別、操作監査証跡は feature-addition → GAP-LGX-182 削除 | — |
| 2.8 L5 | GREEN | NFR-LGX-001 OBS.04（日本語 primary・修正示唆）がメッセージ言語方針を所有。NFR §11 非目標で i18n 明示遅延。キー化/直埋めは DD。敵対的精査: ローカライズ方針は NFR OBS.04 所有 → GAP-LGX-184 削除 | — |
| 2.8 L6 | GREEN | NFR-LGX-001 SEC.05（API キー非ログ化・マスキング）+ SEC 系横断要件が機密非混入を所有。embedding/成果物本文の非混入は NFR.SEC の cross-cutting 領域で SPEC-010 固有判断でない。敵対的精査: NFR SEC 所有 → GAP-LGX-182 削除 | — |
| 2.9 F1 | GREEN | REQ.01、§4 MCP-INV-1 | — |
| 2.9 F2 | GREEN | REQ.01（LGX-COMPAT-001 §4） | — |
| 2.9 F3 | GREEN | REQ.02〜05（各 json キー明示） | — |
| 2.10 D1 | GREEN | REQ.06（読取系決定性） | — |
| 2.10 D2 | GREEN | REQ.06（create 決定性対象外） | — |
| 2.10 D3 | GREEN | REQ.02/06（list 安定降順 + 同規則タイブレーク DD） | — |
| 2.10 D4 | GREEN | REQ.04、SPEC-LGX-004 REQ.02（計測/判定分離） | — |
| 2.10 D5 | GREEN | REQ.05（パーセンタイル凍結 / 補間 DD） | — |
| 2.10 D6 | GREEN | REQ.08、SPEC-LGX-006 REQ.11（consumer 委譲） | — |
| 2.10 D7 | GREEN | REQ.04（算出可能エッジのみ + 可視化） | — |
| 2.10 D8 | GREEN | §4 SCORE-INV-2（次元不一致 Error が検出手段） | — |

**集計**: 総観点 71 / GREEN 71 / RED 0。敵対的精査パス（2026-06-09）で 7 件削除（181/182/183/184/187/188/189）、SPEC 改訂適用（2026-06-10）で 2 件（185/186）解消

## 4. ステータスの決定

敵対的精査パス（2026-06-09）後に RED 観点 2 件（B9 = GAP-LGX-185 / V6 = GAP-LGX-186）が残存していたが、SPEC-LGX-010 v0.2.1（人間承認 2026-06-10）の適用により両 GAP は closed、該当観点は GREEN に再評価された。全観点 GREEN のため本 TP のステータスは `green`。

> **委譲（GAP 非起票）の明示**: 以下は他 SPEC / DD が所有するため RED 起票せず GREEN 委譲とした。
> - 類似度・ヒストグラム集計ロジックの実体 → SPEC-LGX-006 REQ.11（D6）
> - 次元不一致ペアの skip + 集約 Warning（check 経路）→ SPEC-LGX-006 REQ.04（E10 は本 SPEC の report/calibrate への適用として GREEN）
> - check の閾値判定・severity → SPEC-LGX-004 REQ.02（D4）
> - 終了コードの一般定義・ログ出力先・メッセージ言語 → NFR-LGX-001 OBS.02/04/05（L1）
> - スナップショットのテーブル/カラム構造・タイブレーカー実装・補間式 → DD（P5, D3, D5）
> - UC-012/013 の新規生成 → UC フェーズ（本 TP の対象外。§1.3 は予告として GREEN）

## 5. 観点ナレッジベース参照

- `docs/perspectives/core-perspectives.md` §境界値 / §エラーハンドリング / §状態遷移 / §並行性 / §バージョニング・互換性 / §永続化 / §入力検証 / §ライフサイクル / §ロギング・観測性 / §FFI・境界 API
- `docs/perspectives/ux-perspectives.md` §エラー・例外の UX（CLI の終了コード・メッセージ品質）/ §永続化と同期

## 6. 履歴

| 日付 | 変更内容 |
|---|---|
| 2026-06-08 | 初版作成。観点 71 件（GREEN 62 / RED 9）。RED は並行性・運用監査証跡・永続化障害回復・入力境界/ローカライズ・特殊値・モデルバージョン遷移・保持境界・--against 双方解決失敗・baseline 不変性/TOCTOU に集中。GAP-LGX-181〜189 を起票 |
| 2026-06-09 | 敵対的精査パス: 削除 7 件 / 維持 2 件 |
| 2026-06-10 | SPEC 改訂適用（人間承認 2026-06-10、spec-change-proposals/2026-06-09_genuine-gap-resolution-proposals.md）: GENUINE GAP に対応する観点を GREEN 化。GAP-157 は人間裁定・案A、GAP-064 は GraphDag 新設 + DocumentId 行欠落 Error、GAP-120 は凍結契約への加算的拡張承認。ADR-LGX-001〜008 起票 |
| 2026-06-10 | weak GAP 解消適用（人間裁定 fix・承認 2026-06-10、spec-change-proposals/2026-06-10_weak-gap-resolution-proposals.md）: 残存 RED 観点（weak/minor）を全て GREEN 化。個別裁定: GAP-085=打ち切り Info 追加 / GAP-135=永続保持 / GAP-169=タイムアウト導入【v3 差分】。ADR-LGX-009〜011 起票。open GAP 0 となり本 TP は green |
