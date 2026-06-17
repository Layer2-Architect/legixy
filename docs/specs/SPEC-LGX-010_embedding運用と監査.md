Document ID: SPEC-LGX-010

# SPEC-LGX-010: embedding 運用・監査

| 項目 | 内容 |
|------|------|
| Document ID | SPEC-LGX-010 |
| Version | 0.2.1 |
| Status | Accepted（前段ループ反復 1 完了） |
| Date | 2026-06-08 |
| Classification | CONFIDENTIAL |
| 親文書 | SPEC-LGX-001, LGX-COMPAT-001 §4 |
| 対応 NFR | NFR-LGX-001.OBS.02（stdout/stderr 分離）, OBS.05（終了コード） |
| 対応 UC | UC-LGX-010（report = 健全性監査）, UC-LGX-011（calibrate = キャリブレーション）, UC-LGX-012（snapshot = ベースライン凍結ライフサイクル）, UC-LGX-013（drift = standalone 対比）。UC-012/013 は本 SPEC 受理後の UC フェーズで新規生成する（運用者アクター。粒度は §1.3） |

---

## 1. 本文書の位置づけ

### 1.1 目的

embedding ストアを基盤とする運用・監査系 4 コマンド — `snapshot`（ベースライン凍結管理）、`drift`（standalone ドリフト対比）、`report`（健全性監査）、`calibrate`（閾値キャリブレーション）— の要求を定義する。

本 SPEC は前段ループ反復 1 の決定（QSET-LGX-001 Q1 回答、2026-06-07）により新設された。対象 4 コマンドは旧 traceability-engine v3 に**実装のみ存在し SPEC レベルの要求仕様が無かった**機能であり（v3 SPEC-TE-006 では bulk API の consumer として言及されるのみ）、本 SPEC は v3 実測挙動の正準化を基本とし、v3 からの逸脱は全て **【v3 差分】** として明示する。

### 1.2 スコープ

**含む:** 4 コマンドの入出力・引数・既定値・終了コード、スナップショット永続化の要求、check（SPEC-LGX-004）との出力責務境界
**含まない:** embedding 生成と bulk similarity API の実体（→ SPEC-LGX-006）、check 内の閾値判定・severity 付き報告（→ SPEC-LGX-004）、閾値の数値既定（→ NFR-LGX-001 / `.legixy.toml`）、MCP 経由の公開（行わない。§3 REQ.01）

### 1.3 対応 UC と SPEC-LGX-001 との連動

本 SPEC の 4 コマンドは UC-LGX-010（report = 健全性監査）/ UC-LGX-011（calibrate = キャリブレーション）/ UC-LGX-012（snapshot = ベースライン凍結ライフサイクル）/ UC-LGX-013（drift = standalone 対比）に写像する。UC-012/013 は本 SPEC 受理後の UC フェーズで新規生成する。

UC-LGX-012/013 の新規生成に伴い、SPEC-LGX-001 REQ.01/REQ.02 の網羅宣言「UC-LGX-001〜011」は「001〜013」へ再改訂する必要がある。これは UC フェーズ着手時（本 SPEC FCR ACCEPTED 後、ハードルール 9）に SPP-LGX-001（次反復）として処理する。SPEC-LGX-001 v0.5.0 変更履歴は既に「snapshot/drift 系 UC は本 SPEC 受理後の UC フェーズで生成」と予告済みであり、本再改訂は網羅宣言を誠実に保つもの。

**UC 粒度（運用者アクター）**: UC-012（snapshot）= create / list / delete ＋ 復旧フロー（誤 snapshot 削除、空ストア create の非永続）。UC-013（drift）= baseline 有無 × `--against` 3 形式 × モデル解決失敗 × 次元不一致 × **model_version 不一致（GAP-LGX-186）** × 現行ファイル欠落（REQ.03）を代替フローとする。基本は UC-010/011 と同粒度で AI が起草し UC レビューで調整する。

---

## 2. 参照文書

- LGX-COMPAT-001 §3（グローバルオプション）, §4 #5（drift）, #6（report）, #7（calibrate）, #8（snapshot）— 凍結済み引数契約
- SPEC-LGX-001 REQ.02（機能カテゴリ 9）, REQ.08（Surface 分離）
- SPEC-LGX-006 REQ.04（類似度の用途）, REQ.05（check 内ドリフト検出）, REQ.11（bulk similarity API = 本 SPEC との境界面）
- SPEC-LGX-004 REQ.02（check の意味的検証報告）
- NFR-LGX-001.OBS.02（ログは stderr、結果は stdout）, OBS.05（終了コード 0=OK / 1=Error / 2=使用法誤り）
- UC-LGX-010（トレーサビリティ健全性監査）, UC-LGX-011（閾値キャリブレーション）, UC-LGX-012（スナップショット）, UC-LGX-013（standalone ドリフト対比）
- QSET-LGX-001 / 004 / 006 の回答（2026-06-07）, QSET-LGX-010 の回答（2026-06-08）
- v3 実測の根拠（v0.4.0-alpha4 時点の旧リポジトリ traceability-engine.v3）: `crates/te-cli/src/commands/{snapshot,drift,report,calibrate}.rs`, `crates/te-cli/src/main.rs`, `crates/te-embed/src/{drift,similarity,store,orchestrator}.rs`, `crates/te-db/src/connection.rs`

---

## 3. 要求事項

### SPEC-LGX-010.REQ.01: Surface 所属と共通規約

**内容:** 本 SPEC の 4 コマンドは **Admin Surface 専用**であり、MCP（Agent Surface）には公開しない（MCP-INV-1）。共通規約:

- **グローバルオプション**: `--project-root` / `--json` / `--models-dir` を 4 コマンド全てで受理する（LGX-COMPAT-001 §3 / §7 の維持。`--models-dir` が機能上意味を持つのは drift のみ。REQ.03）
- `--json` 指定時は構造化 JSON を標準出力へ返す（v3 実測の正準化。各コマンドの JSON スキーマは REQ.02〜05）
- 引数体系（サブコマンド名・位置引数・フラグ・既定値）は LGX-COMPAT-001 §4 #5〜#8 を維持する
- **終了コードの分類**（NFR-LGX-001.OBS.05、v3 実測と整合）:
  - **使用法誤り（exit 2）** = 引数パーサ層が検出する構文レベルの誤りのみ（未知フラグ・必須位置引数/サブコマンドの欠落・型不正。clap 既定動作。QSET-LGX-004 Q1 回答）
  - **実行エラー（exit 1）** = 受理済みフラグ・引数の値の意味的不正（例: `--buckets 0`、`--against` の形式不正）および実行時失敗
  - **正常終了（exit 0）** = 空ストア・ベースライン欠如等の「結果が空」のケースを含む
- **診断メッセージの出力先**: INFO / WARNING / ERROR は **stderr** に出力する（NFR-LGX-001.OBS.02「ログは stderr、結果は stdout」）。
  **【v3 差分】** v3 は report / calibrate の text モード INFO を stdout に出力していた（`report.rs` / `calibrate.rs` の println、drift は stderr）。NFR 整合とパイプ出力の汚染防止のため stderr に統一する（整合性・堅牢性・実現性の 3 軸比較で採用）。

**根拠:** SPEC-LGX-001 REQ.08、LGX-COMPAT-001 §3/§4、NFR-LGX-001.OBS.02/OBS.05、QSET-LGX-001 Q1 / QSET-LGX-004 Q1 回答
**検証方法:** MCP サーバのツール一覧に 4 コマンドが含まれないこと、グローバルオプション受理の E2E テスト、終了コード分類テスト

### SPEC-LGX-010.REQ.02: snapshot — ベースライン凍結管理

**内容:** `snapshot` は embedding ベースラインのライフサイクルを管理する。`create` / `list` / `delete` のいずれかのサブコマンド指定は**必須**であり、省略は使用法誤り（exit 2、LGX-COMPAT-001 §7「排他・既定挙動の維持」）。

- `snapshot create [--label <L>]`: embeddings ストアの現行全行をスナップショット領域へ複製し、一意な snapshot_id を発行する。複製は単一トランザクションで行う。
  - snapshot_id は `snap-` プレフィクスを持ち（SPEC 凍結）、スナップショット領域内で**一意**である（SPEC 要求）。プレフィクス以降の内部形式（v3 実測: epoch ミリ秒 + 乱数）は consumer から不透明トークンとして扱い、生成方式は **DD で凍結**する。delete target が snapshot_id を受理する外部契約（LGX-COMPAT-001 §4 #8）は内部形式に依存しない（QSET-LGX-010 Q1-b）
  - **label の一意性は強制しない**（v3 実測）。同一 label の複数スナップショットが存在しうる
  - 空ストア時: WARNING（stderr）を通知し exit 0。複製行 0 件のため**スナップショットは永続化されず**（v3 実測: 行複製方式のため。非永続を確定 = QSET-LGX-010 Q1-a）、返却された snapshot_id は以後の list に現れない。`--json` 時は `{"snapshot_id", "label", "node_count": 0, "warning"}` を返す。`warning` 文言には「ストアが空のため永続化されません」を含める（DD で確定）
- `snapshot list`: 既存スナップショットの一覧（snapshot_id / label / node_count / taken_at）を taken_at 降順で出力する（同時刻のタイブレーク規則は DD で確定）。`--json` 時は同フィールドの配列。0 件時は案内メッセージ（text）/ 空配列（json）
- `snapshot delete <target>`: `<target>` は `snapshot_id` または `label:<LABEL>`。
  - label 解決に失敗した場合は ERROR（stderr）+ exit 1
  - snapshot_id 指定で該当行 0 件の場合: text モードでは WARNING（stderr）+ exit 0、`--json` 時は `{"snapshot_id", "deleted_rows": 0}` のみを返し WARNING は出力しない（v3 実測の正準化）
- **label 解決の規則**（delete の `label:<L>` / drift の `--against snapshot:<L>` 共通）: 同一 label が複数存在する場合は **taken_at 最新の 1 件**に解決する（v3 実測）。同時刻タイブレークは DD で確定する。SPEC レベルの要求は「決定論的に 1 件へ解決する」のみ（QSET-LGX-010 Q1-c）。DD のタイブレーカーは **label 解決（本規則）と list 降順安定出力（REQ.06）の両方に同一規則**（例: snapshot_id の全順序による安定タイブレーク）を適用すること

**根拠:** LGX-COMPAT-001 §4 #8、v3 実測（`crates/te-cli/src/commands/snapshot.rs`, `crates/te-embed/src/store.rs` の label 解決）
**検証方法:** create→list→delete の E2E テスト、空ストア・不在 label・重複 label・該当 0 件 delete の境界テスト

### SPEC-LGX-010.REQ.03: drift — standalone ドリフト対比

**内容:** `drift <artifact_id> [--against <snapshot:LABEL|snapshot:ID>]` は、指定成果物の**現行ファイル内容から生成した embedding** とベースラインとの乖離を報告する:

- **drift 値の定義**: `drift = 1.0 − cosine 類似度`。値域 [0.0, 2.0]（v3 実測 `crates/te-embed/src/drift.rs` の正準化）
- **実行時依存**: 現行内容の embedding 生成のため **ONNX モデルの解決が必要**（4 コマンド中 drift のみ。report / calibrate / snapshot は保存済みベクトルのみを使用しモデル不要）。モデルは **`--models-dir` フラグ ＞ 環境変数 `LGX_MODELS_DIR` ＞ 環境変数 `TE_MODELS_DIR`（旧名フォールバック。使用時は stderr に Info で新名を案内）＞ 設定ファイル** の順で解決する。全解決失敗・モデル読込失敗は実行エラー（exit 1）として試行内容を stderr に通知する。
  - **【v3 差分】**（QSET-LGX-010 Q2-a） v3 は環境変数 `TE_MODELS_DIR` のみを参照する（`crates/te-cli/src/model_dir.rs:34,67`）。legixy は `LGX_MODELS_DIR` を正準とし `TE_MODELS_DIR` を旧名フォールバックとして受理する。LGX-COMPAT-001 §3 が凍結するのは `--models-dir` フラグのみで環境変数名は凍結対象外（実機確認済み）。既存の `TE_MODELS_DIR` 設定は引き続き機能するため正当な入力空間の挙動は不変（互換安全）。両変数が同時設定された場合は `LGX_MODELS_DIR` を優先する（v3 に存在しなかった状況のため互換破壊にあたらない）。旧名フォールバックの撤去時期は将来 SPEC 改訂事項として DD へ申し送る
- **ベースライン選択**: `--against` 省略時は embeddings ストアの現行保存行。`--against snapshot:<token>` 指定時は、token をまず label として解決（同一 label 複数時は taken_at 最新。REQ.02）し、解決できなければ snapshot_id とみなす。**`snapshot:label:<LABEL>` の明示判別形式も受理**し label として解決する（v3 実測）
  - **明示 label 形式の解決失敗（spec-change 2026-06-13、ADR-LGX-019、TRIAGE §4 #19）**: `snapshot:label:<LABEL>` の明示判別形式で label 解決に失敗した場合は、snapshot_id へのフォールバックを行わず **ERROR（stderr）+ exit 1** とする。明示的に label と宣言した名前参照の解決失敗は「ベースライン不在（exit 0）」ではなく「指定ミス（exit 1）」として扱い、`snapshot delete label:<L>`（exit 1）および本 REQ 末尾の非対称性原則（壊れた状態を隠さない）と対称化する。【v3 差分】v3 は明示形式でも snapshot_id 扱いにフォールバックし行不在 exit 0 だった。曖昧形式 `snapshot:<token>`（`label:` 接頭辞なし）は従来どおり label→snapshot_id フォールバックし、行不在は exit 0 を維持する
- `snapshot:` プレフィクスを欠く `--against` 値は**実行エラー（exit 1）**として reject する（v3 実測: アプリ層判定。clap 層ではないため exit 2 ではない。REQ.01 の分類に従う）
- ベースラインが存在しない場合（未 embed のノード、スナップショットに当該行なし）: stderr へ INFO を通知し exit 0。`--json` 時は `{"artifact_id", "drift": null, "baseline_available": false}` を stdout に返し、**INFO は stderr に併出する**（v3 実測の正準化。stdout の機械可読性は保たれる）
- `<artifact_id>` が graph.toml に存在しない場合: ERROR（stderr）+ exit 1
- `<artifact_id>` は graph.toml に存在するが**現行ファイルが欠落**している場合（embedding 生成のためのファイル読込失敗）: ERROR（stderr）+ exit 1（QSET-LGX-010 Q2-c）。これは v3 正準でもある（`crates/te-embed/src/orchestrator.rs:160` `read_current_content_for_node` の `read_to_string(...)?` が伝播、`compute_node_drift` doc「Err: …ファイル読込失敗…」）。
  - **非対称性の注記**: ベースライン不在（未 embed / 未 snapshot）は正常なライフサイクル状態として exit 0 で扱う一方、現行ファイル欠落は graph.toml が存在を主張するファイルが消えた**壊れた状態**として exit 1 とする。この非対称は意図的であり、明示指定の対比は壊れた状態を隠さない（本 REQ 冒頭の原則および次元不一致 Error と同列）
- ベースラインと現行 embedding の**次元数が不一致**の場合: 実行エラー（exit 1。QSET-LGX-006 Q4 回答。check 内類似度計算の「集約 Warning + skip」とは異なり、明示指定の対比は失敗を隠さない）
- **ベースライン保存時の model_version と現行 model_version が異なる場合（次元は一致）: 実行エラー（exit 1）**（GAP-LGX-186 対応 — SCORE-INV-2 違反状態。同一次元のまま別バージョンへ遷移したケースは次元検査をすり抜けるため、model_version 文字列照合を一次検出とする。照合は SPEC-LGX-006.REQ.10 の完全一致判定に従う）
- 非有限スコア（NaN/±Inf）が生じた場合は exit 1（REQ.09）
- `--json` 正常時の出力: `{"artifact_id", "drift", "baseline_available": true, "baseline_source": "embeddings" | "snapshot:<id>"}`

**check 内 Drift との書き分け**: SPEC-LGX-004 REQ.02 / SPEC-LGX-006 REQ.05 の Drift は「検証層が content_hash 変化を Warning 報告する」機能。本 REQ の `drift` は「運用層が特定成果物をベースラインと定量対比する」機能であり、同名だが別物である（QSET-LGX-004 Q3 回答）。

**根拠:** LGX-COMPAT-001 §4 #5、QSET-LGX-004 Q3 / QSET-LGX-006 Q4 回答、v3 実測（`crates/te-cli/src/commands/drift.rs`, `crates/te-embed/src/orchestrator.rs`）
**検証方法:** ベースライン有無 × --against 3 形式の組合せテスト、モデル解決失敗テスト、次元不一致テスト、終了コードテスト

### SPEC-LGX-010.REQ.04: report — トレーサビリティ健全性監査

**内容:** `report` は閾値判定を行わない**計測レポート**を出力する:

1. **links**: graph.toml のエッジ（Chain / Custom / ParentChild）のうち、**両端点の embedding が存在し次元が一致するもの**の cosine 類似度 `[{from, to, score, kind}]`（v3 実測: 端点 embedding 不在・次元不一致のエッジは算出対象外）
2. **candidates**: エッジ未定義だが類似度 ≥ `link_candidate_threshold` のリンク漏れ候補 `[{from, to, score}]`（同様に算出可能なペアのみ）
3. **summary**: `{total_links, total_candidates, min_link_score, max_link_score, mean_link_score}`。total_links は**算出対象となったエッジ数**（スキップ後）である

- **スキップの可視化【v3 差分】**: 端点 embedding 不在・次元不一致によりスキップが発生した場合、**集約 Warning 1 件**（スキップ件数と代表理由）を stderr に出力する（QSET-LGX-006 Q4 で確定した「bulk API 類似度計算の集約 Warning + skip」の本コマンドへの適用。v3 は無言スキップ）。`--json` 時の warning 表現（任意フィールド）は DD で確定
- 非有限スコア（NaN/±Inf）のペアは対比に算入しない（skip + 集約 Warning、REQ.09）
- text モード: 人間可読の階層表示（リンク類似度 + 候補一覧 + 統計サマリ）
- `--json` モード: 上記 3 キーの構造化 JSON
- 空ストア時: text モードは INFO（stderr、`embed --all` の実行を促す）+ exit 0。`--json` 時は links/candidates 空配列 + summary（統計値 null）の**空構造**を stdout に返す（exit 0、v3 実測）

**check との責務境界**（QSET-LGX-004 Q4 回答）: check は**判定**（閾値超過のみを severity 付き findings として報告）、report は**計測**（判定せず生スコア + 候補 + 統計を出力）。両者の出力責務は重複しない。report は severity 概念を持たない。

**根拠:** UC-LGX-010、LGX-COMPAT-001 §4 #6、QSET-LGX-004 Q2/Q4 / QSET-LGX-006 Q4 回答、v3 実測（`crates/te-cli/src/commands/report.rs`, `crates/te-embed/src/similarity.rs` のスキップ挙動）
**検証方法:** TS（report 出力スキーマテスト、部分 embed 状態でのスキップ + 集約 Warning テスト）、check 出力との非重複検査

### SPEC-LGX-010.REQ.05: calibrate — 閾値キャリブレーション

**内容:** `calibrate [--buckets <N>] [--recommend]` は全ペア類似度の分布と推奨閾値を出力する:

- 全ペア類似度（O(N²)。次元不一致ペアおよび非有限スコア（NaN/±Inf）ペア〔REQ.09〕はスキップし、スキップ発生時は集約 Warning 1 件を stderr に出力する **【v3 差分】**、QSET-LGX-006 Q4）からヒストグラムを生成
- **ヒストグラムの定義**（v3 実測）: 値域 **[0.0, 1.0] 固定**の等幅 N バケット。域外スコア（負の cosine 等）は clamp して算入し、上限 1.0 は末尾バケットに含める（inclusive）。min/max/mean は clamp 前の生値
- `--buckets` 既定 10。`--buckets 0` は実行エラー（exit 1、v3 実測。REQ.01 の分類に従う）
- 出力: `{pairs, min, max, mean, distribution: [{low, high, count}], thresholds: {similarity_threshold, drift_threshold, link_candidate_threshold}}`（thresholds は設定中の現在値）
- `--recommend` 指定時は `recommended_thresholds` を追加出力する。**推奨閾値の算出はパーセンタイル方式**（QSET-LGX-006 Q2 回答で正準化）:
  - `similarity_threshold` 推奨値 = p25
  - `drift_threshold` 推奨値 = 1.0 − p90
  - `link_candidate_threshold` 推奨値 = p75
  - 参考情報として p10 / p25 / p50 / p75 / p90 を併記する
  - SPEC が凍結するのはパーセンタイル方式（どのパーセンタイルがどの閾値に写るか: p25 / 1.0−p90 / p75）まで。算出式の補間方式（v3 実測 nearest-rank 変種: `sorted[round((n−1)·frac)]`）は実装詳細として **DD で凍結**する（QSET-LGX-010 Q3-a）。算出式の沈黙変更は本 REQ の「既知分布 fixture に対する推奨値一致テスト」が破綻させ、意識的判断 + 変更履歴を強制する（再現性はテスト層で担保）
  - ペア数 0（空ストア、ノード 1 件、または全ペア次元不一致 skip 等）の場合、`recommended_thresholds` は出力されない（v3 実測）。**【v3 差分】**（QSET-LGX-010 Q3-b） `--recommend` 指定かつ pairs=0 のときは stderr に INFO 1 件（「ペア数 0 のため推奨値は算出されません」）を出力する。`--json` の stdout は汚さない。v3 は無言省略していたが、`--recommend` 指定に対し無言で何も返さないのは沈黙的な機能無効化にあたるため可視化する（QSET-LGX-006 Q4 の集約 Warning と同趣旨。pairs=0 は空ストア以外でも発生し空ストア INFO では捕捉できないため `--recommend` 経路に置く）
- text モード: ASCII ヒストグラム + min/max/mean + 現閾値（+ `--recommend` 時は推奨値）
- 空ストア時: text モードは INFO（stderr）+ exit 0。`--json` 時は pairs=0・統計値 null・distribution 空の**空構造**を返す（exit 0、v3 実測）

**根拠:** UC-LGX-011、LGX-COMPAT-001 §4 #7、QSET-LGX-006 Q2/Q4 回答、v3 実測（`crates/te-cli/src/commands/calibrate.rs`, `crates/te-embed/src/similarity.rs` の histogram）
**検証方法:** 既知分布の fixture に対する推奨値一致テスト（nearest-rank 式前提）、--buckets 境界テスト、clamp 境界テスト

### SPEC-LGX-010.REQ.06: 出力の決定性（読取系 3 コマンド）

**内容:** **drift / report / calibrate** の出力は同一入力に対して決定論的である。入力 = graph.toml + embeddings ストア + 設定 + （drift のみ）対象成果物の現行ファイル内容。走査・出力順序は SPEC-LGX-006 REQ.11 の決定性保証（node_id 昇順ロード、SCORE-INV-1 整合）に従う。

`snapshot create` は snapshot_id / taken_at が作成時刻に依存するため**決定性の対象外**とする。`snapshot list` は同一 DB 状態に対し taken_at 降順で安定した出力を返す（同時刻タイブレークは DD で確定）。

**根拠:** UC-LGX-010/011 の関連不変条件、SPEC-LGX-006 REQ.11、v3 実測（snapshot_id 生成は時刻 + 乱数由来）
**検証方法:** 読取系 3 コマンドの同一入力での出力バイト一致テスト

### SPEC-LGX-010.REQ.07: ストレージ境界と非破壊性

**内容:**
- `report` / `calibrate` / `drift` / `snapshot list` は読取専用であり、engine.db・graph.toml・成果物ファイルを変更しない
- **engine.db 不在時【v3 差分】**: 読取専用コマンドおよび `snapshot delete` は **DB ファイルを新規作成せず**、空ストア相当の挙動（REQ.02〜05 の空ストア時挙動）で正常終了する。v3 は DB 接続時にディレクトリ・DB ファイル・スキーマを自動作成していた（`crates/te-db/src/connection.rs`）が、「読取専用」宣言・FB-INV-4（DB 不在時安全性）の趣旨・UC-LGX-010/011 の事後条件（engine.db 不変）と矛盾するため、非作成に統一する（3 軸比較で採用。観測可能な差は副作用ファイルの有無のみ）
  - **DB 不在 ≡ 空ストア**であり、`snapshot delete` の挙動は REQ.02 から導出される（新たな特例ではない。QSET-LGX-010 Q1-d）: `delete label:<L>` は label 解決 0 件で **ERROR + exit 1**、`delete <snapshot_id>` は該当 0 行で **WARNING + exit 0**（`--json` 時は `{"snapshot_id", "deleted_rows": 0}`）。DB の物理存在の有無を exit code に露出させない（露出させると本 REQ の「観測可能な差は副作用ファイルの有無のみ」に反する。`label:<L>` は名前参照であり、名前解決失敗を DB 欠落理由で WARNING に格下げすると project-root 誤り等を覆い隠す）
- `snapshot create` のみ書込み系として engine.db の初期化（不在時の新規作成 + スキーマ作成）とスナップショット領域への書込みを行う。embeddings 本体の行は変更しない
- 4 コマンドはいずれも graph.toml に書き込まない
- スナップショットの具体的なテーブル/カラム構造は DD で定義する

**根拠:** UC-LGX-010/011 の事後条件、FB-INV-4、v3 実測（embedding_snapshots テーブル分離）
**検証方法:** 実行前後の engine.db / graph.toml ハッシュ比較テスト（読取系）、engine.db 不在からの各コマンド起動テスト、書込み範囲検査（snapshot create）

### SPEC-LGX-010.REQ.08: SPEC-LGX-006 との境界面

**内容:** 本 SPEC の 4 コマンドは SPEC-LGX-006 REQ.11 の bulk similarity API — 全エッジスコア算出 / リンク候補抽出 / 全ペアスコア算出 / ヒストグラム集計 / 決定論的全件ロードの各操作（操作名・シグネチャは DD で確定）— の **consumer** であり、類似度計算・ヒストグラム集計のロジックを再実装しない。SPEC-LGX-006 = エンジン（生成・検出）、本 SPEC = 運用コマンド、という責務分担を維持する。

**根拠:** QSET-LGX-001 Q1 / QSET-LGX-006 Q2 回答、SPEC-LGX-006 REQ.11
**検証方法:** 依存方向の構造検査（DD 凍結後）

### SPEC-LGX-010.REQ.09: 非有限スコア（NaN/±Inf）の扱い（GAP-LGX-185 対応）

**内容:** 類似度・drift 値として非有限値（NaN / +Inf / −Inf）が生じた場合の consumer 側フォールバックを以下に確定する（生成側の非ゼロ L2 正規化保証は SPEC-LGX-006 の所在。本 REQ は防御層）:
- **calibrate / report**: 非有限スコアのペアは**対比・統計に算入しない**（skip + 集約 Warning 1 件。次元不一致 skip と同経路）
- **drift**: **exit 1**（明示指定の対比は壊れた状態を隠さない — 次元不一致 Error・現行ファイル欠落 Error と同列の原則）
- **`--json` 出力は非有限値を一切含まない**（JSON 仕様上も NaN/Inf は表現不能）。統計が算出不能な場合は該当フィールドを `null` とする
- 本 REQ は REQ.06（出力の決定性）の前提を支える（非有限値の混入は決定論比較を破壊する）

**根拠:** GAP-LGX-185、SPEC-LGX-006.REQ.04（ゼロベクトル skip + 集約 Warning との整合）
**検証方法:** 非有限値を注入した fixture での calibrate/report skip + Warning テスト、drift exit 1 テスト、--json 出力に NaN/Inf が現れないことの検査

| 不変条件 | 役割 | 対応要求 |
|---------|------|---------|
| CTX-INV-1（決定論保証） | 関連 | REQ.06（読取系 3 コマンドの出力決定性） |
| FB-INV-4（DB 不在時安全性） | 関連 | REQ.07（engine.db 不在時に DB を作成せず空ストア相当で継続） |
| SCORE-INV-1（ハッシュ一致保証） | 関連 | REQ.06（決定論的全件ロードとの整合）, REQ.02（snapshot が content_hash / model_version を含む行を複製しベースラインの同一性情報を保持） |
| SCORE-INV-2（モデルバージョン一致） | 検証 | REQ.03（**model_version 文字列照合〔SPEC-LGX-006.REQ.10 完全一致〕が一次検出**。次元不一致 Error は同次元別バージョン遷移を捕捉できないため補完的検出に位置づけを訂正 — GAP-LGX-186） |
| MCP-INV-1（Agent Surface 限定） | 実装 | REQ.01（4 コマンドを MCP 非公開の Admin Surface に限定） |

**本 SPEC が関与しない不変条件:** CTX-INV-2〜5、FB-INV-1〜3/5、MCP-INV-2〜4、STATE-INV-1/2、SUBNODE-INV-1〜6、CACHE-INV-1〜4

---

## 5. 変更履歴

| 日付 | バージョン | 変更内容 |
|------|----------|---------|
| 2026-06-07 | 0.1.0-draft | 初版（AI 起草）。QSET-LGX-001 Q1 回答（選択肢 B、SPP-LGX-001 差分 1）に基づく新設。v3 実測挙動の正準化を基本とし、QSET-LGX-004 Q3/Q4・QSET-LGX-006 Q2/Q4 の確定事項を取り込む |
| 2026-06-07 | 0.1.1-draft | AI Adversary 6 レンズ検証（must-fix 14 / should-fix 20 / note 14）反映: 終了コード分類を v3 実測に整合（--against 形式不正 / --buckets 0 = exit 1）、決定性要求を読取系 3 コマンドに限定、グローバルオプション 3 種の網羅、drift のモデル依存と解決失敗挙動を追加、report links の「全エッジ」を算出可能エッジに限定 + 集約 Warning 適用、engine.db 不在時の非作成を確定【v3 差分】、診断の stderr 統一【v3 差分】、label 非一意性と最新優先解決、snapshot:label: 形式受理、空ストア時の --json 空構造、ヒストグラム clamp 域、パーセンタイル算出式注記、NFR 参照を OBS.02/OBS.05 に訂正、§4 に FB-INV-4 追加・SCORE-INV-1 理由修正、§5 を 9 項目に拡充しライフサイクルを明記 |
| 2026-06-08 | 0.2.0 | 前段ループ反復 1 完了（QSET-LGX-010 回答 → SPP-LGX-010 承認 → FCR-LGX-010 ACCEPTED）。§5 の 9 引き継ぎ項目を全て確定し §5（旧・前段ループ引き継ぎ事項）を削除、変更履歴を §5 へ繰り上げ。確定: snapshot_id 不透明トークン化（Q1-b）、drift モデル解決の `LGX_MODELS_DIR` リブランド【v3 差分】（Q2-a）、現行ファイル欠落 drift = exit 1 + 非対称性注記（Q2-c）、calibrate pairs=0 INFO【v3 差分】（Q3-b）、DB 不在時 delete 挙動の REQ.02 導出明記（Q1-d）、UC-012/013 新規化と SPEC-001 連動予告（Q4、§1.3）。追認: 空ストア非永続（Q1-a）/ 同時刻タイブレーク DD 委任（Q1-c）/ `--json` INFO 併出（Q2-b）/ パーセンタイル式 DD 委任（Q3-a）。Status を Accepted へ |
| 2026-06-10 | 0.2.1 | TP[SPEC] GAP 解消（人間承認 2026-06-10、2 件単一改訂）: GAP-LGX-185 対応で REQ.09（非有限スコア NaN/±Inf — calibrate/report は skip + 集約 Warning、drift は exit 1、--json は非有限値非出力・統計不能時 null）を新設し REQ.03/04/05 に参照を差込。GAP-LGX-186 対応で REQ.03 に「model_version 不一致（次元一致）→ exit 1」を追加（一次検出は SPEC-LGX-006.REQ.10 の完全一致照合）、§4 SCORE-INV-2 行の過大宣言（次元不一致 Error のみ）を「model_version 照合が一次・次元不一致は補完」に訂正、§1.3 UC-013 代替フローに model_version 不一致を追記。ownership: model_version 照合は drift 出力契約＝本 SPEC 所在（SPEC-LGX-006 は生成・bulk API に責務限定） |
| 2026-06-13 | 0.3.0 | spec-change（ADR-LGX-019、TRIAGE §4 #19）: REQ.03 に明示 label 形式 `snapshot:label:<L>` の解決失敗 = ERROR + exit 1 を追加（snapshot_id フォールバックなし）。`snapshot delete label:<L>`（exit 1）および非対称性原則と対称化。【v3差分】v3 は明示形式でも snapshot_id 扱いで exit 0。曖昧形式 `snapshot:<token>` は exit 0 維持 |
