Document ID: SPEC-LGX-006

# SPEC-LGX-006: embedding とドリフト検出

| 項目 | 内容 |
|------|------|
| Document ID | SPEC-LGX-006 |
| Version | 0.7.0 |
| Status | Approved（人間査読済） |
| Date | 2026-04-17 |
| Classification | CONFIDENTIAL |
| 親文書 | SPEC-LGX-001, LGX-EXT-001 §5.8 |
| 対応 NFR | NFR-LGX-001.PERF.08, REL.06 |
| 対応 UC | UC-LGX-007 |

---

## 1. 本文書の位置づけ

### 1.1 目的

embedding 生成・格納・ドリフト検出の要求を定義する。v0.1.0 の embedding 機能を明文化しつつ、legixy の Contextual Retrieval 対応を追加する。

### 1.2 スコープ

**含む:** embedding 生成の入出力、ONNX モデル要件、格納スキーマ、ドリフト検出
**含まない:** 意味的類似度の閾値（→ NFR-LGX-001）、check の意味的検証の使われ方（→ SPEC-LGX-004）

---

## 2. 参照文書

- LGX-EXT-001 §4.3 engine.db のスキーマ変更
- LGX-EXT-001 §5.7 embedding 生成ロジックの抽象化
- LGX-EXT-001 §5.8 Contextual Retrieval の実装
- v0.1.0 の embed コマンド実装を慣例仕様として参照

---

## 3. 要求事項

### SPEC-LGX-006.REQ.01: ONNX モデル

**内容:** embedding 生成は ONNX モデルを CPU で実行する。GPU 利用は非目標。モデルパスは `.legixy.toml` の `[semantic]` セクションで指定する。

**既定モデル:** **日本語・英語の双方を扱える多言語モデル**を既定とする。具体的には `paraphrase-multilingual-MiniLM-L12-v2`（384 次元、mean pooling、prefix 不要、50+ 言語対応）。これは旧 `all-MiniLM-L6-v2`（英語中心）のドロップイン置換であり、エンジン側のプーリング・正規化（mean pooling + L2 正規化）を変更せずに日本語成果物のドリフト検出を可能にする。
- 出力次元はモデルの出力 shape から動的に確定するため、384 次元以外（例: 768 次元の multilingual-mpnet）にも対応する。
- より高い日本語精度が必要な場合の代替: `intfloat/multilingual-e5-base`（768 次元）や `BAAI/bge-m3`。ただし e5 系は `query:` / `passage:` プレフィックス付与を前提とするため、採用時はエンジンに前処理追加が必要（別 ADR で判断）。

**トークン上限超過時の扱い（GAP-LGX-102 対応）:** モデルの最大入力トークン長はモデル metadata から取得する。入力テキストが上限を超過する場合は**先頭 N トークンで切り捨てて生成**し、切り捨てが発生した場合は**集約 Warning 1 件**（件数）を stderr に出力する【v3 差分: v3 は無言切り捨て】。チャンク分割 + 平均は採らない（複雑性に対し精度利得が不明。長文の実用的対策はサブノード化〔REQ.09〕による粒度分割である旨を注記）。切り捨て境界の厳密挙動は DD で確定する。

**モデル出力 shape の検証（GAP-LGX-103 対応）:** モデル読込時に出力 shape の妥当性（mean pooling 可能な軸構造・正の hidden 次元）を検証し、非適合モデルは **embed 起動不能の Error（exit 1）** とする。診断にはモデルディレクトリ誤配置の可能性を示唆するメッセージを含める。本検証（読込時の構造検査）と SCORE-INV-2（model_version 照合 = 既存 embedding との版整合）は別経路である。

**根拠:** LEGIXY-SPEC-001 §2, NFR-LGX-001.PERF.08、OSS 公開で日英混在ドキュメントを扱う要件、GAP-LGX-102/103。
**検証方法:** モデル読み込みテスト、日本語・英語サンプル文の類似度が妥当な値域に入ること、上限超過入力での切り捨て + 集約 Warning テスト、異常 shape モデルでの exit 1 テスト

### SPEC-LGX-006.REQ.02: embed コマンド

**内容:** `legixy embed --all` は全ノードの embedding を生成し engine.db に格納する。
- `--all`: 全ノード対象
- `--node <ID>`: 個別ノード指定（**複数指定可**、`--all` と排他。指定 ID が graph.toml に未登録なら exit 1）
- `--force`: content_hash 一致でも強制再生成（REQ.10 参照）
- 既存 embedding のうち content_hash が一致するものは再計算しない（`--force` 指定時を除く）
- `embed --json` の出力スキーマ: `{generated: <件数>, skipped: <件数>, failed: <件数>, errors: [{node_id, message}]}` を確定する
- **空テキストの扱い（GAP-LGX-101 対応）**: 本文が空・空白のみ（REQ.03 の正規化後 0 文字。空 content_range のサブノードを含む）のノードは **embedding 生成を skip** し、`skipped` に計上 + 集約 Warning 1 件で可視化する。ゼロベクトルは格納しない（mean pooling の 0 除算を構造的に回避 — REQ.04 のゼロベクトル skip 経路に流入させない）。skip されたノードは embedding 行を持たない**未生成状態**（REQ.05 の 3 状態）として drift・類似度計算から除外される
- **モデル解決失敗（GAP-LGX-106 対応）**: モデルの解決・読込に失敗した場合は **Error + exit 1** とし、試行したパスを stderr に通知する（解決順は SPEC-LGX-010.REQ.03 と同一: `--models-dir` > `LGX_MODELS_DIR` > `TE_MODELS_DIR`〔旧名〕> 設定ファイル。v3 実測〔embed.rs:48-54 anyhow 伝播〕の正準化）。なお **`[semantic] enabled` は check の意味層の有効化のみを制御**し、明示コマンドである embed の実行可否には影響しない（v3 実挙動）

**凍結契約との関係（GAP-LGX-120 対応、人間承認 2026-06-10）:** `--node` / `--force` は LGX-COMPAT-001 §4 #4 への**SPEC 主導の加算的拡張**である（既存呼出 `embed [--all]` は不変＝後方互換）。【v3 差分】v3 の embed は `--all` のみ（`crates/te-cli/src/main.rs:91-94`、`--json` はグローバルフラグ）。拡張の経緯は ADR に記録し、LGX-COMPAT-001 §4 #4/§7 に追記する。MCP 非公開は維持（MCP-INV-1）。

**根拠:** UC-LGX-007, v0.1.0 継承、GAP-LGX-120
**検証方法:** E2E テスト（--node 複数指定・--all 排他 exit 1・--force 再生成・--json スキーマ検証を含む）

### SPEC-LGX-006.REQ.03: embedding データの必須情報

**内容:** 生成された embedding ごとに、少なくとも以下の情報を格納する:
- ノード識別情報（一意）
- ベクトル本体
- ベクトル次元数
- 使用モデルのバージョン識別子
- 元コンテンツのハッシュ（SHA-256）
- Contextual Retrieval 用コンテキストとそのハッシュ（任意）
- 作成時刻

具体的なテーブル/カラム構造は DD で定義する。

**content_hash の正規化手順（GAP-LGX-114 対応）:** content_hash は以下の順で正規化した後の UTF-8 バイト列に対する SHA-256 とする:
1. BOM 除去（NFR COMPAT.07 と整合）
2. 改行統一: CRLF / CR → LF（NFR COMPAT.08 と整合）
3. Unicode 正規化: NFC
4. 末尾正規化（末尾改行の有無の揺れを吸収。厳密挙動は DD で確定）

これによりクロスプラットフォーム（Windows/Linux/macOS の checkout 差・エディタ差）での偽 stale / 偽 fresh を防止し、SCORE-INV-1 を環境非依存に固定する。サブノード content_hash（REQ.12）にも同一手順を適用する。

**根拠:** LGX-EXT-001 §4.3、GAP-LGX-114
**検証方法:** 生成後に上記情報が取得可能であること。BOM 有無・CRLF/LF・NFC/NFD の各バリアントで content_hash が一致するテスト

### SPEC-LGX-006.REQ.04: 意味的類似度の利用

**内容:** 2 ノード間の意味的類似度はコサイン類似度で計算する。以下の用途で使用される:
- SemanticSimilarity: 類似ノード検出
- LinkCandidate: 未定義エッジ候補検出
- Drift: 過去 embedding との乖離検出

しきい値は `.legixy.toml` の `[semantic]` セクションで設定。

**cosine 類似度の値域（GAP-LGX-105 対応）:** 値域は **[-1.0, 1.0]**。L2 正規化済みベクトル（REQ.01）では内積 = cosine 類似度が成立する。**負値は正常な出力**（意味的反対方向）であり消費側（SPEC-LGX-010 REQ.05 の calibrate clamp 等）はこれを前提としてよい。浮動小数点誤差で僅かに域外となる値は **[-1, 1] に clamp** する（完全一致ペアが 1.0 を超えない保証。ゼロベクトル等の特異点は本 REQ 後段の skip 経路）。

**次元不一致時の挙動（前段ループ反復 1 で確定）:** モデル切替等により次元数の異なる embedding が混在する場合（REQ.10 の全再生成が完了するまでの遷移期）、類似度計算は次元不一致ペアを skip し、**集約 Warning 1 件**（skip 件数 + `embed --all` 誘導。ペア毎ではなく集約）を報告する。【v3 差分】v3 は無言 skip（`crates/te-embed/src/similarity.rs:84-86` 等）であり、semantic 検証の静かな無効化 → check の偽 green を防ぐための可視化である。standalone `drift` の次元不一致は Error を維持する（SPEC-LGX-010.REQ.03。明示指定の対比は失敗を隠さない）。

**ゼロベクトル/数値安定性（GAP-LGX-104 対応）:**
- いずれかのベクトルのノルムが 0 のペアの cosine 類似度は **NaN/Inf を返さず、当該ペアを skip + 集約 Warning 1 件**として報告する（次元不一致と同一経路）。standalone `drift` でのゼロベクトルは Error を維持する（SPEC-LGX-010.REQ.03 と整合）
- **浮動小数点推論の値再現性の適用範囲**: CTX-INV-1 / SCORE-INV の決定論保証は**走査・出力「順序」の決定性のみ**を対象とし、ONNX 推論値のビット単位再現性（環境・BLAS 実装差による微小差）は保証対象外とする。微小差は drift_threshold が吸収する

**根拠:** v0.1.0 継承, SPEC-LGX-004.REQ.02、QSET-LGX-006 Q4 回答（2026-06-07）、GAP-LGX-104
**検証方法:** 類似度計算テスト（ゼロベクトルペアで NaN/Inf 非出力 + 集約 Warning のテストを含む）

### SPEC-LGX-006.REQ.05: ドリフト検出

**内容:** ノードの content_hash が前回の embedding 生成時と変化した場合、該当 embedding は「古い」と見なし、check 時に Drift Warning として報告する。embed の再実行で解消する。

**ノードの 3 状態（GAP-LGX-110 対応）:** embedding に関するノード状態を以下の 3 値で定義する:
- **fresh**: embedding 行が存在し content_hash 一致（SCORE-INV-1）
- **stale**: embedding 行が存在し content_hash 不一致
- **未生成**: embedding 行が存在しない（新規追加・未 embed、または REQ.02 の空テキスト skip）

`detect_drift`（REQ.11）は未生成ノードを **DriftFinding(kind=missing) として結果に含める**（戻り型の表現は DD）。check の Drift 報告（SPEC-LGX-004 REQ.02）は stale と未生成を**区別したメッセージ**で Warning 報告する【v3 差分: v3 は行不在を無言 skip — 偽 fresh 黙殺の可視化】。

**根拠:** UC-LGX-007, v0.1.0 継承、GAP-LGX-110
**検証方法:** ドリフトシミュレーションテスト（未 embed ノード混在 fixture で missing が区別報告されることを含む）

### SPEC-LGX-006.REQ.06: Contextual Retrieval（デフォルト無効）

**内容:** legixy は Contextual Retrieval に対応する:
- `[contextual_retrieval]` セクションで `enabled = true` にすると有効化
- 有効時は各ノードの embedding 生成前に、上流コンテキストを LLM API で合成してから embedding 化
- 既定は **無効**（追加の外部 API 呼出しを要するため）

**非決定性との両立（GAP-LGX-113 対応、ADR-LGX-009）:**
- **freshness 判定は content_hash のみ**で行う（SCORE-INV-1 不変。context_hash は freshness に寄与しない）
- 合成済み context は**キャッシュ**し、content_hash が不変である限り**再合成しない**（LLM 応答の揺れを fresh 判定経路から構造的に排除）
- 再生成時（stale または `--force`）の context の揺れは embedding 値の揺れとして許容する — ADR-LGX-003 の「値のビット再現は決定論保証の対象外」に包含される
- 複数ノードの context 合成は**逐次実行を既定**とする（並行化・レート制限の導入は DD 判断。単一呼出のリトライ/タイムアウトは REQ.06.1）
- 既定無効であること（本 REQ）が CTX-INV-1 の決定論前提を守る一次根拠であることを明示する

**根拠:** LGX-EXT-001 §5.8 Contextual Retrieval の実装、GAP-LGX-113、ADR-LGX-009
**検証方法:** 有効/無効切り替えテスト、content_hash 不変時に LLM API が呼ばれない（キャッシュ再利用）テスト

### SPEC-LGX-006.REQ.06.1: Contextual Retrieval 障害時の挙動

**内容:** Contextual Retrieval 有効時に LLM API 呼出しが失敗した場合、以下の階層的フォールバックを行う:
1. **タイムアウト**: 30 秒（`[contextual_retrieval] timeout_sec` で設定可能、暫定値）
2. **リトライ**: 指数バックオフで最大 3 回（初回 1 秒、2 秒、4 秒待機）
3. **永続失敗時のフォールバック**: Contextual Retrieval を**無効扱い**にして通常の embedding 生成を継続
4. **警告記録**: フォールバック発生時は stderr と `observations` テーブルに Warning を記録

API キー不正、ネットワーク不通、レート制限等の全ての障害ケースでシステム全体の動作が継続することを保証する。
**根拠:** VAL-LGX-001 Finding P-04、NFR-LGX-001.REL.02（部分失敗時の継続）
**検証方法:** LLM API モックで失敗を強制した際の通常 embedding 生成継続テスト、タイムアウト・リトライ動作テスト

### SPEC-LGX-006.REQ.07: API キーの取扱い

**内容:** Contextual Retrieval の LLM API キーは環境変数経由で受領する（`.legixy.toml` にキー自体は記載しない）。キー値はログに出力しない。
**根拠:** NFR-LGX-001.SEC.05
**検証方法:** ログ検査テスト

### SPEC-LGX-006.REQ.08: トランザクション境界

**内容:** embedding の生成・格納のトランザクション粒度は**ノード/サブノード単位の 1 トランザクション**とする（GAP-LGX-108 対応 — REQ.09 の部分失敗トレランスとの内部矛盾を解消し、REQ.09 を優先）:
- `embed --all` 全体を単一トランザクションとすることは**禁止**（1 件の失敗で全件ロールバックとなり NFR REL.02〔部分失敗時の継続〕に反するため）
- 1 ノード/サブノードの生成・格納が失敗した場合、当該トランザクションのみ rollback して Error に計上し、後続ノードの処理を継続する（REQ.09 と整合）
- 各トランザクション内では途中中断による部分的不整合は発生しない（NFR REL.06 を粒度内で維持）
- 部分失敗時の終了コードの詳細は SPEC-LGX-010 に委譲する（責務境界）

**根拠:** NFR-LGX-001.REL.06, REL.02、GAP-LGX-108
**検証方法:** 中断テスト（1 ノード失敗時に当該ノードのみ rollback され他ノードの embedding が残ることの確認を含む）

### SPEC-LGX-006.REQ.09: サブノード対応（Phase 2 で実装、LGX-EXT-001 Phase 2）

**内容:** embedding はドキュメントノードに加えサブノードも対象とする。サブノード embedding の生成には Node.content_range（h2/h3 見出しから次見出しまでの byte range、LGX-EXT-001 §4.5.1）で切り出した部分テキストのみを入力とする。**親ドキュメントのテンプレ部分（Document ID 行・ヘッダ表・変更履歴等）は意図的に除外**することで、ISSUE-005 で観測されたテンプレ寄与を排除する。

- `embed_all` のデフォルト挙動: **サブノード含む全ノード**（Phase 1 まではドキュメントのみ）
- 後方互換のため `[semantic].include_subnodes = false` で Phase 1 動作に切替可能
- サブノード embedding の入力テキスト: `content[range.0..range.1]`（親見出し・上位章は含めない）
- 失敗時の挙動: 該当サブノードのみ Error 計上、他ノードは継続（既存の部分失敗トレランス継承）
- **content_range の防御的検証（GAP-LGX-118 対応）**: producer（LGX-EXT-001 §4.5.1 の見出し抽出）を信頼境界内としつつ、embed 側で range の妥当性を検証する。①逆転（`range.0 > range.1`）②ファイル長超過 ③UTF-8 文字境界違反は、**当該サブノードのみ Error 計上 + 後続継続**（本 REQ のトレランスと同経路）。いかなる不正 range でも **panic しない**（NFR SEC.03/04。文字境界安全な切り出し方式は DD）。空 range（`range.0 == range.1`）は不正ではなく REQ.02 の空テキスト skip 経路で扱う（GAP-LGX-101）

**根拠:** SPEC-LGX-002.REQ.05、LGX-EXT-001 Phase 2、ISSUE-005 §2.3 vnstudio Phase 1 ベースライン（全ペア mean=0.6798、テンプレ寄与で底上げ）への対応
**検証方法:** サブノード embedding テスト（embed_all がドキュメント + サブノードを処理し、それぞれ別 embedding を engine.db に格納すること）

### SPEC-LGX-006.REQ.10: モデル更新時の再計算

**内容:** `model_version` が変化した場合、全 embedding を再生成する。`embed --all --force` で強制再生成可能（`--force` の引数仕様は REQ.02）。
再生成完了までの遷移期における類似度計算の挙動（次元不一致ペアの skip + 集約 Warning）は REQ.04 を参照。

**model_version の生成方式（GAP-LGX-115 対応）:** model_version は以下の複合キーとして決定論的に生成する:
- (a) モデル名
- (b) ONNX ファイル内容のハッシュ（同名ファイルの差し替えを検出し偽 fresh を防止）
- (c) 前処理プロファイル（e5 系 `query:`/`passage:` prefix の有無等。REQ.01 の代替モデル採用時に判定へ寄与）
- (d) 出力次元数

「変化した」の判定は model_version 文字列の**完全一致**で行う（SCORE-INV-2 判定の決定論化）。ハッシュの hex 桁数等の表現詳細は DD で確定する。旧モデル embedding からの移行手順は GAP-LGX-116（別管理）と連携する。

**根拠:** ソフトウェア保守性、GAP-LGX-115
**検証方法:** モデル切替テスト（同名 ONNX ファイル差し替えで model_version が変化することの確認を含む）

### SPEC-LGX-006.REQ.11: Bulk similarity API（SEM/RPT/CAL 共通基盤）

**内容:** legixy-embed（crate 名は例示であり DD で凍結、SPEC-LGX-001.REQ.03）は以下の bulk similarity API を公開する（legixy-check SemanticChecker〔SPEC-LGX-004.REQ.02〕/ `report`・`calibrate` コマンド〔SPEC-LGX-010.REQ.04/REQ.05〕の共通基盤）:

- `compute_edge_scores(graph, store) -> Vec<EdgeScore>`: graph.toml の全エッジ（Chain/Custom/ParentChild）に対し cosine 類似度を算出
- `compute_link_candidates(graph, store, threshold) -> Vec<CandidateScore>`: 非エッジペアで類似度 ≥ threshold のものを抽出（O(N²)）
- `compute_all_pair_scores(store) -> Vec<(NodeId, NodeId, f32)>`: 全ペア類似度（calibrate ヒストグラム用、O(N²)）
- `detect_drift(graph, store, project_root) -> Vec<DriftFinding>`: 各ノードのファイル SHA-256 と store の保存済 content_hash を比較
- `histogram(scores, buckets) -> Vec<Bucket>`: ストリーミング対応のヒストグラム集計
- `EmbeddingStore::load_all() -> Vec<EmbeddingRow>`: node_id 昇順で全 embeddings をロード（決定性担保）

戻り値は SCORE-INV-1（決定性）を保証する順序で返却する。

**consumer 側の仕様所在（前段ループ反復 1 で確定）:** 本 API を消費するコマンド群の出力仕様・引数・終了コードは SPEC-LGX-010（embedding 運用・監査）が規定する。`calibrate --recommend` の推奨閾値ロジック（p25 / 1.0−p90 / p75 のパーセンタイル方式）も SPEC-LGX-010.REQ.05 に正準定義がある。本 SPEC はエンジン（生成・検出・bulk API）に責務を限定する。

**根拠:** LEGIXY-SPEC-001 §4（semantic_check）、workflow_2026-04-20_semantic-check-and-reporting.md §1.1
**検証方法:** TS-LGX-005 §10 T-EMB-SIM-001〜004

### SPEC-LGX-006.REQ.12: サブノード embedding の格納項目（Phase 2 新規）

**内容:** `embeddings` テーブルへの永続化時、サブノードノードについて以下の追加情報を保持する:

- `node_id`: サブノード ID（例: `DD-VNS-003#a3f7b2c4e91dfa08` または明示 ID `DD-VNS-003#s:cross-section`）
- `parent_id`: 親ドキュメント ID（例: `DD-VNS-003`）
- `anchor`: 見出しテキスト（例: `## 認証機能`）
- `is_subnode`: 1（boolean、index 用）
- 他のフィールド（`embedding`, `embedding_dim`, `model_version`, `content_hash`, `created_at`, `context`, `context_hash`）はドキュメントノードと同様

格納先テーブルは Phase 1 で予約された列を使用し、新規テーブルは作らない（LGX-EXT-001 §4.3 / §8.2）。
content_hash は **サブノードの content_range 部分のみ** から計算する（親ドキュメント全体ではない）。正規化手順は REQ.03 と同一（BOM 除去 → 改行統一 → NFC → 末尾正規化、GAP-LGX-114 対応）。

検索性能のため `(parent_id, anchor)` の INDEX を追加する。

**根拠:** LGX-EXT-001 §8.2 Phase 1 設計余地の活用、ISSUE-005 検証データ取得のための storage 要求
**検証方法:** サブノード embedding 永続化テスト

---

## 4. 不変条件との関係

| 不変条件 | 役割 | 対応要求 |
|---------|------|---------|
| CTX-INV-1（決定論保証） | 関連 | REQ.08（トランザクション境界により同一入力→同一結果を保つ）。適用範囲は走査・出力**順序**の決定性のみ — ONNX 推論値のビット再現性は対象外（REQ.04、GAP-LGX-104） |
| CTX-INV-2（グラフ整合性） | 実装 | REQ.09（サブノード embedding がグラフ定義と整合） |
| SCORE-INV-1（ハッシュ一致保証） | 実装 | REQ.03（content_hash フィールド管理 + 正規化手順で環境非依存に固定、GAP-LGX-114）, REQ.05（drift 検出は content_hash 変化で判定） |
| SCORE-INV-2（モデルバージョン一致） | 実装 | REQ.03（model_version フィールド管理）, REQ.10（複合キー生成 + 完全一致判定で決定論化、GAP-LGX-115。モデル更新時の再計算） |

**本 SPEC が関与しない不変条件:** CTX-INV-3/4/5, MCP-INV-1〜4、SUBNODE-INV-1〜6、FB-INV-1〜5、STATE-INV-1/2、CACHE-INV-1〜4

---

## 5. 変更履歴

| 日付 | バージョン | 変更内容 |
|------|----------|---------|
| 2026-04-17 | 0.1.0-draft | 初版（AI 起草、v0.1.0 既存機能を明文化 + Contextual Retrieval 追加） |
| 2026-04-17 | 0.1.1-draft | F-01 修正: 不変条件テーブルの CTX-INV-3 誤記を修正。F-02 修正: 親文書と §2 参照の §5.2 → §5.8（Contextual Retrieval の実装）に訂正、§5.7 参照も追加。F-03 修正: REQ.03 から DB スキーマ具体記述を削除し必須情報リストに抽象化 |
| 2026-04-17 | 0.1.2-draft | F-04 修正: §4 表に「役割」列を追加、対象外不変条件（CTX-INV-3/4, MCP-INV-*, SUBNODE-INV-*）を明記 |
| 2026-04-17 | 0.2.0 | 人間査読完了により承認 |
| 2026-04-17 | 0.3.0 | S1-c 対応: REQ.06.1 Contextual Retrieval 障害時の階層的フォールバック（タイムアウト 30 秒 / 指数バックオフ 3 回リトライ / 無効扱いでの継続）を新設（Finding P-04） |
| 2026-04-28 | 0.4.0 | LGX-EXT-001 Phase 2 Block A 対応。REQ.09 サブノード対応を Phase 1 予約 → Phase 2 実装に格上げ（content_range ベース、テンプレ寄与排除）。REQ.12 サブノード embedding 格納項目（parent_id / anchor / is_subnode）を新設 |
| 2026-04-17 | 0.3.1 | S1-d 対応: §4 表に SCORE-INV-1（ハッシュ一致保証、既存 REQ.03/05 実装）と SCORE-INV-2（モデルバージョン一致、既存 REQ.03/10 実装）を追加。対象外不変条件の一覧も CACHE-INV-* と LEGIXY-SPEC-001 §10 の全 INV を網羅 |
| 2026-04-20 | 0.4.0-draft | SEM Block（workflow_2026-04-20_semantic-check-and-reporting.md §2）で REQ.11 Bulk similarity API を新設。legixy-embed に `compute_edge_scores` / `compute_link_candidates` / `compute_all_pair_scores` / `detect_drift` / `histogram` / `EmbeddingStore::load_all` を追加し、legixy-check SemanticChecker / `report` / `calibrate` の 3 ブロック共通基盤とする |
| 2026-06-07 | 0.5.0 | 前段ループ反復 1（QSET-LGX-006 回答 → SPP-LGX-006 承認）対応: ヘッダ Version の不整合（0.3.1 表記 vs 履歴 0.4.0）を解消し 0.5.0 へ。§3 の REQ 物理順序を ID 順（09→10→11→12）に整列（REQ-id 不変）。REQ.04 に次元不一致時の「skip + 集約 Warning【v3 差分】」を確定（drift の Error は SPEC-LGX-010 で維持）。REQ.11 に SPEC-LGX-010 相互参照と crate 名例示化注記を追加。既定モデルの運用整合（CLAUDE.md / trace-check.sh → paraphrase-multilingual-MiniLM-L12-v2）は SPEC 無変更の付随修正として実施 |
| 2026-06-10 | 0.6.0 | TP[SPEC] GAP 解消（人間承認 2026-06-10、5 件単一改訂）: GAP-LGX-104 対応で REQ.04 にゼロベクトル cosine の skip + 集約 Warning と浮動小数点値再現性の適用範囲（順序決定性のみ）を確定、§4 CTX-INV-1 行に適用範囲を明記。GAP-LGX-108 対応で REQ.08 のトランザクション粒度を「ノード/サブノード単位 1 Tx」に確定（REQ.09 優先、--all 全体 1 Tx 禁止）。GAP-LGX-114 対応で REQ.03/REQ.12 に content_hash 正規化手順（BOM→改行→NFC→末尾）を確定。GAP-LGX-115 対応で REQ.10 に model_version 複合キー生成（名前+ONNX 内容ハッシュ+前処理プロファイル+次元）と完全一致判定を確定。GAP-LGX-120 対応（人間承認・凍結契約への加算的拡張）で REQ.02 に --node/--force/--json スキーマを確定【v3 差分: v3 は --all のみ】、ADR 起票 + LGX-COMPAT-001 §4 #4/§7 追記と連動 |
| 2026-06-10 | 0.7.0 | weak GAP 解消（人間裁定 fix・承認 2026-06-10、8 件単一改訂）: GAP-LGX-101 — REQ.02 空テキストは skip + 集約 Warning・未生成状態へ。GAP-LGX-102 — REQ.01 トークン上限超過は先頭切り捨て + 集約 Warning【v3 差分】（分割平均は不採用、サブノード化が対策）。GAP-LGX-103 — REQ.01 読込時 shape 検証・非適合は embed 起動不能 exit 1。GAP-LGX-105 — REQ.04 に cosine 値域 [-1,1]・負値正常・域外 clamp を明示。GAP-LGX-106 — REQ.02 モデル解決失敗 exit 1 + 試行パス通知（SPEC-LGX-010 と同一解決順）、[semantic] enabled は check 専用と明記。GAP-LGX-110 — REQ.05 に 3 状態（fresh/stale/未生成）と detect_drift の missing 包含【v3 差分】。GAP-LGX-113 — REQ.06 に freshness は content_hash のみ・context キャッシュ・逐次既定（ADR-LGX-009）。GAP-LGX-118 — REQ.09 に content_range 防御的検証（panic 禁止・該当のみ Error 継続・空 range は 101 経路） |
