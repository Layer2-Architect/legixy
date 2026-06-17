Document ID: SPEC-LGX-004

# SPEC-LGX-004: 検証

| 項目 | 内容 |
|------|------|
| Document ID | SPEC-LGX-004 |
| Version | 0.8.0 |
| Status | Approved（人間査読済） |
| Date | 2026-04-28 |
| Classification | CONFIDENTIAL |
| 親文書 | SPEC-LGX-001, LGX-EXT-001 §7 |
| 対応 NFR | NFR-LGX-001.PERF.02, OBS.05, OBS.06, REL.02, REL.03 |
| 対応 UC | UC-LGX-001 |

---

## 1. 本文書の位置づけ

### 1.1 目的

`legixy check` および `check --formal` コマンドの動作要求を legixy 向けに明文化する。v0.1.0 検証機能を継承しつつ、サブノード不変条件 SUBNODE-INV-1〜6 の強制を追加する。

### 1.2 スコープ

**含む:** check コマンドの動作、検証カテゴリ、終了コード、severity
**含まない:** embedding 生成（→ SPEC-LGX-006）、意味的類似度の閾値詳細（→ NFR-LGX-001, SPEC-LGX-006）

---

## 2. 参照文書

- LGX-EXT-001 §7 不変条件への影響と追加
- LEGIXY-SPEC-001 §10 不変条件
- 前世代 `old.source/` の check 実装を慣例仕様として参照

---

## 3. 要求事項

### SPEC-LGX-004.REQ.01: check --formal（形式検証）

**内容:** `legixy check --formal` は ONNX モデルを必要とせず、以下のカテゴリを検証する（severity は REQ.15 の割当表が正準）:
- FileExistence: graph.toml で参照されるファイルの存在【Error】
- DocumentId: ファイル内の `Document ID:` 行と graph.toml の ID の一致（不一致【Error】/ 行欠落【Error】）
- ChainIntegrity: chain エッジの整合性【Error】
- OrphanFile: graph.toml に登録されていないファイル【Error】
- **GraphDag: グラフ全体（Chain/Custom/ParentChild 全エッジ種別）の DAG 制約（CTX-INV-4）【Error】**（GAP-LGX-064 対応で新設。【v3 差分】 — v3 はグラフ全体サイクルも `SubnodeDag` カテゴリ名で報告していた〔`crates/te-graph/src/validation.rs:52-65`〕。legixy では適用範囲を正確に表すカテゴリ名 `GraphDag` に分離する）
- Freshness: ファイル更新時刻のドリフト（任意）【Warning】
- SubnodeIdFormat: サブノード ID フォーマット（SUBNODE-INV-6）【Error】
- SubnodeIdUniqueness: サブノード ID 一意性（SUBNODE-INV-3）【Error】
- SubnodeParentIntegrity: サブノード親存在（SUBNODE-INV-1）【Error】
- SubnodePathConsistency: サブノードパス整合性（SUBNODE-INV-2）【Error】
- SubnodeDag: サブノード関与エッジの DAG 制約（SUBNODE-INV-4）【Error】

**空グラフ時の挙動（GAP-LGX-061 対応）:** graph.toml にノードが 1 件も存在しない（または空の）場合、check / check --formal は **finding 0 件・exit 0** で正常終了し、**stderr に Info 1 件**（graph 未構築・成果物未登録の誘導）を出力する【v3 差分: v3 は無言】。graph.toml が物理的に存在しない場合は未初期化（init 誘導）として別経路で扱う。存在するが空のグラフは使用法誤りではない（exit 2 にしない）。

**根拠:** LGX-EXT-001 §7.1、GAP-LGX-064（severity 割当の明示、人間裁定 2026-06-10: GraphDag 新設 + DocumentId 行欠落 Error〔v3 実挙動と一致、`crates/te-check/src/document_id.rs:81`〕）、GAP-LGX-061
**検証方法:** TS-LGX-001 T-VL-001〜007、REQ.15 割当表との突合テスト、空グラフ fixture（finding 0・exit 0・stderr Info）テスト

### SPEC-LGX-004.REQ.02: check（全層検証）

**内容:** `legixy check`（`--formal` なし）は形式検証に加え、以下の意味的検証を実施する:
- SemanticSimilarity: graph.toml のチェーン/カスタム/親子エッジで類似度 < `similarity_threshold` のペアを Warning として報告
- LinkCandidate: 非エッジペアで類似度 ≥ `link_candidate_threshold` のペアを Info として報告（v2 互換、リンク漏れ候補）
- Drift: 全成果物に対し、ファイル内容 content_hash と embeddings テーブルの保存済 hash を比較し、不一致を Warning として報告（ファイル不在も Warning）

embeddings テーブルが空の場合は Info 1 件（「embed --all 未実行」誘導）のみ返却し exit code に影響しない（非致命扱い）。ONNX モデルが必要。

**実装:** legixy-check の `SemanticChecker` が legixy-embed の bulk similarity API（SPEC-LGX-006.REQ.11）を呼び出して検出する（crate 名は例示であり DD で凍結、SPEC-LGX-001.REQ.03）。

**standalone コマンドとの責務境界（前段ループ反復 1 で確定）:** `report`（計測レポート）・standalone `drift`（ベースライン対比）・`snapshot`・`calibrate` は **SPEC-LGX-010（embedding 運用・監査）** が規定する。本 REQ の SemanticSimilarity / LinkCandidate / Drift は check 内の**判定**（閾値判定の結果のみを severity 付き findings として報告し、生スコア一覧は出力しない）であり、SPEC-LGX-010 の**計測**（判定なしの生スコア + 候補 + 統計の出力）と出力責務は重複しない。また本 REQ の Drift（content_hash 比較の Warning）と standalone `drift`（embedding 対比の定量値）は**同名だが別機能**である。
**根拠:** v0.1.0 check 動作, SPEC-LGX-006, workflow_2026-04-20_semantic-check-and-reporting.md §2
**検証方法:** TS-LGX-003 §12 T-SEM-001〜006

### SPEC-LGX-004.REQ.03: severity 4 段階

**内容:** CheckResult は以下 4 段階の severity を持つ:
- `Error`: 必ず修正が必要
- `Warning`: 確認・対処推奨
- `Info`: 参考情報
- `Ok`: 問題なし

**Ok の使用条件（GAP-LGX-065 対応）:** `Ok` は**カテゴリ finding としては発行しない**（v3 実測: te-check の全検査器に Severity::Ok の producer は存在せず、text 出力でも skip される〔reporter.rs:62〕）。全検証 pass の正準表現は **findings 0 件 + 全 counts（error/warning/info）0 + exit 0** である。`Ok` は CheckReport 集計（ok_count）と将来拡張のための**予約 severity** であり、JSON 出力にも Ok finding は現れない。

**根拠:** CLAUDE.md「検証結果への対応方針」、GAP-LGX-065
**検証方法:** TS-LGX-001 T-VL-* の severity 検証、全 pass fixture で findings 空配列を確認するテスト

### SPEC-LGX-004.REQ.04: 終了コード

**内容:** `check` および `check --formal` は以下の終了コードを返す:
- `0`: Error 件数が 0
- `1`: Error 件数が 1 以上
- `2`: 使用法誤り（引数パーサ層〔clap〕が検出する構文レベルの誤り: 未知フラグ・必須引数欠落・型不正等）

exit 2 は check 固有ではなく**全サブコマンド共通の規約**であり、LGX-COMPAT-001 v1.0.1 でグローバル規約として契約化する（本 SPP 付随変更）。受理済みフラグ・引数の値の意味的不正は exit 1（実行エラー）に分類する。

**根拠:** NFR-LGX-001.OBS.05、QSET-LGX-004 Q1 回答（2026-06-07。v3 実測: check 本体は 0/1〔`crates/te-check/src/reporter.rs:50-57`〕+ clap 既定の exit 2 が全コマンドに存在）
**検証方法:** E2E テスト（0/1/2 の 3 値）

### SPEC-LGX-004.REQ.05: 部分失敗時の継続

**内容:** 検証中に一部のファイル読み込みに失敗しても、他のチェックは継続する。失敗は Error として CheckReport に記録される。
**根拠:** NFR-LGX-001.REL.02
**検証方法:** 破損ファイル混在時のテスト

### SPEC-LGX-004.REQ.06: 冪等性

**内容:** 同一入力に対して `check --formal` は常に同一の CheckReport を返す（結果順序含む）。

**全層 check への拡張（GAP-LGX-072 対応）:** 意味検証を含む全層 `check` も冪等性・結果順序保証の対象とする:
- finding の出力順は**安定ソートキー**（severity rank 降順 → category → related_ids。詳細キーは DD で凍結）で決定論的とする
- **類似度スコアの値はビット単位再現の対象外**（ADR-LGX-003 — 決定論保証は走査・出力「順序」のみ。同一環境では実用上再現され、環境差の微小値は drift_threshold が吸収する）
- golden file 比較・CI 差分検知は順序・件数・severity・category で行い、生スコアの厳密一致を期待値にしない

**根拠:** NFR-LGX-001.REL.03、GAP-LGX-072、ADR-LGX-003
**検証方法:** 反復実行テスト（formal は厳密一致、全層は同一環境での反復一致 + 順序安定性）

### SPEC-LGX-004.REQ.07: サブノード不変条件の強制

**内容:** 以下の不変条件違反は必ず Error severity として検出される:
- SUBNODE-INV-1 親存在
- SUBNODE-INV-2 パス整合性
- SUBNODE-INV-3 ID 一意性
- SUBNODE-INV-4 DAG 制約
- SUBNODE-INV-6 ID フォーマット

SUBNODE-INV-5（ID 決定性）は生成ロジック側で保証される（TS でのみ検証）。
**根拠:** LGX-EXT-001 §7.1
**検証方法:** TS-LGX-001 T-VL-002〜007

### SPEC-LGX-004.REQ.08: CheckReport の出力

**内容:** CheckReport は標準出力に出力される。ログは stderr に出力される（stdout のパイプ可能）。`--log-format=json` で JSON Lines 出力対応。
**根拠:** NFR-LGX-001.OBS.02, OBS.03
**検証方法:** 出力形式テスト

### SPEC-LGX-004.REQ.09: freshness 検出

**内容:** v0.1.0 同様、mtime または git 履歴に基づき上流ファイルより古い下流ファイルを Warning として検出する。method は `.legixy.toml` で設定。
**根拠:** v0.1.0 継承
**検証方法:** freshness シナリオテスト

### SPEC-LGX-004.REQ.10: 未解決エッジの Warning 報告（CTX-INV-5 検証）

**内容:** `check` および `check --formal` は、SPEC-LGX-002.REQ.11 で記録された `unresolved_edges`（参照先ノードが存在しないエッジ）を以下の形式で報告する:
- severity: **Warning**（Error ではない）
- category: 新規追加 `UnresolvedEdge`（または既存 `ChainIntegrity` の extension）
- message: エッジの from/to と、どちらのノードが存在しないかを明示
- related_ids: 未解決エッジに関わる ID

未解決エッジが存在しても G1 ゲート（Error=0）は通過する。運用中の一時的参照切れを許容するための設計。
**根拠:** LEGIXY-SPEC-001 §10 CTX-INV-5、VAL-LGX-001 Finding E-01
**検証方法:** 存在しないノード ID を持つエッジを含む graph.toml の check で Warning が報告されるテスト

### SPEC-LGX-004.REQ.11: ID Changelog 宣言検出（IdRedefined）

**内容:** SPEC ファイル内の `## ID Changelog` セクション（Markdown 表形式）または `.legixy.toml` の `[[id_changelog]]` 配列に基づき、`change = "redefined"` 宣言された ID を引用している全下流成果物を列挙し、Warning として報告する。

- **入力ソース**: 以下のいずれか／両方を `.legixy.toml` の `[id_changelog].source` で指定（`spec_header` | `toml_config` | `both`）
  - SPEC ヘッダ: ファイル先頭から `## ID Changelog` 見出しの直後にある Markdown 表（列: `Date | ID | Change | Note`）
  - TOML config: `[[id_changelog]]` 配列要素（フィールド: `spec, date, id, change, note`）
- **検出対象**: `change = "redefined"` のエントリのみ（`new` / `removed` 等は将来拡張）
- **引用走査**: 各再定義 ID について、graph 上 chain 下流のドキュメントノード本体ファイルを行スキャン。デフォルト citation_pattern は `\|\s*{ID}\s*\|`（表行内の単独出現）
- **severity**: Warning
- **category**: `IdRedefined`（新規追加）
- **出力**: 引用箇所をファイルパス + 行番号 + 引用行（先頭 N 文字）で列挙
- **デフォルト**: `[id_changelog].enabled = false`。明示的オプトインで動作（後方互換性のため）
- **G1 ゲート**: 本検査は Warning のみ発行し Error にならないため G1 を阻害しない

**根拠:** ISSUE-001（vnstudio で発生した SPEC §5 NFR 増補に伴う 31 件のセマンティックドリフト未検出問題）
**検証方法:** TS-LGX-003 §12 T-IC-001〜006

### SPEC-LGX-004.REQ.12: ID 引用整合性検査（IdSemanticMismatch）

**内容:** SPEC で定義された ID の **定義文** と、下流成果物の **引用文** に含まれる **数値リテラル + キーワード** を機械的に照合し、不整合を Warning（または Info、設定可）として報告する。REQ.11 の Changelog 宣言漏れに対するセーフティネット。

- **数値抽出**: `\b(\d+(?:\.\d+)?)\s*(ms|秒|分|時間|byte|MB|GB)\b` + 比較演算子（以内 / 以上 / 未満）を抽出
- **単位正規化**: `unit_normalization = true` の場合、ms ↔ 秒 ↔ 分を正規化して比較（200ms = 0.2 秒として一致扱い）
- **キーワード Set 比較**: `[id.semantic_mismatch].keywords` で project-specific 辞書を定義可。デフォルトは空（全 ID で false-positive 抑制のため明示的辞書のみ動作）
- **判定**: SPEC 定義表の `| ID | カテゴリ | 目標値 | ... |` 行から数値・キーワードを抽出し、下流 chain で同一 ID を引用する行と比較。差分があれば Warning（または Info）
- **severity**: 既定 `Info`（false-positive 多発を避けるため）。`severity = "warning"` で昇格可
- **category**: `IdSemanticMismatch`（新規追加）
- **デフォルト**: `[id_semantic_mismatch].enabled = false`
- **G1 ゲート**: Error にならないため G1 を阻害しない

**根拠:** ISSUE-001 §2.2（Changelog 宣言忘れに対する補完防衛）
**検証方法:** TS-LGX-003 §13 T-ISM-001〜005

### SPEC-LGX-004.REQ.13: サブノード単位意味類似度検査（IdSemanticDrift、Phase 2 Block F）

**内容:** SPEC ノードのサブノード（定義側）と、下流（UC/RB/SEQ）のサブノード（引用側）について、**同一 ID を引用しているペア** の embedding 類似度を計算し、閾値を下回る場合に Warning として報告する。Phase 2 Block A（サブノード embedding 登録、SPEC-LGX-006.REQ.09）の上に乗る薄いレイヤとして動作。

ISSUE-001 §2.3 の機能 C 本体実装。機能 A（IdRedefined 宣言）/ 機能 B（IdSemanticMismatch 数値検査）と独立に動作する 3 層防御の **embedding 観点の最終層**。

- **対象ペア**: サブノード embedding（is_subnode = 1）のうち、本文に同一 ID の `citation_pattern` がマッチするもの同士。SPEC 親ドキュメントに属するサブノードを「定義側」、それ以外（UC/RB/SEQ 親）を「引用側」として扱う
- **類似度計算**: 既存 `te_embed::cosine_similarity` を流用
- **閾値判定**: cosine_similarity < `similarity_threshold`（既定 0.75）→ Warning
- **citation_pattern**: 機能 A/B と統一（既定 `\|\s*{ID}\s*\|`）
- **打切り**: 1 ID あたりのペア比較は `max_pairs_per_id`（既定 50）で打切り、Info で通知
- **severity**: Warning（embedding 類似度は false-positive 傾向が低いため、機能 B より厳格）
- **category**: `IdSemanticDrift`（新規追加）
- **デフォルト**: `[id_semantic_drift].enabled = false`（オプトイン）
- **G1 ゲート**: Error を発しないため阻害しない
- **embedding 不在時の挙動**: 該当ノードの embedding が engine.db に無い場合は当該ペアをスキップ（embed 未実行は致命扱いしない）

**前提条件**:
- `legixy embed --all` が実行済（サブノード embedding が `embeddings` テーブルに格納済）
- `[semantic].include_subnodes = true`（既定、Phase 2 alpha1 以降）
- ONNX モデルが利用可能（`embed --all` の前提）

**根拠:** ISSUE-001 §2.3 機能 C、LGX-EXT-001 Phase 2 Block F、ISSUE-005 §2.3（vnstudio Phase 1 ベースライン: ノード単位類似度ではテンプレ寄与で SN 比が低いため、サブノード粒度の embedding 比較が必要）
**検証方法:** TS-LGX-003 §15 T-ISD-001〜005

### SPEC-LGX-004.REQ.14: サブノード ID 衝突検査（SubnodeIdCollision、前段ループ反復 1 新設）

**内容:** `check --formal` は、同一ドキュメント内で正規化後 heading_path が一致する複数の見出し（SPEC-LGX-002.REQ.05 の生成式により同一 ID に縮退するもの、SPEC-LGX-002.REQ.12）を検出し、Warning として報告する:
- severity: **Warning**（Error にしない理由: v3 で check が通っていた既存プロジェクトが移行直後に G1 ゲート〔Error=0〕で fail する互換リスクの回避）
- category: `SubnodeIdCollision`（新規追加）
- message: 親ドキュメント・衝突した見出しテキスト・縮退先 ID を明示
- **検出対象は自動生成サブノード同士の縮退のみ**。graph.toml の明示ノードと衝突して生成スキップされた自動 ID（SPEC-LGX-002.REQ.12 の明示優先スキップ）は本検査の対象外とする（QSET-LGX-002 Q2 回答どおり、明示宣言は意図的な上書きとみなす）
- G1 ゲート: Warning のみのため阻害しない

【v3 差分】v3 は無言で後勝ち縮退し検出機構を持たなかった（`crates/te-graph/src/parser.rs:126-145`）。本検査は v3 サブノード仕様 TE-NEXT-EXT-001（line 725「`check --formal` で検出される」）が約束しながら実装が果たさなかった検出の回復である。

**根拠:** QSET-LGX-002 Q2 回答（2026-06-07）、SUBNODE-INV-3
**検証方法:** 同名見出し 2 つの fixture で Warning 1 件が報告されるテスト

### SPEC-LGX-004.REQ.15: 形式検証カテゴリの severity 割当（GAP-LGX-064 対応、旧 GAP-073/074 吸収）

**内容:** 形式検証（REQ.01）の全カテゴリについて severity 割当を以下の表で**完全に**定める。本表に現れないカテゴリの追加は本 SPEC の改訂を要する（割当完全性の保証）。

> **config 助言 finding の射程注記（spec-change 2026-06-13、ADR-LGX-019、TRIAGE §4 #8）**: 本表の「完全性」および severity の「固定」は、**グラフの検証結果（違反 finding）**のカテゴリと severity を対象とする。v3 が formal check の先頭で発行する**設定ファイルの書き方助言**（`[id.document_id].pattern` への `{id}` 誤記 Warning、`[id].area == "XX"`〔init テンプレ初期値のまま〕Info 等）は「検証カテゴリ」ではなく設定ガイダンスであり、本割当表の完全性宣言の対象外とする。これらは DocumentId 等の検証カテゴリの severity を 3 値化するものではなく、実装は config 助言を検証 finding と分離して出力する。

| カテゴリ | severity | 固定/可変 | 備考 |
|---|---|---|---|
| FileExistence | Error | 固定 | ファイル読取失敗も Error |
| DocumentId（不一致） | Error | 固定 | |
| DocumentId（行欠落） | Error | 固定 | 人間裁定 2026-06-10（厳格）。v3 実挙動と一致。ハードルール 4(c)「Document ID 行必置」と整合 |
| ChainIntegrity | Error | 固定 | |
| OrphanFile | Error | 固定 | |
| GraphDag | Error | 固定 | グラフ全体サイクル（CTX-INV-4）。新カテゴリ【v3 差分: v3 は SubnodeDag 名で報告】 |
| Freshness | Warning | 固定 | 既存 pin（更新時刻ドリフトは助言） |
| SubnodeIdFormat / SubnodeIdUniqueness / SubnodeParentIntegrity / SubnodePathConsistency / SubnodeDag | Error | 固定 | REQ.07 と整合 |
| SubnodeIdCollision | Warning | 固定 | REQ.14 既定（互換リスク回避の理由つき pin） |
| UnresolvedEdge | Warning | 固定 | REQ.10 既定 |
| IdRedefined / IdSemanticMismatch / IdSemanticDrift | Warning/Info | 固定 | REQ.11/12/13 既定（運用補助、G1 非阻害） |

- **Error 列のみ**が G1 ゲート（Error=0）と終了コード（REQ.04: Error>0 → exit 1）に影響する。
- 既に各 REQ（REQ.07/10/11/12/13/14）で pin 済みの severity と本表は重複するが矛盾しない（再掲による完全性明示）。

**根拠:** GAP-LGX-064（+旧 GAP-073 DocumentId severity / 旧 GAP-074 node-level DAG severity を吸収）。LGX-COMPAT-001 v1.0.1（check の Error>0 → exit 1）と整合。CLI/MCP 引数は不変
**検証方法:** 全カテゴリの fixture で報告 severity が本表と一致することの網羅テスト

---

## 4. 不変条件との関係

| 不変条件 | 役割 | 対応要求 |
|---------|------|---------|
| CTX-INV-2（グラフ整合性） | 検証 | REQ.01（ChainIntegrity, FileExistence） |
| CTX-INV-4（DAG 制約） | 検証 | REQ.01（GraphDag、グラフ全体）, REQ.15（severity=Error 固定） |
| CTX-INV-5（未解決エッジの許容性） | 検証 | REQ.10（UnresolvedEdge Warning 報告） |
| SUBNODE-INV-1（親存在） | 検証 | REQ.01, REQ.07（SubnodeParentIntegrity） |
| SUBNODE-INV-2（パス整合性） | 検証 | REQ.01, REQ.07（SubnodePathConsistency） |
| SUBNODE-INV-3（ID 一意性） | 検証 | REQ.01, REQ.07（SubnodeIdUniqueness）, REQ.14（SubnodeIdCollision、自動生成の縮退検出） |
| SUBNODE-INV-4（DAG） | 検証 | REQ.01, REQ.07（SubnodeDag） |
| SUBNODE-INV-6（ID フォーマット） | 検証 | REQ.01, REQ.07（SubnodeIdFormat） |
| CTX-INV-1（決定論保証） | 関連 | REQ.06（冪等性、検証動作の再現性） |

**REQ.11 / REQ.12 / REQ.13 と不変条件の関係:** REQ.11（IdRedefined）・REQ.12（IdSemanticMismatch）・REQ.13（IdSemanticDrift）は legixy 不変条件には直接関与しない **運用補助カテゴリ** であり、SPEC レベルの「同一 ID の意味再定義」を運用上検出するための 3 層防御である。Error を発しないため G1 ゲートを阻害せず、デフォルト OFF のため後方互換性も保たれる。REQ.13 は Phase 2 Block A 完成（SPEC-LGX-006.REQ.09）に依存する embedding ベースの検査であり、A/B が宣言・regex ベースで補えない「表記揺れ + 定義シフトの併発」を最終的に拾う層となる。

**本 SPEC が関与しない不変条件:** CTX-INV-3, MCP-INV-1〜4、SUBNODE-INV-5（ID 決定性は生成ロジック側で保証、本 SPEC の検証対象外）

---

## 5. 変更履歴

| 日付 | バージョン | 変更内容 |
|------|----------|---------|
| 2026-04-17 | 0.1.0-draft | 初版（AI 起草） |
| 2026-04-17 | 0.1.1-draft | F-03 修正: REQ.01/REQ.03 の根拠欄から DD-LGX-001 §3.10 / §2.4 参照を削除 |
| 2026-04-17 | 0.1.2-draft | F-04 修正: §4 表に「役割」列を追加、SUBNODE-INV-1〜4,6 の集約記述を個別行に展開、CTX-INV-2 を追加、対象外不変条件（CTX-INV-3, MCP-INV-*, SUBNODE-INV-5）を明記、CTX-INV-4 名称を「非循環」→「DAG 制約」に訂正 |
| 2026-04-17 | 0.2.0 | 人間査読完了により承認 |
| 2026-04-17 | 0.3.0 | S1-c 対応: REQ.10 未解決エッジの Warning 報告（CTX-INV-5 検証責任）を新設、§4 マトリクスに CTX-INV-5 行を追加（Finding E-01） |
| 2026-04-27 | 0.4.0 | ISSUE-001 対応: REQ.11 IdRedefined 検出、REQ.12 IdSemanticMismatch 検出を新設。§4 マトリクスに REQ.11/REQ.12 の運用補助カテゴリとしての位置付けを追記。デフォルト OFF / Warning（または Info）/ G1 非阻害の方針 |
| 2026-04-28 | 0.5.0 | LGX-EXT-001 Phase 2 Block F（ISSUE-001 機能 C 本体）対応。REQ.13 IdSemanticDrift サブノード単位意味類似度検査を新設。§4 不変条件マトリクスに REQ.13 を 3 層防御（A 宣言 / B regex / C embedding）として位置付け |
| 2026-06-07 | 0.6.0 | 前段ループ反復 1（QSET-LGX-004 回答 → SPP-LGX-004 承認）対応: REQ.04 の exit 2 を全コマンド共通規約として明確化（LGX-COMPAT-001 v1.0.1 と連動）。REQ.02 に SPEC-LGX-010 との責務境界（check=判定 / report=計測、check 内 Drift と standalone drift の書き分け）を明文化、bulk API 参照を REQ.09→REQ.11 に訂正。REQ.14 SubnodeIdCollision 検査（Warning、QSET-LGX-002 Q2 連動【v3 差分】）を新設 |
| 2026-06-10 | 0.7.0 | TP[SPEC] GAP 解消（人間承認 2026-06-10）: GAP-LGX-064 対応（旧 073/074 吸収）で REQ.01 各カテゴリに severity を明示し、グラフ全体 DAG を新カテゴリ **GraphDag** に分離（人間裁定。【v3 差分】v3 は SubnodeDag 名で報告）、DocumentId 行欠落は Error（人間裁定、v3 実挙動と一致）。REQ.15 severity 割当表（割当完全性保証、Error 列のみ G1/exit 1 に影響）を新設。§4 CTX-INV-4 行を GraphDag/REQ.15 連動に更新。CLI/MCP 引数不変 |
| 2026-06-10 | 0.8.0 | weak GAP 解消（人間裁定 fix・承認 2026-06-10）: GAP-LGX-061 対応で REQ.01 に空グラフ時挙動（finding 0・exit 0 + stderr Info【v3 差分】、物理不在は init 誘導の別経路）を追記。GAP-LGX-065 対応で REQ.03 に Ok の使用条件（カテゴリ finding 非発行＝v3 実測、全 pass = findings 0 件、予約 severity）を確定。GAP-LGX-072 対応で REQ.06 を全層 check に拡張（安定ソートキーの順序決定論・スコア値はビット再現対象外 = ADR-LGX-003 整合・golden 比較指針） |
| 2026-06-13 | 0.9.0 | spec-change（ADR-LGX-019、TRIAGE §4 #8）: REQ.15 に config 助言 finding の射程注記を追加。完全性・severity 固定は検証結果（違反 finding）を対象とし、config 由来の設定書き方助言（`{id}` 誤記 Warning・area=="XX" Info）は対象外で DocumentId 等の severity を 3 値化しない（実装は分離出力） |
