Document ID: TS-LGX-007

# TS-LGX-007: embedding 生成とドリフト検出のテスト仕様

> TS は **上流 TP の翻訳**。ゼロから観点を考えない。各 TP 観点を DD-LGX-007 で確定した `legixy-embed` の Rust 型・関数シグネチャ（§2 型 / §3 公開 API surface 17 関数）に即した具体的な入力・期待出力・前提条件へ展開する。

**親 DD**: DD-LGX-007（`legixy-embed`。Embedder / embed_node / embed_all / EmbeddingStore / compute_model_version / normalize_content / content_hash_for / bulk similarity API / detect_drift / cosine_similarity / histogram。公開 API surface 17 関数、§3 凍結）
**継承 TP**: TP-LGX-006（TP[SPEC] SPEC-006、40 観点）, TP-LGX-017（TP[UC] UC-007 フロー、22 観点）, TP-LGX-004（TP[SPEC] SPEC-004 から意味層委譲分 B4/B5/B6/B7/R4/P1/P3 — DD-LGX-007 §8 が TP-004 を引用しており、check が SPEC-006 へ委譲した意味層スコア観点の受け皿として本 TS が所有）

## 1. 対象範囲

このテスト仕様がカバーする DD-LGX-007 の関数 / 型:

- DD-LGX-007 §3 `normalize_content(raw: &str) -> String`（4 段正規化: BOM 除去→CRLF/CR→LF→NFC→末尾改行 1 個正規化）
- DD-LGX-007 §3 `content_hash_for(content: &str) -> String`（normalize_content 後 UTF-8 への SHA-256 hex 64 桁・小文字）
- DD-LGX-007 §3 `Embedder::new(model_dir, model_version) -> Result<Self, EmbedError>`（model.onnx + tokenizer.json + shape 検証合格時のみ Ok）
- DD-LGX-007 §3 `Embedder::embed_node(&self, text, parent_doc, node_id) -> Result<EmbedResult, EmbedError>`（空テキストは Err にせず呼出側 skip 判定）
- DD-LGX-007 §3 `embed_all(graph, store, embedder, options) -> Result<EmbedReport, EmbedError>`（ノード単位 Tx・部分失敗継続・DB 書込のみ Err 昇格）
- DD-LGX-007 §3 `compute_model_version(model_name, onnx_path, profile, dim) -> Result<String, EmbedError>`（`{name}:{onnx_sha256_8hex}:{profile}:{dim}` 複合キー）
- DD-LGX-007 §3 `EmbeddingStore::{new, is_up_to_date, upsert_with_subnode_meta, load_all, load_embedding, create_snapshot}`
- DD-LGX-007 §3 `compute_edge_scores` / `compute_link_candidates` / `compute_all_pair_scores`（bulk API。本 DD 所有・report=DD-010 が consumer、ADR-LGX-021 §2.3）
- DD-LGX-007 §3 `detect_drift(graph, store, project_root) -> Result<Vec<DriftFinding>, EmbedError>`（EmbeddingMissing 包含・node_id ASC）
- DD-LGX-007 §3 `cosine_similarity(a, b) -> f32`（[-1,1] clamp・ゼロノルム skip 経路）
- DD-LGX-007 §3 `histogram(scores, buckets) -> Vec<Bucket>`（**値域 [0,1] 固定**・域外は [0,1] へ clamp・末尾 inclusive・ストリーミング。正準定義は DD-LGX-011 §3 / TS-LGX-011 所有、本 TS は委譲確認に集中）
- DD-LGX-007 §3 `read_current_content_for_node(node, graph, project_root) -> Result<String, EmbedError>`（embed_all と同一経路の content_range 切り出し共有ヘルパ）
- DD-LGX-007 §2 型: `EmbedResult` / `EmbedOptions` / `NodeFilter`{All,Ids} / `EmbedReport`{generated,skipped,failed,errors} / `EmbedErrorItem` / `EmbeddingRow` / `HashMatchState`{Skip,Regen,Missing} / `DriftFinding` / `DriftKind`{ContentChanged,FileMissing,EmbeddingMissing} / `PreprocessProfile`{Plain,E5Prefix} / `ShapeValidation`{Ok,Invalid} / `Bucket` / `EmbedError`(11 variant)

委譲（本 TS 対象外）:
- **ONNX 推論値のビット再現性 / 多言語スコアの数値妥当性**（D-01 値再現性・D-05 値域の具体閾値）→ DD-007 §7 / SPEC-006 REQ.04（順序のみ保証・ビット再現は対象外）。本 TS は順序決定性・skip 経路のみ検証。
- **drift / report / calibrate / snapshot 運用コマンドの出力仕様・引数・終了コード・JSON スキーマ・閾値推奨**（standalone drift・`--against`・ベースライン選択・p25/p90 推奨）→ SPEC-LGX-010 / **UC-LGX-013（standalone drift）** / TS-LGX-010・TS-LGX-013 へ委譲。本 TS は `legixy-embed` のエンジン（生成・格納・検出・bulk API）に集中。
- **embed スループット PERF.08（≥50 nodes/sec 暫定）** → NFR-LGX-001 / bench（DD-007 §8 Bench 行）へ委譲。
- **並行 embed / busy_timeout / WAL の REL 整合**（C-01 ロック）→ NFR-LGX-001 REL.07/SEC.02 へ委譲（DD-007 §7 単一スレッド前提）。
- **Contextual Retrieval LLM 合成・API キーマスキングの詳細**（REQ.06/07）→ Phase 1 はパススルー骨格（DD-007 §4 contextual.rs / preprocessor.rs）。CR フォールバック `Ok(None)` の構造のみケース化、LLM 呼出挙動・`mask_api_key` は contextual 専用 TS / SEC NFR へ委譲。
- **サブノード ID 生成式・content_range 生成（producer 側）** → TS-LGX-003（LGX-EXT-001 §4.5.1）。本 TS は consumer 側の content_range 防御検証のみ。

## 2. ケース一覧

### ケース 1: 空テキスト（正規化後 0 文字）ノードの embed_all skip

- **観点出典**: TP-LGX-006 §2.1 B-01（空テキスト embedding）, §2.6 I-02（空 range は skip 経路）, TP-LGX-017 §2.3 EF1
- **分類**: Integration
- **前提**: graph に 1 ノード。本文が空白のみ（`normalize_content` 後 0 文字）。`store` 空、`embedder` 正常、`options = EmbedOptions::default()`
- **入力**: `embed_all(&graph, &store, &embedder, EmbedOptions::default())`
- **期待**: `Ok(EmbedReport{ generated: 0, skipped: 1, failed: 0, errors: [] })`。当該ノードは `embed_node` を呼ばず（ゼロベクトル非格納）、`store.load_embedding(id)` は `Ok(None)`（未生成状態）。集約 Warning 1 件 stderr。`exit_code == 0`
- **境界条件**: 空テキスト = skip（Err でも generated でもない。REQ.02 / GAP-LGX-101）

### ケース 2: `embed_node` は空テキストでも Err を返さない（責務境界）

- **観点出典**: TP-LGX-006 §2.1 B-01（生成ベクトルの定義）
- **分類**: Unit
- **前提**: `embedder` 正常。`text` = 空文字列 ""（正規化後 0 文字）
- **入力**: `embedder.embed_node("", None, "SPEC-LGX-001")`
- **期待**: `Err` を返さない（DD §3 不変条件: 空テキストの skip 判定は embed_all / orchestration 層の責務）。embed_node 単体では空入力でも `Ok(EmbedResult)` または呼出側に委ねる形（DD 不変条件どおり、空テキスト Err 化しないこと）
- **境界条件**: skip 判定の所有層分離（embed_node ≠ orchestrator）

### ケース 3: 巨大テキスト（トークン上限超過）の先頭切り捨て + 集約 Warning

- **観点出典**: TP-LGX-006 §2.1 B-02（巨大テキスト切り捨て）
- **分類**: Integration
- **前提**: モデル最大トークン長を超える本文を持つノードを含む graph。`embedder` 正常
- **入力**: `embed_all(&graph, &store, &embedder, EmbedOptions::default())`
- **期待**: チャンク分割せず先頭 N トークンで切り捨てて生成。`generated` に計上（失敗ではない）。`AggregatedWarnings.truncated_count >= 1` → ループ後に集約 Warning 1 件 stderr。`Ok(report)`、`exit_code == 0`
- **境界条件**: 上限超過 = 切り捨て継続（Err ではない。REQ.01 / GAP-LGX-102。v3 差分: v3 無言）

### ケース 4: モデル shape 検証失敗 → Embedder::new が Err → exit 1

- **観点出典**: TP-LGX-006 §2.1 B-03（異常 output shape）, §2.2 E-01, TP-LGX-004 §2.2 E4
- **分類**: Integration
- **前提**: model.onnx は存在するが出力 shape が mean pooling 不能（0 次元 / 軸構造不適合）
- **入力**: `Embedder::new(model_dir, model_version)`
- **期待**: `Err(EmbedError::ModelShapeInvalid{ reason })`。reason にモデルディレクトリ誤配置示唆を含む。呼出側（legixy-cli）が exit 1（全体 abort、embed_all 呼出前）
- **境界条件**: 読込時 shape 検証 ≠ SCORE-INV-2 版照合（別経路。REQ.01 GAP-LGX-103）

### ケース 5: モデル解決/読込失敗 → Embedder::new が Err → exit 1

- **観点出典**: TP-LGX-006 §2.2 E-01/E-02（モデル不在・解決順序）, TP-LGX-017 §2.2 AF1
- **分類**: Integration
- **前提**: model.onnx または tokenizer.json が解決パス上に不在
- **入力**: `Embedder::new(missing_model_dir, model_version)`
- **期待**: `Err(EmbedError::ModelLoadFailed{ path, reason })`。reason に試行したパスのリストを含む（解決順 `--models-dir` > `LGX_MODELS_DIR` > `TE_MODELS_DIR` > 設定）。exit 1
- **境界条件**: 解決順序は ADR-LGX-016。`[semantic] enabled` は embed 実行可否に非影響

### ケース 6: `normalize_content` — BOM 除去（クロスプラットフォーム一致の前提）

- **観点出典**: TP-LGX-006 §2.5 P-03（content_hash 正規化）, TP-LGX-004 §2.6 P1（drift 比較元正規化）
- **分類**: Unit
- **前提**: なし（純関数）
- **入力**: `normalize_content("\u{FEFF}hello")` と `normalize_content("hello")`
- **期待**: 両者とも `"hello"`（末尾改行正規化込み）。BOM (U+FEFF) が除去される
- **境界条件**: 4 段の段 1（BOM 除去、NFR COMPAT.07）

### ケース 7: `normalize_content` — CRLF / CR → LF 統一

- **観点出典**: TP-LGX-006 §2.5 P-03, TP-LGX-004 §2.6 P1
- **分類**: Unit
- **前提**: なし
- **入力**: `normalize_content("a\r\nb\rc")`
- **期待**: `"a\nb\nc"`（末尾改行正規化適用後）。CRLF と単独 CR がともに LF へ
- **境界条件**: 4 段の段 2（改行統一、NFR COMPAT.08）

### ケース 8: `normalize_content` — NFC 正規化（NFD → NFC）

- **観点出典**: TP-LGX-006 §2.5 P-03, §2.6 I-01（正規化前 Unicode）
- **分類**: Unit
- **前提**: なし
- **入力**: `normalize_content("\u{0065}\u{0301}")`（NFD: e + combining acute）と `normalize_content("\u{00E9}")`（NFC: é）
- **期待**: 両者の戻り値が等しい（ともに NFC の é）
- **境界条件**: 4 段の段 3（NFC、I-01 と同一正規化方針 GAP-LGX-114）

### ケース 9: `normalize_content` — 末尾改行揺れ吸収（末尾 1 改行へ正規化）

- **観点出典**: TP-LGX-006 §2.5 P-03（偽 stale/偽 fresh 防止）
- **分類**: Unit
- **前提**: なし
- **入力**: `normalize_content("x")` / `normalize_content("x\n")` / `normalize_content("x\n\n\n")`
- **期待**: 3 者すべて同一文字列（末尾改行 1 個へ正規化。DD §3 不変条件「1 末尾改行に正規化」）
- **境界条件**: 4 段の段 4（末尾正規化。境界: 0 改行 / 1 改行 / 多数改行が同値）

### ケース 10: `normalize_content` 冪等性（property）

- **観点出典**: TP-LGX-006 §2.9 D-04 / §2.5 P-03（決定性）, DD-007 §8 Property 行
- **分類**: Property-based（proptest）
- **生成器**: 任意の `String`（BOM・CR・CRLF・NFD・末尾改行を含みうる Unicode 文字列）
- **不変条件**: `normalize_content(normalize_content(s)) == normalize_content(s)`（2 回適用 == 1 回適用）
- **反例ハンドリング**: shrink して最小の非冪等例を記録

### ケース 11: `content_hash_for` 決定性 + クロスプラットフォーム一致（property）

- **観点出典**: TP-LGX-006 §2.5 P-03/P-04（一意性）, §2.9 D-04, DD-007 §8 Property 行
- **分類**: Property-based（proptest）
- **生成器**: 論理的に同一だが表現差（BOM 有無・CRLF/LF・NFC/NFD・末尾改行数）を持つ文字列ペア
- **不変条件**: 表現差のみのペアは `content_hash_for` が一致。戻り値は常に SHA-256 hex 64 桁・小文字 `[0-9a-f]{64}`。同一論理内容 → 同一ハッシュ（SCORE-INV-1 環境非依存）
- **反例ハンドリング**: shrink して最小の不一致例を記録

### ケース 12: `cosine_similarity` 値域境界（完全一致=1.0 / 直交=0.0 / 反対=-1.0）

- **観点出典**: TP-LGX-006 §2.1 B-05（cosine 値域境界）, TP-LGX-004 §2.1 B4（similarity 閾値境界の値域前提）
- **分類**: Unit
- **前提**: L2 正規化済みベクトル
- **入力**: (a) `cosine_similarity([1,0], [1,0])`、(b) `cosine_similarity([1,0], [0,1])`、(c) `cosine_similarity([1,0], [-1,0])`
- **期待**: (a) `1.0`、(b) `0.0`、(c) `-1.0`。負値は正常出力（意味的反対）
- **境界条件**: 値域 [-1.0, 1.0] の上限・中央・下限（REQ.04）

### ケース 13: `cosine_similarity` clamp（浮動小数点誤差で域外 → [-1,1] へ）

- **観点出典**: TP-LGX-006 §2.1 B-05（域外 clamp）, TP-LGX-004 §2.1 B4
- **分類**: Unit
- **前提**: 内積が浮動小数点誤差で僅かに 1.0 超 / -1.0 未満となるベクトル対
- **入力**: 内積 ≈ 1.0000001 / ≈ -1.0000001 を生む `a, b`
- **期待**: 戻り値が `<= 1.0` かつ `>= -1.0`（[-1,1] に clamp。完全一致が 1.0 を超えない保証。REQ.04 GAP-LGX-105）
- **境界条件**: 上限 +1 / 下限 -1 のちょうど・超過

### ケース 14: ゼロノルムベクトルの cosine は skip 経路（NaN/Inf 非返却）

- **観点出典**: TP-LGX-006 §2.1 B-04（ゼロベクトル 0 除算・NaN）
- **分類**: Unit
- **前提**: 一方または双方のノルムが 0。DD §3 不変条件: 呼出側でゼロノルム検査後に呼ぶ想定（内部 assert なし）。呼出側（bulk API）が AggregatedWarnings.zero_norm_count に計上し skip
- **入力**: bulk API 経由でゼロノルム embedding を含むペアを処理（`compute_all_pair_scores` 等）
- **期待**: 当該ペアを skip（結果に含めない）+ 集約 Warning 1 件 stderr。NaN/Inf を結果へ出さない。`Ok(scores)`
- **境界条件**: ゼロノルム = skip（v3 差分: v3 は 0.0 返却。REQ.04）。standalone drift のゼロノルム Error は SPEC-010 / TS-013 へ委譲

### ケース 15: bulk API — ノード 0 件 / 1 件（O(N²) ペア数 0）

- **観点出典**: TP-LGX-006 §2.1 B-06（bulk API ペア 0）
- **分類**: Integration
- **前提**: `store` に embedding 行が 0 件、または 1 件のみ
- **入力**: `compute_all_pair_scores(&store)` / `compute_link_candidates(&graph, &store, 0.8)` / `compute_edge_scores(&graph, &store)`
- **期待**: いずれも `Ok(vec![])`（ペアが構成できない = 空 Vec、Err ではない）
- **境界条件**: N=0 / N=1 の下限（ペア数 = N(N-1)/2 = 0）

### ケース 16: bulk API — 次元不一致ペアの skip + 集約 Warning

- **観点出典**: TP-LGX-006 §2.2 E-05（次元不一致集約 Warning）, §2.5b V-02（混在期）
- **分類**: Integration
- **前提**: `store` に 384 次元と 768 次元の embedding が混在（model_version 遷移期）
- **入力**: `compute_edge_scores(&graph, &store)` / `compute_link_candidates(&graph, &store, t)` / `compute_all_pair_scores(&store)`
- **期待**: 次元不一致ペアを skip。`AggregatedWarnings.dim_mismatch_count >= 1` → 集約 Warning 1 件（skip 件数 + `embed --all` 誘導）stderr。返却は一致次元ペアのみ。`Ok(scores)`
- **境界条件**: 次元不一致 = skip（v3 差分: v3 無言 skip。REQ.04）。standalone drift の次元不一致 Error は SPEC-010 / TS-013 委譲

### ケース 17: bulk API 返却順序の決定性（property）

- **観点出典**: TP-LGX-006 §2.9 D-02（返却順序決定性、SCORE-INV-1）
- **分類**: Property-based（proptest）
- **生成器**: 任意の embedding 行集合（node_id・次元ランダム）から `store` を構築
- **不変条件**: `compute_edge_scores` 出力順は `graph.edges()` 挿入順、`compute_link_candidates` は (from, to) 昇順、`compute_all_pair_scores` は i < j 昇順。同一入力で常に同一順序。`load_all` は `ORDER BY node_id ASC`
- **反例ハンドリング**: shrink して最小の順序不一致例を記録

### ケース 18: `histogram` — 値域 [0,1] 固定・均等幅・末尾バケット inclusive

- **観点出典**: TP-LGX-006 §2.1 B-05（値域）, TP-LGX-004 §2.1 B5（閾値・ヒストグラム境界）
- **分類**: Unit
- **前提**: スコア列 `[0.0, 0.5, 1.0]`、`buckets = 2`。値域は **[0,1] 固定**（DD-007 §3 訂正済 = v3 `similarity.rs` L225 `score.clamp(0.0,1.0)`・`bucket_width=1.0/buckets`）
- **入力**: `histogram([0.0, 0.5, 1.0].into_iter(), 2)`
- **期待**: 2 バケット（span [0,1]・幅均等 `bucket_width = 1.0/2 = 0.5` → `[0.0,0.5)` と `[0.5,1.0]`）。`0.0` は最下位バケット（bucket[0]）、`0.5` は bucket[1]、末尾バケット上限 inclusive のため `1.0` も bucket[1] に計上。`bucket[0].count == 1`、`bucket[1].count == 2`、count 合計 == 3。各 `Bucket{low, high}` の low/high が span [0,1] を均等分割（bucket[0]={low:0.0,high:0.5}, bucket[1]={low:0.5,high:1.0}）
- **境界条件**: 値域 [0,1] 固定（DD-007 §3 訂正済。旧 [-1,1] 記述は誤りで撤回）。末尾上限 inclusive（v3 同様）。境界値 1.0 が末尾へ落ちる
- **委譲注**: 正準 histogram は DD-LGX-011（calibrate）所有（ADR-LGX-021 §2.3）。本ケースは `legixy-embed` の histogram ユーティリティの値域・均等幅・末尾 inclusive の主検証を担うが、calibrate 運用文脈での正準検証（ベースライン選択・p25/p90 推奨に供するバケット集計）は **TS-LGX-011 へ委譲**する（カバレッジ表に明記）

### ケース 19: `histogram` — [0,1] 外の clamp（負値は 0.0 へ・上限超過は 1.0 へ）

- **観点出典**: TP-LGX-006 §2.1 B-05（負値・clamp）, TP-LGX-004 §2.1 B5
- **分類**: Unit
- **前提**: スコア列に `-1.5` / `-0.5`（負域外）と `1.5`（上限域外）を含む。値域は **[0,1] 固定**（DD-007 §3 訂正済 = v3 `similarity.rs` L225 `score.clamp(0.0,1.0)`）。`buckets = 4` → `bucket_width = 1.0/4 = 0.25`、バケットは `[0.00,0.25)` / `[0.25,0.50)` / `[0.50,0.75)` / `[0.75,1.00]`
- **入力**: `histogram([-1.5, -0.5, 0.0, 0.5, 1.0, 1.5].into_iter(), 4)`
- **期待**: 域外値は [0,1] に clamp して計上 — 負値 `-1.5` / `-0.5` は **0.0 に clamp** され bucket[0]（`[0.00,0.25)`）に算入、`1.5` は **1.0 に clamp** され末尾 bucket[3]（`[0.75,1.00]`、上限 inclusive）に算入。`0.0` も bucket[0]、`0.5` は bucket[2]、`1.0` は末尾 bucket[3]。clamp 後の最下位バケット個別 count `bucket[0].count == 3`（-1.5, -0.5, 0.0）、最上位バケット個別 count `bucket[3].count == 2`（1.0, 1.5）。総 count == 6
- **境界条件**: 値域 [0,1] 固定（DD-007 §3 訂正済。旧 [-1,1] 記述は誤りで撤回）。下限 0・上限 +1 の clamp（負値 = 0.0、>1 = 1.0）。clamp 後の最下位/最上位バケット個別 count を検証

### ケース 20: `compute_model_version` 複合キー書式

- **観点出典**: TP-LGX-006 §2.5b V-01（model_version 生成方式）
- **分類**: Unit
- **前提**: 既知の model_name・ONNX ファイル・profile・dim
- **入力**: `compute_model_version("paraphrase-multilingual-MiniLM-L12-v2", onnx_path, PreprocessProfile::Plain, 384)`
- **期待**: `Ok` で `{model_name}:{onnx_sha256_8hex}:{profile}:{dim}` の 4 要素複合キー形式（DD §3）。すなわち先頭が `"paraphrase-multilingual-MiniLM-L12-v2:"`、続く 8 桁 hex（`[0-9a-f]{8}`）、`profile` 部位、末尾 `:384` で構成される。`profile` 部位は **`PreprocessProfile::Plain` に対する決定的な固定文字列であること**（具体値は実装 / DD §5 で確定。DD-007 が profile の文字列表現を明示していないため、本 TS では「`Plain` が常に同一の決定的トークンに写像され、`E5Prefix` とは異なる文字列となること」を検証し、リテラル `"plain"` には依存しない）。同一 ONNX なら同一文字列（冪等）
- **境界条件**: 複合キー 4 要素（名前 / ONNX 8hex / profile / dim）。`profile` 部位は `Plain` ≠ `E5Prefix` の決定的固定文字列（リテラル値は DD §5 確定待ちのため非依存）。E5Prefix profile は別書式部位

### ケース 21: `compute_model_version` — 同名 ONNX 差し替えで model_version 変化

- **観点出典**: TP-LGX-006 §2.5b V-01（変化判定）, §2.4 S-03（model_version 遷移）, SPEC-006 REQ.10 検証方法
- **分類**: Unit
- **前提**: 同一ファイル名だが内容（バイト列）の異なる 2 つの ONNX
- **入力**: 同名・別内容の ONNX に対し `compute_model_version(...)` を 2 回
- **期待**: 2 つの model_version 文字列が**不一致**（ONNX 内容ハッシュ部 `{onnx_sha256_8hex}` が変わる）。完全一致判定で「変化した」を検出（偽 fresh 防止、SCORE-INV-2）
- **境界条件**: 同名差し替え検出（REQ.10 GAP-LGX-115）

### ケース 22: `EmbeddingStore::is_up_to_date` — SCORE-INV-1 + SCORE-INV-2 双方一致で skip

- **観点出典**: TP-LGX-006 §2.3 S-02（hash 一致 skip）, §2.5b V-01（SCORE-INV-2）, TP-LGX-017 §2.5 DF3
- **分類**: Unit
- **前提**: `store` に node_id の行があり content_hash=H, model_version=V
- **入力**: (a) `is_up_to_date(id, H, V)`、(b) `is_up_to_date(id, H', V)`（hash 不一致）、(c) `is_up_to_date(id, H, V')`（version 不一致）
- **期待**: (a) `Ok(true)`（双方一致 → Skip）、(b) `Ok(false)`（Regen）、(c) `Ok(false)`（Regen）
- **境界条件**: content_hash AND model_version の双方一致でのみ true（SCORE-INV-1 ∧ SCORE-INV-2。片方不一致は再生成）

### ケース 23: `HashMatchState` 3 状態判定（Skip / Regen / Missing）

- **観点出典**: TP-LGX-006 §2.3 S-01（未生成 vs 古い）, §2.4 S-04（個別 ID skip 可否）, DD-007 §8 Unit 行
- **分類**: Unit
- **前提**: `load_embedding` の戻り値（行あり一致 / 行あり不一致 / 行なし）
- **入力**: 各状態に対する HashMatchState 判定ロジック
- **期待**: 行あり ∧ content_hash+model_version 一致 → `Skip`、行あり ∧ いずれか不一致 → `Regen`、行なし（`Ok(None)`）→ `Missing`
- **境界条件**: 3 状態の網羅（未生成=Missing と古い=Regen を区別）

### ケース 24: `embed_all --force` で content_hash 一致でも強制再生成

- **観点出典**: TP-LGX-006 §2.3 S-02（--force × skip）, §2.4 S-04, TP-LGX-017 §2.2 AF2
- **分類**: Integration
- **前提**: `store` に最新（content_hash 一致）の行が既存。`options.force = true`、`node_filter = All`
- **入力**: `embed_all(&graph, &store, &embedder, EmbedOptions{ force: true, ..default() })`
- **期待**: content_hash 一致でも再計算し `generated` に計上（skipped にしない）。`upsert_with_subnode_meta` を呼ぶ
- **境界条件**: `--force` は SCORE-INV-1 skip を上書き（REQ.10 / REQ.02）

### ケース 25: `embed_all` NodeFilter::Ids — 未登録 ID で Err（exit 1）

- **観点出典**: TP-LGX-006 §2.4 S-04（個別 ID 指定）, §2.8 F-03（個別指定契約）, TP-LGX-017 §2.6 R5
- **分類**: Integration
- **前提**: `node_filter = NodeFilter::Ids(vec!["SPEC-LGX-999"])`（graph.toml 未登録）
- **入力**: `embed_all(&graph, &store, &embedder, EmbedOptions{ node_filter: Ids(...), ..default() })`
- **期待**: `Err(EmbedError::NodeNotFound("SPEC-LGX-999"))`（意味的不正 → 呼出側 exit 1）。`--all` と `--node` の構文的排他は clap `conflicts_with`（exit 2）で別経路
- **境界条件**: 未登録 ID = 意味的不正 exit 1（構文誤り exit 2 と区別）

### ケース 26: `embed_all` 部分失敗継続 — content_range 防御検証失敗を errors 計上後続継続

- **観点出典**: TP-LGX-006 §2.2 E-03（部分失敗 vs Tx 境界）, §2.6 I-02（content_range 不正）, TP-LGX-017 §2.3 EF1/EF3, TP-LGX-004 §2.2 E1
- **分類**: Integration
- **前提**: 3 ノードのうち中央が content_range 逆転（start > end）またはファイル長超過。他 2 ノードは正常
- **入力**: `embed_all(&graph, &store, &embedder, EmbedOptions::default())`
- **期待**: `Ok(EmbedReport{ generated: 2, skipped: 0, failed: 1, errors: [EmbedErrorItem{ node_id, message }] })`。`InvalidContentRange` は Err に昇格せず EmbedErrorItem へ変換。`report.failed == report.errors.len()`。panic しない。後続ノードは処理継続。`exit_code == 1`（failed > 0）
- **境界条件**: 部分失敗 = errors 計上 + 継続（v3 差分: v3 は全文 fallback）。UTF-8 境界違反も `str::is_char_boundary` で検出し InvalidContentRange（GAP-LGX-118）

### ケース 27: `embed_all` ノード単位 Tx — 推論失敗ノードのみ rollback・後続継続

- **観点出典**: TP-LGX-006 §2.2 E-03/E-04（部分失敗・終了コード分類）, §2.4 C-02（Tx 部分不整合非発生）, TP-LGX-017 §2.3 EF1/EF3
- **分類**: Integration
- **前提**: あるノードの `embed_node` が `OnnxInferenceError` を返す。他は正常
- **入力**: `embed_all(...)`
- **期待**: 当該ノードの Tx のみ rollback（そのノードの行は更新されない＝既存維持 or 未生成のまま）、`EmbedErrorItem` に変換し `errors` push、後続ノードは upsert される。`report.failed >= 1`、`exit_code == 1`
- **境界条件**: ノード単位 1 Tx（REQ.08）。1 ノード失敗が他ノードのコミットを巻き込まない（部分コミット成立）

### ケース 28: `embed_all` DB 接続異常は Err 昇格（全体 abort）

- **観点出典**: TP-LGX-006 §2.5 P-05（書込失敗）, §2.2 E-03, TP-LGX-004 委譲なし（embed 固有）
- **分類**: Integration
- **前提**: `upsert_with_subnode_meta` が `DbError`（接続異常・ディスクフル相当）を返す
- **入力**: `embed_all(...)`
- **期待**: `Err(EmbedError::Db(_))`（部分失敗継続経路ではなく全体 abort へ昇格）。ノード単位 Tx 内の DbError は embed_all が Err 化
- **境界条件**: DB 接続異常 = Err 昇格（exit 1）/ ノード内論理失敗 = errors 継続。両経路の分離

### ケース 29: `embed_all` 決定性 — 同一入力 → 同一 EmbedReport（property）

- **観点出典**: TP-LGX-006 §2.9 D-01（順序決定性。値再現性は委譲）, DD-007 §8 Property 行（D1 proptest）
- **分類**: Property-based（proptest）
- **生成器**: 任意の graph（正常 / 空テキスト / content_range 不正を混在）と固定 embedder
- **不変条件**: 同一入力に対し `embed_all` の `EmbedReport`（generated / skipped / failed の各件数、errors の node_id 集合と順序）が常に同一。ベクトル値のビット再現性は対象外（順序・件数の決定性のみ）
- **反例ハンドリング**: shrink して最小の件数/順序不一致例を記録

### ケース 30: `EmbedReport` --json スキーマ（v3 差分: failed フィールド + errors オブジェクト）

- **観点出典**: TP-LGX-006 §2.8 F-02（--json スキーマ）, TP-LGX-017 §2.5 DF1
- **分類**: Contract
- **前提**: 部分失敗を含む embed_all 結果
- **入力**: `serde_json::to_string(&report)`（`EmbedReport` の Serialize）
- **期待**: JSON に `generated` / `skipped` / `failed`（数値）/ `errors`（オブジェクト配列、各 `{node_id, message}`）の 4 フィールド。v3 のタプル列・failed 欄なし形式ではない。`failed == errors.len()`。集約 Warning は JSON に含めず stderr のみ
- **境界条件**: v3 差分（failed 追加・errors オブジェクト化。SUPP-006 §2.5-d: --json でも warning 欄なし）

### ケース 31: `detect_drift` — stale（content_hash 不一致）を ContentChanged で報告

- **観点出典**: TP-LGX-006 §2.3 S-01（drift 状態）, TP-LGX-004 §2.6 P1/P3（drift 比較元・網羅）, §2.1 B6
- **分類**: Integration
- **前提**: `store` の行 content_hash とファイル現内容の `read_current_content_for_node` 結果が不一致
- **入力**: `detect_drift(&graph, &store, &project_root)`
- **期待**: `Ok(vec![DriftFinding{ node_id, stored_hash: Some(H), current_hash: Some(H'), kind: DriftKind::ContentChanged }])`。現内容は embed_all と同一経路（normalize + content_range）で計算
- **境界条件**: stale = ContentChanged（stored・current 双方 Some）

### ケース 32: `detect_drift` — 未生成ノードを EmbeddingMissing で結果に包含

- **観点出典**: TP-LGX-006 §2.3 S-01（未生成 vs 古い）, §2.5 P-03, TP-LGX-017 §2.3 EF3
- **分類**: Integration
- **前提**: graph にノードがあるが `store` に embedding 行なし（`load_embedding` = `Ok(None)`）
- **入力**: `detect_drift(&graph, &store, &project_root)`
- **期待**: `DriftFinding{ node_id, stored_hash: None, current_hash: Some(H'), kind: DriftKind::EmbeddingMissing }` を結果に**含む**（v3 差分: v3 は無言 skip → 偽 fresh 黙殺）
- **境界条件**: 未生成 = EmbeddingMissing（stored=None）。検出対象から除外しない（REQ.05）

### ケース 33: `detect_drift` — ファイル読込不能を FileMissing で報告

- **観点出典**: TP-LGX-006 §2.3 S-01, TP-LGX-004 §2.1 B6（drift 対象不在）, TP-LGX-017 §2.3 EF2（運用は SPEC-010 委譲）
- **分類**: Integration
- **前提**: `store` に embedding 行はあるが、ノードのファイル実体が読めない（削除・権限）
- **入力**: `detect_drift(&graph, &store, &project_root)`
- **期待**: `DriftFinding{ node_id, stored_hash: Some(H), current_hash: None, kind: DriftKind::FileMissing }`。panic せず DriftFinding 化
- **境界条件**: ファイル不在 = FileMissing（current=None）。3 種 DriftKind の網羅（ContentChanged / FileMissing / EmbeddingMissing）

### ケース 34: `detect_drift` 出力順 node_id ASC（決定性）

- **観点出典**: TP-LGX-006 §2.9 D-02（順序決定性）, TP-LGX-004 §2.10 D1
- **分類**: Unit
- **前提**: 複数ノードに drift が混在（順不同に格納）
- **入力**: `detect_drift(&graph, &store, &project_root)`
- **期待**: 戻り Vec が node_id 昇順でソート済（DD §3 不変条件「出力順は node_id ASC」）
- **境界条件**: 決定的出力順（SCORE-INV-1 順序決定性）

### ケース 35: `read_current_content_for_node` — embed_all と同一経路の content_range 切り出し

- **観点出典**: TP-LGX-006 §2.6 I-02（content_range）, TP-LGX-017 §2.6 R3（drift on-the-fly 生成）
- **分類**: Unit
- **前提**: サブノード（content_range あり）と ドキュメントノード（range なし）
- **入力**: `read_current_content_for_node(&node, &graph, &project_root)`
- **期待**: サブノードは content_range 部分のみ切り出し（親全体ではない）。ドキュメントノードは全文。embed_all（生成時）と detect_drift（検証時）が同一ヘルパを通り content_hash が一致する経路（SUPP-006 §2.3-e、ISSUE-003 BUG-3 fix）。range 不正は `Err(InvalidContentRange)`（panic なし）
- **境界条件**: 生成と検証の content 切り出し経路一致（偽 drift 防止）

### ケース 36: `EmbeddingStore::load_all` — node_id ASC 決定性

- **観点出典**: TP-LGX-006 §2.9 D-02（SCORE-INV-1）, §2.5 P-01
- **分類**: Unit
- **前提**: `store` に複数行（挿入順は任意）
- **入力**: `store.load_all()`
- **期待**: `Ok(Vec<EmbeddingRow>)` が node_id 昇順（DD §3 `ORDER BY node_id ASC`）。bulk API の決定性の基盤
- **境界条件**: ロード順の決定性

### ケース 37: `EmbeddingStore::load_embedding` — 未登録は Ok(None)

- **観点出典**: TP-LGX-006 §2.3 S-01（未生成判定）, §2.5 P-01
- **分類**: Unit
- **前提**: `store` に当該 node_id の行なし
- **入力**: `store.load_embedding("SPEC-LGX-001")`
- **期待**: `Ok(None)`（Err ではない。HashMatchState::Missing 判定に使用）
- **境界条件**: 未登録 = Ok(None)（行なしはエラーではない）

### ケース 38: `EmbeddingStore::upsert_with_subnode_meta` — INSERT OR REPLACE の冪等 upsert

- **観点出典**: TP-LGX-006 §2.5 P-02（サブノード格納項目）, §2.6 永続化, TP-LGX-017 §2.1 BF3
- **分類**: Integration
- **前提**: 既存行ありの node に対し同一 EmbedResult で再 upsert
- **入力**: `store.upsert_with_subnode_meta(&node, &result)` を 2 回
- **期待**: `Ok(())`。2 回適用後も行は 1 件（INSERT OR REPLACE による冪等）。ノード単位 1 Tx（REQ.08）。parent_id / anchor / is_subnode を含む格納（REQ.12）
- **境界条件**: upsert 冪等性（多重実行で重複行を作らない）

### ケース 39: `EmbeddingStore::create_snapshot` — 全行コピー・行数返却

- **観点出典**: TP-LGX-006 §2.5 P-01（永続化）, TP-LGX-017 §2.1 BF3
- **分類**: Integration
- **前提**: `store` の embeddings に N 行
- **入力**: `store.create_snapshot("snap-001", Some("baseline"))`
- **期待**: `Ok(N)`（コピーした行数）。1 Tx で embedding_snapshots へ全行コピー
- **境界条件**: スナップショット行数 = 現 embeddings 行数

### ケース 40: CR フォールバック骨格 — LLM 失敗時 Ok(None)（Phase 1 パススルー）

- **観点出典**: TP-LGX-006 §2.7 L-01（CR フォールバック Warning）, §2.4 C-03（CR 並行は委譲）, TP-LGX-017 §2.4 AT3
- **分類**: Unit
- **前提**: ContextualConfig 有効・LLM API 呼出がタイムアウト/リトライ尽き（または Phase 1 パススルー骨格）
- **入力**: `synthesize_with_fallback(...)`（DD §6 CR フォールバック）
- **期待**: `Ok(None)`（CR 無効扱いで通常 embedding 継続）。フォールバック時 stderr Warning。`Err` に昇格しない
- **境界条件**: CR 失敗 = Ok(None) 継続（REQ.06.1）。LLM 呼出挙動・mask_api_key・observations 記録は contextual 専用 TS / SEC NFR へ委譲

### ケース 41: 終了コード契約 0/1/2（LGX-COMPAT-001 凍結）

- **観点出典**: TP-LGX-006 §2.2 E-04（終了コード分類）, §2.8 F-01（凍結引数）, TP-LGX-017 §2.3 EF4, §2.6 R5
- **分類**: Contract
- **前提**: (a) 全件成功（skipped 含む failed=0）、(b) 部分失敗 failed>0 または モデル shape/解決失敗、(c) `--all` と `--node` 排他違反（clap 構文層）
- **入力**: それぞれ embed_all → exit 変換 / CLI ディスパッチ
- **期待**: (a)→0、(b)→1、(c)→2。未登録 ID（意味的不正）は exit 1（exit 2 ではない）
- **境界条件**: exit 2 = clap 構文層限定（`conflicts_with`）/ failed>0・モデル失敗・未登録 ID = exit 1（NFR OBS.05）

### ケース 42: 出力先分離（EmbedReport=stdout / 集約 Warning・ログ=stderr）

- **観点出典**: TP-LGX-006 §2.7 L-04（skip 集約 Warning 出力先）, §2.8 F-02, TP-LGX-017 §2.5 DF1, TP-LGX-004 §2.9 O1
- **分類**: Integration
- **前提**: 空テキスト skip・次元不一致・切り捨て のいずれかを含む embed_all 実行
- **入力**: CLI 実行（`embed --all` / `embed --all --json`）
- **期待**: `EmbedReport`（人間可読 or `--json`）は stdout、集約 Warning（dim_mismatch / zero_norm / truncated / empty_skip）とログは stderr。`--json` 指定時も Warning は stderr のみ（JSON スキーマに warning 欄なし）
- **境界条件**: stdout/stderr チャネル分離（NFR OBS.02、SUPP-006 §2.5-d）

### ケース 43: `compute_link_candidates` — `score >= threshold` 境界（= 含む / < 除外）

- **観点出典**: TP-LGX-004 §2.1 B5（link_candidate 閾値境界 `=`, `<`）, TP-LGX-006 §2.1 B-05（cosine 値域境界の閾値適用）
- **分類**: Integration
- **前提**: `store` に L2 正規化済み embedding を持つ 3 ノード A, B, C を格納。ペアスコアを制御値で固定する決定的スタブ embedding を用い、`cos(A,B) == t` ちょうど、`cos(A,C) == t - ε`（閾値未満）、`cos(B,C) == t + ε`（閾値超）となるよう構成（`t = 0.8`）。`graph` には 3 ノードが登録済み
- **入力**: `compute_link_candidates(&graph, &store, 0.8)`
- **期待**: 結果（`Vec<CandidateScore>`）に `score == t`（= 0.8）ちょうどのペア (A,B) が**含まれる**（v3 `similarity.rs` L131 `if score >= threshold` の `>=` により境界値は採用）。`score == t + ε` のペア (B,C) も含まれる。`score < t` のペア (A,C) は**除外**される。返却順は (from, to) 昇順。結果集合 = {(A,B), (B,C)}、(A,C) を含まない。`Ok(candidates)`
- **境界条件**: `>=` 閾値の境界 — ちょうど `= t` は採用、`< t` は除外（v3 `similarity.rs` L131 `score >= threshold`）。本 TS が bulk API（`compute_link_candidates`）所有のため、この閾値比較式の検証は TS-LGX-001（check 内 IdSemanticDrift / SemanticChecker の `<` 判定）へ委譲不可で本 TS が所有する（histogram / N=0,1 ケースは比較式を通らない名目 mapping のため別途必要）

## 3. 観点カバレッジ表

| TP § | 観点 | カバーする TS ケース |
|---|---|---|
| TP-006 §2.1 B-01 空テキスト embedding | 境界値 | ケース 1, 2 |
| TP-006 §2.1 B-02 巨大テキスト切り捨て | 境界値 | ケース 3 |
| TP-006 §2.1 B-03 異常 output shape | 境界値 | ケース 4 |
| TP-006 §2.1 B-04 ゼロベクトル cosine | 境界値 | ケース 14 |
| TP-006 §2.1 B-05 cosine 値域境界 | 境界値 | ケース 12, 13（cosine 値域 [-1,1]）, 18, 19（histogram 値域 [0,1]・clamp。正準 histogram 検証は TS-LGX-011 委譲）|
| TP-006 §2.1 B-06 ノード 0/1 件 bulk | 境界値 | ケース 15 |
| TP-006 §2.2 E-01 モデル不在/読込失敗 | エラー | ケース 4, 5 |
| TP-006 §2.2 E-02 モデル解決順序 | エラー | ケース 5 |
| TP-006 §2.2 E-03 部分失敗 vs Tx 境界 | エラー | ケース 26, 27, 28 |
| TP-006 §2.2 E-04 終了コード分類 | エラー | ケース 41 |
| TP-006 §2.2 E-05 次元不一致集約 Warning | エラー | ケース 16 |
| TP-006 §2.3 S-01 未生成ノード drift | 状態 | ケース 23, 31, 32, 33, 37 |
| TP-006 §2.3 S-02 content_hash skip × --force | 状態 | ケース 22, 24 |
| TP-006 §2.3 S-03 model_version 遷移期 | 状態 | ケース 16, 21 |
| TP-006 §2.3 S-04 個別 ID × --all × --force | 状態 | ケース 24, 25, 23 |
| TP-006 §2.4 C-01 並行 embed/ロック | 並行 | NFR-LGX-001 REL.07/SEC.02 へ委譲 |
| TP-006 §2.4 C-02 Tx 中 kill/電源断 | 並行 | ケース 27（ノード単位 Tx rollback。電源断は NFR REL.06 委譲）|
| TP-006 §2.4 C-03 CR 並行/レート | 並行 | ケース 40（骨格）/ contextual 専用 TS へ委譲 |
| TP-006 §2.5 P-01 必須情報取得可能性 | 永続化 | ケース 36, 37, 39 |
| TP-006 §2.5 P-02 サブノード格納項目 | 永続化 | ケース 38 |
| TP-006 §2.5 P-03 content_hash 正規化 | 永続化 | ケース 6, 7, 8, 9, 10, 11, 31, 32 |
| TP-006 §2.5 P-04 同一テキスト複数ノード一意性 | 永続化 | ケース 11 |
| TP-006 §2.5 P-05 ディスクフル/権限 | 永続化 | ケース 28 |
| TP-006 §2.5b V-01 model_version 生成/変化判定 | 互換 | ケース 20, 21, 22 |
| TP-006 §2.5b V-02 768次元混在期 | 互換 | ケース 16 |
| TP-006 §2.5b V-03 e5 プレフィックス | 互換 | ケース 20（PreprocessProfile::E5Prefix 書式部位）/ ADR 委譲（範囲外） |
| TP-006 §2.5b V-04 v0.1.0 旧 embedding | 互換 | ケース 21（model_version 不一致再生成）/ UC-009 移行へ委譲 |
| TP-006 §2.6 I-01 Unicode/制御文字/絵文字 | 入力 | ケース 8（NFC）/ トークナイザ絵文字処理は DD・TS-003 委譲 |
| TP-006 §2.6 I-02 content_range 不正値 | 入力 | ケース 26, 35 |
| TP-006 §2.7 L-01 CR フォールバック Warning | 観測 | ケース 40 |
| TP-006 §2.7 L-02 API キーマスキング | 観測 | mask_api_key=contextual/SEC NFR へ委譲（NFR-LGX-001 SEC.05）|
| TP-006 §2.7 L-03 embed 進捗表示 | 観測 | NFR-LGX-001 USE.03 へ委譲 |
| TP-006 §2.7 L-04 skip 集約 Warning 出力先 | 観測 | ケース 42 |
| TP-006 §2.8 F-01 embed 凍結引数受理 | 互換 | ケース 41 |
| TP-006 §2.8 F-02 embed --json スキーマ | 互換 | ケース 30, 42 |
| TP-006 §2.8 F-03 個別ノード指定契約 | 互換 | ケース 25 |
| TP-006 §2.9 D-01 浮動小数点値再現性 | 決定性 | ケース 29（順序決定性）/ ビット再現性は DD §7・SPEC-006 REQ.04 へ委譲（対象外）|
| TP-006 §2.9 D-02 bulk API 返却順序決定性 | 決定性 | ケース 17, 34, 36 |
| TP-006 §2.9 D-03 CR 非決定性 vs freshness | 決定性 | ケース 40 / freshness は content_hash のみ（ケース 11, 22）|
| TP-006 §2.9 D-04 mean pooling + L2 固定 | 決定性 | ケース 10, 11（正規化決定性）/ 推論パイプライン値は DD §7 委譲 |
| TP-006 §2.9 D-05 多言語混在妥当性 | 決定性 | ケース 8（NFC 一致）/ 値域の数値妥当性は NFR・SPEC-010 へ委譲 |
| TP-017 §2.1 BF1 embed ステップ連鎖 | UC フロー | ケース 1, 22, 23, 24, 26 |
| TP-017 §2.1 BF2 drift ステップ連鎖 | UC フロー | ケース 31, 35 |
| TP-017 §2.1 BF3 embed 成功時事後条件観察可能性 | UC フロー | ケース 38, 39 |
| TP-017 §2.1 BF4 --subnodes 連鎖 | UC フロー | ケース 35, 38 |
| TP-017 §2.2 AF1 代替フロー 2a exit コード | UC フロー | ケース 5, 41 |
| TP-017 §2.2 AF2 代替フロー 3b 事後収束 | UC フロー | ケース 24 |
| TP-017 §2.2 AF3 drift 代替フロー網羅 | UC フロー | ケース 33 / standalone drift 運用は SPEC-010・TS-013 委譲 |
| TP-017 §2.3 EF1 embed 部分失敗継続 | UC フロー | ケース 26, 27 |
| TP-017 §2.3 EF2 drift 失敗パス | UC フロー | ケース 33 / SPEC-010・TS-013 委譲 |
| TP-017 §2.3 EF3 エラー時不変条件保持 | UC フロー | ケース 27, 32 |
| TP-017 §2.3 EF4 エラー通知（出力先/終了コード）| UC フロー | ケース 41, 42 |
| TP-017 §2.4 AT1 embed 書込権限 | アクター | ケース 38（書込観察）/ NFR SEC.01 委譲 |
| TP-017 §2.4 AT2 drift 責任境界（推奨のみ）| アクター | SPEC-010・TS-013 委譲（運用出力）|
| TP-017 §2.4 AT3 --subnodes フェーズ前提 | アクター | ケース 35, 40 |
| TP-017 §2.5 DF1 embed 入出力データフロー | データフロー | ケース 38, 42 |
| TP-017 §2.5 DF2 drift JSON/null 出力 | データフロー | SPEC-010・TS-013 委譲（--json スキーマ運用）|
| TP-017 §2.5 DF3 SCORE-INV-1 skip ロジック | データフロー | ケース 22, 23 |
| TP-017 §2.6 R1 embed/drift フロー分離 | 領域 | ケース 1（生成）vs 31（検証）の責務分離 |
| TP-017 §2.6 R2 SCORE-INV-2 事後条件整合 | 領域 | ケース 20, 21, 22 |
| TP-017 §2.6 R3 drift Step2 on-the-fly 生成 | 領域 | ケース 35 |
| TP-017 §2.6 R4 drift 閾値判定フロー | 領域 | SPEC-010 REQ.03・TS-013 委譲（閾値出所・--json）|
| TP-017 §2.6 R5 凍結済み引数契約整合 | 領域 | ケース 25, 41 |
| TP-004 §2.1 B4 similarity 閾値境界（=,<）| 境界値（意味層委譲受け皿）| ケース 12, 13（cosine 値域）/ check 内 `<` 判定は TS-LGX-001 委譲 |
| TP-004 §2.1 B5 link_candidate 閾値境界 | 境界値（受け皿）| ケース 43（`compute_link_candidates` `>= threshold` 境界 — `=` 含む / `<` 除外、本 TS 所有）, 18, 19（histogram 境界 — 正準は TS-LGX-011 委譲）, 15（candidate 空）|
| TP-004 §2.1 B6 drift 対象不在 severity | 境界値（受け皿）| ケース 33（FileMissing）/ check 内 severity は TS-001 委譲 |
| TP-004 §2.1 B7 max_pairs_per_id 打切り | 境界値 | check/IdSemanticDrift 側＝TS-001 委譲（embed エンジン非所有）|
| TP-004 §2.6 P1 drift 比較元不在/古い | 永続化（受け皿）| ケース 31, 32, 37 |
| TP-004 §2.6 P3 drift 網羅範囲 | 永続化（受け皿）| ケース 31, 32, 33, 34（全ノード走査）|
| TP-004 §2.11 R4 SemanticSimilarity 対象エッジ | 領域（受け皿）| ケース 16（compute_edge_scores 対象）/ check 適用は TS-001 委譲 |

> 継承 TP 観点はすべて本テーブルで TS ケースまたは明示委譲先に mapping 済み（人間ゲート判断対象）。本 TS は `legixy-embed` エンジン（生成・格納・検出・bulk API・正規化・cosine/histogram）の入力・期待・境界・決定性に集中し、ONNX 推論値のビット再現性（DD §7）・standalone drift / report / calibrate / snapshot の運用仕様（SPEC-010 / UC-013 / TS-010・013）・並行制御（NFR REL.07）・性能（NFR PERF.08 bench）・check 内の閾値判定と severity（TS-LGX-001）・**calibrate 運用文脈の正準 histogram 検証（TS-LGX-011、ADR-LGX-021 §2.3）**・mask_api_key（SEC NFR）を責務上委譲する。`compute_link_candidates` の `>= threshold` 境界（ケース 43）は本 TS が bulk API 所有のため TS-LGX-001 委譲不可で本 TS が所有する。TP-004 引用分（DD-007 §8）は意味層スコアの値域・正規化・全ノード走査の受け皿として本 TS が cosine/histogram/detect_drift で具体化し、check への適用境界は TS-LGX-001 が所有する。histogram の値域は **[0,1] 固定**（DD-007 §3 訂正済 = v3 `similarity.rs` L225 `score.clamp(0.0,1.0)`。旧 [-1,1] 記述は誤りで撤回）。

## 4. テスト技法選択

- 同値分割: HashMatchState（Skip/Regen/Missing）・DriftKind（ContentChanged/FileMissing/EmbeddingMissing）・EmbedError variant・NodeFilter（All/Ids）・exit コード（0/1/2）の各クラスに 1 ケース以上。
- 境界値分析: cosine 値域（上限 +1 / 中央 0 / 下限 -1 / 域外 clamp）、histogram（値域 [0,1] 固定・末尾 inclusive・[0,1] 外 clamp で負値→0.0/>1→1.0）、`compute_link_candidates` の `>= threshold` 境界（= 含む / < 除外、ケース 43）、normalize_content 末尾改行（0/1/多数）、bulk API ペア数（N=0/1）。
- Property-based（proptest）: normalize_content 冪等性（ケース 10）、content_hash_for 決定性・クロスプラットフォーム一致（ケース 11）、bulk API 返却順序決定性（ケース 17）、embed_all EmbedReport 決定性（ケース 29）。
- 状態遷移: HashMatchState 3 状態（ケース 23）、DriftKind 3 状態（ケース 31/32/33）、embed_all 部分失敗継続経路 vs Err 昇格経路（ケース 26/27/28）。

## 5. テスト基盤

- 言語: Rust（CLI 本体、crate `legixy-embed`）
- フレームワーク: cargo test
- Property-based: proptest（DD-007 §8 Property 行）
- モック: ONNX 推論は固定ベクトルを返すスタブ Embedder（値再現性は対象外のため決定的スタブで順序・件数を検証）。EmbeddingStore は in-memory rusqlite Connection（`:memory:`）。LLM API（CR）は Phase 1 パススルー骨格 / フォールバックスタブ。

## 6. 関連 TC

| TS ケース | 対応 TC | 場所 |
|---|---|---|
| ケース 1〜42 | TC-LGX-007（RED → GREEN で確定）| `crates/legixy-embed/tests/` および各 `src/*.rs` の `#[cfg(test)]`（content.rs / embedder.rs / orchestrator.rs / store.rs / similarity.rs / drift.rs / model_version.rs）|

> TC への具体的な関数名・ファイル分割は TS-to-TC(RED) 段階で確定する（本 TS はケース定義まで）。
