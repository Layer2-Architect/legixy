Document ID: TP-LGX-021

# TP-LGX-021: UC-LGX-011「閾値キャリブレーション」観点（UC レベル）

> TP は **テストケース** ではなく **観点リスト**。UC レベル TP は「ユースケースのフロー記述に問いかける質問のリスト」として書く。SPEC レベル TP（TP-LGX-010）が「仕様が答えるか」を問うのに対し、UC レベル TP は「フローが先行成果物（親 SPEC）を観察可能なステップへ忠実かつ完全に具体化しているか」を問う。

**親**: UC-LGX-011
**ステータス**: green
**最終更新**: 2026-06-13

## 1. 対象スコープ

この TP は UC-LGX-011「閾値キャリブレーション」の全フロー（基本フロー Step 1〜7、代替フロー 2a/1a/3a、事後条件）に UC レベル観点をぶつける。

- 対象: UC-LGX-011 全節（概要・アクター・事前条件・基本フロー・代替フロー・事後条件・関連不変条件・関連 SPEC / NFR）
- 親 SPEC: SPEC-LGX-010 REQ.05（calibrate コマンド）, REQ.01（共通規約・終了コード）, REQ.06（出力決定性）, REQ.07（ストレージ境界・非破壊性）
- 関連 SPEC §: SPEC-LGX-006 REQ.11（bulk similarity API — `compute_all_pair_scores` / `histogram`）, REQ.04（cosine 値域 [-1,1] / clamp）, NFR-LGX-001.OBS.06（ユーザー向け構造化出力）, LGX-COMPAT-001 §4 #7（calibrate 凍結済み引数契約）
- 委譲方針: calibrate の全ペア算出・ヒストグラム集計・パーセンタイル方式推奨閾値ロジックの**規定そのもの**は TP-LGX-010（green 確定済）が所有する。本 TP はそれらを再検証せず、「UC-011 のフロー記述が SPEC-LGX-010 REQ.05 の規定を観察可能なステップとして正しく具体化しているか」のみを問う。

## 2. 観点リスト

### 2.1 基本フロー（ステップ連鎖の整合）

- [ ] 観点 BF1: 各ステップの事後条件が後続ステップの前提を満たすか（Step1 コマンド受理 → Step2 embeddings テーブル全件ロード → Step3 全ペア類似度算出 → Step4 ヒストグラム生成 → Step5/6 text/JSON 出力 → Step7 exit 0 の連鎖整合）
- [ ] 観点 BF2: Step2 「全件ロード」とStep3 「全ペア算出（O(N²)）」の段階区分が UC で観察可能か。`te_embed::compute_all_pair_scores` の呼出が Step3 に明示されており、「ロード済みデータを入力にペア算出」という依存関係が表現されているか
- [ ] 観点 BF3: Step5/Step6 の出力分岐（text モード / `--json` モード）が基本フローで網羅されているか。`--json` 非指定時のデフォルトが text モードであることが UC で観察可能か
- [ ] 観点 BF4: Step7 の exit 0 収束が成功時事後条件（「標準出力にヒストグラム + 閾値が出力される」「engine.db は不変」）と対応するか。後続 UC（アクターが `.legixy.toml` を編集する判断を得る）の前提として参照可能か

### 2.2 代替フロー（分岐網羅）

- [ ] 観点 AF1: 代替フローの網羅性確認。明示分岐（2a embeddings 空 / 1a --buckets 0 / 3a 全ペア算出失敗）が SPEC-LGX-010 REQ.05 の境界ケース（空ストア・--buckets 0 exit 1・次元不一致スキップ・`--recommend` 使用）を被覆しているか
- [ ] 観点 AF2: `--recommend` 分岐の非網羅。SPEC-LGX-010 REQ.05 は `--recommend` 指定時に `recommended_thresholds` を追加出力し、pairs=0 のとき INFO + 非出力を規定する。UC-011 の基本フロー・代替フローに `--recommend` の分岐が存在しない
- [ ] 観点 AF3: 2a（embeddings 空）の事後条件収束。UC が「exit 0 で終了」と明示しているか。SPEC-LGX-010 REQ.05（空ストア時: text モード INFO + exit 0 / `--json` 時: 空構造 + exit 0）との収束確認
- [ ] 観点 AF4: 1a（--buckets 0）の exit コード明示。UC が「エラーメッセージ + exit 1」と記述しており SPEC-LGX-010 REQ.01（値の意味的不正 = exit 1）と一致するか確認
- [ ] 観点 AF5: 各代替フローへの遷移条件が明示されているか（2a=「embeddings が空の場合」、1a=「--buckets 0 指定時」、3a=「全ペア算出失敗時」）

### 2.3 例外フロー（失敗パス）

- [ ] 観点 EF1: 次元不一致スキップの失敗パス欠落。SPEC-LGX-010 REQ.05 は次元不一致ペアを「スキップ + 集約 Warning 1 件（stderr）」として規定するが、UC の基本フロー・代替フローにこの部分失敗パスが記述されていない
- [ ] 観点 EF2: 非有限スコア（NaN/±Inf）の失敗パス欠落。SPEC-LGX-010 REQ.09 は calibrate での NaN/Inf ペアを「skip + 集約 Warning 1 件」として規定するが、UC のフロー記述に現れていない
- [ ] 観点 EF3: エラー時の状態不変条件。3a（全ペア算出失敗）で exit 1 となる場合に engine.db が不変（読取のみ）であることが UC 事後条件で担保されているか（SPEC-LGX-010 REQ.07 への参照として）
- [ ] 観点 EF4: Step3 の失敗境界。3a は「全ペア算出失敗時」を扱うが、「一部ペア算出失敗（次元不一致/特殊値によるスキップ）」との区別が UC 上で観察可能か

### 2.4 アクター遷移と権限

- [ ] 観点 AT1: アクター（PM / 設定管理者 / 設計者 / QA リード）の権限が一貫しているか。calibrate は読取専用コマンドであり、全アクターが同一権限で実行可能であることが UC で一貫しているか
- [ ] 観点 AT2: 責任境界。UC 事後条件「アクターが出力を根拠に `.legixy.toml` の閾値を編集する判断を得る（閾値変更自体は別手順）」でシステム=計測出力 / アクター=判断・編集 の分担が明示されているか
- [ ] 観点 AT3: 複数アクターの使用文脈（プロジェクト立ち上げ時 / ONNX モデル切り替え時 / false positive・negative 判断時）が、UC フローの基本フローで対応できるか。フロー記述が特定アクター文脈に特化せず汎用的か

### 2.5 データフロー

- [ ] 観点 DF1: 入出力データの分離。入力（embeddings テーブル + `.legixy.toml` 現閾値設定 + `--buckets` 引数）→ 出力（ヒストグラム + 統計サマリ + 現閾値一覧 = stdout / 診断メッセージ = stderr）の分離が UC で観察可能か（SPEC-LGX-010 REQ.01【v3 差分】: INFO/WARNING は stderr, 結果は stdout）
- [ ] 観点 DF2: stdout/stderr 分離の不明示。UC の基本フロー Step5/6 では出力内容の記述はあるが「stdout に出力」「診断は stderr」という出力先の明示がなく、NFR-LGX-001.OBS.06（構造化出力）や SPEC-LGX-010 REQ.01 との整合が UC レベルで観察できない
- [ ] 観点 DF3: engine.db 非破壊の観察可能性。UC 事後条件「engine.db は不変（読取のみ）」が明示されており SPEC-LGX-010 REQ.07 と整合するか

### 2.6 領域固有観点（calibrate / 閾値キャリブレーション）

- [ ] 観点 R1: `--buckets N` の境界値が UC で網羅されているか。既定値（10）は事前条件・基本フロー Step4 に明示あり。`--buckets 0` は 1a で扱われる。上限・負値・型不正については UC が言及していないが、SPEC-LGX-010 REQ.01（型不正 = exit 2）への委譲で解決可能か
- [ ] 観点 R2: ヒストグラムの値域定義（[0.0, 1.0] 固定・域外 clamp・min/max/mean は clamp 前生値）が UC フローで観察可能か。UC の基本フロー Step4/5/6 は出力項目を列挙するが clamp 挙動の言及がなく、SPEC-LGX-010 REQ.05 への委譲として問題ないか
- [ ] 観点 R3: 出力の決定性（同一 embeddings → 同一ヒストグラム）が UC の関連不変条件 SCORE-INV-1 で担保されており、SPEC-LGX-010 REQ.06（読取系 3 コマンドの出力決定性）と整合するか
- [ ] 観点 R4: 終了コード契約（Step7 exit 0 / 1a exit 1）が LGX-COMPAT-001 §4 #7 の凍結契約と整合するか。SPEC-LGX-010 REQ.01 の 3 分類（exit 0/1/2）と UC 記述の一致
- [ ] 観点 R5: `--json` モードの出力スキーマ（pairs / min / max / mean / distribution / thresholds）が UC 基本フロー Step6 に示されており、SPEC-LGX-010 REQ.05 の JSON スキーマと完全一致するか。`--recommend` 追加時の `recommended_thresholds` キーが UC に未記載の点の評価

## 3. RED / GREEN 判定

| 観点 | 判定 | 親 SPEC / UC §で回答（委譲先） | 関連 GAP |
|---|---|---|---|
| 2.1 BF1 ステップ連鎖整合 | GREEN | 基本フロー Step1〜7 が事前条件（embed --all 実行済・.legixy.toml 設定あり）→ Step2 全件ロード → Step3 全ペア算出 → Step4 ヒストグラム → Step5/6 出力 → Step7 exit 0 と連鎖。各事後条件が後続前提を満たす | — |
| 2.1 BF2 段階区分の観察可能性 | GREEN | Step3 に `te_embed::compute_all_pair_scores` を明示。Step2 ロード → Step3 算出 → Step4 `te_embed::histogram` の依存関係が UC フローで観察可能。エンジン API の詳細は SPEC-LGX-006 REQ.11 へ委譲 | — |
| 2.1 BF3 text/JSON 出力分岐の網羅 | GREEN | 基本フロー Step5（text モード）と Step6（`--json` モード）を別ステップで記述。デフォルト = text モードはフローの流れから観察可能（Step5 から Step6 は `--json` 指定時への分岐として配置）。詳細スキーマは SPEC-LGX-010 REQ.05 へ委譲 | — |
| 2.1 BF4 成功時事後条件の観察可能性 | GREEN | 事後条件「標準出力にヒストグラム + 閾値が出力される」「engine.db は不変」「アクターが判断を得る」の 3 つが明示。後続アクション（.legixy.toml 編集）はアクター責務として分離。外部観察可能 | — |
| 2.2 AF1 代替フロー網羅性 | RED | `--recommend` 分岐が UC に未記載（詳細は AF2）。次元不一致スキップ（集約 Warning）も未記載（詳細は EF1）。SPEC-LGX-010 REQ.05 の境界ケースを UC フローが完全には被覆していない。【WEAK: SPEC-010 委譲で解決可。UC フロー記述への明示は任意か必須かの裁定要】 | GAP-LGX-281 |
| 2.2 AF2 --recommend 分岐の非網羅 | RED | SPEC-LGX-010 REQ.05 は `--recommend` 指定時の `recommended_thresholds` 追加出力と pairs=0 時の INFO + 非出力を規定するが、UC-011 の基本フロー・代替フローに `--recommend` フラグの使用パターンが記述されていない。キャリブレーション UC の主要機能であり、QA リードが使用する代替フローの欠落として評価。【GENUINE 寄り: --recommend はキャリブレーション中核機能。UC 概要は言及なし、フローに代替分岐なし】 | GAP-LGX-282 |
| 2.2 AF3 2a（空ストア）事後条件収束 | GREEN | 2a「embeddings が空の場合: INFO 出力して exit 0」。SPEC-LGX-010 REQ.05（空ストア: INFO + exit 0）と一致。`--json` 時の空構造については UC が text モード以外に言及していないが SPEC-010 委譲として GREEN | — |
| 2.2 AF4 1a（--buckets 0）exit コード | GREEN | 1a「エラーメッセージ + exit 1」が明示。SPEC-LGX-010 REQ.01（値の意味的不正 = exit 1）および TP-LGX-010 B4 で確立済みの規定と一致 | — |
| 2.2 AF5 遷移条件の明示 | GREEN | 2a「embeddings が空の場合」、1a「--buckets 0 指定時」、3a「全ペア算出失敗時」と発火条件を明示 | — |
| 2.3 EF1 次元不一致スキップの欠落 | RED | SPEC-LGX-010 REQ.05（次元不一致ペアのスキップ + 集約 Warning 1 件 stderr【v3 差分】）が UC フローに反映されていない。3a は「算出失敗」を扱うが「部分スキップ＋継続」のパスは別物。事後条件「標準出力にヒストグラム + 閾値が出力される」は partial skip 後も成立するが、集約 Warning の stderr 出力が観察可能化されていない。【WEAK: SPEC-010 REQ.05 委譲で解決可】 | GAP-LGX-283 |
| 2.3 EF2 非有限スコアの失敗パス欠落 | RED | SPEC-LGX-010 REQ.09（calibrate: NaN/Inf ペアを skip + 集約 Warning 1 件）が UC に未記述。次元不一致と同一経路だが独立したケースとして規定されている。【WEAK: SPEC-010 REQ.09 委譲で解決可。EF1 と同性質】 | GAP-LGX-284 |
| 2.3 EF3 エラー時の状態不変条件 | GREEN | UC 事後条件「engine.db は不変（読取のみ）」が明示。3a の exit 1 ケースでも engine.db 不変が適用される。SPEC-LGX-010 REQ.07（読取系非破壊）へ委譲として整合 | — |
| 2.3 EF4 全件失敗 vs 部分スキップの区別 | RED | 3a「全ペア算出失敗時: anyhow エラーコンテキスト付きで exit 1」はシステム全体の算出失敗を扱う。一方 EF1/EF2 の「部分スキップ継続」とは異なるパスだが、UC がこの区別を明示していない。「一部ペアのスキップ（部分成功 exit 0）」と「全件失敗（exit 1）」の境界が UC レベルで観察不能。【GENUINE 寄り: exit 0 vs exit 1 の境界が消費者にとって重要な観察ポイント】 | GAP-LGX-285 |
| 2.4 AT1 アクター権限の一貫性 | GREEN | PM / 設定管理者 / 設計者 / QA リードとも読取専用 calibrate を同一権限で実行。calibrate は engine.db を変更しない（事後条件明示）。権限差は本質的に存在せず UC 記述と一貫 | — |
| 2.4 AT2 責任境界（計測 vs 判断・編集） | GREEN | UC 事後条件「アクターが出力を根拠に `.legixy.toml` の閾値を編集する判断を得る（閾値変更自体は別手順）」でシステム = 計測出力 / アクター = 判断・編集 の分担を明示 | — |
| 2.4 AT3 複数アクター文脈の汎用性 | GREEN | 基本フローは「legixy calibrate [--buckets N] [--json] を実行する」と汎用的に記述。アクター文脈（プロジェクト立ち上げ / モデル切り替え / FP・FN 判断）は概要と典型的な判断材料節で補足されており、フロー本体はアクター中立 | — |
| 2.5 DF1 入出力データ分離の明示 | GREEN | 事前条件（embeddings テーブル N 件 + `.legixy.toml` 設定あり）が入力を規定。事後条件（ヒストグラム + 閾値 = stdout, engine.db = 不変）が出力を規定。SPEC-LGX-010 REQ.01 stdout/stderr 分離は TP-LGX-010 L1 で確立済み → 委譲 | — |
| 2.5 DF2 stdout/stderr 分離の不明示 | RED | UC 基本フロー Step5（text モード: ASCII ヒストグラム + 最小/最大/平均 + 現閾値一覧）と Step6（--json モード: JSON 構造）が「標準出力に出力」という記述を持たない。2a の INFO は「出力」とあるが出力先（stdout/stderr）が不明示。SPEC-LGX-010 REQ.01【v3 差分】（INFO/WARNING は stderr、結果は stdout）が UC フローに反映されておらず、消費者が観察可能な出力先が不明確。【WEAK: SPEC-010 REQ.01 委譲で解決可】 | GAP-LGX-286 |
| 2.5 DF3 engine.db 非破壊の観察可能性 | GREEN | UC 事後条件「engine.db は不変（読取のみ）」を明示。SPEC-LGX-010 REQ.07（読取系の非破壊性）と整合。STATE-INV-1 も関連不変条件に列挙 | — |
| 2.6 R1 --buckets 境界値の委譲 | GREEN | UC は既定値 10（Step4 明示）と `--buckets 0` エラー（1a）を記述。上限・負値・型不正は SPEC-LGX-010 REQ.01（型不正 = exit 2）+ TP-LGX-010 B4/B5 で確立済みの規定へ委譲として GREEN | — |
| 2.6 R2 ヒストグラム値域・clamp の委譲 | GREEN | UC は ヒストグラム生成（Step4）と出力項目（Step5/6）を記述。[0.0,1.0] 固定・域外 clamp・clamp 前統計の詳細は SPEC-LGX-010 REQ.05 + TP-LGX-010 B6 で確立済みの規定へ委譲。UC フローはヒストグラム計算を `te_embed::histogram` として参照し整合 | — |
| 2.6 R3 出力の決定性 | GREEN | UC 関連不変条件 SCORE-INV-1（同一 embeddings → 同一ヒストグラム）を明示。SPEC-LGX-010 REQ.06（読取系 3 コマンドの出力決定性）+ TP-LGX-010 D1 で確立済みの規定と整合 | — |
| 2.6 R4 終了コード契約 | GREEN | 基本フロー Step7（exit 0）+ 1a（exit 1）+ 3a（exit 1）が明示。SPEC-LGX-010 REQ.01 3 分類（exit 0/1/2）と整合。LGX-COMPAT-001 §4 #7 凍結契約は TP-LGX-010 E1/F2 で確立済みへ委譲 | — |
| 2.6 R5 --json スキーマ整合（--recommend 欠落評価） | RED | UC 基本フロー Step6 の JSON スキーマ（`{"pairs": N, "min", "max", "mean", "distribution": [...], "thresholds": {...}}`）は SPEC-LGX-010 REQ.05 の出力定義と一致する。ただし `--recommend` 指定時の `recommended_thresholds` キーが UC Step6 に未記載（AF2 と連動）。スキーマ自体は委譲として GREEN だが、`--recommend` 分岐の欠落を改めて確認。【GAP-LGX-282 に集約（AF2 と同一 GAP）】 | GAP-LGX-282 |

集計: **全 24 観点 / GREEN 17 / RED 7**（RED は AF1 / AF2 / EF1 / EF2 / EF4 / DF2 / R5、R5 は AF2 に集約のため実質独立 RED は 6 件）

## 4. ステータスの決定

RED 観点が複数件（AF1 / AF2 / EF1 / EF2 / EF4 / DF2 を中心に 6 件独立 GAP）残存するため、本 TP のステータスは `**ステータス**: red`。

- AF2（--recommend 分岐非網羅）は GENUINE 寄り: calibrate の主要機能であり、QA リードが使用する代替フローとして UC に記述が必要と判断。パーセンタイル方式推奨閾値は SPEC-LGX-010 REQ.05 に正準定義があり、UC レベルでは `--recommend` の存在とフロー分岐のみを記述すれば足りる。
- EF4（全件失敗 vs 部分スキップの境界）は GENUINE 寄り: exit 0（部分スキップ継続）と exit 1（全件失敗）の境界は消費者にとって重要な観察ポイントであり、UC フローへの明示が望ましい。
- EF1 / EF2 / DF2 / AF1 は WEAK 候補: いずれも SPEC-LGX-010 REQ.05/REQ.09/REQ.01 が既に答えており、UC フロー記述への明示反映が任意か必須かの裁定が必要。
- 敵対的精査パスで GENUINE / WEAK / OUT_OF_SCOPE を確定し、WEAK 確定分は人間裁定（UC フローへの追記 or drop）を経て close。全観点 GREEN 化後に本 TP を green へ更新する。

## 5. 観点ナレッジベース参照

この TP の生成時に参照したナレッジベース章節:

- `docs/perspectives/core-perspectives.md` §UC レベル観点（基本フロー / 代替フロー / 例外フロー / アクター遷移と権限 / データフロー）
- `docs/perspectives/core-perspectives.md` §汎用観点（境界値=--buckets / エラーハンドリング / 状態遷移 / 入力検証）
- 親 SPEC: SPEC-LGX-010 REQ.01/05/06/07/08/09
- エンジン SPEC: SPEC-LGX-006 REQ.04/11
- 委譲先 TP: TP-LGX-010（embedding 運用・監査、green 確定済）/ TP-LGX-006（embedding とドリフト検出、green 確定済）
- LGX-COMPAT-001 §4 #7（calibrate 凍結済み引数契約）

UX 層観点（Undo/フォーカス/タッチ等）は CLI 計測コマンドには本質的に N/A のため、診断出力の観察可能性（DF2）以外はスキップした。

## 6. 履歴

| 日付 | 変更内容 |
|---|---|
| 2026-06-13 | 初版作成。UC レベル観点 24 件（GREEN 17 / RED 7、うち R5 は AF2 に集約のため独立 GAP は 6 件）。GAP-LGX-281〜286 を起票 |

## 7. 解消（2026-06-13、敵対的精査裁定後）

本 TP が起票した GAP[UC] は全て closed。内訳: **WEAK=方針B（委譲容認）** / **REFUTED=棄却** / **GENUINE=UC 修正で解消**（A/B/C、人間承認 2026-06-13）。§3 表の判定列は初版（起票時）の draft 判定を保持する（精査の履歴として温存）。全 RED 観点は上記裁定で解消したため本 TP は **green**。各 GAP の最終状態は当該 GAP ファイル（§5）と docs/spec-change-proposals/2026-06-13_uc-gap-adversarial-review.md を参照。
