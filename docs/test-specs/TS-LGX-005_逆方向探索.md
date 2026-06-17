Document ID: TS-LGX-005

# TS-LGX-005: 逆方向探索（investigate）のテスト仕様

> TS は **上流 TP の翻訳**。ゼロから観点を考えない。各 TP 観点を DD-LGX-005 で確定した型・関数シグネチャに即した具体的な入力・期待出力・前提条件へ展開する。

**親 DD**: DD-LGX-005
**継承 TP**: TP-LGX-005（TP[SPEC] グラフ走査、31 観点）, TP-LGX-015（TP[UC] UC-005 逆方向探索フロー、20 観点）

## 1. 対象範囲

このテスト仕様がカバーする DD-LGX-005 の関数 / 型:

- DD-LGX-005 §3 `legixy_nav::investigate(graph: &TraceGraph, start_ids: &[String], db: Option<&rusqlite::Connection>, drift_threshold: f32) -> Result<PrunedTraversalResult, NavError>`
- DD-LGX-005 §3 `legixy_nav::investigate_with_depth(graph: &TraceGraph, start_ids: &[String], db: Option<&rusqlite::Connection>, drift_threshold: f32, max_depth: Option<usize>) -> Result<InvestigateOutcome, NavError>`
- DD-LGX-005 §3 `legixy_nav::render_pruned(result: &PrunedTraversalResult, format: ReportFormat) -> String`（打ち切り**非発生**時用）
- DD-LGX-005 §3 `legixy_nav::render_outcome(outcome: &InvestigateOutcome, format: ReportFormat) -> String`（v1.1 新設。`outcome.truncated == true` のとき summary 行へ `"truncated":true,"excluded":K` を付加。打ち切り可観測性整形経路）
- DD-LGX-005 §2 型: `VisitedNode` / `MultiTraversalResult` / `SuspiciousNode` / `PrunedTraversalResult` / `InvestigateOutcome`{truncated, excluded_count} / `ReportFormat`{Text, JsonLines} / `NavError`{Io, InvalidInput}
- DD-LGX-005 §4 内部: `DriftPruner::prune`（db=None / Some / 照会失敗の 3 経路、ベストエフォート）

NodeId は `String`（ADR-LGX-021 §2.1。`VisitedNode.id` / `SuspiciousNode.id` / `start_ids` 要素 / `edges_traversed` のタプル要素はすべて `String`）。

委譲（本 TS 対象外）:
- drift スコアの**数値妥当性**（算出式・SCORE-INV-1）→ SPEC-LGX-006 / TP-LGX-006 所有。本 TS は drift_score を fixture 投入値として扱い、閾値判定・整列の決定性のみ検証する。
- 走査性能予算 PERF.02（ノード 1,000 + エッジ 2,000 で < 500ms）→ NFR-LGX-001 / bench（criterion）へ委譲（DD §8 Bench 行）。
- 並行アクセス整合性 REL.07/08（concurrent write）→ NFR / legixy-db 責務（DD §7）。
- BFS の隣接処理順・IndexMap 挿入順・system 生成 parent_child エッジの整列規則 → SPEC-LGX-002 REQ.08 / TP-LGX-005 が委譲明示（観点 26）。本 TS は固定された順序を**消費した結果の決定性**のみを検証する。
- `--max-depth` の値型・範囲検証（負値・非数値・usize 上限超）→ 引数パーサ（clap）層、構文誤り exit 2（DD §2.3）。本 TS は API 層の `Option<usize>` 受理後の挙動に限定。
- グローバル `--project-root` / `--json` / `--models-dir` 受理、サブコマンド名・位置引数互換 → CLI ディスパッチ層 / LGX-COMPAT-001 §3/§4 #12 契約。本 TS はケース 16/17 で `--json` の機能化（render の JsonLines 出力スキーマ）に限定。

本 TS は「investigate が SPEC-005 の規定を DD-005 の型で正しく具体化しているか」を検証する。

## 2. ケース一覧

### ケース 1: max_depth = 0 → 起点のみ（depth 0）返る

- **観点出典**: TP-LGX-005 §2.1 観点 1（`max_depth = 0` で起点のみ、深度 0、REQ.04）
- **分類**: Unit
- **前提**: 起点 `"DD-LGX-005"` を含む線形チェーン（上流に複数ノード）。`db = None`、`drift_threshold = 0.3`
- **入力**: `investigate_with_depth(&graph, &["DD-LGX-005".to_string()], None, 0.3, Some(0))`
- **期待**: `Ok(outcome)`。`outcome.result.traversal.visited` は要素 1（`VisitedNode{ id: "DD-LGX-005", depth: 0, .. }`）。`depth_map["DD-LGX-005"] == 0`。`edges_traversed` 空。`outcome.truncated == true`（起点に未訪問の上流隣接が存在するため）、`outcome.excluded_count == 起点の逆方向隣接数`
- **境界条件**: 深度下限 0 = 起点のみ。打ち切りは depth==0 境界で発生

### ケース 2: max_depth 未指定（None）と巨大値の結果一致

- **観点出典**: TP-LGX-005 §2.1 観点 2（省略=無制限と巨大値で一致、REQ.04 既定=無制限）, §2.5 観点 13（互換対象 (d)）
- **分類**: Integration
- **前提**: 起点から逆方向に到達可能なノードが有限（深度最大 D < 1,000,000）。`db = None`
- **入力**: `investigate_with_depth(.., None)` と `investigate_with_depth(.., Some(1_000_000))`、さらに `investigate(&graph, start, None, 0.3)`
- **期待**: 3 者の `traversal.visited`（順序含む）・`depth_map`・`edges_traversed` が完全一致。`None`/巨大値ともに `truncated == false`、`excluded_count == 0`。`investigate` は `investigate_with_depth(.., None)` と同一の `PrunedTraversalResult`（DD §3 不変条件: max_depth=None ⇒ investigate 同一）
- **境界条件**: 深度上限なし＝到達可能全集合。巨大値 ≥ 実深度なら打ち切り不発

### ケース 3: max_depth 打ち切り発生 → truncated=true / excluded_count>0（exit 不変）

- **観点出典**: TP-LGX-005 §2.1 観点 5（打ち切り発生の観測可能性、GAP-LGX-085）, TP-LGX-015 §2.6 R2（--max-depth 適用）
- **分類**: Integration
- **前提**: 起点から深度 3 まで到達可能なグラフ。`db = None`
- **入力**: `investigate_with_depth(&graph, start, None, 0.3, Some(1))`
- **期待**: `Ok(outcome)`。`outcome.result.traversal.visited` は深度 0/1 のノードのみ。`outcome.truncated == true`。`outcome.excluded_count == K`（K = 深度境界ノード depth==1 から出る未訪問隣接数の合計、DD §6 近似定義）。`exit_code` は不変＝ **0**（stdout 結果集合・終了コードは打ち切りで変わらない、SPEC-005.REQ.04）。Info は呼出側が stderr へ出す（戻り値自体は exit に非影響）
- **境界条件**: 打ち切り = 部分集合 + truncated フラグ。到達集合は決定論的、exit 0 維持

### ケース 4: 起点ノード不在 → 空 visited・exit 0（エラーではない、D 裁定）

- **観点出典**: TP-LGX-005 §2.2 観点 6（不在起点=空結果・非エラー、REQ.05）, §2.10 観点 23（空結果時 exit 0）, TP-LGX-015 §2.3 EF3（起点 ID 不在の挙動）
- **分類**: Unit
- **前提**: `graph.toml` に存在しない ID。`db = None`
- **入力**: `investigate(&graph, &["NONEXISTENT-LGX-999".to_string()], None, 0.3)`
- **期待**: `Ok(result)`（`Err` ではない）。`result.traversal.visited` 空、`depth_map` 空、`edges_traversed` 空、`suspicious_nodes` 空。呼出側 `exit_code == 0`。任意文字列起点（形式不正な ID）も同じく「不在」に収束し空結果・exit 0
- **境界条件**: **D 裁定**。起点不在 = 空の探索結果 + exit 0（exit 1 ではない、SPEC-005.REQ.05 / DD §2.3 代替フロー / DD §3 不変条件）。SUPP-005 の UC 代替フロー 2a との矛盾は SPEC 後勝ちで exit 0

### ケース 5: 空グラフ（ノード 0）/ 単一ノード孤立 → 起点のみ or 空

- **観点出典**: TP-LGX-005 §2.1 観点 4（空グラフ・単一ノード・孤立起点）, §2.8 観点 18（隣接ゼロ＝起点 1 件）
- **分類**: Unit
- **前提**: (a) ノード 0 の空グラフ、(b) 単一ノード・エッジ 0 で起点 = その 1 ノード。`db = None`
- **入力**: (a) `investigate(&empty_graph, &["X".to_string()], None, 0.3)`、(b) `investigate(&single_node_graph, &["X".to_string()], None, 0.3)`
- **期待**: (a) 起点不在に収束 → `visited` 空、exit 0（ケース 4 と同根）。(b) `visited` 要素 1（`VisitedNode{ id:"X", depth:0 }`）、`edges_traversed` 空、`suspicious_nodes` 空、exit 0
- **境界条件**: 規模下限。隣接ゼロの孤立起点は「起点 1 件」（深度 0）、空グラフは「空・非エラー」

### ケース 6: 逆方向走査の方向性（to→from を辿る）

- **観点出典**: TP-LGX-005 §2.11 観点 28（from→to 順方向、逆は to→from）, TP-LGX-015 §2.6 R1（逆方向 BFS = 上流方向 = to→from）
- **分類**: Unit
- **前提**: エッジ `A --chain--> B`、`B --chain--> C`（chain は from→to）。起点 = `C`。`db = None`
- **入力**: `investigate(&graph, &["C".to_string()], None, 0.3)`
- **期待**: `visited` に C(depth 0), B(depth 1), A(depth 2) を含む（逆方向＝to→from を辿り上流へ）。順方向の下流（C より先）は含まない。`edges_traversed` は初訪問を生んだ逆向き辿り（spanning tree、グラフ向き表現 (B,C)/(A,B) を含む、DD §2.1 注）
- **境界条件**: 逆方向走査は `to→from`（SPEC-005.REQ.02 / REQ.01 一般則）。順方向走査 (impact) との区別

### ケース 7: BFS 決定論性（同一入力 → 同一 visited 順・depth_map）

- **観点出典**: TP-LGX-005 §2.11 観点 25（IndexMap 挿入順に従う visited 順・depth_map 決定性、REQ.03 / REL.05 / CTX-INV-1）, §2.3 観点 10（BFS レベル＝深度）, TP-LGX-015 §2.5 DF3（visited 順の独立決定性）
- **分類**: Property-based（proptest）
- **生成器**: 任意の DAG（ノード集合・chain/custom/parent_child エッジ・起点）を生成。同一論理グラフを複数回構築（同一 IndexMap 挿入順）
- **不変条件**: 同一 `(graph, start_ids, None, threshold)` に対し `investigate` は常に同一順序の `visited`・同一 `depth_map`・同一 `edges_traversed` を返す（バイト一致）。多経路到達ノードの `depth_map` は最短深度（最初の到達深度）を保持し再訪で更新しない（REQ.03/04 の BFS 定義的帰結）
- **反例ハンドリング**: shrink して最小の順序不一致 / 深度不一致例を記録

### ケース 8: DAG 破れ（サイクル）・self-loop でも有限停止

- **観点出典**: TP-LGX-005 §2.11 観点 29（サイクルあり入力で有限停止、REQ.06）, 観点 30（self-loop で起点を再出力せず停止）, §2.2 観点 8（panic/unwrap が本番経路に残らない）
- **分類**: Unit
- **前提**: (a) サイクル `A→B→C→A` を含むグラフ、(b) self-loop `X→X` を含むグラフ。`db = None`
- **入力**: (a) `investigate(&cyclic_graph, &["A".to_string()], None, 0.3)`、(b) `investigate(&selfloop_graph, &["X".to_string()], None, 0.3)`
- **期待**: 両者とも有限時間で `Ok` 返却（無限ループしない）。各ノードは visited に 1 回のみ（(b) の X は再出力されない）。panic / unwrap で異常終了しない
- **境界条件**: visited による停止保証（REQ.06）。self-loop は最小サイクルで visited 抑止に内包

### ケース 9: read-only 不変（graph / db を変更しない）

- **観点出典**: TP-LGX-005 §2.4 観点 12（読み取り専用メモリ内操作）, §2.6 観点 16（engine.db / graph.toml への書き込みを伴わない）, TP-LGX-015 §2.1 BF3（読み取り専用事後条件）, §2.4 AT1（両アクター同一権限）
- **分類**: Property/Integration
- **前提**: 任意のグラフ + drift スコアを持つ engine.db fixture（`db = Some(&conn)`）
- **入力**: `investigate(&graph, start, Some(&conn), 0.3)` 実行前後の graph スナップショット・engine.db ファイルハッシュ
- **期待**: 実行前後で graph（借用 `&TraceGraph`）と engine.db が不変（DD §5 借用・read-only、§7 snapshot 読取）。複数回呼び出しでも結果同一・状態不変
- **境界条件**: `&TraceGraph` / `Option<&rusqlite::Connection>` の借用による read-only 保証

### ケース 10: db = None → suspicious_nodes 空・走査結果のみ（ベストエフォート、代替フロー 3a）

- **観点出典**: TP-LGX-005 §2.2（部分成功）, TP-LGX-015 §2.2 AF3（3a: ドリフトスコアなしで走査結果のみ）, §2.3 EF2（engine.db 不在は 3a へ、exit 1 ではない）
- **分類**: Unit
- **前提**: `db = None`（engine.db 不在 / embedding 未生成相当）。起点は到達可能ノードを持つ
- **入力**: `investigate(&graph, start, None, 0.3)`
- **期待**: `Ok(result)`。`result.traversal.visited` / `depth_map` は通常通り（drift 不在の影響なし）。`result.suspicious_nodes` 空（`Vec` 空配列）。`result.drift_threshold == 0.3`。`exit_code == 0`（失敗パスに落ちない、REQ.05 代替フロー 3a / NFR REL.02）
- **境界条件**: DB 不在 = 致命的でない degraded。suspicious は空、visited/depth_map は非影響

### ケース 11: db = Some・DB 照会失敗 → suspicious_nodes 空・継続（NavError に昇格しない）

- **観点出典**: TP-LGX-005 §2.2（ベストエフォート）, DD §6（SQL 照会失敗は NavError に昇格させず stderr 警告 + 空 suspicious で継続、NFR REL.02）, DD §8 Unit「DriftPruner::prune 照会失敗経路」
- **分類**: Unit/Integration
- **前提**: `db = Some(&conn)` だが scores テーブル不在 / 不正スキーマ（`rusqlite::Error` を誘発）
- **入力**: `investigate(&graph, start, Some(&conn), 0.3)`
- **期待**: `Ok(result)`（`Err(NavError)` ではない）。`suspicious_nodes` 空。stderr に `[nav] drift pruning skipped: ...` 警告（DD §6）。`visited`/`depth_map` は通常通り。`exit_code == 0`
- **境界条件**: SQL 失敗 ≠ 致命。DB=None と同一の degraded・非致命帰結（NavError へ昇格させない）

### ケース 12: db = Some・drift スコア閾値判定 → suspicious_nodes 抽出（threshold 以上）

- **観点出典**: TP-LGX-005 §2.9 観点 20（drift 値の判定基準・閾値）— 数値妥当性は SPEC-006 委譲、本ケースは閾値**判定ロジック**のみ, TP-LGX-015 §2.1 BF1（Step3 drift 参照 → Step4 マーク連鎖）
- **分類**: Integration
- **前提**: engine.db fixture で visited 内ノードに drift スコア付与（例: N1=0.5, N2=0.3, N3=0.1）。`drift_threshold = 0.3`
- **入力**: `investigate(&graph, start, Some(&conn), 0.3)`
- **期待**: `suspicious_nodes` に N1(0.5), N2(0.3) を含み N3(0.1 < 0.3) を含まない。各 `SuspiciousNode{ id, drift_score, type_code, path }` は対応 visited から type_code/path をクローン（DD §5）。`drift_score` は scores テーブルの `MAX(value) where score_type='drift'`（DD §2.1）
- **境界条件**: 閾値 = 以上（>=）で抽出。境界値 drift==threshold は含む（N2=0.3 採用）。drift 算出式は SPEC-006 委譲

### ケース 13: suspicious_nodes の整列決定性（drift 降順・同値 id 昇順）

- **観点出典**: TP-LGX-015 §2.5 DF3（suspicious スコア降順保証）, DD §2.1（drift_score 降順・同値 id 昇順 stable sort）, DD §8 Property「suspicious 整列決定性」
- **分類**: Property-based（proptest）
- **生成器**: 任意の `Vec<SuspiciousNode>`（id / drift_score / type_code / path をランダム生成、drift 同値ペアを含む）
- **不変条件**: 同一入力に対し `suspicious_nodes` は常に drift_score 降順、drift 同値内では id 昇順（stable）の一意順序。`investigate` 全体でも同一 `(graph, start, db, threshold)` → 同一 suspicious 順序
- **反例ハンドリング**: shrink して最小の順序逆転例を記録

### ケース 14: render_pruned Text 書式（v3 互換）

- **観点出典**: TP-LGX-005 §2.9 観点 19（出力に含める情報: ID 走査順 / type / path / depth / 使用エッジ、REQ.09）, DD §3 出力書式の凍結（Text）, TP-LGX-015 §2.5 DF1（出力構造の観察可能性）
- **分類**: Unit
- **前提**: visited = [N0(depth 0), N1(depth 1)]、suspicious = [S1(drift 0.5)]、`drift_threshold = 0.3`
- **入力**: `render_pruned(&result, ReportFormat::Text)`
- **期待**: DD §3 凍結書式に一致する文字列。visited 各行 `{id} (type={type_code}, depth={depth}, path={path})`、`Suspicious nodes (drift_threshold=0.3):` セクション + `{id} (drift={score}, type=.., path=..)`、サマリ `Summary: visited=2, suspicious=1`。drift 値は Text 経路では `f32` の Display 表現を直接出力する（v3 reporter.rs の Text 経路は `sn.drift_score` の生 f32 を直接 Display；`f32_to_clean_f64` は JsonLines 経路のみで適用される。挙動束縛は不変）
- **境界条件**: Text = v3 互換既定（ReportFormat::default()）。フィールド順・ラベルは凍結

### ケース 15: render_pruned JsonLines 書式（--json 機能化、v3 差分）

- **観点出典**: TP-LGX-005 §2.5 観点 14（--json 機能化が引数体系を壊さない、REQ.09）, §2.9 観点 19, TP-LGX-015 §2.5 DF1, DD §3 出力書式の凍結（JsonLines）
- **分類**: Unit/Contract
- **前提**: visited = [N0(depth 0)]、suspicious = [S1(drift 0.3)]、`drift_threshold = 0.3`、`truncated = false`
- **入力**: `render_pruned(&result, ReportFormat::JsonLines)`
- **期待**: 1 行 1 JSON オブジェクトの JSON Lines。visited 行 `{"id":"N0","type":"..","depth":0,"path":".."}`、suspicious 行 `{"suspicious":{"id":"S1","drift":0.3,"type":"..","path":".."}}`、summary 行 `{"summary":{"visited":1,"suspicious":1,"drift_threshold":0.3}}`。各行は valid JSON、フィールド名は DD §3 凍結に一致。`drift` は `f32_to_clean_f64` 短表現
- **境界条件**: JsonLines = 受理済み `--json` フラグの機能化（v3 は Text 固定で無視）。スキーマは DD 凍結（フィールド改名・削除は次版 SPEC 改訂）

### ケース 16: render_outcome の JsonLines summary truncated フラグ（打ち切り時のみ付加）

- **観点出典**: TP-LGX-005 §2.1 観点 5（打ち切りの観測可能性、GAP-LGX-085）, DD §3「truncated フラグ: InvestigateOutcome.truncated=true のとき summary 行に "truncated":true,"excluded":K を追加」, DD §3 `render_outcome`（v1.1 新設の打ち切り可観測性整形経路）
- **分類**: Contract
- **対象 API**: `render_outcome(outcome: &InvestigateOutcome, format: ReportFormat) -> String`（DD §3、v1.1 新設）。`render_outcome` は内部で `outcome.result`（= `PrunedTraversalResult`）を `render_pruned` 相当で整形し、`outcome.truncated == true` のとき summary 行へ truncated 注記を加える。`PrunedTraversalResult` は truncated を持たないため、truncated 反映の整形は `render_pruned` ではなく `render_outcome` が担う（旧記述の `render_pruned` 束縛は API gap、DD v1.1 で解消）
- **前提**: (a) `InvestigateOutcome{ truncated: false, excluded_count: 0, result: .. }`、(b) `InvestigateOutcome{ truncated: true, excluded_count: 4, result: .. }`
- **入力**: 各 outcome を `render_outcome(&outcome, ReportFormat::JsonLines)` で render
- **期待**: (a) summary 行に truncated フィールドなし（`{"summary":{"visited":..,"suspicious":..,"drift_threshold":..}}`）＝ `render_pruned(&outcome.result, JsonLines)` と完全一致（DD §3 `render_outcome` 不変条件: truncated=false 時は render_pruned と同一出力）。(b) summary 行に `"truncated":true,"excluded":4` を追加。truncated=false 時は付加しない（後方互換: 既存スキーマ不変）
- **境界条件**: truncated フラグは JSON スキーマ加算的拡張（DD 凍結対象、フィールド削除・改名は次版 SPEC 改訂）。打ち切り発生時のみ可視化。truncated 反映の責務は `render_outcome`（打ち切り発生経路）、`render_pruned` は打ち切り非発生時用と整合

### ケース 17: 終了コード契約 0/1/2（LGX-COMPAT-001 §4 #12 凍結）

- **観点出典**: TP-LGX-005 §2.10 観点 23（空結果 exit 0）, §2.1 観点 3（構文誤り exit 2）, TP-LGX-015 §2.6 R4（終了コード契約）, §2.3 EF1（graph.toml 不在・破損）
- **分類**: Contract
- **前提**: (a) 正常走査（起点存在・不在いずれも結果取得成功）、(b) graph ロード失敗（`legixy_graph::GraphError`）/ `NavError`、(c) `--max-depth` に負値・非数値（clap 構文層）
- **入力**: それぞれ investigate 経路 / CLI ディスパッチ
- **期待**: (a)→ exit **0**（起点不在の空結果・打ち切り発生を含めすべて 0、DD §2.3）。(b)→ exit **1**（実行時失敗 = NavError/GraphError を呼出側が変換、DD §6）。(c)→ exit **2**（引数構文誤り = clap、DD §2.3）
- **境界条件**: exit 2 は構文層限定。意味的事象（起点不在・DB 不在・打ち切り）は exit 0、実行時失敗のみ exit 1。--max-depth 値検証は clap 層委譲

### ケース 18: 出力先分離（走査結果=stdout / Info・警告=stderr）

- **観点出典**: TP-LGX-015 §2.5 DF1（stdout/stderr 分離）, DD §6（打ち切り Info / drift 警告は stderr）, NFR OBS.02
- **分類**: Integration
- **前提**: `--max-depth` 打ち切り発生（Info）かつ db 照会失敗（警告）を同時に誘発するグラフ + 不正 db
- **入力**: CLI 実行（`investigate <start> --max-depth N` / `--json`）
- **期待**: 走査結果（render_pruned 出力）は **stdout**。`[nav] info: max-depth {N} truncated traversal; {K} reachable node(s) excluded` と `[nav] drift pruning skipped: ..` は **stderr**（DD §6、NFR OBS.02）。stdout のみリダイレクトしても結果集合は欠落・混入しない
- **境界条件**: チャネル分離。Info/警告は exit に非影響（exit 0 維持）

### ケース 19: MCP 非公開（investigate は Admin Surface 限定）

- **観点出典**: TP-LGX-005 §2.10 観点 24（走査は MCP に公開されない、MCP-INV-1、REQ.10）, TP-LGX-015 §2.6 R5（UC アクターは CLI のみ）
- **分類**: Contract
- **前提**: MCP サーバ（ts-mcp）が公開する 3 ツール一覧
- **入力**: MCP ツールカタログ（query_context / list_nodes 等）に investigate / impact が含まれないことの検査
- **期待**: MCP 公開ツールに走査系（investigate）が**存在しない**。CLI Admin Surface のみで提供（REQ.10、MCP-INV-1）
- **境界条件**: Admin Surface 限定。走査は MCP（Agent Surface）に露出しない

## 3. 観点カバレッジ表

| TP § | 観点 | カバーする TS ケース |
|---|---|---|
| TP-005 §2.1 観点 1 max_depth=0 | 境界値 | ケース 1 |
| TP-005 §2.1 観点 2 無制限/巨大値一致 | 境界値 | ケース 2 |
| TP-005 §2.1 観点 3 --max-depth 不正値 | 境界値 | ケース 17(c)（clap 層 exit 2 / 値検証は委譲） |
| TP-005 §2.1 観点 4 空/単一/孤立起点 | 境界値 | ケース 5（+ 4 起点不在） |
| TP-005 §2.1 観点 5 打ち切り観測可能性 | 境界値 | ケース 3, 16 |
| TP-005 §2.2 観点 6 不在起点=空・非エラー | エラー | ケース 4 |
| TP-005 §2.2 観点 7 未解決エッジ除外結果 | エラー | SPEC-LGX-002 REQ.11 / SPEC-LGX-004 check へ委譲（走査は解決済部分グラフ上） |
| TP-005 §2.2 観点 8 panic/unwrap 不在 | エラー | ケース 8 |
| TP-005 §2.3 観点 9 visited 初期状態・再訪抑止 | 状態 | ケース 7, 8 |
| TP-005 §2.3 観点 10 BFS レベル↔depth_map | 状態 | ケース 1, 7 |
| TP-005 §2.3 観点 11 多経路時 depth 規則（最短） | 状態 | ケース 7（property 不変条件） |
| TP-005 §2.4 観点 12 読み取り専用メモリ内 | 並行 | ケース 9（+ REL.07/08 NFR 委譲） |
| TP-005 §2.5 観点 13 省略=無制限 互換 | 互換 | ケース 2 |
| TP-005 §2.5 観点 14 --json 機能化非破壊 | 互換 | ケース 15 |
| TP-005 §2.5 観点 15 サブコマンド/位置引数互換 | 互換 | CLI ディスパッチ / LGX-COMPAT-001 §4 #12 契約へ委譲（ケース 17/19 で終端確認） |
| TP-005 §2.6 観点 16 書き込み非伴 | 永続化 | ケース 9 |
| TP-005 §2.7 観点 17 起点 ID 形式検証 | 入力 | ケース 4（任意文字列起点 = 不在に収束） |
| TP-005 §2.8 観点 18 隣接ゼロ=起点 1 件 | ライフ | ケース 5(b) |
| TP-005 §2.9 観点 19 出力情報の確定 | 観測 | ケース 14, 15 |
| TP-005 §2.9 観点 20 drift 判定基準・閾値 | 観測 | ケース 12（閾値判定ロジック）/ drift 数値妥当性は SPEC-LGX-006・TP-LGX-006 委譲 |
| TP-005 §2.9 観点 21 未解決エッジ可視化 | 観測 | SPEC-LGX-002 REQ.11 / SPEC-LGX-004 check へ委譲（観点 7 同根） |
| TP-005 §2.10 観点 22 グローバルフラグ受理 | 境界 API | CLI ディスパッチ / LGX-COMPAT-001 §3 へ委譲 |
| TP-005 §2.10 観点 23 空結果 exit 0 | 境界 API | ケース 4, 17(a) |
| TP-005 §2.10 観点 24 MCP 非公開 | 境界 API | ケース 19 |
| TP-005 §2.11 観点 25 IndexMap 順 visited 決定性 | 領域 | ケース 7 |
| TP-005 §2.11 観点 26 種別間整列規則 | 領域 | SPEC-LGX-002 REQ.08（IndexMap/格納順）へ委譲（消費結果の決定性は ケース 7） |
| TP-005 §2.11 観点 27 custom from→to 方向 | 領域 | ケース 6（from→to 一般則） |
| TP-005 §2.11 観点 28 parent_child 方向 | 領域 | ケース 6 |
| TP-005 §2.11 観点 29 DAG 破れ有限停止 | 領域 | ケース 8 |
| TP-005 §2.11 観点 30 self-loop 停止 | 領域 | ケース 8 |
| TP-005 §2.11 観点 31 種別フィルタ非提供 | 領域 | ケース 6（3 種別常に対象、--kind なしは互換契約 LGX-COMPAT-001 §4 委譲） |
| TP-015 §2.1 BF1 ステップ連鎖整合 | UC フロー | ケース 6→12（BFS→drift→マーク連鎖） |
| TP-015 §2.1 BF2 事前条件↔Step3 前提 | UC フロー | ケース 10, 11（engine.db 存在しても embedding 不在 → 3a 収束） |
| TP-015 §2.1 BF3 事後条件観察可能性 | UC フロー | ケース 9, 14 |
| TP-015 §2.2 AF1 分岐網羅 | UC フロー | ケース 2（max_depth 省略）, 10（3a）, 12（1a 収束） |
| TP-015 §2.2 AF2 代替 1a 収束 | UC フロー | ケース 12（drift_threshold 適用 → 同一出力形式） |
| TP-015 §2.2 AF3 代替 3a 出力差分 | UC フロー | ケース 10（suspicious 空・visited/depth_map 通常通り） |
| TP-015 §2.2 AF4 3a 遷移条件 | UC フロー | ケース 10, 11（db=None / 照会失敗 → 同一 3a 帰結） |
| TP-015 §2.3 EF1 graph 不在・破損 | UC フロー | ケース 17(b)（GraphError → exit 1） |
| TP-015 §2.3 EF2 engine.db 不在↔3a 一貫性 | UC フロー | ケース 10（exit 1 でなく 3a 収束） |
| TP-015 §2.3 EF3 起点 ID 不在 exit 0 | UC フロー | ケース 4 |
| TP-015 §2.4 AT1 アクター権限一貫性 | UC フロー | ケース 9（両アクター read-only 同一権限） |
| TP-015 §2.4 AT2 責任境界（探索 vs 是正） | UC フロー | ケース 14, 15（Step5 返却で終端、是正含まない出力） |
| TP-015 §2.5 DF1 入出力型・stdout/stderr 分離 | UC フロー | ケース 14, 15, 18 |
| TP-015 §2.5 DF2 drift-threshold 優先規則 | UC フロー | CLI 引数解決 / LGX-COMPAT-001 §6 へ委譲（CLI 値 > 設定値、ケース 12 が値適用を確認） |
| TP-015 §2.5 DF3 suspicious 降順・visited 独立 | UC フロー | ケース 13（降順整列）, 7（visited 独立決定性） |
| TP-015 §2.6 R1 逆方向 BFS = to→from | UC フロー | ケース 6 |
| TP-015 §2.6 R2 --max-depth 適用 | UC フロー | ケース 1, 3 |
| TP-015 §2.6 R3 出力 3 フィールド↔REQ.09 | UC フロー | ケース 14, 15（visited/suspicious/depth_map） |
| TP-015 §2.6 R4 終了コード契約 | UC フロー | ケース 17 |
| TP-015 §2.6 R5 MCP 非公開 UC 整合 | UC フロー | ケース 19 |

> 継承 TP の全 51 観点（TP-005 31 + TP-015 20）は本テーブルで TS ケースまたは明示委譲先に mapping 済み（人間ゲート判断対象）。drift 数値妥当性は SPEC-LGX-006/TP-LGX-006、性能は NFR-LGX-001/bench、並行整合は NFR/legixy-db、IndexMap 格納順は SPEC-LGX-002、未解決エッジ報告は SPEC-LGX-004 check へ責務上委譲し、本 TS は legixy-nav の逆方向 BFS 走査・打ち切り可観測性（truncated/excluded_count）・drift ベストエフォート枝刈り・整形（Text/JsonLines）・決定性・read-only・exit 契約に集中する。

## 4. テスト技法選択

- 同値分割: db 状態（None / Some 正常 / Some 照会失敗）、起点（存在 / 不在 / 孤立 / 任意文字列）、max_depth（None / 0 / 中間 / 巨大値）を `Result<T, NavError>` の Ok 系で分割（NavError は実行時失敗のみ、起点不在は Ok）。
- 境界値分析: max_depth 下限 0（ケース 1）/ 無制限 None（ケース 2）/ 打ち切り境界（ケース 3）、drift 閾値 == threshold（ケース 12 で含む）、visited 件数 0/1（ケース 4/5）。
- Property-based: BFS visited 順・depth_map・edges_traversed の決定性（ケース 7）、suspicious_nodes の drift 降順・id 昇順整列の決定性（ケース 13）を不変条件として property 化。
- 状態遷移: BFS 進行（visited 初期化 → レベル進行 → 再訪抑止 → 有限停止）をケース 7/8 で網羅。

## 5. テスト基盤

- 言語: Rust（主 crate `legixy-nav`）
- フレームワーク: cargo test
- Property-based: proptest（ケース 7, 13）
- モック: なし。engine.db は rusqlite の in-memory / tmpfile fixture で代替（drift スコア・不正スキーマを fixture 投入）。グラフは graph.toml fixture からロード or プログラム構築。

## 6. 関連 TC

| TS ケース | 対応 TC | 場所 |
|---|---|---|
| ケース 1 | TC-LGX-NNN | legixy-nav/tests/investigate_depth.rs |
| ケース 2 | TC-LGX-NNN | legixy-nav/tests/investigate_depth.rs |
| ケース 3 | TC-LGX-NNN | legixy-nav/tests/investigate_depth.rs |
| ケース 4 | TC-LGX-NNN | legixy-nav/tests/investigate.rs |
| ケース 5 | TC-LGX-NNN | legixy-nav/tests/investigate.rs |
| ケース 6 | TC-LGX-NNN | legixy-nav/tests/multi_traverser.rs |
| ケース 7 | TC-LGX-NNN | legixy-nav/tests/prop_traversal.rs |
| ケース 8 | TC-LGX-NNN | legixy-nav/tests/multi_traverser.rs |
| ケース 9 | TC-LGX-NNN | legixy-nav/tests/investigate.rs |
| ケース 10 | TC-LGX-NNN | legixy-nav/tests/drift_pruner.rs |
| ケース 11 | TC-LGX-NNN | legixy-nav/tests/drift_pruner.rs |
| ケース 12 | TC-LGX-NNN | legixy-nav/tests/drift_pruner.rs |
| ケース 13 | TC-LGX-NNN | legixy-nav/tests/prop_suspicious.rs |
| ケース 14 | TC-LGX-NNN | legixy-nav/tests/reporter.rs |
| ケース 15 | TC-LGX-NNN | legixy-nav/tests/reporter.rs |
| ケース 16 | TC-LGX-NNN | legixy-nav/tests/reporter.rs |
| ケース 17 | TC-LGX-NNN | legixy-nav/tests/cli_investigate.rs |
| ケース 18 | TC-LGX-NNN | legixy-nav/tests/cli_investigate.rs |
| ケース 19 | TC-LGX-NNN | ts-mcp/test/tools.test.ts（MCP カタログ検査） |
