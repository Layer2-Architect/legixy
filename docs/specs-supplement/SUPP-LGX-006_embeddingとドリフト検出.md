Document ID: SUPP-LGX-006

# SUPP-LGX-006: SPEC-LGX-006（embedding とドリフト検出）実装補完情報

| 項目 | 内容 |
|------|------|
| Document ID | SUPP-LGX-006 |
| 対象 SPEC | SPEC-LGX-006 Version 0.7.0（2026-06-10 承認） |
| Status | AI生成・非正準・人間査読待ち |
| Date | 2026-06-12 |

> **本文書は SPEC 本文の変更ではなく実装のための補完情報（参考資料）である。SPEC 変更には人間承認が必要（SPEC-LGX-001 §7.1）。**
>
> 補完の根拠は主に旧文書群 `legixy.old.p1/docs/`（以下「old.p1」）と旧実装 `traceability-engine.v3.chg_to_lexigy/`（以下「v3 実装」。crate ディレクトリ名は `lx-*`、LGX-COMPAT-001 の表記は `te-*` だが同一物）から採取した。[補完] = 根拠付きで内容を特定済み。[要決定] = 人間の判断（または DD 段階の凍結）が必要。

---

## §1 未解決参照（SPEC が参照するが新リポジトリに存在しない文書）

新リポジトリには `docs/specs/` の SPEC 10 件しか無い。SPEC-LGX-006 が参照する以下の文書は新リポジトリに不在であり、実装時は下表の所在から参照する（または新リポジトリへの取り込みを検討する）。

| # | 参照 ID | SPEC 内の参照箇所 | 必要な理由 | 所在（確認済み） |
|---|---------|------------------|-----------|------------------|
| 1 | LGX-EXT-001（サブノード仕様 v0.2.1） | ヘッダ親文書、§2、REQ.03/06/09/12 根拠 | embeddings テーブル拡張（§4.3）、サブノード ID 規則・content_range（§4.5.1）、embedding 抽象化（§5.7）、Contextual Retrieval（§5.8）、Phase 1 予約列（§8.2）の正準定義 | `legixy.old.p1/docs/legixy_subnode_spec_v0.2.1.md` |
| 2 | LEGIXY-SPEC-001（基盤仕様） | REQ.01 根拠（§2）、REQ.11 根拠（§4 semantic_check）、§4 不変条件（§10） | CTX-INV/SCORE-INV 等の不変条件の正準定義、semantic_check の位置づけ | `legixy.old.p1/docs/legixy_foundational_spec.md` |
| 3 | NFR-LGX-001 | ヘッダ「対応 NFR」、REQ.01/03/06.1/07/08/09 根拠 | PERF.08（≥50 nodes/sec【暫定】）、REL.02/06、SEC.03/04/05、COMPAT.07/08 の数値・条件 | `legixy.old.p1/docs/nfr/NFR-LGX-001_非機能要件.md`（行 86, 108-110, 127-128, 154, 158） |
| 4 | UC-LGX-007 | ヘッダ「対応 UC」、REQ.02/05 根拠 | embed/drift の基本フロー（前処理→トークン化→推論→Mean Pooling→L2 正規化）、事前条件（model.onnx + tokenizer.json） | `legixy.old.p1/docs/usecases/UC-LGX-007_embedding生成とドリフト検出.md` |
| 5 | LGX-COMPAT-001 v1.1.0 | REQ.02「凍結契約との関係」 | CLI 引数互換契約（§3 グローバルオプション・exit 2 規約、§4 #4 embed 行、§7 加算的拡張の規律） | `legixy.old.p1/docs/legixy_cli_compat_reference.md` |
| 6 | GAP-LGX-101/102/103/104/105/106/108/110/113/114/115/118/120（13 件） | REQ.01〜12 の各「GAP-LGX-NNN 対応」 | 各確定仕様の検討経緯・選択肢（実装判断の背景） | `legixy.old.p1/docs/gap-analysis/GAP-LGX-1NN_*.md`（13 ファイル全て存在確認済み） |
| 7 | GAP-LGX-116（旧モデル移行） | REQ.10「GAP-LGX-116（別管理）と連携」 | 旧モデル embedding からの移行手順 | **存在しない（削除済み）**。`legixy.old.p1/docs/test-perspectives/TP-LGX-006_embeddingとドリフト検出.md` 行 128 にて「DUPLICATE（GAP-LGX-115 派生）+ OUT_OF_SCOPE として GAP-LGX-116 削除。移行は UC-LGX-009 / NFR COMPAT.04 が所有」と裁定済み。SPEC 本文の参照は当該裁定より古い記述の残存（SPEC 修正は人間承認事項） |
| 8 | ADR-LGX-003 / ADR-LGX-009 | REQ.04/06（決定論モデル、CR 非決定性） | 順序決定性のみ保証・正規化ハッシュ・複合 model_version（003）、context キャッシュ + content_hash 限定 freshness（009）の判断記録 | `legixy.old.p1/docs/adr/ADR-LGX-003_embedding-determinism-model.md`、`ADR-LGX-009_contextual-retrieval-determinism.md`。REQ.02 の「拡張の経緯は ADR に記録」の実体は `ADR-LGX-002_embed-node-force-additive-extension.md` |
| 9 | QSET-LGX-006（Q4 回答 2026-06-07） | REQ.04 根拠 | 次元不一致 skip + 集約 Warning の決定経緯と Warning 文言例 | `legixy.old.p1/docs/frontend-pass/questionnaires/QSET-LGX-006_embeddingとドリフト検出.md` §Q4 |
| 10 | SPP-LGX-006 | 変更履歴 0.5.0 | 前段ループ反復 1 の機械的差分内容 | `legixy.old.p1/docs/spec-patches/SPP-LGX-006_embeddingとドリフト検出.md` |
| 11 | VAL-LGX-001 Finding P-04 | REQ.06.1 根拠 | CR 障害時フォールバック要求の出所 | old.p1 の `docs/validation/` は空。同等物は v3 実装側 `docs/validation/VAL-LX-001_外部照合記録.md`（Finding P-04【MEDIUM】行 159、対応状況行 416。ID 体系が `LX` であることに注意） |
| 12 | TS-LGX-005 §10 T-EMB-SIM-001〜004 | REQ.11 検証方法 | bulk similarity API のテスト仕様 | old.p1 の `docs/test-specs/` は空。同等物は v3 実装側 `docs/test-specs/TS-LX-005_embedding生成とドリフト検出.md` §10（T-EMB-SIM-001〜004 の 4 件が完全一致で存在） |
| 13 | workflow_2026-04-20_semantic-check-and-reporting.md §1.1/§2 | REQ.11 根拠 | bulk API 新設の計画文書 | `traceability-engine.v3.chg_to_lexigy/claudedocs/workflow_2026-04-20_semantic-check-and-reporting.md` |
| 14 | ISSUE-005 §2.3 | REQ.09 根拠 | vnstudio Phase 1 ベースライン実測（テンプレ寄与）の数値 | `traceability-engine.v3.chg_to_lexigy/issues/ISSUE-005_template-similarity-noise-floor.md` |

**新リポジトリ内で解決する参照（不足ではない）**: SPEC-LGX-001/002/004/010（`legixy/docs/specs/` に存在。SPEC-LGX-010.REQ.03 のモデル解決順・REQ.05 の calibrate パーセンタイル正準定義も確認済み）。DevProc_V4 文書は `legixy/docs/DevPorc/`（ディレクトリ名 typo に注意）に vendored 済み。

---

## §2 実装に必要だが SPEC 内で未規定の事項

### 2.1 モデルファイル

**[補完] 2.1-a モデルディレクトリの構成と配置規約**
モデルディレクトリは `models/<モデル名>/` 配下に `model.onnx` + `tokenizer.json` の 2 ファイル（UC-LGX-007 事前条件、v3 実装 `crates/lx-embed/src/embedder.rs:54-68` が両ファイルの存在を検査し `ModelLoadFailed` を返す）。既定解決パスは `<project_root>/models/<config.semantic.model>`（v3 実装 `crates/lx-cli/src/model_dir.rs:55-62`）。

**[補完] 2.1-b 既定モデルの実体の所在**
既定モデル `paraphrase-multilingual-MiniLM-L12-v2` の ONNX 実体（model.onnx + tokenizer.json）は **`legixy.old.p1/models/paraphrase-multilingual-MiniLM-L12-v2/` に配置済み**（確認済み）。v3 実装側には旧モデル `models/all-MiniLM-L6-v2/` のみ。取得元の前例は HuggingFace（v3 `deploy/manual.md` §5.4: `https://huggingface.co/sentence-transformers/all-MiniLM-L6-v2`）。

**[要決定] 2.1-c モデルの入手・配布手順**
新リポジトリにモデル実体が無い。選択肢: (a) old.p1/models からコピーして同梱（ライセンス: paraphrase-multilingual-MiniLM-L12-v2 は Apache-2.0、再配布可だが OSS 公開時のリポジトリサイズ要検討）、(b) HF からのダウンロード + ONNX エクスポートスクリプトを提供（HF 公式リポジトリに ONNX が同梱されているかの確認、無ければ optimum 等での変換手順の文書化が必要）、(c) 別配布チャネル。配置検証手順（ファイルハッシュの pin 等）も未規定。

**[要決定] 2.1-d トークン上限の取得元**
REQ.01 は「最大入力トークン長はモデル metadata から取得する」とするが、取得元の具体（tokenizer.json の truncation/model_max_length、ONNX モデルのメタデータ、または設定値）が未確定。v3 実装はトークン上限を扱わず `tokenizer.encode(text, true)` をそのまま流す（embedder.rs:125。tokenizer.json 内の truncation 設定に暗黙依存 = SPEC の言う「v3 は無言切り捨て」）。切り捨て境界の厳密挙動（特殊トークン分の確保等）は SPEC 自身が DD 確定事項と明記。

**[要決定] 2.1-e 出力 shape 検証（GAP-LGX-103）の合格条件の具体化**
「mean pooling 可能な軸構造・正の hidden 次元」の機械的判定（rank=3 [batch, seq, hidden] の確認方法、動的次元の扱い）は DD で確定が必要。v3 実装に shape 検証は無い（embedder.rs:191 は `shape.last()` を読むのみ）。

### 2.2 データ形式・スキーマ

**[補完] 2.2-a embeddings テーブルの具体 DDL（REQ.03 の「DD で定義」の前例）**
v3 実装 `crates/lx-db/src/schema.rs:37-54`:

```sql
CREATE TABLE IF NOT EXISTS embeddings (
    node_id TEXT PRIMARY KEY,
    embedding BLOB NOT NULL,
    embedding_dim INTEGER NOT NULL,
    model_version TEXT NOT NULL,
    content_hash TEXT NOT NULL,
    context TEXT NULL,
    context_hash TEXT NULL,
    parent_id TEXT NULL,
    anchor TEXT NULL,
    is_subnode INTEGER NOT NULL DEFAULT 0,
    created_at TEXT DEFAULT (datetime('now'))
);
CREATE INDEX idx_embeddings_parent ON embeddings (parent_id, anchor);   -- REQ.12 の INDEX
CREATE INDEX idx_embeddings_is_subnode ON embeddings (is_subnode);
```

REQ.03 の必須情報・REQ.12 の追加項目を全て満たす。既存 DB への列追加 migration（PRAGMA table_info → ALTER TABLE、CREATE INDEX より先に実行）の前例は schema.rs:186-220。なお LGX-EXT-001 §4.3 の `is_subnode` は 0/1/2 の 3 値（2 = 明示サブノード）だが v3 の store 書込（store.rs:101-104）は 0/1 に縮退している。REQ.12 は「1（boolean、index 用）」とするため SPEC 準拠は 0/1 で足りるが、LGX-EXT-001 との差は DD で明記したほうがよい。

**[補完] 2.2-b ベクトルの直列化形式**
f32 little-endian 連結 BLOB（v3 実装 store.rs:297-313 `f32_slice_to_bytes` / `bytes_to_f32_vec`。unsafe 不使用、4 バイト未満の端数は切り捨て）。

**[補完] 2.2-c freshness 判定（is_up_to_date）の条件**
`content_hash` と `model_version` の**双方一致**で skip（SCORE-INV-1 + SCORE-INV-2、v3 実装 store.rs:35-53）。REQ.02 本文は「content_hash 一致」のみ言及だが、REQ.10（model_version 変化で全再生成）と合わせると双方一致が正しい実装である。

**[補完] 2.2-d embed --json の v3 実出力（差分実装の起点）**
v3 の `EmbedReport`（orchestrator.rs:51-56）は `{generated, skipped, errors: [["node_id","message"],...]}`（タプル列、`failed` 欄無し）。SPEC 0.7.0 の確定スキーマ `{generated, skipped, failed, errors: [{node_id, message}]}` は**新形式**であり、`failed`（= errors の件数）の追加と errors 要素のオブジェクト化が必要。

**[補完] 2.2-e observations テーブル（REQ.06.1 の Warning 記録先）**
v3 実装 schema.rs:71-81: `observations(id, source, category, severity, message, related_ids, context_json, status, created_at)`。category は CLI 上 `compile_miss` / `review_correction` / `manual_note` の 3 値（LGX-COMPAT-001 §4.1）。

**[要決定] 2.2-f CR フォールバック Warning の observations 記録の語彙**
v3 実装は stderr 出力のみで observations に**書いていない**（contextual.rs:119-124 `emit_cr_warning`）。REQ.06.1 を満たすには source/category/severity に入れる値の決定が必要（既存 category 3 値に該当が無い。新 category を足すなら SPEC-LGX-007（フィードバックループ）との整合確認が必要）。

### 2.3 アルゴリズム

**[補完] 2.3-a embedding 生成パイプラインの具体**
v3 実装 embedder.rs:100-227: 前処理（Phase 1 はパススルー、preprocessor.rs）→ tokenizer encode → ONNX 推論（入力 3 系統 `input_ids` / `attention_mask` / `token_type_ids`、形状 (1, seq_len)、GraphOptimizationLevel::Level3）→ attention_mask 加重 Mean Pooling → L2 正規化。出力次元は出力テンソル shape の最終軸から動的取得（REQ.01 の「出力 shape から動的に確定」と一致）。注意: モデルによっては `token_type_ids` 入力を持たない場合があり、入力名の動的検査（session の入力一覧を見て与える）を DD で検討すること（paraphrase-multilingual-MiniLM-L12-v2 は BERT 系で 3 入力が前例どおり動く想定だが要実機確認）。

**[補完] 2.3-b cosine 類似度と drift の式**
`cosine_similarity = dot / (||a||·||b||)`（drift.rs:24-43）、`drift = 1.0 − cosine`、値域 [0.0, 2.0]（drift.rs:9-19）。v3 のゼロノルム時の挙動は「0.0 を返す」（drift.rs:38-39）だが、SPEC 0.7.0 REQ.04 は標準経路で **skip + 集約 Warning** に変更済み（v3 差分）。clamp [-1,1] も v3 に無い新規実装。

**[補完] 2.3-c bulk API（REQ.11）の前例実装と決定性**
v3 実装 similarity.rs に 6 関数全ての前例あり: `compute_edge_scores`（graph.edges() 挿入順）、`compute_link_candidates`（node_id 昇順ペア・既存エッジは無向で除外・threshold は `>=`）、`compute_all_pair_scores`（i<j の昇順）、`detect_drift`、`histogram`（[0,1] 均等幅、末尾バケット上限 inclusive、域外 clamp）、`EmbeddingStore::load_all`（ORDER BY node_id ASC、store.rs:165-190）。結果型 `EdgeScore` / `CandidateScore` / `DriftFinding` / `Bucket` の定義は similarity.rs:24-55。ただし次元不一致・ゼロ次元の**無言 skip**（similarity.rs:84-86, 124-126, 155-157）は SPEC で集約 Warning 化が必要（v3 差分）。

**[補完] 2.3-d content_hash 計算の v3 実態（正規化は新規実装）**
v3 は前処理後テキストの生バイト SHA-256（hex 小文字 64 桁、embedder.rs:239-243）であり、REQ.03 の 4 段正規化（BOM 除去→CRLF/CR→LF→NFC→末尾正規化）は**全て新規実装**。NFC には `unicode-normalization` 等の crate 追加が必要。

**[要決定] 2.3-e detect_drift 側のハッシュ計算経路の統一**
v3 の `detect_drift`（similarity.rs:171-214）は**ファイル全文**の生 SHA-256 を保存済み content_hash と比較する。一方 embed 側はサブノードについて **content_range 切り出し + 前処理後**のハッシュを保存する（orchestrator.rs:109-112）。つまり v3 ではサブノード行が detect_drift で恒常的に偽 drift になる構造的不整合がある（compute_node_drift は ISSUE-003 BUG-3 で修正済みだが detect_drift は未修正）。新実装では「正規化 + content_range 切り出し」を embed / detect_drift で**共有する単一関数**にすること。共有層の設計は DD 事項。

**[要決定] 2.3-f 末尾正規化の厳密挙動**
SPEC 自身が「厳密挙動は DD で確定」とする（候補: 末尾の改行列を 0 個に正規化 vs 1 個に正規化。ADR-LGX-003 残存リスク欄も DD 凍結時の fixture pin を要求）。

**[要決定] 2.3-g model_version 複合キーの表現**
(a) モデル名 + (b) ONNX 内容ハッシュ + (c) 前処理プロファイル + (d) 次元、の連結書式・ハッシュ hex 桁数・区切り文字・前処理プロファイルの識別子語彙（例: `plain` / `e5-prefix`）が未確定（SPEC が DD 委譲を明記）。v3 は `config.semantic.model` の文字列をそのまま model_version に使う（embed.rs:52）ため**全面的に新規実装**。ONNX ファイルハッシュ計算（数十 MB）の起動時コストの扱い（キャッシュするか）も DD 判断。

### 2.4 設定・しきい値

**[補完] 2.4-a `.legixy.toml` [semantic] の既定値**
v3 実装 loader.rs:235-239 / LGX-COMPAT-001 §6 / v3 manual.md §11 より:

```toml
[semantic]
enabled = true            # check の意味層のみ制御（embed の実行可否には無関係 = REQ.02）
model = "paraphrase-multilingual-MiniLM-L12-v2"   # legixy 既定（v3 実測は all-MiniLM-L6-v2）
similarity_threshold = 0.4
drift_threshold = 0.3
link_candidate_threshold = 0.7
include_subnodes = true   # 未指定時 true（REQ.09 の Phase 1 互換切替は false）

[contextual_retrieval]
enabled = false           # REQ.06 既定無効。v3 の構造体はこの 1 キーのみ（model.rs:180-183）
```

なお LGX-COMPAT-001 §6 の `[semantic] vector_store = "docs/traceability/vectors/"` キーは旧バイナリ実測値として記録されているが、v3 の SemanticConfig 構造体（model.rs:168-178）には存在しない（読み捨て）。ISSUE-003（vector_store 設定の stale）参照。

**[補完] 2.4-b しきい値の運用上の注意（ISSUE-005 / ISSUE-004）**
similarity_threshold 0.4 はテンプレ寄与で底上げされた分布では誤検知を生む（ISSUE-005 §5.3）。vnstudio 実測では link_candidate_threshold ≈ 0.85 が暫定推奨（ISSUE-005 §2.3）。Phase 2（サブノード embedding）移行後は分布が大きく変わるため再 calibrate 推奨の注記が必須（ISSUE-005 §5.2）。推奨閾値ロジック（p25 / 1.0−p90 / p75）の正準は SPEC-LGX-010.REQ.05（新リポジトリに存在）。

**[要決定] 2.4-c [contextual_retrieval] の追加キー**
REQ.06.1 は `timeout_sec`（既定 30、暫定値）のみ設定可能と明記。`max_retries` / バックオフ初期値 / provider / model 等を設定可能にするかは未規定（v3 manual.md §11 も「Phase 2 以降の運用開始時に確定予定」と保留）。CrOptions の既定値の前例: timeout_sec=30 / max_retries=3 / base_backoff_ms=1000（contextual.rs:15-23）。

### 2.5 CLI・エラー処理

**[補完] 2.5-a モデル解決失敗時のエラー内容（REQ.02 / GAP-LGX-106）**
v3 実装 model_dir.rs:12-27 の 3 エラー型（NotFound: 探索パス列挙 / InvalidDir / ModelFileMissing）とメッセージが「試行したパスを stderr に通知」の前例。legixy では解決順に `LGX_MODELS_DIR` を正準として追加し `TE_MODELS_DIR` は旧名フォールバック + Info 案内（SPEC-LGX-010.REQ.03、新リポジトリに正準定義あり）。

**[補完] 2.5-b v3 の embed CLI 挙動（互換の基準点）**
`embed` フラグ無し → exit 2（使用法誤り、embed.rs:26-29）。部分失敗（errors 非空）→ exit 1（embed.rs:99-101）。`--node` / `--force` は v3 に存在しない（SPEC 主導の加算的拡張、ADR-LGX-002 / LGX-COMPAT-001 §4 #4）。進捗 spinner（indicatif）は NFR USE.03 対応。

**[要決定] 2.5-c `--node` と `--all` の排他違反時の終了コード**
REQ.02 検証方法に「--all 排他 exit 1」とあるが、LGX-COMPAT-001 §3 のグローバル規約では「引数パーサ層が検出する構文レベルの誤りは exit 2」。clap の `conflicts_with` で実装すると exit 2 になる。どちらの解釈か（パーサ層で弾く=2 / 受理後に意味検査で弾く=1）の確認が必要。SPEC 本文の「指定 ID が graph.toml に未登録なら exit 1」は意味的不正なので exit 1 で確定。

**[要決定] 2.5-d 集約 Warning の文言・JSON 併出**
文言の前例は QSET-LGX-006 Q4 回答の例「次元不一致により N ペアの類似度計算を skip（model_version 遷移中。`embed --all` で再生成してください）」のみ。空テキスト skip・トークン切り捨ての集約 Warning 文言、および `--json` 指定時に Warning を JSON へ含めるか（stderr のみか）は未規定。`embed --json` スキーマに warning 欄は無いため stderr のみが整合的だが、確認が必要。

**[要決定] 2.5-e DriftFinding(kind=missing) の型表現**
SPEC が「戻り型の表現は DD」と明記。v3 の `DriftFinding { node_id, stored_hash, current_hash, missing_file: bool }` の `missing_file` は「**ファイル**不在」であり、REQ.05 の「未生成（embedding 行不在）」とは別概念。v3 は embedding 行不在ノードを無言 skip する（similarity.rs:184-186）ため、kind 列挙（例: ContentChanged / FileMissing / EmbeddingMissing）の新設が必要。check 側の stale / 未生成の区別メッセージ文言も未規定。

### 2.6 Contextual Retrieval

**[補完] 2.6-a 実装骨格の前例**
v3 実装 contextual.rs に LlmClient trait（`complete(&self, prompt, timeout_sec)`）、`synthesize_with_fallback`（初回 + 最大 3 リトライ = 4 試行、指数バックオフ 1s/2s/4s、永続失敗で Ok(None) 返却 + stderr Warning = REQ.06.1 の階層的フォールバックと一致）、プロンプト生成 `build_prompt`（親文書 + 対象ノード ID から ≤100 words の文脈要約を要求）、`mask_api_key`（sk-ant- / sk- / AIza / Bearer の 4 パターンを `***REDACTED***` 置換 = NFR SEC.05）の骨格がある。LLM プロバイダは LGX-EXT-001 §5.8 で「Claude API」と明記。

**[要決定] 2.6-b 具象 LLM クライアントと API キー**
v3 は trait のみで**具象クライアント未実装・CLI から未配線**（embed.rs:73 で `contextual: None` ハードコード）。決定が必要: API キーの環境変数名（`ANTHROPIC_API_KEY` が自然だが REQ.07 は「環境変数経由」としか規定しない）、使用モデル・エンドポイント・max_tokens、HTTP クライアント crate。

**[要決定] 2.6-c context キャッシュの保存場所・無効化条件**
ADR-LGX-009 が「DD 委譲」と明記。embeddings テーブルの `context` / `context_hash` 列が Phase 1 予約済み（LGX-EXT-001 §8.2）であり、これを「content_hash 不変なら context を再利用」のキャッシュ実体とするのが自然だが、`--force` 時に context を再合成するか再利用するかの細部が未確定（REQ.06 は「再生成時（stale または --force）の context の揺れは許容」とするので再合成と読める）。

### 2.7 サブノード（REQ.09/12）

**[補完] 2.7-a content_range 切り出しと親ファイル読込の前例**
v3 実装 orchestrator.rs:79-107: サブノードは parent_id を辿って**親ドキュメントのファイル**を読み、`content[range.0..range.1]` を embedding 入力にする。`read_current_content_for_node`（orchestrator.rs:145-175）が embed / drift 共通の切り出しヘルパ（ISSUE-003 BUG-3 fix、冪等性担保）。サブノード ID の生成規則（`{親ID}#{SHA-256 先頭16hex}` / `{親ID}#s:{英数}`）と見出し正規化は LGX-EXT-001 §4.5.1-§4.6。

**[補完] 2.7-b v3 の range 防御の差分（GAP-LGX-118）**
v3 は range をファイル長で clamp し、UTF-8 境界違反時は `from_utf8().unwrap_or(&full_content)` で**全文へフォールバック**する（orchestrator.rs:100-106）。SPEC 0.7.0 は①逆転②長超過③境界違反を「当該サブノードのみ Error 計上 + 継続」と確定したため、このフォールバックは**廃止して Error 化**する必要がある（v3 差分）。文字境界安全な切り出し方式（`str::is_char_boundary` 検査等）は DD 事項。

**[補完] 2.7-c REQ.09 根拠の実測値**
ISSUE-005 §2.3（vnstudio、v0.3.0 + all-MiniLM-L6-v2、112 ノード）: 全ペア mean=0.6798、chain リンク mean=0.6081（全ペアより低い逆転現象）、≥0.94 の高類似ペア 21 件はテンプレ共有由来、`embed --all` 3.15 秒、engine.db 332KB。Phase 2 後の期待: 全ペア mean ≈0.4 への低下と chain mean の逆転解消（ISSUE-005 §2.2/§2.3）。

### 2.8 性能

**[要決定] 2.8-a PERF.08（≥50 nodes/sec）の再評価**
NFR-LGX-001 行 86 自身が【暫定・要再評価】（L12 は L6 比で層数約 2 倍 → スループット低下見込み）と明記。参考実測: v3 + L6 モデルで 112 ノード / 3.15s ≈ **35.6 nodes/sec**（ISSUE-005 §2.3）であり、既定モデル変更後は 50 nodes/sec は未達の可能性が高い。実装後ベンチマーク（NFR §13 整合）で閾値見直しの人間判断が必要。

---

## §3 用語・前提の補完

| 用語 | 定義・補足 | 根拠 |
|------|-----------|------|
| CTX-INV-1（決定論保証） | 同じ入力に対して常に同じコンテキスト結果を返す。本 SPEC では走査・出力**順序**の決定性のみに適用範囲を限定（推論値のビット再現は対象外） | LEGIXY-SPEC-001 §10（行 225）、ADR-LGX-003 |
| CTX-INV-2（グラフ整合性） | 返される成果物はグラフ定義と矛盾しない | LEGIXY-SPEC-001 §10（行 226） |
| SCORE-INV-1（ハッシュ一致保証） | ノードのハッシュが一致するスコアのみ fresh とする | LEGIXY-SPEC-001 §10（行 244） |
| SCORE-INV-2（モデルバージョン一致） | 現在のモデルバージョンと一致するスコアのみ有効 | LEGIXY-SPEC-001 §10（行 245） |
| MCP-INV-1 | Agent Surface 限定 — MCP は compile_context / observe / get_compile_audit の 3 ツールのみ。embed の MCP 非公開の根拠 | LEGIXY-SPEC-001 §10.4、LGX-EXT-001 §6.1 |
| Mean Pooling | attention_mask を重みとした出力テンソルのトークン方向加重平均 | UC-LGX-007 基本フロー 3-d、v3 embedder.rs:193-209 |
| Contextual Retrieval | サブノードの親文書内での位置づけを示す文脈情報を LLM（Claude API）で生成し embedding 対象テキストの前に付加する手法（Anthropic 提唱） | LGX-EXT-001 §5.8 |
| content_range | h2/h3 見出しから次見出しまでの byte range（サブノードの本文範囲） | LGX-EXT-001 §4.5.1、SPEC-LGX-006.REQ.09 |
| サブノード ID | 自動生成 `{親ID}#{SHA-256 先頭16hex}`（ハッシュ対象は `親ID\|見出し階層パス`、トリム + 連続空白統合の正規化のみ）/ 明示 `{親ID}#s:{英数 1-63 文字}` | LGX-EXT-001 §4.5.1-4.5.3、§4.6 |
| engine.db | プロジェクト内 SQLite。LGX-EXT-001 §4.3 は `.legixy/engine.db` と記すが v3 実装の実体は `.trace-engine/engine.db`（manual.md §6.1）。**ディレクトリ名のリブランド方針（旧名フォールバック要否）は SPEC-LGX-006 のスコープ外だが embed 実装が依存するため、SPEC-LGX-008（マイグレーション）側の決定を確認すること**【要決定（所管確認）】 | LGX-EXT-001 §4.3、v3 manual.md §6.1 |
| fresh / stale / 未生成 | SPEC-LGX-006.REQ.05 で定義済み（embedding 行の有無 × content_hash 一致） | SPEC 本文 |
| 集約 Warning | ペア毎・ノード毎ではなく件数集約で 1 件だけ stderr 報告する Warning（Warning 洪水防止と偽 green 防止の両立） | QSET-LGX-006 Q4 回答（精密化 2026-06-07） |
| QSET / SPP / FCR / TP / GAP / DD / ADR / VAL / TS | DevProc_V4.1 の成果物タイプ（質問書 / SPEC パッチ / 前段クローズ記録 / テスト観点 / ギャップ分析 / 詳細設計 / 設計判断記録 / 妥当性確認 / テスト仕様） | `legixy/docs/DevPorc/02-typecodes.md`（vendored 済み） |
| 前段ループ反復 1 | QSET-LGX-006 発行 → 回答 → SPP-LGX-006 承認（2026-06-07）の 1 巡 | old.p1 frontend-pass/ |
| vnstudio | ドッグフーディング先の別プロジェクト（ISSUE-005 の観測元） | ISSUE-005 ヘッダ |
| ISSUE-005 テンプレ寄与 | 成果物共通テンプレ（Document ID 行・ヘッダ表・変更履歴等）が embedding 空間の共通方向ベクトルとして類似度を底上げする現象 | ISSUE-005 §1.1 |
| standalone `drift` | 運用層の `drift <artifact_id>` コマンド（SPEC-LGX-010.REQ.03 所管）。check 内 Drift Warning（本 SPEC REQ.05）とは別物 | SPEC-LGX-010 REQ.03「check 内 Drift との書き分け」 |
| drift 値 | `1.0 − cosine_similarity`、値域 [0.0, 2.0]。drift_threshold（既定 0.3）と比較 | v3 drift.rs:9-19、設定既定値 |

---

## §4 旧実装からの参考情報

### 4.1 該当 crate と責務対応

| 新名（SPEC-LGX-001.REQ.03 例示） | v3 実装ディレクトリ | 本 SPEC との対応 |
|---|---|---|
| legixy-embed | `crates/lx-embed/` | REQ.01-06.1, 09-12 の主実装（Embedder / EmbeddingStore / bulk API / CR / drift 計算） |
| legixy-db | `crates/lx-db/` | REQ.03/12 の embeddings テーブル DDL・migration |
| legixy-cli | `crates/lx-cli/` | REQ.02 の embed コマンド・モデル解決 |
| legixy-core | `crates/lx-core/` | [semantic] / [contextual_retrieval] 設定の読込・既定値 |
| legixy-graph | `crates/lx-graph/` | Node.content_range / parent_id / anchor / SubnodeKind の供給元 |
| legixy-check | `crates/lx-check/` | REQ.04/05 の消費側（SemanticChecker、SPEC-LGX-004 所管） |

注: LGX-COMPAT-001 §2 はワークスペース構成を `te-*` と記すが、参照可能な実体 `traceability-engine.v3.chg_to_lexigy/` のディレクトリは `lx-*`。SPEC 中の行番号引用（`embed.rs:48-54`、`te-cli/src/main.rs:91-94`、`te-embed/src/similarity.rs:84-86`）は lx-* 側のファイルでほぼ同位置に確認できる（embed.rs:47-49 = モデル解決 anyhow 伝播、similarity.rs:84-86 = 次元不一致無言 skip）。

### 4.2 主要参照ファイル（絶対パス）

- `traceability-engine.v3.chg_to_lexigy/crates/lx-embed/src/embedder.rs` — 生成パイプライン・SHA-256
- `traceability-engine.v3.chg_to_lexigy/crates/lx-embed/src/store.rs` — EmbeddingStore（upsert 1 Tx / load_all 昇順 / snapshot API）
- `traceability-engine.v3.chg_to_lexigy/crates/lx-embed/src/similarity.rs` — bulk API 6 関数 + CheckResult 変換
- `traceability-engine.v3.chg_to_lexigy/crates/lx-embed/src/drift.rs` — cosine / drift 式
- `traceability-engine.v3.chg_to_lexigy/crates/lx-embed/src/orchestrator.rs` — embed_all（部分失敗継続・content_range 切り出し・skip 判定）
- `traceability-engine.v3.chg_to_lexigy/crates/lx-embed/src/contextual.rs` — CR フォールバック・mask_api_key
- `traceability-engine.v3.chg_to_lexigy/crates/lx-embed/src/preprocessor.rs` — 前処理（Phase 1 パススルー）
- `traceability-engine.v3.chg_to_lexigy/crates/lx-db/src/schema.rs` — DDL・migration
- `traceability-engine.v3.chg_to_lexigy/crates/lx-cli/src/commands/embed.rs`、`src/model_dir.rs` — CLI 層
- `traceability-engine.v3.chg_to_lexigy/crates/lx-core/src/config/model.rs`、`config/loader.rs` — 設定既定値
- `traceability-engine.v3.chg_to_lexigy/models/all-MiniLM-L6-v2/` — 旧既定モデル実体
- `legixy.old.p1/models/paraphrase-multilingual-MiniLM-L12-v2/` — **新既定モデル実体（model.onnx + tokenizer.json 配置済み）**
- `traceability-engine.v3.chg_to_lexigy/docs/test-specs/TS-LX-005_embedding生成とドリフト検出.md` — テスト仕様 29 件（§10 に T-EMB-SIM-001〜004）
- `traceability-engine.v3.chg_to_lexigy/deploy/manual.md` — モデル配置（§5.4）・設定スキーマ（§11）・snapshot 容量目安（§snapshot: 384 次元 ×4byte ≈ 1.5KB/ノード）

### 4.3 embedding 関連の既知問題（issues/）

| ISSUE | 内容 | 本 SPEC への含意 |
|---|---|---|
| ISSUE-001 | semantic ID 再定義検出 | サブノード粒度類似度検査（機能 C）の動機。REQ.09 と接続 |
| ISSUE-002 | drift baseline 管理（embedding_snapshots テーブル / snapshot コマンド） | SPEC-LGX-010 所管だが store.rs の snapshot API が同居。model_version 不一致 snapshot との drift は DimensionMismatch Error（manual.md 行 1439） |
| ISSUE-003 | vector_store 設定 stale、BUG-1（include_subnodes 反映漏れ）、BUG-2（ALTER TABLE 順序）、BUG-3（drift の content_range 不整合） | 2.2-a の migration 順序、2.3-e のハッシュ経路統一の根拠 |
| ISSUE-004 | calibrate 推奨閾値（percentile ロジック） | SPEC-LGX-010.REQ.05 の前史。Phase 2 後の再 calibrate 注記必須（§5.2） |
| ISSUE-005 | テンプレ相似ノイズ床 | REQ.09 の直接根拠（§2.3 ベースライン実測） |

### 4.4 v3 差分（SPEC 0.7.0 で新規実装が必要な振る舞い）一覧

SPEC 本文に【v3 差分】と明記されたもの + 本調査で確認した実装差分の集約:

1. トークン上限超過の集約 Warning（v3: 無言切り捨て）
2. 読込時 shape 検証 + exit 1（v3: 検証なし）
3. 空テキスト skip + 集約 Warning（v3: そのまま embed）
4. `--node` / `--force` フラグ（v3: `--all` のみ）
5. `--json` スキーマの `failed` 欄 + errors オブジェクト化（v3: タプル列）
6. 次元不一致・ゼロベクトルの skip + 集約 Warning（v3: 無言 skip / ゼロノルム 0.0 返却）
7. cosine の [-1,1] clamp（v3: なし）
8. detect_drift の未生成 = DriftFinding(kind=missing) 包含（v3: 無言 skip）
9. content_hash の 4 段正規化（v3: 生バイト SHA-256）
10. model_version 複合キー（v3: 設定のモデル名文字列そのまま）
11. content_range 防御的検証で Error 計上（v3: 全文フォールバック）
12. CR Warning の observations 記録（v3: stderr のみ）
13. モデル解決の `LGX_MODELS_DIR` 正準化（v3: `TE_MODELS_DIR` のみ）

---

（以上。本文書は AI 生成の参考資料であり、記載の [要決定] 13 項目は人間の裁定または DD 段階での凍結を要する。）
