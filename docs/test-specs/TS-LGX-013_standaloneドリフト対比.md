Document ID: TS-LGX-013

# TS-LGX-013: standalone ドリフト対比（drift）のテスト仕様

> TS は **上流 TP の翻訳**。ゼロから観点を考えない。各 TP 観点を DD-LGX-013 で確定した `legixy-embed` drift サブシステムの型・関数シグネチャに即した具体的な入力・期待出力・前提条件へ展開する。

**親 DD**: DD-LGX-013
**継承 TP**: TP-LGX-010（TP[SPEC] embedding 運用・監査、drift 関連観点 = §2.1 B8 / §2.2 E2〜E6,E9 / §2.3 S2 / §2.6 V4〜V6 / §2.7 I1 / §2.10 D1,D8）, TP-LGX-023（TP[UC] UC-013 フロー、§2.1 BF1〜BF4 / §2.2 AF1〜AF6 / §2.3 EF1〜EF5 / §2.4 AT1〜AT3 / §2.5 DF1〜DF3 / §2.6 R1〜R6）, TP-LGX-009（DD-013 §8 引用、drift サブシステムの Unit/Integration/Property 分類の出典）

## 1. 対象範囲

このテスト仕様がカバーする DD-LGX-013 の関数 / 型:

- DD-LGX-013 §3 `legixy_embed::drift::run(graph: &TraceGraph, store: &EmbeddingStore, config: &Config, artifact_id: &Id, against: AgainstSpec) -> Result<DriftResult, DriftError>`
- DD-LGX-013 §3 `legixy_embed::drift::resolve_model(config: &Config, models_dir_override: Option<&Path>) -> Result<ResolvedModel, DriftError>`
- DD-LGX-013 §3 `legixy_embed::drift::parse_against(raw: Option<&str>) -> Result<AgainstSpec, DriftError>`
- DD-LGX-013 §3 `legixy_embed::drift::compute_drift(current: &CurrentEmbedding, baseline: &BaselineEmbedding) -> Result<f32, DriftError>`
- DD-LGX-013 §3 `legixy_embed::drift::exit_code(result: &Result<DriftResult, DriftError>) -> i32`
- DD-LGX-013 §2 型: `DriftResult`{drift:Option\<f32\>, baseline_available:bool, baseline_source:Option\<BaselineSource\>} / `ResolvedModel`{model_version,dim,source} / `ModelSource`{Flag,EnvLgx,EnvTe,ConfigFile} / `ArtifactRef` / `CurrentEmbedding` / `BaselineEmbedding`{..,source:BaselineSource} / `BaselineSource`{Embeddings,Snapshot(String)} / `AgainstSpec`{Embeddings,SnapshotToken(String),SnapshotLabelExplicit(String)} / `ModelVersion`(newtype) / `IntegrityCheckResult`{Ok,DimMismatch,ModelVersionMismatch} / `DriftError`（11 バリアント）

委譲（本 TS 対象外）:
- **現行 embedding の生成本体**（ONNX 推論・mean pooling・L2 正規化・content_range 切り出し）の数値妥当性と決定性 → TS-LGX-007（embed 生成。DD-013 §1「embed 生成は TS-007 委譲」/ §4 `drift/embed.rs` は embed_all と同一経路）。本 TS は `compute_drift` 以降の対比ロジックに集中する。
- **cosine 類似度の数値妥当性・値域 [-1,1]**（SPEC-LGX-006 REQ.04 = エンジン責務）→ TS-LGX-007。本 TS は drift = 1.0 − cosine の値域 [0.0,2.0] への写像と非有限防御のみを検証する。
- **EmbeddingStore の行ロード・snapshot 行複製のスキーマ整合・WAL** → DD-007 所有（`EmbeddingStore` は DD-007 所有 → 本サブシステムへ委譲渡し）/ NFR-LGX-001 PERF.07。本 TS は store を test double（空ストア / 既知行 / snapshot 行）として与え、drift 側の解決・照合・算出を検証する。
- **性能予算（384 次元スループット）** → DD-013 §8 Bench / NFR-LGX-001。
- **並行アクセス整合性・TOCTOU**（外部更新中の drift）→ NFR-LGX-001 SEC.02 / REL.07（TP-LGX-010 §2.4 C4 / §2.3 S6 / TP-LGX-023 AT3 が NFR 委譲を確立済）。本 TS は read-only 不変（実行前後でストア/グラフ/FS 不変）のみ検証する。
- **snapshot / report / calibrate コマンド本体**（TP-LGX-010 の B1〜B7,B10 / E1,E7,E8,E10〜E12 / S1,S3〜S6 / P1〜P5 / V1〜V3,V7 / I2〜I5 / L1〜L6 / F1〜F3 / D2〜D7 等）→ 当該 UC の TS（snapshot=UC-012, calibrate=UC-011, report=別 UC）。本 TS は drift サブシステムが SPEC-010 REQ.03/REQ.06/REQ.07/REQ.09 と UC-013 フローを DD-013 の型で正しく具体化しているかに集中する。

本 TS は「`legixy_embed::drift::*` が SPEC-LGX-010 REQ.03（standalone 対比）/ REQ.06（決定性）/ REQ.07（read-only・DB 不在時非作成）/ REQ.09（非有限スコア）と UC-013 フローを、DD-013 の型・終了コード規約・非対称性原則で正しく実装しているか」を検証する。

## 2. ケース一覧

### ケース 1: `parse_against(None)` → `AgainstSpec::Embeddings`

- **観点出典**: TP-LGX-023 §2.2 AF1（`--against` 省略 = embeddings ストア現行行）, TP-LGX-010 §2.7 I1
- **分類**: Unit
- **前提**: `--against` 未指定
- **入力**: `parse_against(None)`
- **期待**: `Ok(AgainstSpec::Embeddings)`
- **境界条件**: 省略時のデフォルト経路（baseline = embeddings ストア現行行）

### ケース 2: `parse_against(Some("snapshot:rel-2024"))` → `SnapshotToken`（label 優先・id フォールバック）

- **観点出典**: TP-LGX-023 §2.2 AF1, TP-LGX-010 §2.7 I1（3 形式受理）
- **分類**: Unit
- **前提**: `snapshot:<token>` 曖昧形式（label とも snapshot_id とも解釈可能）
- **入力**: `parse_against(Some("snapshot:rel-2024"))`
- **期待**: `Ok(AgainstSpec::SnapshotToken("rel-2024".into()))`（`snapshot:` プレフィクスを剥がし token 部のみ保持）
- **境界条件**: 曖昧形式 = label 優先 → snapshot_id フォールバックの解析（解決自体は ケース 12/13）

### ケース 3: `parse_against(Some("snapshot:label:nightly"))` → `SnapshotLabelExplicit`

- **観点出典**: TP-LGX-023 §2.2 AF1, TP-LGX-010 §2.7 I1（`snapshot:label:<L>` 明示判別形式）
- **分類**: Unit
- **前提**: `snapshot:label:<L>` 明示 label 形式（`label:` を含む label の escape 経路 = TP-010 §2.7 I5 で確立）
- **入力**: `parse_against(Some("snapshot:label:nightly"))`
- **期待**: `Ok(AgainstSpec::SnapshotLabelExplicit("nightly".into()))`
- **境界条件**: 明示 label 形式 = label 解決失敗を exit 1 にする経路（曖昧形式の exit 0 と非対称、ケース 14 と対）

### ケース 4: `parse_against(Some("foo-bar"))`（`snapshot:` プレフィクス欠如）→ `DriftError::InvalidAgainstFormat`

- **観点出典**: TP-LGX-010 §2.2 E2（プレフィクス欠如 → exit 1）, TP-LGX-023 §2.6 R5（アプリ層判定 = exit 1、clap exit 2 でない）, §2.2 AF1（1a）
- **分類**: Unit
- **前提**: `snapshot:` プレフィクスを欠く `--against` 値
- **入力**: `parse_against(Some("foo-bar"))`
- **期待**: `Err(DriftError::InvalidAgainstFormat { value: "foo-bar".into() })`。`exit_code(&Err(..)) == 1`
- **境界条件**: アプリ層 reject = exit 1（clap 構文層の exit 2 と区別。DD §2.3 終了コード規約）

### ケース 5: `compute_drift` 同一ベクトル（cosine=1.0）→ drift=0.0（下限）

- **観点出典**: TP-LGX-010 §2.1 B8（drift 値域 [0.0,2.0]）, TP-LGX-023 §2.6 R2（drift = 1.0 − cosine）
- **分類**: Unit
- **前提**: `current.vector == baseline.vector`（L2 正規化済、dim/model_version 一致）
- **入力**: `compute_drift(&current, &baseline)`（両者 `vector = [0.6, 0.8]`, dim=2, 同一 model_version）
- **期待**: `Ok(0.0)`（cosine=1.0 → drift = 1.0 − 1.0）。`drift.is_finite()`
- **境界条件**: 値域下限 0.0（完全一致）

### ケース 6: `compute_drift` 直交ベクトル（cosine=0.0）→ drift=1.0（中央）

- **観点出典**: TP-LGX-010 §2.1 B8, TP-LGX-023 §2.6 R2
- **分類**: Unit
- **前提**: `current.vector ⊥ baseline.vector`（dim/model_version 一致）
- **入力**: `compute_drift(&current, &baseline)`（`current=[1.0,0.0]`, `baseline=[0.0,1.0]`）
- **期待**: `Ok(1.0)`（cosine=0.0 → drift=1.0）
- **境界条件**: 値域中央 1.0（無相関）

### ケース 7: `compute_drift` 逆向きベクトル（cosine=−1.0）→ drift=2.0（上限）

- **観点出典**: TP-LGX-010 §2.1 B8（値域上限 2.0）, TP-LGX-023 §2.6 R2
- **分類**: Unit
- **前提**: `current.vector == −baseline.vector`（dim/model_version 一致）
- **入力**: `compute_drift(&current, &baseline)`（`current=[1.0,0.0]`, `baseline=[-1.0,0.0]`）
- **期待**: `Ok(2.0)`（cosine=−1.0 → drift = 1.0 − (−1.0)）
- **境界条件**: 値域上限 2.0（完全逆相関）。0/中央/最大 を別ケースに分離（ケース 5/6/7）

### ケース 8: `compute_drift` 非有限スコア（NaN/±Inf）→ `DriftError::NonFiniteScore`

- **観点出典**: TP-LGX-023 §2.3 EF4（6a 非有限 → exit 1, SPEC-LGX-010 REQ.09）, §2.5 DF2（非有限値が `--json` に現れない）, DD-013 §6 非有限スコア防御
- **分類**: Unit
- **前提**: cosine 計算結果が `f32::is_finite() == false`（例: ゼロベクトル混入でゼロ除算 → NaN、または極端値で Inf）
- **入力**: `compute_drift(&current, &baseline)`（cosine が NaN/±Inf を生む病的ベクトル対）
- **期待**: `Err(DriftError::NonFiniteScore)`。`exit_code(&Err(..)) == 1`。serde_json シリアライズ前に捕捉（serde_json 挙動に非依存、DD §6）
- **境界条件**: 次元/model_version reject をすり抜けた残余の数値破綻（EF4）。非有限値は `Ok` の `drift` に絶対に格納されない

### ケース 9: 次元不一致 → `IntegrityCheckResult::DimMismatch` → `DriftError::DimMismatch` → exit 1

- **観点出典**: TP-LGX-010 §2.2 E6（次元不一致 → exit 1）, §2.10 D8（次元不一致 = SCORE-INV-2 検出手段）, TP-LGX-023 §2.1 BF4 / §2.3 EF5（5a, 壊れた状態を隠さない）
- **分類**: Unit（integrity.rs）
- **前提**: `current.dim=384`, `baseline.dim=256`（model_version 照合到達前に次元検査が先行 = BF4 二段検査の一段目）
- **入力**: 整合性検査 → `run` 経由で `DriftError::DimMismatch{current_dim:384, baseline_dim:256}`
- **期待**: `Err(DriftError::DimMismatch{current_dim:384, baseline_dim:256})`。`exit_code == 1`
- **境界条件**: 次元検査が第一段。明示対比は失敗を隠さない（check の skip+Warning とは別、E6）

### ケース 10: 次元一致・model_version 不一致 → `IntegrityCheckResult::ModelVersionMismatch` → `DriftError::ModelVersionMismatch` → exit 1

- **観点出典**: TP-LGX-010 §2.6 V6（同一次元・別 model_version の SCORE-INV-2 検出）, TP-LGX-023 §2.1 BF4 / §2.6 R4（model_version 完全一致 = SCORE-INV-2 一次検出、次元検査は補完）
- **分類**: Unit（integrity.rs）
- **前提**: `current.dim == baseline.dim == 384`（次元検査をすり抜ける）だが `current.model_version != baseline.model_version`（例: ONNX 差し替え・前処理プロファイル変更で SHA256 前 16 hex が変化）
- **入力**: 整合性検査 → `run` 経由で `DriftError::ModelVersionMismatch{current, baseline}`
- **期待**: `Err(DriftError::ModelVersionMismatch{current: "<m1>", baseline: "<m2>"})`。`exit_code == 1`
- **境界条件**: 同一次元のモデル切替は次元不一致で捕捉できない → model_version 文字列の完全一致照合が一次検出（SCORE-INV-2、SPEC-LGX-006 REQ.10 完全一致委譲）。BF4 の二段検査の二段目

### ケース 11: 整合性検査 OK（次元一致・model_version 一致）→ `IntegrityCheckResult::Ok` → drift 算出続行

- **観点出典**: TP-LGX-023 §2.1 BF1（Step5 検査 → Step6 算出の連鎖整合）, §2.5 DF3（照合入力の供給）
- **分類**: Unit（integrity.rs）
- **前提**: `current.dim == baseline.dim`, `current.model_version == baseline.model_version`
- **入力**: 整合性検査
- **期待**: `IntegrityCheckResult::Ok`。後続 `compute_drift` へ進行（drift 値を算出）
- **境界条件**: 二段検査の通過条件（次元一致 AND model_version 完全一致）。BF4 の連鎖が Step6 へ到達

### ケース 12: `run` E2E（embed 済みノード・baseline=embeddings 現行行）→ `DriftResult{drift:Some(_), baseline_available:true, baseline_source:Some(Embeddings)}` → exit 0

- **観点出典**: TP-LGX-023 §2.1 BF1/BF3（基本フロー Step1〜7 連鎖・成功時事後条件）, TP-LGX-010 §2.7 I1（省略時 = embeddings）, §2.5 DF3
- **分類**: Integration
- **前提**: graph.toml に `artifact_id` 登録済・現行ファイル存在・モデル解決可能・embeddings ストアに当該 artifact の現行行あり（dim/model_version 一致）。`against = AgainstSpec::Embeddings`
- **入力**: `run(&graph, &store, &config, &artifact_id, AgainstSpec::Embeddings)`
- **期待**: `Ok(DriftResult{artifact_id, drift: Some(d), baseline_available: true, baseline_source: Some(BaselineSource::Embeddings)})` で `0.0 <= d <= 2.0`。`exit_code(&Ok(..)) == 0`
- **境界条件**: 標準成功パス。`baseline_source == Embeddings`（DF2 正常スキーマ 4 キー相当）

### ケース 13: `run` baseline=`snapshot:<token>`（label 優先解決成功）→ `baseline_source:Some(Snapshot(<id>))` 解決後 id 反映

- **観点出典**: TP-LGX-023 §2.6 R6（label 入力 → 出力 baseline_source は解決後 snapshot_id）, §2.2 AF1/AF6, TP-LGX-010 §2.7 I1
- **分類**: Integration
- **前提**: snapshot 行が label="nightly" で 1 件存在（snapshot_id="snap-abc123"）。`against = AgainstSpec::SnapshotToken("nightly")`。dim/model_version は現行と一致
- **入力**: `run(&graph, &store, &config, &artifact_id, AgainstSpec::SnapshotToken("nightly".into()))`
- **期待**: `Ok(DriftResult{.., baseline_available: true, baseline_source: Some(BaselineSource::Snapshot("snap-abc123".into()))})`。出力は label でなく**解決後 snapshot_id**（R6）
- **境界条件**: token→label 解決成功 → baseline_source に解決後 id を反映（label のまま返さない）

### ケース 14: `run` baseline=`snapshot:label:<L>`（明示 label 形式・解決失敗）→ `DriftError::LabelNotFound` → exit 1

- **観点出典**: TP-LGX-010 §2.2 E7 相当（明示 label 解決失敗 → exit 1）, TP-LGX-023 §2.2 AF5 / §2.3 EF1（5a/前提崩壊系 exit 1）, DD-013 §2.3（`snapshot delete label:<L>` と対称）
- **分類**: Integration
- **前提**: snapshot ストアに label="missing" が 0 件。`against = AgainstSpec::SnapshotLabelExplicit("missing")`
- **入力**: `run(&graph, &store, &config, &artifact_id, AgainstSpec::SnapshotLabelExplicit("missing".into()))`
- **期待**: `Err(DriftError::LabelNotFound{label: "missing"})`。`exit_code == 1`
- **境界条件**: 明示 label 形式の解決失敗 = exit 1（曖昧形式 SnapshotToken の行不在 = exit 0、ケース 15 と非対称。DD §2.3）

### ケース 15: `run` baseline=`snapshot:<token>`（曖昧形式・行不在）→ `DriftResult{baseline_available:false}` → exit 0

- **観点出典**: TP-LGX-010 §2.2 E9（双方解決失敗 = baseline-absent exit 0）, TP-LGX-023 §2.2 AF2/AF4（4a baseline 不在 exit 0）, §2.5 DF2（baseline 不在スキーマ drift:null）
- **分類**: Integration
- **前提**: token="ghost" が label にも snapshot_id にも一致しない（曖昧形式 SnapshotToken で解決したが当該行なし）。`against = AgainstSpec::SnapshotToken("ghost")`
- **入力**: `run(&graph, &store, &config, &artifact_id, AgainstSpec::SnapshotToken("ghost".into()))`
- **期待**: `Ok(DriftResult{artifact_id, drift: None, baseline_available: false, baseline_source: None})`。`exit_code(&Ok(..)) == 0`
- **境界条件**: 曖昧形式の token 未解決最終分岐 = exit 0（正常ライフサイクル。明示 label のケース 14 と非対称）。drift=None → `--json` で drift:null（DF2）

### ケース 16: `run` engine.db 不在 → DB 新規作成せず → `DriftResult{baseline_available:false}` → exit 0【v3 差分・REQ.07】

- **観点出典**: TP-LGX-010 §2.3 S2（DB 不在 ≡ 空ストア・非作成）, TP-LGX-023 §2.3 EF2（engine.db 不在時も DB 新規作成しない）, DD-013 §3 注記（REQ.07【v3 差分】）
- **分類**: Integration
- **前提**: `.legixy/engine.db` ファイルが存在しない。`EmbeddingStore` は空ストア相当を返す（`run` は `Err(DriftError::Db)` を受け取らない）。`against = AgainstSpec::Embeddings`
- **入力**: `run(&graph, &store, &config, &artifact_id, AgainstSpec::Embeddings)`
- **期待**: `Ok(DriftResult{baseline_available: false, drift: None, baseline_source: None})`。`exit_code == 0`。**実行後も `.legixy/engine.db` が新規作成されていない**（FS 検査）
- **境界条件**: DB 不在 = baseline 不在の上位ケース、かつ DB ファイルを副作用で作らない（REQ.07【v3 差分】の核心。Err にしない・作成しない）

### ケース 17: `run` artifact_id が graph.toml 未登録 → `DriftError::ArtifactNotFound` → exit 1

- **観点出典**: TP-LGX-010 §2.2 E4（artifact 不在 → exit 1）, TP-LGX-023 §2.2 AF4（3a graph.toml 不在 exit 1）, §2.3 EF5（壊れた状態 exit 1）
- **分類**: Integration
- **前提**: `artifact_id` が graph.toml の `[[nodes]]` に未登録
- **入力**: `run(&graph, &store, &config, &unregistered_id, AgainstSpec::Embeddings)`
- **期待**: `Err(DriftError::ArtifactNotFound{artifact_id})`。`exit_code == 1`
- **境界条件**: 成果物未登録 = exit 1。baseline 不在 exit 0 と区別（4a との非対称、AF4）

### ケース 18: `run` graph.toml は登録主張だが現行ファイル欠落 → `DriftError::FileNotFound` → exit 1

- **観点出典**: TP-LGX-010 §2.2 E5（現行ファイル欠落 → exit 1, 非対称性注記）, TP-LGX-023 §2.2 AF4（3b）/ §2.3 EF5（壊れた状態を隠さない）
- **分類**: Integration
- **前提**: `artifact_id` は graph.toml 登録済だが `file_path` の実ファイルが存在しない
- **入力**: `run(&graph, &store, &config, &artifact_id, AgainstSpec::Embeddings)`
- **期待**: `Err(DriftError::FileNotFound{artifact_id, path})`。`exit_code == 1`
- **境界条件**: graph.toml が主張する壊れた状態 = exit 1（3b）。baseline 不在 exit 0（4a）との意図的非対称（EF5）

### ケース 19: `resolve_model` 解決順序 4 経路（Flag > EnvLgx > EnvTe > ConfigFile）

- **観点出典**: TP-LGX-010 §2.6 V4（4 段解決順）/ V5（両変数同時 → LGX 優先）, TP-LGX-023 §2.2 AF3（モデル解決系分岐網羅）
- **分類**: Unit（model.rs）
- **前提**: 各経路の有効/無効を組み合わせた 4 シナリオ:
  - (a) `models_dir_override=Some(path)` 提供 → `ResolvedModel.source == ModelSource::Flag`
  - (b) override=None・`LGX_MODELS_DIR` set → `source == EnvLgx`
  - (c) override=None・`LGX_MODELS_DIR` unset・`TE_MODELS_DIR` set → `source == EnvTe`
  - (d) 全 env unset・`config.semantic.model` + project_root/models/ で解決 → `source == ConfigFile`
  - (e) `LGX_MODELS_DIR` と `TE_MODELS_DIR` 両 set → `source == EnvLgx`（LGX 優先、V5）
- **入力**: `resolve_model(&config, models_dir_override)` を各シナリオで
- **期待**: 各 `Ok(ResolvedModel{ source: <期待 ModelSource> })`
- **境界条件**: 解決優先順位の一意性（Flag > EnvLgx > EnvTe > ConfigFile）

### ケース 20: `resolve_model` 旧名 `TE_MODELS_DIR` 使用時の stderr Info 案内

- **観点出典**: TP-LGX-010 §2.6 V4（旧名フォールバック + Info 案内）, TP-LGX-023 §2.2 AF3（2b）/ §2.3 EF3（severity・出力先 stderr）
- **分類**: Integration
- **前提**: `LGX_MODELS_DIR` unset・`TE_MODELS_DIR` set で解決（`ModelSource::EnvTe`）
- **入力**: `resolve_model(&config, None)`（CLI 層が stderr Info を発行）
- **期待**: `Ok(ResolvedModel{source: EnvTe})` + stderr に旧名使用の Info 案内（`legixy-cli` 層担当。本サブシステムは `ModelSource::EnvTe` を返し CLI 層が案内を発行）
- **境界条件**: 旧名フォールバックは機能継続（exit 非影響）+ 案内（移行誘導）。diag 出力先は stderr（OBS.02）

### ケース 21: `resolve_model` 全経路失敗 → `DriftError::ModelNotFound{tried_paths}` → exit 1

- **観点出典**: TP-LGX-010 §2.2 E3（モデル解決失敗 → exit 1 + 試行内容 stderr）, TP-LGX-023 §2.2 AF3（2a）/ §2.3 EF1
- **分類**: Integration
- **前提**: override=None・全 env unset・config 経路にもモデルなし（全 4 経路で発見失敗）
- **入力**: `resolve_model(&config, None)`
- **期待**: `Err(DriftError::ModelNotFound{tried_paths: vec![...]})` で `tried_paths` が試行した全経路を非空で列挙（stderr 通知の情報源、L3 情報十分性）。`exit_code == 1`
- **境界条件**: 全経路試行後に失敗（部分試行で早期 Err にしない）。試行内容を保持

### ケース 22: `exit_code` 契約（baseline あり=0 / baseline 不在=0 / Err=1）

- **観点出典**: TP-LGX-010 §2.2 E1（exit 3 分類）, TP-LGX-023 §2.2 AF5（収束）/ §2.6 R5（exit 契約一致）, DD-013 §3 `exit_code` 不変条件
- **分類**: Contract
- **前提**: 3 ケース — (a) `Ok(DriftResult{baseline_available:true})`, (b) `Ok(DriftResult{baseline_available:false})`, (c) `Err(任意 DriftError)`
- **入力**: それぞれ `exit_code(&result)`
- **期待**: (a)→0、(b)→0、(c)→1。**baseline_available の true/false に依らず Ok は 0**（DD §3 不変条件）。clap 構文エラー（exit 2）は本関数の対象外（CLI 層・LGX-COMPAT-001 §3 凍結）
- **境界条件**: exit 0/1 の二値（本関数）。exit 2 は構文層限定（DD §2.3）。LGX-COMPAT-001 §4 #5 凍結契約

### ケース 23: `run` の read-only 不変（graph / store / FS を変更しない）

- **観点出典**: TP-LGX-010 §2.5 P2（読取系非破壊）, TP-LGX-023 §2.3 EF2 / §2.4 AT1（読取専用・全アクター同一権限）, DD-013 §3/§5（read-only 借用）
- **分類**: Property/Integration
- **前提**: 任意の入力（成功・baseline 不在・各種 Err いずれも）
- **入力**: `run(...)` 実行前後の graph.toml / engine.db（存在時）/ 成果物ファイルのハッシュ
- **期待**: 実行前後でハッシュ不変（`&TraceGraph`/`&EmbeddingStore`/`&Config`/`&Id` 借用のみ、§5 read-only）。Err 時も中間状態破壊なし。DB 不在時は DB を作らない（ケース 16 と整合）
- **境界条件**: 借用による read-only 保証（SPEC-LGX-010 REQ.07）。3 アクター（設計者/運用者/QA）同一権限の根拠（AT1）

### ケース 24: `compute_drift` の決定性（property: 同一ベクトル対 → 同一 drift 値）

- **観点出典**: TP-LGX-010 §2.10 D1（読取系決定性、同一入力→同一出力）, TP-LGX-023 §2.6 R3（読取系決定性事後条件）, DD-013 §3 `run` 冪等性
- **分類**: Property-based（proptest）
- **生成器**: 任意の `Vec<f32>` ベクトル対（同一 dim、有限値、L2 正規化済とみなす）を proptest で生成
- **不変条件**: 同一 `(current.vector, baseline.vector)` に対し `compute_drift` は常に同一 `f32` を返す（ビット一致、SCORE-INV-1 決定性順序）。複数回呼出しで分岐なし
- **反例ハンドリング**: shrink して最小の非決定例を記録（実装上は浮動小数演算順序の固定で決定性を保証）

### ケース 25: `compute_drift` の値域 [0.0, 2.0]（property: 任意の有限正規化ベクトル対）

- **観点出典**: TP-LGX-010 §2.1 B8（値域 [0.0,2.0]）, TP-LGX-023 §2.6 R2, DD-013 §3 `compute_drift` 不変条件
- **分類**: Property-based（proptest）
- **生成器**: 任意の dim（>=1）の有限値ベクトル対を生成し L2 正規化（ゼロベクトルは生成器で除外 or 非有限ケース 8 へ分離）
- **不変条件**: `compute_drift` が `Ok(d)` を返す全ての有限正規化入力で `0.0 <= d <= 2.0`。非有限が混入したら `Err(NonFiniteScore)`（`Ok` に域外値は現れない）
- **反例ハンドリング**: shrink して値域違反 or 非有限漏れの最小例を記録

### ケース 26: drift と check 内 Drift の別機能性（baseline_source 出力契約による責務境界）

- **観点出典**: TP-LGX-023 §2.6 R1（check 内 Drift content_hash 変化 Warning との別機能性）, §2.5 DF1（入出力データ型・stdout/stderr 分離）
- **分類**: Integration（責務境界確認）
- **前提**: standalone drift（`legixy_embed::drift::run`、embedding 対比の定量値）と check 内 Drift（SPEC-LGX-004 / SPEC-LGX-006 REQ.05 の content_hash 変化 Warning）は別経路
- **入力**: `run(...)` が返す `DriftResult` の構造
- **期待**: `DriftResult` は定量 drift 値（`Option<f32>`）+ `baseline_source` を持ち、severity / Warning 概念を持たない（check の finding とは型が別）。drift 値 = stdout、診断 = stderr（CLI 層、DF1）
- **境界条件**: standalone drift（計測値・severity なし）/ check 内 Drift（判定・Warning）の機能境界（R1）。本サブシステムは算出と出力のみ、判断材料の解釈はアクター責務（AT2）

## 3. 観点カバレッジ表

| TP § | 観点 | カバーする TS ケース |
|---|---|---|
| TP-010 §2.1 B8 drift 値域 [0.0,2.0] | 境界値 | ケース 5, 6, 7（下限/中央/上限）, 25（property）|
| TP-010 §2.1 B1〜B7,B9,B10 snapshot/calibrate/report 境界 | 境界値 | UC-012/011/report 各 TS へ委譲（B9 NaN は本 TS の drift 経路 = ケース 8）|
| TP-010 §2.2 E1 exit 3 分類 | エラー | ケース 22（0/1 は本関数、2 は構文層委譲）|
| TP-010 §2.2 E2 プレフィクス欠如 exit 1 | エラー | ケース 4 |
| TP-010 §2.2 E3 モデル解決失敗 exit 1 | エラー | ケース 21 |
| TP-010 §2.2 E4 artifact 不在 exit 1 | エラー | ケース 17 |
| TP-010 §2.2 E5 現行ファイル欠落 exit 1 | エラー | ケース 18 |
| TP-010 §2.2 E6 次元不一致 exit 1 | エラー | ケース 9 |
| TP-010 §2.2 E7 label 解決失敗 exit 1 | エラー | ケース 14（drift の明示 label 形式）|
| TP-010 §2.2 E9 双方解決失敗 baseline-absent exit 0 | エラー | ケース 15 |
| TP-010 §2.2 E8,E10〜E12 snapshot/report/calibrate | エラー | UC-012/011/report 各 TS へ委譲 |
| TP-010 §2.3 S2 DB 不在 ≡ 空ストア・非作成 | 状態 | ケース 16 |
| TP-010 §2.3 S1,S3〜S6 snapshot ライフサイクル | 状態 | UC-012 TS へ委譲 |
| TP-010 §2.5 P2 読取系非破壊 | 永続化 | ケース 16, 23 |
| TP-010 §2.5 P1,P3〜P5 snapshot 永続化 | 永続化 | UC-012 TS へ委譲（EmbeddingStore は DD-007 所有）|
| TP-010 §2.6 V4 解決順 4 段・旧名 Info | 互換 | ケース 19, 20 |
| TP-010 §2.6 V5 両変数同時 LGX 優先 | 互換 | ケース 19(e) |
| TP-010 §2.6 V6 同一次元・別 model_version | 互換 | ケース 10 |
| TP-010 §2.6 V1〜V3,V7 引数契約/snap-/撤去時期 | 互換 | UC-012/CLI 凍結契約・将来 SPEC へ委譲 |
| TP-010 §2.7 I1 --against 3 形式受理 | 入力 | ケース 1, 2, 3 |
| TP-010 §2.7 I2〜I5 delete target/label 境界 | 入力 | UC-012 TS へ委譲 |
| TP-010 §2.10 D1 読取系決定性 | 決定性 | ケース 24（property）|
| TP-010 §2.10 D8 次元不一致 = SCORE-INV-2 検出 | 領域 | ケース 9, 10 |
| TP-010 §2.10 D2〜D7 snapshot/report/calibrate 決定性 | 領域 | UC-012/011/report 各 TS へ委譲 |
| TP-010 §2.4 C1〜C4 並行性 | 並行 | NFR-LGX-001 SEC.02/REL.07 へ委譲（ケース 23 read-only 不変が前提担保）|
| TP-010 §2.8 L1〜L6 / §2.9 F1〜F3 観測性/CLI 契約 | 観測/境界 | stderr 出力は CLI 層（ケース 20 で source 返却まで）、MCP 非公開・json スキーマは CLI 層/UC-012 へ委譲 |
| TP-023 §2.1 BF1 ステップ連鎖整合 | UC フロー | ケース 11, 12 |
| TP-023 §2.1 BF2 モデル依存の射程 | UC フロー | ケース 19, 20, 21（drift のみモデル依存）|
| TP-023 §2.1 BF3 成功時事後条件 | UC フロー | ケース 12, 23 |
| TP-023 §2.1 BF4 二段検査の連鎖 | UC フロー | ケース 9, 10, 11 |
| TP-023 §2.2 AF1 --against 形式分岐網羅 | UC フロー | ケース 1, 2, 3, 4, 12, 13 |
| TP-023 §2.2 AF2 token 二段解決の最終分岐 | UC フロー | ケース 15（曖昧形式の未解決 = exit 0）|
| TP-023 §2.2 AF3 モデル解決系分岐網羅 | UC フロー | ケース 19, 20, 21 |
| TP-023 §2.2 AF4 exit1/exit0 非対称 | UC フロー | ケース 14, 15, 17, 18（3a/3b exit1 vs 4a exit0）|
| TP-023 §2.2 AF5 代替フロー収束 | UC フロー | ケース 22（収束先 exit）+ 14〜18 |
| TP-023 §2.2 AF6 重複 label 最新解決 | UC フロー | ケース 13（label 解決）|
| TP-023 §2.3 EF1 各ステップ失敗パス | UC フロー | ケース 14, 17, 18, 21, 9, 10, 8 |
| TP-023 §2.3 EF2 エラー時状態不変 | UC フロー | ケース 16, 23 |
| TP-023 §2.3 EF3 エラー通知 severity/出力先 | UC フロー | ケース 20（stderr Info）, 15（baseline 不在 INFO 経路）|
| TP-023 §2.3 EF4 非有限スコア失敗パス | UC フロー | ケース 8 |
| TP-023 §2.3 EF5 壊れた状態 exit1 vs 正常 exit0 | UC フロー | ケース 14, 15, 17, 18 |
| TP-023 §2.4 AT1 アクター権限一貫性 | アクター | ケース 23（read-only 同一権限）|
| TP-023 §2.4 AT2 責任境界（算出 vs 解釈）| アクター | ケース 26 |
| TP-023 §2.4 AT3 実行中外部更新/TOCTOU | アクター | NFR-LGX-001 SEC.02/REL.07 へ委譲（ケース 23 read-only 前提）|
| TP-023 §2.5 DF1 入出力型・stdout/stderr 分離 | データ | ケース 26（型分離）, 20（stderr）|
| TP-023 §2.5 DF2 --json スキーマ完全性 | データ | ケース 12（4 キー正常）, 15（3 キー drift:null）, 8（非有限非出力）|
| TP-023 §2.5 DF3 baseline 同一性情報 | データ | ケース 11, 12, 13（embeddings/snapshot 行供給）|
| TP-023 §2.6 R1 check 内 Drift との別機能性 | 領域 | ケース 26 |
| TP-023 §2.6 R2 drift 値定義 | 領域 | ケース 5, 6, 7, 25 |
| TP-023 §2.6 R3 読取系決定性事後条件 | 領域 | ケース 24 |
| TP-023 §2.6 R4 二層検出の位置づけ | 領域 | ケース 9, 10 |
| TP-023 §2.6 R5 終了コード契約一致 | 領域 | ケース 4, 22 |
| TP-023 §2.6 R6 baseline_source 解決後 id 反映 | 領域 | ケース 13 |
| TP-009 §2.2/§2.3 drift Unit/Integration/Property 分類 | テスト分類 | DD-013 §8 引用。embedding 算出本体は TS-LGX-007 委譲、本 TS は drift 対比ロジックをケース 1〜26 で被覆 |

> 継承 TP 観点はすべて本テーブルで TS ケースまたは明示委譲先に mapping 済み（人間ゲート判断対象）。embedding 生成本体・cosine 数値妥当性は TS-LGX-007、snapshot/report/calibrate コマンドは各 UC の TS、性能は bench/NFR、並行/TOCTOU は NFR SEC.02/REL.07 へ委譲し、本 TS は `legixy_embed::drift::*` のベースライン解決・整合性検査（次元/model_version 二段）・非有限防御・drift 算出値域・終了コード非対称性（baseline 不在 exit 0 / 壊れた状態 exit 1）・read-only 不変・決定性に集中する。
