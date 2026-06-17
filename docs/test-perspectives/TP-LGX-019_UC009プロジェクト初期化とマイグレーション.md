Document ID: TP-LGX-019

# TP-LGX-019: UC-LGX-009「プロジェクト初期化とマイグレーション」観点（UC レベル）

> TP は **テストケース** ではなく **観点リスト**。UC レベル TP は「ユースケースのフロー記述に問いかける質問のリスト」として書く。SPEC レベル TP（TP-LGX-008）が「仕様が答えるか」を問うのに対し、UC レベル TP は「フローが先行成果物（親 SPEC）を観察可能なステップへ忠実かつ完全に具体化しているか」を問う。

**親**: UC-LGX-009
**ステータス**: green
**最終更新**: 2026-06-13

## 1. 対象スコープ

この TP は UC-LGX-009「プロジェクト初期化とマイグレーション」の全フロー（init 基本フロー Step 1〜2、migrate 基本フロー Step 1〜7、代替フロー 2a/2b、事前条件・事後条件）に UC レベル観点をぶつける。

- 対象: UC-LGX-009 全節（概要・アクター・事前条件・基本フロー・代替フロー・事後条件）
- 親 SPEC: SPEC-LGX-008（マイグレーション）REQ.01〜REQ.13
- 関連 SPEC §: SPEC-LGX-008.REQ.02（非破壊性・冪等性・atomic 書込・確定順序）、REQ.02a（退避命名規約）、REQ.03a（破損検出）、REQ.06（migrate コマンド引数）、REQ.07（init コマンド）、REQ.08（失敗時メッセージ・成功時変更サマリ）、REQ.09（バージョン検出）、REQ.13（設定ファイル探索順）、LGX-COMPAT-001 §4 #1/#2（init/migrate 引数凍結契約）、LGX-COMPAT-001 §3（終了コード凍結契約）
- 委譲方針: migrate / init の各セマンティクス（非破壊性・冪等性・バージョン検出・ID マッピング全単射保証・設定ファイル探索 4 ケース等の**規定そのもの**）は TP-LGX-008（green 確定済）が所有する。本 TP はそれらを再検証せず、「UC-LGX-009 のフロー記述が SPEC-LGX-008 の規定を観察可能なステップとして正しく具体化しているか」のみを問う。

## 2. 観点リスト

### 2.1 基本フロー（ステップ連鎖の整合）

- [ ] 観点 BF1: init フローのステップ連鎖整合。Step 1（コマンド受理）→ Step 2（5 成果物生成）の事後条件「有効な legixy プロジェクト構造が作成される」が各生成物（`.legixy.toml` / `graph.toml` / 各タイプディレクトリ / `.legixy/` / `matrix.md`）の外部観察可能性と一致するか
- [ ] 観点 BF2: migrate フロー Step 2（v0.1.0 読み込み: a. `.legixy.toml` 解析 / b. matrix.md パース）の事後条件が後続 Step 3（graph.toml 生成）の前提（ノード/エッジ生成の材料）を満たすか
- [ ] 観点 BF3: migrate フロー Step 3〜6（graph.toml 生成 → `.legixy.toml` 変換 → feedback.db→engine.db 移行 → vectors.bin インポート）の処理順序と SPEC-LGX-008.REQ.02 の確定順序（DB コミット先行→平文ファイル atomic 確定）との整合
- [ ] 観点 BF4: migrate フロー Step 7（移行レポート出力）が SPEC-LGX-008.REQ.08 の成功時変更サマリ（生成/更新ファイル一覧・書き換え ID 件数・バックアップ場所）を UC 観察可能なステップとして記述しているか
- [ ] 観点 BF5: init フローで生成される 5 成果物が SPEC-LGX-008.REQ.07 の規定（`.legixy.toml`=ICONIX 8 typecode + `[id.document_id]`、`docs/traceability/graph.toml`=空、8 ディレクトリ+`.gitkeep`、`.legixy/`=.gitignore 付き、`engine.db`=初期スキーマ）と UC フロー記述の「以下を生成する」5 項目が対応しているか

### 2.2 代替フロー（分岐網羅）

- [ ] 観点 AF1: 代替フローの網羅性。init 側は「2a. `.legixy.toml` が既に存在する」のみだが、SPEC-LGX-008.REQ.07 で判定対象の「既存」は `.legixy.toml` / `.trace-engine.toml` / `docs/traceability/graph.toml` / `.legixy/engine.db` の **4 種**。UC の 2a は「`.legixy.toml` が既に存在する」とのみ記述しており、他 3 種の既存判定ケースが分岐として列挙されているか
- [ ] 観点 AF2: migrate 側の代替フロー 2b（プロジェクト未発見）に加え、SPEC-LGX-008.REQ.06 が定義する `--dry-run` 実行パスと `--to` 省略時のデフォルト動作が代替フローまたは基本フロー注記として観察可能か
- [ ] 観点 AF3: 各代替フローの事後条件収束。2a/2b とも「ERROR を報告する」で終了し、exit コードが SPEC-LGX-008 / LGX-COMPAT-001 §3 の規約（exit 1 = 実行時エラー / exit 2 = 使用法誤り）のどちらに収束するかが UC で観察可能か

### 2.3 例外フロー（失敗パス）

- [ ] 観点 EF1: migrate Step 2 での破損入力（engine.db open/クエリ失敗・TOML パース失敗・必須構造欠落）の失敗パスが代替フロー/例外フローに列挙されているか。SPEC-LGX-008.REQ.03a（破損検出→Error 中断・原本温存・部分移行なし）が UC フロー記述に現れていない【GENUINE: 破損検出は migrate の中核安全保証。UC フローでの明示が必要か、SPEC 委譲で十分かの裁定が必要】
- [ ] 観点 EF2: migrate Step 3〜6 での各段階失敗パス（ディスクフル・権限エラー・中断）が例外フローとして定義されているか。SPEC-LGX-008.REQ.02（SQLite トランザクション・atomic rename・確定順序）および REQ.08（失敗段階・バックアップ場所・リカバリ手順提示）がフロー記述として具体化されているか【WEAK: SPEC-008 REQ.02/REQ.08 委譲で解決可。UC への明示列挙は任意】
- [ ] 観点 EF3: init での部分生成失敗（例: `.legixy.toml` 生成直後に `graph.toml` 生成が失敗した場合）の中間状態と原本保護が例外フローで定義されているか【WEAK: SPEC-008 REQ.07 の「既存ファイルがある場合エラー」と REQ.02a 退避命名で大筋被覆。UC への明示は任意】

### 2.4 アクター遷移と権限

- [ ] 観点 AT1: アクター（開発者 CLI）の権限前提の一貫性。init/migrate ともカレントディレクトリへの書き込み権限を前提とするが、UC の事前条件でその権限前提が明示されているか
- [ ] 観点 AT2: 責任境界。migrate Step 5 の「feedback.db → engine.db 移行」・Step 6 の「vectors.bin → embeddings テーブルインポート」はシステム責務であり、移行後の Git commit はアクター責務（STATE-INV-2）であることの分担が UC で観察可能か

### 2.5 データフロー

- [ ] 観点 DF1: migrate の入出力データの型・制約。入力（v0.1.0 プロジェクトルートパス）→ 出力（graph.toml / migration-id-map.toml / 変換済 engine.db / 変換済 `.legixy.toml` / 移行レポート）の対応が UC フローで観察可能か
- [ ] 観点 DF2: migrate Step 6「vectors.bin があれば embeddings テーブルにインポートする」の条件分岐（vectors.bin 不在時の挙動）が UC フロー記述として明示されているか。SPEC-LGX-008.REQ.05（Phase 1 はドキュメントノードのみ・サブノード情報は含めない）との整合が観察可能か
- [ ] 観点 DF3: init で生成される `docs/traceability/matrix.md`（空テンプレート）が UC に列挙されているが、SPEC-LGX-008.REQ.07 には matrix.md の init 生成が明記されていない。この UC 記述の根拠（親 SPEC への対応）が確認できるか

### 2.6 領域固有観点（マイグレーション / 初期化）

- [ ] 観点 R1: 設定ファイル探索順。migrate Step 2a「`.legixy.toml` を解析」は単一ファイルを想定しているが、SPEC-LGX-008.REQ.13 の探索順（`.legixy.toml` → `.trace-engine.toml` フォールバック）が UC フロー記述で観察可能か
- [ ] 観点 R2: バージョン検出の UC 可視性。migrate フローが v0.1.0 プロジェクトを読み込む前に SPEC-LGX-008.REQ.09（PRAGMA user_version / `[graph]` セクション有無によるバージョン自動検出）を実行するステップが UC に観察可能か【WEAK: SPEC-008 REQ.09 の規定そのものは TP-LGX-008 が所有。UC フローでのステップ明示は任意】
- [ ] 観点 R3: 非破壊性と退避の UC 可視性。migrate フローで元ファイルの退避（SPEC-LGX-008.REQ.02a: `<名>.bak.{epoch}` 命名）が UC の観察可能なステップとして現れているか【WEAK: SPEC-008 REQ.02/REQ.02a 委譲で解決可。UC への明示は任意】
- [ ] 観点 R4: terminate 条件と exit コードの UC 観察可能性。init / migrate の成功時 exit 0 が UC 事後条件として観察可能か。代替フローの ERROR 報告が exit 1 に収束することが UC で明示されているか（LGX-COMPAT-001 §3 / §4 との整合）
- [ ] 観点 R5: ID マッピングの UC 可視性。migrate Step 3「ノードとエッジを生成する」は旧 ID → 新 ID の書き換えと migration-id-map.toml 生成（SPEC-LGX-008.REQ.11）を内包するはずだが、UC フローで観察可能なステップとして現れているか【WEAK: SPEC-008 REQ.11 委譲で解決可。UC への明示は任意】
- [ ] 観点 R6: init `--force` オプションの UC 可視性。SPEC-LGX-008.REQ.07 および LGX-COMPAT-001 §4 #1 が規定する `--force` 上書きパスが UC のどこに現れているか（基本フロー注記・代替フロー・事前条件のいずれかで観察可能か）【WEAK: LGX-COMPAT-001 §4 #1 委譲で解決可。UC への明示は任意】

## 3. RED / GREEN 判定

| 観点 | 判定 | 親 SPEC / UC §で回答（委譲先） | 関連 GAP |
|---|---|---|---|
| 2.1 BF1 init ステップ連鎖整合 | GREEN | Step 1 受理 → Step 2「以下を生成する」の 5 成果物列挙。事後条件「有効な legixy プロジェクト構造が作成される」が観察可能。SPEC-LGX-008.REQ.07 の生成物規定は TP-LGX-008 L-1/L-2 へ委譲 | — |
| 2.1 BF2 migrate Step 2 → Step 3 前提充足 | GREEN | Step 2 で `.legixy.toml` 解析 + matrix.md パースの 2 サブステップを明示。これが Step 3「各行からノード/エッジ生成」の材料として連鎖。SPEC-LGX-008.REQ.03 の抽出規則は TP-LGX-008 I-1/I-2 へ委譲 | — |
| 2.1 BF3 migrate Step 3〜6 の処理順序と REQ.02 確定順序整合 | RED | UC の Step 5（feedback.db→engine.db）→ Step 6（vectors.bin インポート）→ Step 7（レポート）という順序は DB 系操作を Step 4〜6 にまたがらせる。SPEC-LGX-008.REQ.02 は「DB コミット先行 → 平文ファイル（graph.toml 等）atomic 確定」を要求するが、UC の Step 3（graph.toml 生成）が Step 5 の DB 移行より前に来ておりこの確定順序と乖離しているか、または意図が別か不明。【GENUINE: 中断時の安全性に直結する実行順序。UC フローと SPEC 確定順序の一致・不一致を人間裁定する必要がある】 | GAP-LGX-261 |
| 2.1 BF4 migrate Step 7 の変更サマリ具体化 | RED | Step 7「移行レポートを出力する」は SPEC-LGX-008.REQ.08 の成功時変更サマリ（生成/更新ファイル一覧・書き換え ID 件数・バックアップ場所）を一行で包んでいるが、stdout/stderr 分離・`--format json` 対応・STATE-INV-2 運用支援という観察可能な具体性が UC フロー記述に反映されていない。【WEAK: SPEC-008 REQ.08 委譲で解決可。UC への明示は任意】 | GAP-LGX-262 |
| 2.1 BF5 init 生成物の UC 記述と REQ.07 の対応 | RED | UC の init Step 2 は 5 項目を列挙するが、SPEC-LGX-008.REQ.07 は「`.legixy/engine.db` を初期スキーマで作成」を必須生成物として定義している一方、UC の列挙に engine.db が存在しない（UC には「`.legixy/` ディレクトリ（.gitignore 付き）」とのみ記述）。REQ.07 の engine.db 作成が UC フロー記述に観察可能でない。【GENUINE: 後続 UC が engine.db 存在を前提とする可能性があり、UC 事後条件の完全性に影響する】 | GAP-LGX-263 |
| 2.2 AF1 init 既存ファイル判定の分岐網羅 | RED | UC の代替フロー 2a は「`.legixy.toml` が既に存在する場合」のみ。SPEC-LGX-008.REQ.07 は「既存」判定対象を `.legixy.toml` / `.trace-engine.toml` / `docs/traceability/graph.toml` / `.legixy/engine.db` の 4 種と規定。`.trace-engine.toml` 既存・`graph.toml` 既存・`engine.db` 既存の各ケースが UC 代替フローに現れていない。【GENUINE: 特に `.trace-engine.toml` 既存は旧プロジェクトへの後付け init シナリオで重要。UC 分岐の非網羅】 | GAP-LGX-264 |
| 2.2 AF2 `--dry-run` パスと `--to` 既定値の観察可能性 | GREEN | `--dry-run` は SPEC-LGX-008.REQ.06 + TP-LGX-008 B-4/O-4 へ委譲。`--to` 既定は LGX-COMPAT-001 §4 #2 へ委譲。UC フローが基本フロー（通常 migrate）を記述するのは自然であり、--dry-run パスの UC 分岐非列挙は委譲容認の範囲内 | — |
| 2.2 AF3 代替フローの exit コード収束 | RED | UC の 2a/2b はともに「ERROR を報告する」で終了するが、これが exit 1（実行時エラー）か exit 2（使用法誤り）かが UC に観察可能でない。2a（既存ファイルあり）は実行時エラー（exit 1）と使用法誤り（exit 2）のどちらとも解釈しうる。LGX-COMPAT-001 §3 凍結契約との整合が UC 事後条件に反映されていない。【WEAK: LGX-COMPAT-001 §3 / TP-LGX-008 F-4 へ委譲で解決可。UC への exit コード明示は任意】 | GAP-LGX-265 |
| 2.3 EF1 破損入力の失敗パス欠落 | RED | migrate Step 2 では `.legixy.toml` 解析と matrix.md パースを行うが、破損入力（TOML パース失敗・必須構造欠落・engine.db open 失敗）の失敗パスが代替フロー/例外フローに一切列挙されていない。SPEC-LGX-008.REQ.03a（破損検出→Error 中断・原本温存・部分移行なし）が UC フロー記述として観察可能でない。【GENUINE: 破損検出は migrate の中核安全保証。「対象が無い」（空入力正常終了）と「対象が壊れている」（Error）の区別が UC で観察可能でない】 | GAP-LGX-266 |
| 2.3 EF2 Step 3〜6 各段階失敗パスの欠落 | RED | migrate Step 3〜6 はいずれも失敗パスが定義されていない。SPEC-LGX-008.REQ.02 は DB トランザクション・atomic rename・確定順序・中断耐性を規定するが、これらが UC の観察可能なフローとして現れていない。【WEAK: SPEC-008 REQ.02/REQ.08 委譲で解決可。UC への各段階失敗パス列挙は任意】 | GAP-LGX-267 |
| 2.3 EF3 init 部分生成失敗の中間状態 | GREEN | init の部分生成失敗は SPEC-LGX-008.REQ.07（`--force` 上書き＋REQ.02a 退避）の派生問題。TP-LGX-008 E-3/E-5 が所有。UC フロー記述レベルでの例外パス非列挙は委譲容認の範囲内 | — |
| 2.4 AT1 書き込み権限前提の事前条件明示 | GREEN | init/migrate ともカレントディレクトリへの書き込み権限はファイルシステム標準前提。UC 事前条件への明示は通常省略される水準であり委譲容認 | — |
| 2.4 AT2 Git commit 責任境界の明示 | GREEN | migrate 事後条件「v0.1.0 の全データが legixy 形式に変換される」は DB/ファイル変換の完了を指し、Git commit までが完了という STATE-INV-2 の運用意図は SPEC-LGX-008 §4 + TP-LGX-008 S-5 へ委譲で一貫 | — |
| 2.5 DF1 migrate 入出力データの対応 | GREEN | migrate フロー全体を通じた入力（`--from <PATH>` の v0.1.0 プロジェクト）→ 出力（graph.toml / engine.db / `.legixy.toml` / レポート）の対応が Step 2〜7 の連鎖として観察可能。出力ファイルの型詳細は SPEC-LGX-008 REQ 各条へ委譲 | — |
| 2.5 DF2 vectors.bin 不在時の分岐 | RED | migrate Step 6「vectors.bin があれば embeddings テーブルにインポートする」は「あれば」という条件分岐を記述しているが、不在時の挙動（スキップして正常終了 / 警告発行 / 別フロー）が UC に観察可能でない。SPEC-LGX-008.REQ.05（Phase 1 はドキュメントノードのみ）との整合も不明。【WEAK: REQ.05 および TP-LGX-008 L-3 委譲で解決可。「あれば」の分岐明示で UC フロー記述としてはむしろ網羅方向だが、不在時の事後状態が不明確】 | GAP-LGX-268 |
| 2.5 DF3 init の matrix.md 生成と REQ.07 との対応 | RED | UC init Step 2 は「`docs/traceability/matrix.md`（空テンプレート）」の生成を列挙するが、SPEC-LGX-008.REQ.07 には matrix.md の init 生成が明記されていない（REQ.07 は `.legixy.toml`・`graph.toml`・`engine.db`・8 ディレクトリ+`.gitkeep` が規定対象）。この UC 記述は親 SPEC の規定を超えており、裏付けとなる REQ/§が確認できない。【GENUINE: UC が SPEC に規定のない生成物を追加しているのは UC フロー記述のスコープ逸脱の可能性。人間裁定が必要】 | GAP-LGX-269 |
| 2.6 R1 設定ファイル探索順の UC 可視性 | RED | migrate Step 2a「`.legixy.toml` を解析」は `.legixy.toml` のみを参照するが、SPEC-LGX-008.REQ.13 の探索順（`.legixy.toml` 優先 / `.trace-engine.toml` フォールバック）が UC フロー記述として観察可能でない。`.trace-engine.toml` のみ存在する v0.1.0 プロジェクトに対する migrate の挙動が UC で不明。【GENUINE: migrate の主要入力ファイルの探索順が UC 基本フローから見えない。旧プロジェクト移行の現実的シナリオで影響あり】 | GAP-LGX-270 |
| 2.6 R2 バージョン検出ステップの UC 可視性 | GREEN | バージョン自動検出（SPEC-LGX-008.REQ.09）の規定そのものは TP-LGX-008 V-1/V-2 が所有。UC フローが migrate の前提条件として「v0.1.0 プロジェクトが存在する」を事前条件に置いており、バージョン検出を UC フロー中に明示しないことは委譲容認の範囲内 | — |
| 2.6 R3 退避命名の UC 可視性 | GREEN | SPEC-LGX-008.REQ.02a 退避命名規約は TP-LGX-008 P-5 へ委譲。UC フロー記述に退避ステップが現れないことは委譲容認の範囲内 | — |
| 2.6 R4 exit コードの UC 観察可能性 | GREEN | 基本フロー成功時の exit 0 は事後条件の「有効なプロジェクト構造が作成される」「全データが変換される」から自然に観察可能（成功=exit 0 は LGX-COMPAT-001 §3 の凍結契約として確立）。exit コードの UC 明示は委譲容認。代替フローの exit コード収束は AF3 で別途評価 | — |
| 2.6 R5 ID マッピングの UC 可視性 | GREEN | SPEC-LGX-008.REQ.11（旧 ID 書き換え・migration-id-map.toml 生成）は TP-LGX-008 V-8 が所有。UC の Step 3「ノードとエッジを生成する」は REQ.03 の変換ルールを要約しており、ID マッピング詳細の非列挙は委譲容認の範囲内 | — |
| 2.6 R6 `--force` オプションの UC 可視性 | GREEN | SPEC-LGX-008.REQ.07 + LGX-COMPAT-001 §4 #1 へ委譲。UC の代替フロー 2a「ERROR を報告する」は `--force` なし時の挙動。`--force` あり時は上書き（REQ.07 規定）で基本フローに合流する構造であり、UC への明示は任意 | — |

集計: **全 22 観点 / GREEN 13 / RED 9**（RED: BF3[GENUINE] / BF4[WEAK] / BF5[GENUINE] / AF1[GENUINE] / AF3[WEAK] / EF1[GENUINE] / EF2[WEAK] / DF2[WEAK] / DF3[GENUINE] / R1[GENUINE]）

> 補足: EF2 と DF2 と AF3 は観点番号が 10 件だが BF5 と DF3 が 1 観点ずつで計 22 観点、RED 9 件（GENUINE 5: BF3/BF5/AF1/EF1/DF3 → GAP 261/263/264/266/269、GENUINE+R1 で GAP-270 追加）。以下 GAP 内訳で詳細。

## 4. ステータスの決定

RED 観点が 9 件（GENUINE 6: BF3/BF5/AF1/EF1/DF3/R1、WEAK 3: BF4/AF3/EF2、WEAK 1: DF2）残存するため、本 TP のステータスは `red`。

- GENUINE 判定 6 件（GAP-LGX-261/263/264/266/269/270）は UC フロー記述として人間裁定が必要な実質的ギャップ。特に BF3（migrate 処理順序と SPEC 確定順序の乖離）、BF5（engine.db 生成の UC 欠落）、AF1（init 既存判定分岐の非網羅）、EF1（破損検出失敗パス欠落）、DF3（matrix.md 生成の親 SPEC 裏付け不在）、R1（設定ファイル探索順の UC 不可視）は下流成果物（RBA/SEQD/DD）の前提として影響が大きい。
- WEAK 判定（BF4/AF3/EF2/DF2）は親 SPEC の対応 REQ で答えが存在し、UC への明示反映が任意か必須かの裁定により close 可能。
- 全観点 GREEN 化後に本 TP を green へ更新する。

## 5. 観点ナレッジベース参照

この TP の生成時に参照したナレッジベース章節:

- `docs/perspectives/core-perspectives.md` §UC レベル観点（基本フロー / 代替フロー / 例外フロー / アクター遷移と権限 / データフロー）
- `docs/perspectives/core-perspectives.md` §汎用観点（永続化 / 状態遷移 / エラーハンドリング / 入力検証 / バージョニング・互換性）
- 親 SPEC: SPEC-LGX-008.REQ.01〜REQ.13
- 委譲先 TP: TP-LGX-008（マイグレーション SPEC レベル観点、green 確定済）
- LGX-COMPAT-001 §3 / §4 #1/#2（init/migrate 引数・終了コード凍結契約）

## 6. 履歴

| 日付 | 変更内容 |
|---|---|
| 2026-06-13 | 初版作成。UC レベル観点 22 件（GREEN 13 / RED 9）。GAP-LGX-261〜270 のうち 10 件を起票（GENUINE 6 件: 261/263/264/266/269/270、WEAK 4 件: 262/265/267/268） |

## 7. 解消（2026-06-13、敵対的精査裁定後）

本 TP が起票した GAP[UC] は全て closed。内訳: **WEAK=方針B（委譲容認）** / **REFUTED=棄却** / **GENUINE=UC 修正で解消**（A/B/C、人間承認 2026-06-13）。§3 表の判定列は初版（起票時）の draft 判定を保持する（精査の履歴として温存）。全 RED 観点は上記裁定で解消したため本 TP は **green**。各 GAP の最終状態は当該 GAP ファイル（§5）と docs/spec-change-proposals/2026-06-13_uc-gap-adversarial-review.md を参照。
