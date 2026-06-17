Document ID: TP-LGX-006

# TP-LGX-006: embedding とドリフト検出（観点リスト）

> TP は **テストケース** ではなく **観点リスト**。「仕様文書に問いかける質問のリスト」として書く。

**親**: SPEC-LGX-006
**ステータス**: green
**最終更新**: 2026-06-09

## 1. 対象スコープ

本 TP は SPEC-LGX-006（embedding 生成・格納・ドリフト検出のエンジン責務）の全 REQ に観点をぶつける。

- 対象: SPEC-LGX-006 §3 REQ.01〜REQ.12, §4 不変条件との関係
- 関連 SPEC §:
  - SPEC-LGX-010（embedding 運用・監査）— `drift` / `report` / `calibrate` / `snapshot` の出力仕様・引数・終了コード・閾値推奨ロジックは **SPEC-LGX-010 が正準**（SPEC-006 REQ.11 consumer 注記）。本 TP では当該観点を SPEC-LGX-010 へ委譲（delegate）として GREEN 判定する。
  - SPEC-LGX-004 REQ.02 — check 内の閾値判定（`<` / `≥` の比較方向、severity 付与）は SPEC-004 が正準。
  - NFR-LGX-001 — PERF.08（embedding スループット）, REL.06（トランザクション境界）, SEC.05（API キーマスキング）, OBS.05（終了コード分類）。
  - LGX-COMPAT-001 §3/§4 #4 — `embed` の凍結済み引数契約（`[--all]`、グローバル `--project-root`/`--json`/`--models-dir`）。

**責務境界の整理（GREEN/RED 判定の前提）:**
SPEC-006 は **エンジン**（embedding 生成・格納スキーマ・cosine 計算・drift 検出ロジック・bulk similarity API）に責務を限定する（REQ.11 注記）。`embed` コマンド本体（LGX-COMPAT-001 §4 #4、REQ.02/REQ.10）は SPEC-006 が owner。`drift`/`report`/`calibrate`/`snapshot` の運用コマンドは SPEC-LGX-010 が owner。したがって本 TP の RED は **embed コマンド本体と生成・格納・計算ロジックの未定義箇所** に集中する。運用コマンド側の観点は SPEC-LGX-010 へ委譲する。

## 2. 観点リスト

### 2.1 境界値

- [ ] 観点 B-01: 空テキスト・空白のみのノード本文に対する embedding 生成（mean pooling のトークン列が空/ほぼ空）の挙動と、生成されるベクトルの定義（REQ.01/REQ.02）
- [ ] 観点 B-02: 巨大テキスト（ドキュメントノード本文が ONNX モデルの最大トークン長 128〜512 を超過）の切り捨て/分割/エラーのいずれか（REQ.01/REQ.02）
- [ ] 観点 B-03: 出力次元の動的確定（REQ.01）における異常 shape（0 次元、3 次元以上の予期しない出力 shape、shape 取得失敗）の扱い
- [ ] 観点 B-04: ゼロベクトル（degenerate embedding）または L2 ノルム 0 のベクトル同士の cosine 類似度計算（0 除算・NaN の扱い、REQ.04/REQ.11）
- [ ] 観点 B-05: cosine 類似度の値域境界（完全一致 = 1.0、直交 = 0.0、反対 = -1.0、負値）の定義と返却（REQ.04/REQ.11）
- [ ] 観点 B-06: ノード 0 件・1 件のグラフに対する bulk API（`compute_all_pair_scores` 等の O(N²) でペア数 0）の挙動（REQ.11）

### 2.2 エラーハンドリング

- [ ] 観点 E-01: `embed` 実行時に ONNX モデルが不在/読込失敗（破損・権限）した場合の挙動と終了コード（REQ.01。`drift` のモデル解決は SPEC-010 REQ.03 にあるが `embed` 側は未記述）
- [ ] 観点 E-02: モデル不在時の `--models-dir` / 環境変数 / 設定の解決順序が `embed` で適用されるか（SPEC-010 REQ.03 は `drift` 専用に解決順を規定。`embed` の owner は SPEC-006）
- [ ] 観点 E-03: `embed --all` 途中で 1 ノードの生成に失敗（推論失敗・ファイル読込失敗）した場合の部分失敗トレランスと、REQ.08 の単一トランザクション境界との整合（ドキュメントノードの失敗で全ロールバックか部分コミットか）
- [ ] 観点 E-04: `embed` の終了コード分類（全失敗/部分失敗/正常）の定義（NFR-LGX-001.OBS.05 は汎用、`embed` 固有の分類が SPEC-006 に無い）
- [ ] 観点 E-05: 次元不一致時の集約 Warning（REQ.04【v3 差分】）— GREEN（明示記述あり）。検証は対象。standalone `drift` の Error 維持も明示（SPEC-010 REQ.03 へ委譲）

### 2.3 状態遷移・ライフサイクル

- [ ] 観点 S-01: 一度も embed されていないノード（embedding 行が存在しない）の状態と drift 検出の扱い（REQ.05 は content_hash「変化」のみ規定。未生成 vs 古い の区別）
- [ ] 観点 S-02: content_hash 一致による再計算スキップ（REQ.02）と `--force`（REQ.10）の関係。`--force` 時は content_hash 一致でも再計算するか
- [ ] 観点 S-03: model_version 変化時の全再生成（REQ.10）の遷移期（次元混在期）に対する drift 検出と類似度計算の整合（REQ.04 へ委譲記述あり → 確認）
- [ ] 観点 S-04: `embed` の対象指定（個別ノード ID、REQ.02）と `--all` / `--force` の組合せ（個別 ID + `--force`、個別 ID 指定時の content_hash スキップ可否）

### 2.4 並行性

- [ ] 観点 C-01: 2 つの `embed` プロセスの同時実行、または `embed`（書込）と `check`/`report`（読取）の並行時の engine.db ロック挙動（REQ.08 は単一トランザクションのみ。busy_timeout=NFR REL.07 との連携が SPEC-006 に無い）
- [ ] 観点 C-02: `embed` のトランザクション中にプロセス kill / 電源断が起きた場合の部分不整合の非発生（REQ.08）— GREEN（明示）。検証対象
- [ ] 観点 C-03: Contextual Retrieval の LLM API 呼出（REQ.06）が embedding 生成と並行/直列のいずれか、複数ノードの context 合成を並行実行する場合のレート制限・順序保証

### 2.5 永続化

- [ ] 観点 P-01: embedding データの必須情報（REQ.03）が全て取得可能か — GREEN（必須項目列挙あり、具体スキーマは DD）。検証対象
- [ ] 観点 P-02: サブノード embedding の格納項目（parent_id / anchor / is_subnode、REQ.12）と `(parent_id, anchor)` INDEX — GREEN（明示）。検証対象
- [ ] 観点 P-03: content_hash の算出対象の正規化（CRLF/LF 改行差、BOM、UTF-8 正規化）。同一論理内容が改行コード差で別ハッシュになると SCORE-INV-1 freshness が偽 stale/偽 fresh になる（REQ.03/REQ.05、NFR COMPAT.07/08）
- [ ] 観点 P-04: 同一 content_hash を持つ別ノード（ハッシュ衝突は SHA-256 で実質起きないが、同一テキストの複数ノード）に対する格納とノード識別の一意性（REQ.03 ノード識別情報の一意性）
- [ ] 観点 P-05: ディスクフル/書込権限エラー時の `embed` 格納失敗の扱い（REQ.08 トランザクション境界の外側のエラー）

### 2.5b バージョニング・互換性

- [ ] 観点 V-01: `model_version` 識別子の生成方式と「変化した」の判定基準（REQ.03/REQ.10、SCORE-INV-2）。モデルファイルのハッシュか、設定文字列か、何が変わると再生成トリガになるか
- [ ] 観点 V-02: 768 次元等の代替モデル（multilingual-mpnet / e5-base、REQ.01）への切替時、既存 384 次元 embedding との混在期の取扱い（REQ.10 全再生成 + REQ.04 skip へ委譲記述 → 確認）
- [ ] 観点 V-03: e5 系モデル採用時の `query:`/`passage:` プレフィックス前処理（REQ.01 で「別 ADR で判断」と委譲）— GREEN（明示的に範囲外＋委譲先記述）
- [ ] 観点 V-04: v0.1.0 既存 embedding（旧モデル all-MiniLM-L6-v2、英語中心）の読込時挙動とドロップイン置換（REQ.01）。旧 embedding の model_version 不一致による再生成（REQ.10 へ繋がるか）

### 2.6 入力検証

- [ ] 観点 I-01: ノード本文に含まれる制御文字・サロゲートペア・正規化前 Unicode・絵文字（ZWJ/異体字）・BiDi 制御の embedding 入力としての扱い（REQ.01 多言語対応、NFR COMPAT.07）
- [ ] 観点 I-02: サブノード content_range（REQ.09）が不正（range.0 > range.1、範囲外、空 range）な場合の入力検証

### 2.7 ロギング・観測性

- [ ] 観点 L-01: Contextual Retrieval フォールバック時の Warning が stderr + observations テーブルに記録（REQ.06.1）— GREEN（明示）。検証対象
- [ ] 観点 L-02: API キーがログ・DB・エラー情報（スタックトレース/HTTP レスポンス）に混入しないマスキング（REQ.07、NFR SEC.05）— GREEN（明示）。検証対象
- [ ] 観点 L-03: `embed --all` の長時間処理に対する進捗表示（NFR USE.03/PERF.08）が SPEC-006 で要求/委譲されているか
- [ ] 観点 L-04: 次元不一致 skip の集約 Warning の出力先・文言（REQ.04 は「集約 Warning 1 件 + embed --all 誘導」と記述）— GREEN（明示）。出力先 stderr は SPEC-010 REQ.01 へ委譲

### 2.8 FFI / 境界 API（embed CLI 引数 — LGX-COMPAT-001）

- [ ] 観点 F-01: `embed` の凍結済み引数（`[--all]`、グローバル `--project-root`/`--json`/`--models-dir`）の受理（LGX-COMPAT-001 §4 #4 / §3）— GREEN（compat 契約で凍結）。`embed --all --force`（REQ.10）の `--force` が compat 表に無い点の整合
- [ ] 観点 F-02: `embed` の `--json` 出力スキーマ（生成件数・スキップ件数・失敗件数）の定義（グローバル `--json` は全コマンド受理 = LGX-COMPAT-001 §3。`embed` 固有スキーマが SPEC-006 に無い）
- [ ] 観点 F-03: REQ.02 の「個別ノード ID 指定オプション」のフラグ名・形式（LGX-COMPAT-001 §4 #4 は `[--all]` のみ列挙。個別指定オプションの引数契約が未確定）

### 2.9 領域固有観点（ML / embedding determinism）

- [ ] 観点 D-01: 浮動小数点 ONNX 推論の決定性 — 同一入力・同一モデルで生成されるベクトル本体が再現可能か（CPU SIMD・スレッド数・BLAS 実装差による非決定性）。CTX-INV-1（決定論保証）・SCORE-INV-1 との整合（REQ.04/REQ.11 は「順序の決定性」を保証するが「ベクトル値の再現性」は未記述）
- [ ] 観点 D-02: bulk API の返却順序の決定性（REQ.11 node_id 昇順ロード、SCORE-INV-1）— GREEN（明示）。検証対象
- [ ] 観点 D-03: Contextual Retrieval 有効時（REQ.06）の context が LLM 合成のため非決定的。context_hash の freshness 判定（REQ.03 の context_hash）と、同一ノード再 embed 時に context が変わると content_hash 不変でも embedding が変わる問題。CTX-INV-1 / SCORE-INV-1 との整合
- [ ] 観点 D-04: mean pooling + L2 正規化の固定（REQ.01）— GREEN（明示、モデル切替でも不変）。検証対象
- [ ] 観点 D-05: 多言語混在テキスト（日英混在 1 文、CJK + Latin）の embedding 妥当性（REQ.01 検証方法に「日本語・英語サンプル文の類似度が妥当な値域」とあり）— GREEN（検証方法に明示）。値域の具体は閾値 → NFR/SPEC-010 へ委譲

## 3. RED / GREEN 判定

| 観点 | 判定 | SPEC / 委譲先で回答 | 関連 GAP |
|---|---|---|---|
| B-01 空テキスト embedding | GREEN | (該当なし) ※敵対的精査: WEAK_OR_PADDED、人間判断で drop 可【2026-06-10 解消: weak GAP fix 適用（人間裁定）により GAP closed】 | GAP-LGX-101 |
| B-02 巨大テキスト切り捨て | GREEN | (該当なし) ※敵対的精査: DD/トークナイザ層、WEAK_OR_PADDED【2026-06-10 解消: weak GAP fix 適用（人間裁定）により GAP closed】 | GAP-LGX-102 |
| B-03 異常 output shape | GREEN | (該当なし) ※敵対的精査: ONNX ランタイム境界 = DD、WEAK_OR_PADDED【2026-06-10 解消: weak GAP fix 適用（人間裁定）により GAP closed】 | GAP-LGX-103 |
| B-04 ゼロベクトル cosine 0除算 | GREEN | (該当なし) ※敵対的精査: GENUINE（数値正当性 + D-01 値再現性）【2026-06-10 解消: SPEC 改訂適用（人間承認）により GAP closed】 | GAP-LGX-104 |
| B-05 cosine 値域境界 | GREEN | REQ.04 は「cosine で計算」のみ。値域定義なし ※敵対的精査: cosine [-1,1] は数学的性質、SPEC-010 REQ.05 が clamp 済み、WEAK_OR_PADDED【2026-06-10 解消: weak GAP fix 適用（人間裁定）により GAP closed】 | GAP-LGX-105 |
| B-06 ノード 0/1 件の bulk API | GREEN | REQ.11（O(N²) ペア抽出）+ SPEC-010 REQ.04/05 空ストア委譲 | — |
| E-01 embed モデル不在/読込失敗 | GREEN | (該当なし。drift は SPEC-010 REQ.03) ※敵対的精査: 終了コードは NFR OBS.05 + SPEC-010 REQ.03 前例で類推可、WEAK_OR_PADDED【2026-06-10 解消: weak GAP fix 適用（人間裁定）により GAP closed】 | GAP-LGX-106 |
| E-02 embed モデル解決順序 | GREEN | SPEC-LGX-010 REQ.03（`--models-dir` ＞ `LGX_MODELS_DIR` ＞ `TE_MODELS_DIR` ＞ 設定）+ LGX-COMPAT-001 §3（`--models-dir` 全コマンド共通）。embed の解決順序は同一基盤を共有。GAP-LGX-106 と同根のため DUPLICATE として GAP-LGX-107 削除 | — |
| E-03 embed 部分失敗 vs Tx 境界 | GREEN | REQ.08（Tx 境界）/ REQ.09（サブノード部分失敗）が矛盾しうる ※敵対的精査: GENUINE（SPEC-006 内の真の内部矛盾）【2026-06-10 解消: SPEC 改訂適用（人間承認）により GAP closed】 | GAP-LGX-108 |
| E-04 embed 終了コード分類 | GREEN | NFR-LGX-001.USE.04（「Error 1件以上 → 終了コード 1」）+ OBS.05（0=OK/1=Error/2=使用法誤り）が部分失敗 = exit 1 を直接回答。ALREADY_ANSWERED として GAP-LGX-109 削除 | — |
| E-05 次元不一致集約 Warning | GREEN | REQ.04【v3 差分】 | — |
| S-01 未生成ノードの drift | GREEN | REQ.05 は content_hash「変化」のみ ※敵対的精査: SCORE-INV-1 で行不在は自明に non-fresh、detect_drift 戻り型は DD、WEAK_OR_PADDED【2026-06-10 解消: weak GAP fix 適用（人間裁定）により GAP closed】 | GAP-LGX-110 |
| S-02 content_hash skip × --force | GREEN | REQ.02（hash 一致 skip）+ REQ.10（--force 強制再生成） | — |
| S-03 model_version 遷移期 | GREEN | REQ.10 + REQ.04 委譲（次元混在 skip） | — |
| S-04 個別ID × --all × --force | GREEN | embed の引数契約は GAP-LGX-120 に統合（F-03 と同根）。LGX-COMPAT-001 §4 #4 が `embed [--all]` のみ凍結する点の整合は GAP-LGX-120 が扱う。DUPLICATE として GAP-LGX-111 削除 | — |
| C-01 並行 embed / 読書ロック | GREEN | NFR-LGX-001.REL.07（busy_timeout 上限 5000 ms、超過時 Error）+ SEC.02（WAL + busy_timeout で同時書込破損なし）が並行制御を所有。GAP 本文も「NFR REL.07/SEC.02 への委譲で解決しうる」と自認。ALREADY_ANSWERED として GAP-LGX-112 削除 | — |
| C-02 Tx 中の kill/電源断 | GREEN | REQ.08（部分不整合非発生）, NFR REL.06 | — |
| C-03 Contextual Retrieval 並行/レート | GREEN | REQ.06/06.1 はリトライ/タイムアウトのみ。並行・順序未記述 ※敵対的精査: Contextual Retrieval は REQ.06 で既定無効、非決定性は有効時のみ顕在、WEAK_OR_PADDED【2026-06-10 解消: weak GAP fix 適用（人間裁定）により GAP closed】 | GAP-LGX-113 |
| P-01 必須情報の取得可能性 | GREEN | REQ.03（7 項目列挙、スキーマは DD） | — |
| P-02 サブノード格納項目 + INDEX | GREEN | REQ.12 | — |
| P-03 content_hash 正規化 | GREEN | REQ.03/05 は SHA-256 のみ。改行/BOM/正規化未定義 ※敵対的精査: GENUINE（SCORE-INV-1 の決定性が正規化に依存、NFR COMPAT.07/08 は IO 層のみ）【2026-06-10 解消: SPEC 改訂適用（人間承認）により GAP closed】 | GAP-LGX-114 |
| P-04 同一テキスト複数ノードの一意性 | GREEN | REQ.03（ノード識別情報の一意性） | — |
| P-05 ディスクフル/権限 | GREEN | NFR SEC.01（権限）, REL.01（破損耐性）へ委譲 | — |
| V-01 model_version 生成方式/変化判定 | GREEN | REQ.03/10 は「識別子」「変化」とのみ。生成方式・判定基準なし ※敵対的精査: GENUINE（SCORE-INV-2 判定の核心、REQ.10 再生成トリガの前提）【2026-06-10 解消: SPEC 改訂適用（人間承認）により GAP closed】 | GAP-LGX-115 |
| V-02 768次元等への切替混在期 | GREEN | REQ.10（全再生成）+ REQ.04（skip）+ REQ.01（動的次元） | — |
| V-03 e5 プレフィックス前処理 | GREEN | REQ.01（範囲外、別 ADR 委譲を明示） | — |
| V-04 v0.1.0 旧 embedding 読込 | GREEN | model_version による旧/新区別は GAP-LGX-115 の派生（両者 384 次元のため model_version でしか区別不可）。移行は UC-LGX-009 / NFR COMPAT.04 が所有。DUPLICATE（115 派生）+ OUT_OF_SCOPE として GAP-LGX-116 削除 | — |
| I-01 Unicode/制御文字/絵文字入力 | GREEN | content_hash 正規化（NFC 等）と同一の正規化方針を共有すべきで GAP 本文も「GAP-LGX-114 と同一」と自認。トークナイザの絵文字処理は DD。DUPLICATE として GAP-LGX-117 削除（114 が keeper） | — |
| I-02 content_range 不正値検証 | GREEN | REQ.09/12 は range 使用のみ。不正 range の検証なし ※敵対的精査: content_range 生成は LGX-EXT-001 §4.5.1 の責務、UTF-8 panic 防止は NFR SEC.03/04 が既にカバー、WEAK_OR_PADDED【2026-06-10 解消: weak GAP fix 適用（人間裁定）により GAP closed】 | GAP-LGX-118 |
| L-01 Contextual Retrieval Warning 記録 | GREEN | REQ.06.1（stderr + observations） | — |
| L-02 API キーマスキング | GREEN | REQ.07, NFR SEC.05 | — |
| L-03 embed 進捗表示 | GREEN | NFR-LGX-001.USE.03 が「長時間処理（embed --all 等）で進捗表示」と embed --all を名指しで所有。GAP 本文も「重要度は低」「NFR USE.03 への委譲で解決しうる軽微」と自認。ALREADY_ANSWERED として GAP-LGX-119 削除 | — |
| L-04 skip 集約 Warning 出力先/文言 | GREEN | REQ.04 + SPEC-010 REQ.01（stderr 委譲） | — |
| F-01 embed 凍結引数受理 | GREEN | LGX-COMPAT-001 §4 #4 / §3 | — |
| F-02 embed --json スキーマ | GREEN | グローバル --json は受理（§3）だが embed 固有スキーマ未定義 ※敵対的精査: GENUINE（embed の引数/出力契約が SPEC-006 と LGX-COMPAT-001 §4 #4〔`[--all]` のみ凍結〕の間で不完結。`--force`/個別 ID が凍結契約外 = ハードルール 7 抵触）【2026-06-10 解消: SPEC 改訂適用（人間承認）により GAP closed】 | GAP-LGX-120 |
| F-03 個別ノード指定の引数契約 | GREEN | REQ.02「個別ノード ID 指定オプション」+ F-02 と同根。GAP-LGX-120 に統合（S-04/GAP-111 もここへ集約）【2026-06-10 解消: SPEC 改訂適用（人間承認）により GAP closed】 | GAP-LGX-120 |
| D-01 浮動小数点推論の値再現性 | GREEN | REQ.04/11 は順序のみ。ベクトル値の再現性未記述。CTX-INV-1 影響。GAP-LGX-104 に統合（§2.9 注記） ※敵対的精査: GENUINE（GAP-LGX-104 の一部として維持）【2026-06-10 解消: SPEC 改訂適用（人間承認）により GAP closed】 | GAP-LGX-104 |
| D-02 bulk API 返却順序決定性 | GREEN | REQ.11（node_id 昇順、SCORE-INV-1） | — |
| D-03 Contextual Retrieval 非決定性 vs freshness | GREEN | REQ.03 context_hash / REQ.06 は決定性影響を未記述【2026-06-10 解消: weak GAP fix 適用（人間裁定）により GAP closed】 | GAP-LGX-113 と関連、独立計上下記 |
| D-04 mean pooling + L2 固定 | GREEN | REQ.01 | — |
| D-05 多言語混在の妥当性 | GREEN | REQ.01 検証方法 | — |

> **判定上の注記（D-01 / D-03 の GAP 割当）:** D-01（浮動小数点推論の値再現性）は B-04/B-05 とは別根の決定性問題のため独立 GAP を起票せず、最重要の **GAP-LGX-104（ゼロベクトル/数値安定性）** に含めるとスコープが混ざるため、本 TP では D-01 を独立観点として残し GAP は起票しない（CTX-INV-1 の値再現性は実装層の決定性テストで担保される性質であり、SPEC レベルでは「決定性を保証する」宣言の有無のみが問題。SPEC-006 §4 は CTX-INV-1 を「関与しない不変条件」に挙げず REQ.08 経由で「関連」とするが値再現性は未言及）。**→ この曖昧さ自体を GAP-LGX-104 の影響範囲に追記し、D-01 を GAP-LGX-104 に紐付ける。** D-03 は GAP-LGX-113（Contextual Retrieval 並行/順序）と同じ REQ.06 系の決定性問題のため GAP-LGX-113 に統合する。

（上記注記に従い、判定表の D-01 関連 GAP は GAP-LGX-104、D-03 は GAP-LGX-113 とする。）

## 4. ステータスの決定

敵対的精査パス（2026-06-09）後も RED 観点が 13 件残るため、本 TP のステータスは `**ステータス**: green`。

> 2026-06-10 追記（weak GAP fix 適用後）: 残存していた weak/minor GAP も SPEC 改訂（人間裁定 fix・承認 2026-06-10）で全件 closed。全観点 GREEN のためステータスを green に更新。

> 2026-06-10 追記: GENUINE GAP は SPEC 改訂（人間承認 2026-06-10）で全件 closed（本 TP の該当観点を GREEN 化）。残る RED は weak/minor（人間判断で drop 可）のみであり、weak 裁定が完了するまでステータスは red を維持する。
残存 GAP（GAP-LGX-101/102/103/104/105/106/108/110/113/114/115/118/120）が close されたら GREEN に再評価する。

**集計（敵対的精査パス 2026-06-09 反映）:**
- 総観点数: 40
- GREEN: 27（初版 20 + 精査で GREEN 化 7: E-02/E-04/S-04/C-01/V-04/I-01/L-03）
- RED: 13（残存 GAP 13 件）
  - GENUINE（irrefutable）: 5 — GAP-LGX-104（ゼロベクトル/D-01 値再現性）, 108（Tx 境界 vs 部分失敗矛盾）, 114（content_hash 正規化）, 115（model_version 生成/変化判定）, 120（embed 引数/--json 契約 vs 凍結境界）
  - minor / WEAK_OR_PADDED（人間判断で drop 可）: 8 — GAP-LGX-101, 102, 103, 105, 106, 110, 113, 118
- 削除（敵対的精査で REFUTE）: 7 — GAP-LGX-107（→106 DUPLICATE）, 109（NFR USE.04/OBS.05 ALREADY_ANSWERED）, 111（→120 DUPLICATE）, 112（NFR REL.07/SEC.02 ALREADY_ANSWERED）, 116（→115 DUPLICATE + UC-009/COMPAT.04 OUT_OF_SCOPE）, 117（→114 DUPLICATE）, 119（NFR USE.03 ALREADY_ANSWERED）

## 5. 観点ナレッジベース参照

この TP の生成時に参照したナレッジベース章節:

- `docs/perspectives/core-perspectives.md` §境界値, §エラーハンドリング, §状態遷移, §並行性, §バージョニング・互換性, §永続化, §入力検証, §ライフサイクル, §ロギング・観測性, §FFI/境界 API
- `docs/perspectives/ux-perspectives.md` §待機/進捗の UX（L-03 進捗表示）, §エラー・例外の UX（E-04 バルク一部失敗表示）
- 領域固有（ML/embedding determinism）: 浮動小数点推論の非決定性、モデルバージョン migration、ゼロベクトル数値安定性、Unicode 正規化（本 TP で新規追加。AT 経由で core-perspectives.md へ昇格候補）

## 6. 履歴

| 日付 | 変更内容 |
|---|---|
| 2026-06-08 | 初版作成。観点 40 件（GREEN 20 / RED 20）。GAP-LGX-101〜120 を起票。運用コマンド（drift/report/calibrate/snapshot）観点は SPEC-LGX-010 へ委譲 |
| 2026-06-09 | 敵対的精査パス: 削除 7 件 / 維持 13 件 |
| 2026-06-10 | SPEC 改訂適用（人間承認 2026-06-10、spec-change-proposals/2026-06-09_genuine-gap-resolution-proposals.md）: GENUINE GAP に対応する観点を GREEN 化。GAP-157 は人間裁定・案A、GAP-064 は GraphDag 新設 + DocumentId 行欠落 Error、GAP-120 は凍結契約への加算的拡張承認。ADR-LGX-001〜008 起票 |
| 2026-06-10 | weak GAP 解消適用（人間裁定 fix・承認 2026-06-10、spec-change-proposals/2026-06-10_weak-gap-resolution-proposals.md）: 残存 RED 観点（weak/minor）を全て GREEN 化。個別裁定: GAP-085=打ち切り Info 追加 / GAP-135=永続保持 / GAP-169=タイムアウト導入【v3 差分】。ADR-LGX-009〜011 起票。open GAP 0 となり本 TP は green |
