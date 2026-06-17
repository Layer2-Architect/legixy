Document ID: TP-LGX-023

# TP-LGX-023: UC-LGX-013「standalone ドリフト対比」観点（UC レベル）

> TP は **テストケース** ではなく **観点リスト**。UC レベル TP は「ユースケースのフロー記述に問いかける質問のリスト」として書く。SPEC レベル TP（TP-LGX-010 / TP-LGX-006）が「仕様が答えるか」を問うのに対し、UC レベル TP は「フローが先行成果物（親 SPEC）を観察可能なステップへ忠実かつ完全に具体化しているか」を問う。

**親**: UC-LGX-013
**ステータス**: green
**最終更新**: 2026-06-13

## 1. 対象スコープ

この TP は UC-LGX-013「standalone ドリフト対比」の全フロー（基本フロー Step 1〜7、代替フロー 1a/2a/2b/3a/3b/4a/4b/5a/5b/6a、事後条件）に UC レベル観点をぶつける。

- 対象: UC-LGX-013 全節（概要・アクター・事前条件・基本フロー・代替フロー・事後条件・典型的な判断材料・関連不変条件・関連 SPEC/NFR）
- 親 SPEC: SPEC-LGX-010（embedding 運用・監査）REQ.03（drift = standalone 対比）/ REQ.06（読取系決定性）/ REQ.07（読取専用・DB 不在時非作成）/ REQ.09（非有限スコア）/ REQ.01（共通規約・exit 分類）
- 関連 SPEC §（委譲先）:
  - SPEC-LGX-006 REQ.10（model_version 完全一致判定 = drift の model_version 照合の判定規則）/ REQ.04（cosine 値域 [-1,1]・次元不一致時 drift は Error 維持・ゼロベクトル drift は Error 維持）= エンジン責務
  - SPEC-LGX-006 §4 SCORE-INV-1（決定性順序）/ SCORE-INV-2（モデルバージョン一致）
  - LGX-COMPAT-001 §3（`--models-dir` グローバルオプション）/ §4 #5（drift の凍結済引数契約）/ NFR-LGX-001.OBS.06（ユーザー向け構造化出力）, OBS.02（stdout/stderr 分離）, OBS.05（終了コード）
- 関連 UC: UC-LGX-012（snapshot 凍結ライフサイクル — `--against snapshot:` の基準点を供給）, UC-LGX-011（drift_threshold キャリブレーション — 判断材料の閾値供給）
- 委譲方針: drift の**判定/算出セマンティクスそのもの**（drift = 1.0 − cosine の定義・値域 [0.0,2.0]・終了コード分類・次元/model_version/非有限の reject 規定・モデル解決順序・ベースライン選択規則の**規定**）は TP-LGX-010（green 確定済）/ TP-LGX-006（green 確定済）が所有する。本 TP はそれらを再検証せず、「UC-013 のフロー記述が SPEC-010/006 の規定を観察可能なステップとして正しく具体化しているか」のみを問う。
- **新規 UC 注記**: 本 UC は 2026-06-13 の UC フェーズ拡張（SPEC-001 v0.8.0 同期、運用者アクター）で新規生成された。親 SPEC（SPEC-LGX-010 v0.2.1）の drift 被覆は前段ループ反復 1 + GAP-185/186 解消で厚いが、UC フロー記述の連鎖・網羅は本 TP で初めて精査される。force-GREEN せず、フローレベルの真のギャップは素直に RED+GAP とする。

## 2. 観点リスト

### 2.1 基本フロー（ステップ連鎖の整合）

- [ ] 観点 BF1: 各ステップの事後条件が後続ステップの前提を満たすか（Step1 コマンド受理 → Step2 モデル解決 → Step3 現行ファイル読込 + embedding 生成 → Step4 ベースライン選択 → Step5 次元/model_version 検査 → Step6 drift 算出 → Step7 出力・exit の連鎖整合）
- [ ] 観点 BF2: Step2（モデル解決）と Step3（現行 embedding 生成）の依存順序が観察可能か。drift のみがモデル依存（report/calibrate/snapshot はモデル不要）という事前条件の射程が UC で観察可能か
- [ ] 観点 BF3: 成功時事後条件（stdout に drift 値・exit 0・engine.db/graph.toml/成果物ファイル不変）が外部観察可能で、判断材料（drift_threshold 比較）の前提として参照可能か
- [ ] 観点 BF4: Step5 の二段検査（次元数一致 → model_version 完全一致）の順序とそれぞれの失敗が別ステップ（5a / 5b）へ写像されているか。次元一致だが model_version 不一致のケースが次元検査をすり抜けて model_version 照合に到達する連鎖が観察可能か

### 2.2 代替フロー（分岐網羅）

- [ ] 観点 AF1: `--against` 入力形式の分岐網羅。基本フロー Step4 の 3 形式（`snapshot:<LABEL>` 暗黙 / `snapshot:<ID>` / `snapshot:label:<LABEL>` 明示判別）+ 省略時（embeddings ストア現行行）+ プレフィクス欠如（1a → exit 1）が列挙されているか
- [ ] 観点 AF2: ベースライン選択（Step4）の **token 二段解決の最終分岐**の網羅。「token をまず label として解決 → 解決失敗なら snapshot_id とみなす」とあるが、その snapshot_id **も**存在しない（label にも ID にも一致しない）最終分岐がどの代替フロー（4a baseline-absent exit 0 か / 不正参照 exit 1 か）に収束するかが UC で観察可能か
- [ ] 観点 AF3: モデル解決系分岐の網羅。2a（全解決失敗・読込失敗 → exit 1）/ 2b（旧名 `TE_MODELS_DIR` 解決 → Info 案内 + 続行 / 両変数同時設定時 `LGX_MODELS_DIR` 優先）が解決順序（`--models-dir` > `LGX_MODELS_DIR` > `TE_MODELS_DIR` > 設定）の各段を被覆しているか
- [ ] 観点 AF4: artifact 存在系分岐の網羅。3a（graph.toml 不在 → exit 1）/ 3b（graph.toml 存在だが現行ファイル欠落 → exit 1）と 4a（ベースライン不在 → exit 0）の **exit 1 / exit 0 非対称**が UC フローで明示的に区別されているか
- [ ] 観点 AF5: 各代替フローの事後条件収束。前提崩壊系（1a/2a/3a/3b/5a/5b/6a → exit 1）が「実行エラー exit 1」へ、正常ライフサイクル系（4a → exit 0）が「正常終了 exit 0」へそれぞれ収束し、両者の境界が UC で一貫しているか
- [ ] 観点 AF6: 4b（`snapshot:<L>` の label 同一複数 → taken_at 最新へ決定論的解決。UC-LGX-012 delete と同一規則）の発火条件と解決規則が明示されているか

### 2.3 例外フロー（失敗パス）

- [ ] 観点 EF1: 各ステップでの失敗パスが定義されているか。Step2（モデル）/ Step3（ファイル読込・embedding 生成）/ Step4（ベースライン解決）/ Step5（検査）/ Step6（算出）の各失敗が代替/例外フローに列挙されているか
- [ ] 観点 EF2: エラー時の状態が不変条件を満たすか（drift は読取専用 → engine.db/graph.toml/成果物ファイル不変、engine.db 不在時も DB を新規作成しない、が全エラーパスで担保されているか）
- [ ] 観点 EF3: エラー時のユーザ通知（ERROR/INFO/Info の severity 区分と出力先 stderr）が各失敗パスで定義されているか。`--json` 時の INFO 併出（4a baseline-absent）で stdout の機械可読性が保たれるか
- [ ] 観点 EF4: 非有限スコア（NaN/±Inf）の失敗パス。Step6 の drift 算出で非有限値が生じた場合（6a → exit 1）が、次元/model_version reject をすり抜けた残余の数値破綻として例外フローに位置づけられているか
- [ ] 観点 EF5: `<artifact_id>` は正常だがベースライン保存時と現行で次元が異なる Step5 失敗（5a）と、現行ファイル欠落 Step3 失敗（3b）の両方が「壊れた状態を隠さない」原則の下で exit 1 に収束し、4a の exit 0（正常ライフサイクル）と区別されているか

### 2.4 アクター遷移と権限

- [ ] 観点 AT1: アクター（設計者 / 運用者 / QA リード）の権限・状態が一貫しているか。drift が読取専用であり 3 アクターが同一権限で実行可能であることが UC で一貫しているか
- [ ] 観点 AT2: 責任境界。システムは drift 値の**算出と出力**のみを行い、判断材料（drift ≈ 0.0 / threshold 超過 / baseline_available: false の解釈と是正アクション）はアクター責務であることの分担が明示されているか
- [ ] 観点 AT3: drift 実行中に対象成果物ファイル / graph.toml / embeddings ストア / 参照中 snapshot が外部更新・削除（並行アクセス・TOCTOU）された場合の整合性前提が UC レベルで成立しているか

### 2.5 データフロー

- [ ] 観点 DF1: 入出力データの型・制約。入力（graph.toml + embeddings ストア + 設定 + 現行ファイル内容 + `--against` token）→ 出力（drift 値=stdout / 診断=stderr）の分離が UC で観察可能か
- [ ] 観点 DF2: `--json` 出力スキーマの完全性。正常時 `{"artifact_id","drift","baseline_available":true,"baseline_source":"embeddings"|"snapshot:<id>"}` / baseline 不在時 `{"artifact_id","drift":null,"baseline_available":false}` の 2 スキーマが代替フロー（4a）と整合し、非有限値（6a）が `--json` に現れない（REQ.09）ことが UC で観察可能か
- [ ] 観点 DF3: ベースライン同一性情報のデータフロー。drift の対比に必要な baseline 側の次元数・model_version が「embeddings ストア現行行 or snapshot 複製行」のどちらからも供給され、Step5 の照合入力として整合するか（snapshot は content_hash/model_version を含む行を複製 — UC-012 事後条件）

### 2.6 領域固有観点（standalone drift / embedding 対比 UC）

- [ ] 観点 R1: check 内 Drift（content_hash 変化 Warning。SPEC-LGX-004 REQ.02 / SPEC-LGX-006 REQ.05）と本 UC の standalone drift（embedding 対比の定量値）の**別機能性**が UC 概要で明示され、機能境界の混同が防がれているか
- [ ] 観点 R2: drift 値の定義（drift = 1.0 − cosine 類似度・値域 [0.0,2.0]）が UC で観察可能か（算出ロジックの実体は SPEC-LGX-010 REQ.03 / SPEC-LGX-006 REQ.04 へ委譲）
- [ ] 観点 R3: 読取系決定性。同一入力（graph.toml + embeddings ストア + 設定 + 現行ファイル内容）に対する drift 出力の決定論性が UC 事後条件で明示されているか（SPEC-LGX-010 REQ.06 / SCORE-INV-1 へ委譲）
- [ ] 観点 R4: model_version 不一致検出（5b）が SCORE-INV-2 違反状態の一次検出であり、次元検査（5a）が補完であるという二層検出の位置づけが UC の関連不変条件で観察可能か（判定規則は SPEC-LGX-006 REQ.10 完全一致へ委譲）
- [ ] 観点 R5: 終了コード契約（0/1/2）が UC 事後条件・代替フローと LGX-COMPAT-001 §3/§4 #5 の凍結契約で一致するか。特に 1a（プレフィクス欠如）が clap 層でなくアプリ層判定のため exit 1（exit 2 でない）であることが UC で明示されているか
- [ ] 観点 R6: baseline_source の出力値（`"embeddings"` / `"snapshot:<id>"`）が、`--against` で label を渡しても**解決後の snapshot_id** を反映するか（Step4 で label→snapshot_id 解決 → Step7 出力 `snapshot:<id>` の連鎖が観察可能か）

## 3. RED / GREEN 判定

| 観点 | 判定 | 親 SPEC / UC §で回答（委譲先） | 関連 GAP |
|---|---|---|---|
| 2.1 BF1 ステップ連鎖整合 | GREEN | 基本フロー Step1〜7 が事前条件（init 済・graph.toml 登録・現行ファイル存在・モデル解決可能・ベースライン存在）→ Step2/3 解決・生成 → Step4 選択 → Step5 検査 → Step6/7 算出・出力・exit と連鎖。各事後条件が後続前提を満たす | — |
| 2.1 BF2 モデル依存の射程の観察可能性 | GREEN | 事前条件「4 コマンド中 drift のみの実行時依存」+ Step2（モデル解決順序明示）でモデル依存が drift 固有であることが観察可能。解決順序の規定は SPEC-LGX-010 REQ.03【v3 差分】へ委譲 | — |
| 2.1 BF3 成功時事後条件の観察可能性 | GREEN | 事後条件「stdout に drift 値」「engine.db/graph.toml/成果物ファイル不変（読取専用・DB 不在時非作成）」+ Step7 exit 0。外部観察可能・判断材料の前提として参照可。決定性は SPEC-LGX-010 REQ.06/REQ.07 へ委譲 | — |
| 2.1 BF4 二段検査の連鎖観察可能性 | GREEN | Step5「次元数一致・model_version 完全一致を検査」+ 5a（次元不一致 → exit 1）/ 5b（model_version 不一致・次元一致 → exit 1）。5b の「同一次元のまま別バージョンへ遷移したケースは次元検査をすり抜けるため model_version 文字列照合が一次検出」で連鎖が観察可能。判定規則は SPEC-LGX-006 REQ.10 へ委譲 | — |
| 2.2 AF1 `--against` 形式分岐網羅 | GREEN | Step4 が 3 形式（`snapshot:<token>` label→ID フォールバック / `snapshot:label:<LABEL>` 明示）+ 省略時（embeddings ストア現行行）を列挙、1a がプレフィクス欠如を exit 1 で reject。受理形式の規定は SPEC-LGX-010 REQ.03（I1）へ委譲 | — |
| 2.2 AF2 token 二段解決の最終分岐網羅 | RED | UC Step4 は「token をまず label として解決し、解決できなければ snapshot_id とみなす」と二段解決を記すが、その **snapshot_id も存在しない（label にも ID にも一致しない）最終分岐**がどこへ収束するかをフロー記述に列挙していない。4a は「スナップショットに**当該行**なし」を baseline-absent(exit 0) とするが、これは「snapshot は存在するが artifact 行が無い」ケースであり、「指定 token がいかなる snapshot にも解決しない」ケースとは別。SPEC レベルでは TP-LGX-010 E9 が「baseline-absent exit 0 規則が token 未解決の最終分岐を自然に包含」として GREEN 委譲済だが、UC フロー記述に当該分岐の遷移が明示されていない。【WEAK: SPEC-010 REQ.03 委譲で解決可。UC への明示列挙は任意だが、二段解決を本文に書いた以上その終端を書くのが連鎖整合上望ましい】 | GAP-LGX-301 |
| 2.2 AF3 モデル解決系分岐網羅 | GREEN | 2a（全解決失敗・読込失敗 → stderr 通知 + exit 1）/ 2b（`TE_MODELS_DIR` 解決 → Info 案内 + 続行 / 両変数同時 `LGX_MODELS_DIR` 優先）。解決順序 4 段の規定は SPEC-LGX-010 REQ.03【v3 差分】（V4/V5）へ委譲 | — |
| 2.2 AF4 exit 1/exit 0 非対称の明示 | GREEN | 3a（graph.toml 不在 → exit 1）/ 3b（現行ファイル欠落 → exit 1、「壊れた状態を隠さない。4a の exit 0 との非対称は意図的」と明示）/ 4a（ベースライン不在 → exit 0）。非対称性注記は SPEC-LGX-010 REQ.03 と一致 | — |
| 2.2 AF5 代替フロー事後条件収束 | GREEN | 前提崩壊系（1a/2a/3a/3b/5a/5b/6a）が「実行エラー exit 1」、正常ライフサイクル系（4a）が exit 0 へ収束。境界は SPEC-LGX-010 REQ.01（exit 分類）+ AF4 非対称で一貫 | — |
| 2.2 AF6 重複 label 解決規則の明示 | GREEN | 4b「`snapshot:<L>` の label が同一複数 → taken_at 最新の 1 件へ決定論的解決（UC-LGX-012 の delete と同一規則）」で発火条件と規則を明示。同時刻タイブレークは SPEC-LGX-010 REQ.02（DD 委任）へ委譲 | — |
| 2.3 EF1 各ステップ失敗パス定義 | GREEN | Step2→2a、Step3→3a/3b、Step4→1a/4a/4b、Step5→5a/5b、Step6→6a と各ステップに失敗パスが対応。失敗パス被覆は AF2（token 未解決最終分岐）を除き網羅 | （→ AF2） |
| 2.3 EF2 エラー時状態の不変条件保持 | GREEN | 事後条件「engine.db/graph.toml/成果物ファイル不変（読取専用。engine.db 不在時も DB 新規作成しない）」が全パスで担保。SPEC-LGX-010 REQ.07（読取専用・DB 不在時非作成）へ委譲 | — |
| 2.3 EF3 エラー通知（severity・出力先） | GREEN | 2a（stderr 通知）/ 2b（stderr Info）/ 3a/3b（ERROR stderr）/ 4a（INFO stderr、`--json` 時 stdout に drift:null 併出）と severity・出力先を明示。stdout/stderr 分離は NFR-LGX-001.OBS.02 / SPEC-LGX-010 REQ.01 へ委譲 | — |
| 2.3 EF4 非有限スコアの失敗パス位置づけ | GREEN | 6a「非有限スコア（NaN/±Inf）発生 → exit 1（SPEC-LGX-010 REQ.09）」を例外フローに明示。次元/model_version reject の残余の数値破綻として位置づけ、規定は REQ.09 へ委譲 | — |
| 2.3 EF5 壊れた状態 exit 1 vs 正常 exit 0 区別 | GREEN | 3b（現行ファイル欠落 exit 1）/ 5a（次元不一致 exit 1）が「壊れた状態を隠さない」、4a（ベースライン不在 exit 0）が「正常なライフサイクル状態」と本文で明示区別。非対称は SPEC-LGX-010 REQ.03 と一致 | — |
| 2.4 AT1 アクター権限の一貫性 | GREEN | 設計者 / 運用者 / QA リードとも読取専用 drift を同一権限で実行。権限差は本質的に存在せず UC 記述と一貫（事後条件「読取専用」） | — |
| 2.4 AT2 責任境界（算出 vs 解釈・是正） | GREEN | 「典型的な判断材料」節がシステム=算出/出力、アクター=解釈（drift≈0.0 / threshold 超過 / baseline_available:false の意味判断と是正アクション）の分担を明示 | — |
| 2.4 AT3 実行中外部更新・TOCTOU の整合性前提 | GREEN | 並行アクセス/TOCTOU（解決〜読取間の snapshot 削除等）は NFR-LGX-001 SEC.02（WAL+busy_timeout）/ REL.07 の射程（TP-LGX-010 S6/C4 で確立済・並行モデルは NFR 所有）。drift 読取専用前提で UC レベル整合 | — |
| 2.5 DF1 入出力データ型・stdout/stderr 分離 | GREEN | 入力（graph.toml + embeddings ストア + 設定 + 現行ファイル内容 + `--against`）→ 出力（drift 値=stdout / 診断=stderr）。分離は SPEC-LGX-010 REQ.01 / NFR-LGX-001.OBS.02 へ委譲、UC Step7 + 事後条件と整合 | — |
| 2.5 DF2 `--json` スキーマの完全性 | GREEN | Step7（正常スキーマ 4 キー）+ 4a（baseline 不在スキーマ 3 キー drift:null）+ 6a（非有限値非出力 → REQ.09）が代替フローと整合。各スキーマは SPEC-LGX-010 REQ.03/REQ.09（F3）へ委譲 | — |
| 2.5 DF3 ベースライン同一性情報のデータフロー | GREEN | Step5 の照合入力（次元数・model_version）が「embeddings ストア現行行 or snapshot 複製行」から供給。snapshot 側は content_hash/model_version を含む行複製（UC-012 事後条件 + SPEC-LGX-010 §4 SCORE-INV-1）でデータフロー整合 | — |
| 2.6 R1 check 内 Drift との別機能性 | GREEN | UC 概要が「`check` 内の Drift（content_hash 変化 Warning。SPEC-LGX-004 REQ.02 / SPEC-LGX-006 REQ.05）とは**別物**」と明示。機能境界は SPEC-LGX-010 REQ.03「check 内 Drift との書き分け」と一致 | — |
| 2.6 R2 drift 値定義の観察可能性 | GREEN | 基本フロー Step6「drift = 1.0 − cosine 類似度」+ 概要「値域 [0.0, 2.0]」。算出ロジック実体は SPEC-LGX-010 REQ.03 / SPEC-LGX-006 REQ.04（B8）へ委譲 | — |
| 2.6 R3 読取系決定性の事後条件明示 | GREEN | 事後条件「同一入力（graph.toml + embeddings ストア + 設定 + 現行ファイル内容）に対して決定論的」+ 関連不変条件 SCORE-INV-1/出力の決定性。規定は SPEC-LGX-010 REQ.06（D1）へ委譲 | — |
| 2.6 R4 二層検出の位置づけ観察可能性 | GREEN | 関連不変条件「SCORE-INV-2: model_version 完全一致照合が一次検出、次元検査は補完（SPEC-LGX-010 §4 / GAP-LGX-186）」を明示。判定規則は SPEC-LGX-006 REQ.10 完全一致（V6/D8）へ委譲 | — |
| 2.6 R5 終了コード契約一致（exit 1 vs 2 境界） | GREEN | 1a「プレフィクスを欠く `--against` 値: 実行エラー exit 1（アプリ層判定のため exit 2 ではない）」を明示。0/1/2 契約は LGX-COMPAT-001 §3/§4 #5 + SPEC-LGX-010 REQ.01（E2/V1/V2）へ委譲 | — |
| 2.6 R6 baseline_source の解決後 ID 反映 | RED | Step7 の `--json` 正常スキーマは `baseline_source: "embeddings" | "snapshot:<id>"` と **解決後 snapshot_id** を出力契約に持つ。一方 Step4 は `--against snapshot:<L>`（label 入力）→ snapshot_id 解決の二段解決を記すが、Step4（解決）→ Step7（`snapshot:<id>` 出力）の連鎖、すなわち「label を渡しても出力 baseline_source は解決後の id を反映する」点をフローの観察可能ステップとして明示していない。label 入力時の baseline_source 値（label をそのまま返すのか解決後 id を返すのか）が UC フロー記述から一意に読み取れない。【WEAK: SPEC-010 REQ.03 の JSON スキーマ定義（`snapshot:<id>`）で id 反映は規定済。UC フローへの解決→出力連鎖の明示は任意だが、二段解決を本文に持つ以上、出力が解決結果を反映する旨の 1 文があると観察可能性が上がる】 | GAP-LGX-302 |

集計: **全 24 観点 / GREEN 22 / RED 2**（RED は AF2 / R6、いずれも WEAK 候補）

## 4. ステータスの決定

RED 観点が 2 件（AF2 / GAP-LGX-301、R6 / GAP-LGX-302）残存するため、本 TP のステータスは `**ステータス**: red`。

- いずれも WEAK 候補。親 SPEC-LGX-010 REQ.03（+ TP-LGX-010 で green 確定済）が機能セマンティクスとしては既に答えており、争点は「UC が本文に書いた**二段解決ロジック**（token→label→ID フォールバック）の終端（AF2: 解決失敗の最終分岐 / R6: 解決結果の出力反映）を、フロー記述の連鎖整合として明示すべきか」である。
- 新規 UC のため UC フロー記述は本 TP で初精査。二段解決という分岐ロジックを UC 本文に持ち込んだことに伴う「分岐の終端の明示不足」であり、SPEC 委譲で機能は担保されるが連鎖整合の観点では追記が望ましい。
- 敵対的精査パスで GENUINE / WEAK / OUT_OF_SCOPE を確定し、WEAK 確定分は人間裁定（UC フローへの追記 or drop）を経て close。全観点 GREEN 化後に本 TP を green へ更新する。

## 5. 観点ナレッジベース参照

この TP の生成時に参照したナレッジベース章節:

- `docs/perspectives/core-perspectives.md` §UC レベル観点（基本フロー / 代替フロー / 例外フロー / アクター遷移と権限 / データフロー）
- `docs/perspectives/core-perspectives.md` §汎用観点（境界値=drift 値域 [0.0,2.0]・非有限スコア NaN/±Inf・終了コード境界 / エラーハンドリング=部分成功・cause チェーン / 状態遷移=baseline 有無 / 入力検証=`--against` 段階分離 / ロギング・観測性=stderr 分離）
- `docs/perspectives/ux-perspectives.md` §エラー・例外の UX（モデル解決失敗 stderr 通知・baseline 不在 INFO の可読性に適用）
- 親 SPEC: SPEC-LGX-010.REQ.01/REQ.03/REQ.06/REQ.07/REQ.09、§4 SCORE-INV-1/SCORE-INV-2
- 関連 SPEC: SPEC-LGX-006.REQ.04/REQ.10（cosine 値域・次元/ゼロベクトル時 drift Error 維持・model_version 完全一致）
- 委譲先 TP: TP-LGX-010（embedding 運用・監査 SPEC レベル観点、green 確定済）/ TP-LGX-006（embedding・ドリフト検出 SPEC レベル観点、green 確定済）
- 関連 UC: UC-LGX-012（snapshot ベースライン供給）/ UC-LGX-011（drift_threshold キャリブレーション）
- LGX-COMPAT-001 §3 / §4 #5（`--models-dir` グローバルオプション・drift 凍結済引数契約）

UX 層観点（Undo/フォーカス/タッチ等）は CLI 対比コマンドには本質的に N/A のため、エラー UX（モデル解決失敗・baseline 不在の通知可読性）以外はスキップした。

## 6. 履歴

| 日付 | 変更内容 |
|---|---|
| 2026-06-13 | 初版作成。UC レベル観点 24 件（GREEN 22 / RED 2）。GAP-LGX-301（AF2 token 二段解決の最終分岐の連鎖明示不足）/ GAP-LGX-302（R6 baseline_source の解決後 id 反映の連鎖明示不足）を起票 |

## 7. 解消（2026-06-13、敵対的精査裁定後）

本 TP が起票した GAP[UC] は全て closed。内訳: **WEAK=方針B（委譲容認）** / **REFUTED=棄却** / **GENUINE=UC 修正で解消**（A/B/C、人間承認 2026-06-13）。§3 表の判定列は初版（起票時）の draft 判定を保持する（精査の履歴として温存）。全 RED 観点は上記裁定で解消したため本 TP は **green**。各 GAP の最終状態は当該 GAP ファイル（§5）と docs/spec-change-proposals/2026-06-13_uc-gap-adversarial-review.md を参照。
