Document ID: TS-LGX-006

# TS-LGX-006: 順方向探索（impact）のテスト仕様

> TS は **上流 TP の翻訳**。ゼロから観点を考えない。各 TP 観点を DD-LGX-006 で確定した型・関数シグネチャに即した具体的な入力・期待出力・前提条件へ展開する。

**親 DD**: DD-LGX-006（順方向探索 / `legixy-nav`。`impact` / `detect_truncation` / `emit_truncation_info` / `render_multi` / `MultiTraverser::traverse_forward_multi`。`TraversalResult` / `TruncationInfo`）
**親 SPEC**: SPEC-LGX-005（グラフ走査）
**継承 TP**: TP-LGX-005（TP[SPEC] グラフ走査、31 観点）, TP-LGX-016（TP[UC] UC-006 順方向探索フロー、22 観点）。DD-LGX-006 §8 引用の TP-LGX-005 T-GT-001/003/004/005・REQ.04/REQ.09・TP-LGX-016 R3/DF2/AF2/EF1 を含む。

## 1. 対象範囲

このテスト仕様がカバーする DD-LGX-006 の関数 / 型:

- DD-LGX-006 §3 `legixy_nav::impact(graph: &TraceGraph, start_ids: &[String], max_depth: Option<usize>) -> Result<MultiTraversalResult, NavError>`
- DD-LGX-006 §3 `legixy_nav::detect_truncation(graph: &TraceGraph, result: &MultiTraversalResult, max_depth: usize) -> TruncationInfo`
- DD-LGX-006 §3 `legixy_nav::emit_truncation_info(info: &TruncationInfo)`（副作用: stderr）
- DD-LGX-006 §3 `legixy_nav::render_multi(result: &MultiTraversalResult, format: ReportFormat) -> String`（impact 出力面のみ。書式定義の正典は DD-LGX-005、本 TS は impact 結果に対する適用を検証）
- DD-LGX-006 §3 `MultiTraverser::traverse_forward_multi(graph, start_ids, max_depth) -> MultiTraversalResult`
- DD-LGX-006 §2 型: `TraversalResult`{visited, edges_traversed, depth_map}、`TruncationInfo`{truncated, excluded_count, max_depth}、`ReportFormat`{Text, JsonLines}（適用面）、`NavError`{Io, InvalidInput}（参照）。共有型 `VisitedNode` / `MultiTraversalResult` は DD-LGX-005 正典・本 TS は impact 出力の構築結果として参照（`NodeId = String`）

委譲（本 TS 対象外）:
- **共有 BFS 走査ロジック**（`traverse_forward` 単起点 BFS の visited 順・depth_map 構築・再訪抑止・循環停止の純アルゴリズム検証）→ **TS-LGX-005**（DD-005 が `legixy-nav` 共有走査の正典。本 TS は `impact` / `traverse_forward_multi` が同ロジックへ委譲する事実と impact 固有の集約結果を検証）。
- **逆方向走査（`investigate` / `DriftPruner` / drift 値・suspicious nodes）** → TS-LGX-005（DD-LGX-005 管轄、UC-005 所有）。
- **共有型 `VisitedNode` / `MultiTraversalResult` / `NavError` / `ReportFormat` の型不変条件・serde ラウンドトリップの正典検証** → TS-LGX-005（DD-005 §2 正典）。本 TS は impact が構築する具体インスタンスの内容を検証。
- **`render_multi` の Text / JsonLines 書式定義の正典検証** → TS-LGX-005（DD-005 reporter.rs 正典）。本 TS は impact 結果へ適用したときの内容（visited 行・summary）の整合のみ検証。
- **`--max-depth` の値型・範囲（負値・非数値・usize 超え）の構文検証** → `legixy-cli` 引数パーサ層（clap、exit 2）。DD-006 §2.3 が「引数構文誤り → exit 2 は cli 層」と明示。TP-005 観点 3 / TP-016 AF1 と整合。
- **性能予算（PERF.02、ノード1,000+エッジ2,000で<500ms）** → NFR-LGX-001 / criterion bench（DD-006 §8 Bench 行）。
- **並行アクセス整合性（graph.toml 外部同時更新）** → 対象外（DD-006 §7、UC-006 事後条件「読み取り専用」）。NFR REL 系へ委譲。
- **graph.toml ロード失敗の捕捉と exit 1 変換**の cli 層実装 → `legixy-cli`（DD-006 §6、GAP-235）。本 TS はライブラリ層の境界（`impact` が `Result` を返す契約）まで。

本 TS は「impact 固有（`TraversalResult` / `TruncationInfo` / `impact` / `detect_truncation` / `emit_truncation_info` / 順方向多起点マージ）が SPEC-005 の規定を DD-006 の型で正しく具体化しているか」を検証する。共有走査ロジックは TS-005 へ委譲明示する。

## 2. ケース一覧

### ケース 1: `max_depth = Some(0)` → 起点のみ（深度 0）

- **観点出典**: TP-LGX-005 §2.1 観点 1（`max_depth=0` で起点のみ）, TP-LGX-016 §2.6 R3
- **分類**: Unit
- **前提**: `TraceGraph` に起点 `UC-LGX-001`（type=UC）+ chain 後続 `RBA-LGX-001` が存在
- **入力**: `impact(&graph, &["UC-LGX-001".into()], Some(0))`
- **期待**: `Ok(result)` かつ `result.visited == [VisitedNode{ id:"UC-LGX-001", type_code:"UC", path:_, depth:0 }]`。`result.depth_map["UC-LGX-001"] == 0`。後続ノードを含まない
- **境界条件**: 下限 max_depth=0 = 起点 1 件（深度 0、後続レベル切り捨て）

### ケース 2: `max_depth = None`（無制限）と巨大値が一致

- **観点出典**: TP-LGX-005 §2.1 観点 2 / 観点 13（省略時=無制限、互換対象 (d)）
- **分類**: Unit
- **前提**: 線形チェーン `A→B→C→D`（深度 0..3）の `TraceGraph`
- **入力**: `impact(&graph, &["A".into()], None)` と `impact(&graph, &["A".into()], Some(usize::MAX))`
- **期待**: 両者の `visited`（順序含む）・`depth_map` が完全一致。到達可能全 4 ノードを含む
- **境界条件**: 無制限 = 巨大値（usize 上限）で到達集合不変。省略時既定が v3 互換 (d)

### ケース 3: `max_depth` 境界打ち切り（深度 = max_depth まで含む、超過は除外）

- **観点出典**: TP-LGX-005 §2.1 観点 1（境界）, §2.3 観点 10（BFS レベル = depth_map）, DD-006 §8 T-GT-003
- **分類**: Unit
- **前提**: 線形チェーン `A(0)→B(1)→C(2)→D(3)` の `TraceGraph`
- **入力**: `impact(&graph, &["A".into()], Some(2))`
- **期待**: `result.visited` の id 集合 == {A, B, C}（depth 0/1/2）。D（depth 3）を含まない。`depth_map` は A=0, B=1, C=2 のみ
- **境界条件**: 上限 = max_depth のノードは含む、max_depth+1 のノードは除外（境界 + 境界+1 の別ケース化を本ケース内で網羅）

### ケース 4: 存在しない起点 → 空結果・exit 0（NavError に昇格しない）

- **観点出典**: TP-LGX-005 §2.2 観点 6 / §2.7 観点 17 / §2.10 観点 23, TP-LGX-016 §2.2 AF2, DD-006 §8 T-GT-004（REQ.05、GAP-234/ADR-019 確定）
- **分類**: Unit
- **前提**: `TraceGraph` に `XYZ-LGX-999` が存在しない（任意文字列起点も同様に「不在」へ収束）
- **入力**: `impact(&graph, &["XYZ-LGX-999".into()], None)`
- **期待**: `Ok(result)`（`Err(NavError)` ではない）かつ `result.visited.is_empty()` かつ `result.depth_map.is_empty()`。cli 層 exit 0
- **境界条件**: 起点不在 = 空結果（非エラー）。「ERROR を報告する」(UC 2a) は exit 1 ではなく exit 0（GAP-234 で表現矛盾解消）

### ケース 5: 空 `start_ids` → 空結果

- **観点出典**: TP-LGX-005 §2.1 観点 4（空入力境界）, DD-006 §3 不変条件（空 start_ids は空結果）
- **分類**: Unit
- **前提**: 任意の非空 `TraceGraph`
- **入力**: `impact(&graph, &[], None)`
- **期待**: `Ok(result)` かつ `result.visited.is_empty()`、`result.start_ids.is_empty()`
- **境界条件**: 起点列 0 件 = 空集合（境界値の下限）

### ケース 6: 空グラフ / 単一ノード（孤立）→ 起点 1 件相当

- **観点出典**: TP-LGX-005 §2.1 観点 4 / §2.8 観点 18（孤立起点 = 起点 1 件）
- **分類**: Unit
- **前提**: (a) ノード 0・エッジ 0 の `TraceGraph`、(b) 単一ノード `A`・エッジ 0
- **入力**: (a) `impact(&empty, &["A".into()], None)`、(b) `impact(&single, &["A".into()], None)`
- **期待**: (a) 起点不在につき空結果（ケース 4 帰結）。(b) `visited == [VisitedNode{id:"A", depth:0, ...}]`（隣接 0 = 最後の 1 件相当）
- **境界条件**: 孤立起点（出次数 0）= 起点のみ深度 0

### ケース 7: 単起点 impact の委譲（`traverse_forward_multi` → 共有 BFS）

- **観点出典**: TP-LGX-005 §2.3 観点 9/10（visited 初期・BFS レベル）→ **共有ロジックは TS-005 委譲**、本ケースは impact からの委譲経路を検証
- **分類**: Unit
- **前提**: 分岐グラフ `A→B, A→C, B→D`（IndexMap 挿入順 B, C, D）
- **入力**: `impact(&graph, &["A".into()], None)` と `MultiTraverser::traverse_forward_multi(&graph, &["A".into()], None)`
- **期待**: `impact` の `Ok(result)` の `visited` が `traverse_forward_multi` の `visited` と一致（impact は内部で委譲）。visited 順 = BFS（A, B, C, D ＝ 挿入順）
- **境界条件**: impact は `traverse_forward_multi` へ委譲する薄いラッパー（純 BFS 検証は TS-005）

### ケース 8: 多起点マージ（入力順 × 各起点 BFS 順、既出は追加しない）

- **観点出典**: TP-LGX-005 §2.11 観点 25（決定論順序）, TP-LGX-016 §2.1 BF4（複数起点セマンティクス）, DD-006 §8 Unit（多起点マージ）
- **分類**: Unit
- **前提**: グラフ `A→C, B→C, B→E`。起点 `["A", "B"]`
- **入力**: `impact(&graph, &["A".into(), "B".into()], None)`
- **期待**: `visited` = A の BFS（A, C）→ 次に B の BFS（B は新規, C は既出で追加せず, E は新規）→ 結果 [A, C, B, E]。`start_ids == ["A","B"]`。C は既出のため重複なし
- **境界条件**: 多起点 = start_ids 入力順 × 各起点 BFS 順マージ、既出ノードは再追加しない（BF4 確定セマンティクス）

### ケース 9: 多起点での `depth_map` は min(depth) 記録

- **観点出典**: TP-LGX-005 §2.3 観点 11（複数経路の最短深度）, DD-006 §2.1（多起点では min(depth)）
- **分類**: Unit
- **前提**: グラフ `A→X→T`（T は A から深度 2）, `B→T`（T は B から深度 1）。起点 `["A", "B"]`
- **入力**: `impact(&graph, &["A".into(), "B".into()], None)`
- **期待**: `result.depth_map["T"] == 1`（A 経由 2 と B 経由 1 のうち最小）。先に A 経由で 2 を記録しても B 経由 1 で更新（min 保持）
- **境界条件**: 同一ノードへ複数起点/複数深度到達時、depth_map は最短深度（min）を保持

### ケース 10: 不在起点を含む多起点 → 不在分を読み飛ばし

- **観点出典**: TP-LGX-005 §2.2 観点 6 / §2.7 観点 17, DD-006 §3 不変条件（不在起点は読み飛ばし）
- **分類**: Unit
- **前提**: グラフに `A` は存在、`GHOST` は不在。起点 `["A", "GHOST"]`
- **入力**: `impact(&graph, &["A".into(), "GHOST".into()], None)`
- **期待**: `Ok(result)`。`visited` は A 起点の到達集合のみ（GHOST 分は欠落）。`start_ids == ["A","GHOST"]`（入力列は保持）。`Err` に昇格しない
- **境界条件**: 一部不在起点は当該起点のみスキップ、他起点の走査は継続（部分継続）

### ケース 11: 循環グラフでも有限停止（visited 抑止）→ impact 経路

- **観点出典**: TP-LGX-005 §2.2 観点 8 / §2.11 観点 29（DAG 破れ停止）→ **純 BFS 停止は TS-005 委譲**、本ケースは impact が停止結果を返すことを検証
- **分類**: Unit
- **前提**: サイクル `A→B→C→A`（DAG 破れ）。起点 `A`
- **入力**: `impact(&graph, &["A".into()], None)`
- **期待**: 有限時間で `Ok(result)` を返す。`visited` の id 集合 == {A, B, C}（各 1 回、起点 A 再出力なし）。無限ループせず
- **境界条件**: visited による循環防止で有限停止。self-loop（`A→A`）も最小サイクルとして同帰結（停止アルゴリズム正典は TS-005）

### ケース 12: custom / parent_child エッジは from→to 順方向のみ辿る

- **観点出典**: TP-LGX-005 §2.11 観点 27/28（custom/parent_child の from→to 方向）, DD-006 §8 Unit（custom/parent_child エッジ from→to）, REQ.01 一般則 / REQ.08
- **分類**: Unit
- **前提**: グラフに custom エッジ `A→B`, parent_child エッジ `A→A.1`, 逆向き参照用に `Z→A`（custom）。起点 `A`
- **入力**: `impact(&graph, &["A".into()], None)`
- **期待**: `visited` の id 集合 == {A, B, A.1}（from=A の順方向出エッジのみ）。`Z` は含まない（`Z→A` は to=A で逆方向、順方向では辿らない）
- **境界条件**: 全エッジ種別で順方向 = from→to。逆向きエッジ（to=起点）は順方向走査の対象外

### ケース 13: `detect_truncation` 打ち切り発生（境界ノードの未訪問隣接をカウント）

- **観点出典**: TP-LGX-005 §2.1 観点 5（打ち切り可観測性）, TP-LGX-016 §2.6 R3, DD-006 §6 算定法 / §8 Unit（detect_truncation）, REQ.04
- **分類**: Unit
- **前提**: 線形+分岐 `A(0)→B(1)→C(2)`, `C→D`, `C→E`（D/E は深度 3 で除外される）。先に `impact(&graph, &["A".into()], Some(2))` で result（visited={A,B,C}）を得る
- **入力**: `detect_truncation(&graph, &result, 2)`
- **期待**: `TruncationInfo{ truncated:true, excluded_count:2, max_depth:2 }`。境界深度ノード C（`depth_map["C"]==max_depth==2`）の未訪問隣接 D, E の 2 件を集計
- **境界条件**: 境界深度ノード（depth==max_depth）の未訪問隣接数 = excluded_count（DD §6 オプション(b) 近似、到達集合下界推定）

### ケース 14: `detect_truncation` 打ち切りなし → `truncated = false`

- **観点出典**: TP-LGX-005 §2.1 観点 5, DD-006 §8 Unit（truncated=false: 超過ノードなし）, REQ.04
- **分類**: Unit
- **前提**: 線形 `A(0)→B(1)→C(2)`（深度 3 以降のノードが存在しない）。`impact(&graph, &["A".into()], Some(2))` の result（visited={A,B,C}、C は出次数 0）
- **入力**: `detect_truncation(&graph, &result, 2)`
- **期待**: `TruncationInfo{ truncated:false, excluded_count:0, max_depth:2 }`。境界ノード C の未訪問隣接 0 件
- **境界条件**: max_depth 指定でも境界ノードの未訪問隣接 0 件 → truncated=false（到達集合が完全）

### ケース 15: `emit_truncation_info` は truncated 時のみ stderr 出力（stdout・exit 不変）

- **観点出典**: TP-LGX-005 §2.1 観点 5 / §2.9 観点 19, TP-LGX-016 §2.5 DF1 / §2.6 R3, DD-006 §3 emit_truncation_info, REQ.04（v3 差分）
- **分類**: Integration
- **前提**: (a) `TruncationInfo{truncated:true, excluded_count:2, max_depth:2}`、(b) `TruncationInfo{truncated:false, ...}`
- **入力**: それぞれ `emit_truncation_info(&info)`
- **期待**: (a) stderr に `[nav] info: max-depth 2 truncated traversal; 2 reachable node(s) excluded` を 1 件出力。(b) stderr へ何も出力しない。両者とも stdout 不変・終了コード不変（exit 0）
- **境界条件**: 副作用は stderr のみ・truncated=false では無出力。打ち切り Info 出力時も exit 0（REQ.04 終了コード不変）

### ケース 16: `impact` E2E → `render_multi` Text 書式（v3 互換）

- **観点出典**: TP-LGX-005 §2.9 観点 19, TP-LGX-016 §2.1 BF1 / §2.6 R4, DD-006 §3 render_multi（書式正典は DD-005、本ケースは impact 結果への適用）
- **分類**: Integration
- **前提**: graph.toml fixture → ロード → `impact(&graph, &["UC-LGX-001".into()], None)` で result（visited n 件）
- **入力**: `render_multi(&result, ReportFormat::Text)`
- **期待**: visited 各ノードに `{id} (type={t}, depth={d}, path={p})` 形式の行、末尾に `Summary: visited={n}` 行。行順 = `result.visited` の BFS 順
- **境界条件**: Text 既定書式が v3 互換（書式定義検証は TS-005 へ委譲、本ケースは impact 結果への適用整合のみ）

### ケース 17: `impact` E2E → `render_multi` JsonLines 書式（--json 機能化、v3 差分）

- **観点出典**: TP-LGX-005 §2.5 観点 14（--json 機能化）, TP-LGX-016 §2.5 DF2, DD-006 §3 render_multi（JsonLines）, REQ.09
- **分類**: Integration
- **前提**: `impact` 結果 result（visited n 件）
- **入力**: `render_multi(&result, ReportFormat::JsonLines)`
- **期待**: visited 各行が JSON オブジェクト `{"id","type","depth","path"}`、末尾行が `{"summary":{"visited":n}}`。各行が独立した有効 JSON（JSON Lines）
- **境界条件**: --json は受理済みフラグの機能化、引数体系不変。スキーマ検証の正典は TS-005、本ケースは impact 結果への適用整合

### ケース 18: 起点不在の impact 結果に `render_multi` を適用 → 空表示・summary 0

- **観点出典**: TP-LGX-005 §2.1 観点 4 / §2.9 観点 19, TP-LGX-016 §2.2 AF2 / §2.3 EF3
- **分類**: Integration
- **前提**: `impact(&graph, &["GHOST".into()], None)` の空 result
- **入力**: `render_multi(&result, ReportFormat::Text)` と `render_multi(&result, ReportFormat::JsonLines)`
- **期待**: Text は visited 行なし + `Summary: visited=0`。JsonLines は visited 行なし + `{"summary":{"visited":0}}`。エラー文字列を出力しない（空結果は正常）
- **境界条件**: 空結果は ERROR ではなく visited=0 の正常出力（exit 0、AF2 確定）

### ケース 19: BFS visited 順・depth_map の決定論性（property）

- **観点出典**: TP-LGX-005 §2.11 観点 25 / §2.4 観点 12, TP-LGX-016 §2.4 AT3, DD-006 §8 Property-based（REQ.03、NFR REL.05、CTX-INV-1、T-GT-005）
- **分類**: Property-based（proptest）
- **生成器**: 任意の DAG / 非 DAG `TraceGraph`（ノード・エッジ・種別をランダム生成）と任意の `start_ids` 部分集合・任意の `max_depth: Option<usize>`
- **不変条件**: 同一 `(&graph, start_ids, max_depth)` を 2 回 `impact` 呼び出し → `Ok` 同士で `visited`（順序含む）・`depth_map`（IndexMap 挿入順含む）が完全一致。read-only（呼び出し前後で graph 不変）
- **反例ハンドリング**: shrink して visited 順 / depth_map が不一致になる最小グラフを記録
- **境界条件**: 同一入力 → 同一出力（決定論性）。共有走査ロジックの決定論検証の正典は TS-005、本 property は impact 公開 API での決定論保証を確認

### ケース 20: `impact` の read-only 不変（グラフを変更しない、property/integration）

- **観点出典**: TP-LGX-005 §2.6 観点 16, TP-LGX-016 §2.3 EF2 / §2.4 AT1 / §2.5 DF3, DD-006 §5（read-only 借用）
- **分類**: Property/Integration
- **前提**: 任意の `TraceGraph`（成功・空結果いずれも）
- **入力**: `impact(&graph, ..)` 実行前後の graph のハッシュ
- **期待**: 実行前後で graph が不変（`&TraceGraph` 借用のみ、engine.db への書き込みなし）。複数回呼び出しても副作用なし
- **境界条件**: read-only（借用）保証。impact は engine.db 非依存（TP-016 R1）、graph.toml への書き込みなし

### ケース 21: 終了コード契約 0（正常 / 起点不在）の確認（contract）

- **観点出典**: TP-LGX-005 §2.10 観点 23, TP-LGX-016 §2.6 R2 / §2.1 BF3, DD-006 §2.3 終了コード対応, REQ.05 / NFR OBS.05
- **分類**: Contract
- **前提**: (a) 到達ノードあり、(b) 起点不在 = 空結果、(c) 打ち切り Info 出力時
- **入力**: それぞれ cli 層が `impact` を呼んで exit 判定
- **期待**: (a)→exit 0、(b)→exit 0（空結果は非エラー、GAP-234）、(c)→exit 0（打ち切り Info でも不変、REQ.04）。引数構文誤り exit 2・graph ロード失敗 exit 1 は cli 層所有（→ 委譲）
- **境界条件**: 正常完了（空結果含む）= exit 0。exit 1（ロード失敗）/ exit 2（構文）は cli 層・本 TS 対象外

### ケース 22: `impact` E2E（graph.toml fixture）→ visited 正確・打ち切り stderr・--json

- **観点出典**: TP-LGX-016 §2.1 BF1 / §2.6 R3 / §2.5 DF2（DD-006 §8 Integration impact E2E）
- **分類**: Integration
- **前提**: graph.toml fixture（chain + custom + parent_child を含む）をロード
- **入力**: (a) `impact(&graph, &["UC-LGX-001".into()], None)`、(b) `impact(.., Some(2))` で打ち切り発生 → `detect_truncation` → `emit_truncation_info`、(c) `render_multi(.., JsonLines)`
- **期待**: (a) visited が fixture の到達集合と一致（stdout）。(b) 打ち切り発生時 stderr に Info 1 件、stdout の visited は深度境界まで。(c) JsonLines スキーマ準拠
- **境界条件**: E2E でフロー連鎖（Step1 受理 → Step2 BFS → Step3 打ち切り → Step4 返却）と stdout/stderr 分離を観察

### ケース 23: `edges_traversed` の内容直接 assert（spanning-tree・初訪問エッジのみ・グラフ向き (from,to)・多起点マージ蓄積）

- **観点出典**: TP-LGX-005 §2.9 観点 19（出力情報に edge）, TP-LGX-016 §2.6 R4（使用エッジ情報）, DD-006 §2.1 `MultiTraversalResult.edges_traversed`（spanning tree エッジ (from, to) グラフ向き・初訪問を生んだエッジのみ・多起点では min(depth) 経路で蓄積）
- **分類**: Unit
- **前提**: 固定グラフ fixture `A→B, B→C, A→C`（IndexMap 挿入順 = A の出エッジ `A→B` が `A→C` より先、`B→C` は B の出エッジ）。`C` には A 経由（深度 2: A→B→C）と A 直接（深度 1: A→C）の 2 経路があるが、BFS は深度 1 の `A→C` を初訪問エッジとし、`B→C`(深度 2 経路) は C が既訪問のため spanning-tree に含めない。さらに多起点マージ検証用に第 2 起点 `D`（`D→E`）を持つ拡張 fixture を併用
- **入力**: (a) `impact(&graph, &["A".into()], None)`、(b) `impact(&graph2, &["A".into(), "D".into()], None)`（graph2 = `A→B, B→C, A→C, D→E`）
- **期待**:
  - (a) `result.edges_traversed == vec![("A".to_string(),"B".to_string()), ("A".to_string(),"C".to_string())]`。初訪問を生んだエッジのみ（B は `A→B` で初訪問、C は深度 1 の `A→C` で初訪問）。既訪問 C への重複エッジ `B→C` は含まない。各タプルはグラフ向き `(from, to)`（探索向き = グラフ向き、逆転しない）
  - (b) `result.edges_traversed == vec![("A".to_string(),"B".to_string()), ("A".to_string(),"C".to_string()), ("D".to_string(),"E".to_string())]`。第 1 起点 A の spanning-tree エッジに続いて第 2 起点 D の初訪問エッジ `D→E` を**蓄積**（多起点マージ時、各起点の初訪問エッジを入力順で連結。既訪問ノードへ向かうエッジは追加しない）
- **境界条件**: `edges_traversed` は初訪問エッジのみの spanning-tree（visited ノード数 − 起点数 ＝ エッジ数、森の場合）。タプルはグラフ向き `(from, to)` で、逆向きにしない。**`edges_traversed` は `MultiTraversalResult` の内部保持フィールドであり、`render_multi`（Text / JsonLines）の出力には一切現れない**（DD-006 §2.1「Text/JSON 出力には含まれない（内部保持のみ）」、v3 実測）。よって本ケースは render 出力ではなく構築された結果オブジェクトのフィールドを直接 assert する

## 3. 観点カバレッジ表

### 3.1 TP-LGX-005（TP[SPEC] グラフ走査、31 観点）

| TP § | 観点 | カバーする TS ケース |
|---|---|---|
| TP-005 §2.1 観点 1 max_depth=0 起点のみ | 境界値 | ケース 1 |
| TP-005 §2.1 観点 2 無制限=巨大値 | 境界値 | ケース 2 |
| TP-005 §2.1 観点 3 --max-depth 不正値 | 境界値 | cli 引数パーサ層（exit 2）へ委譲 |
| TP-005 §2.1 観点 4 空グラフ/単一/孤立起点 | 境界値 | ケース 5, 6 |
| TP-005 §2.1 観点 5 打ち切り可観測性 | 境界値 | ケース 13, 14, 15 |
| TP-005 §2.2 観点 6 不在起点=空・exit 0 | エラー | ケース 4, 10 |
| TP-005 §2.2 観点 7 全エッジ未解決除外 | エラー | SPEC-LGX-002 REQ.11 / check（TS-LGX-002 系）へ委譲（走査は解決済み部分グラフ上） |
| TP-005 §2.2 観点 8 panic/unwrap 非残留 | エラー | ケース 11（停止性）/ 純 BFS safety は TS-005 委譲 |
| TP-005 §2.3 観点 9 visited 初期・再訪抑止 | 状態 | ケース 7, 11（impact 経路）/ 共有ロジック正典は TS-005 |
| TP-005 §2.3 観点 10 BFS レベル=depth_map | 状態 | ケース 1, 3 |
| TP-005 §2.3 観点 11 複数経路の最短深度 | 状態 | ケース 9（多起点 min depth） |
| TP-005 §2.4 観点 12 読み取り専用・並行更新 | 並行 | ケース 19, 20（read-only）/ 並行整合性は NFR 委譲 |
| TP-005 §2.5 観点 13 max_depth 省略=無制限 互換 | 互換 | ケース 2 |
| TP-005 §2.5 観点 14 --json 機能化 互換 | 互換 | ケース 17 |
| TP-005 §2.5 観点 15 サブコマンド名/引数 互換 | 互換 | cli 層 / LGX-COMPAT-001 §4 へ委譲（CLI ディスパッチは DD-006 境界外）|
| TP-005 §2.6 観点 16 書き込み非伴 | 永続化 | ケース 20（read-only）|
| TP-005 §2.7 観点 17 起点 ID 形式検証 | 入力 | ケース 4, 10（任意文字列起点=不在へ収束）|
| TP-005 §2.8 観点 18 孤立起点=起点 1 件 | ライフ | ケース 6(b) |
| TP-005 §2.9 観点 19 出力情報（順/type/path/depth/edge）| 観測 | 順/type/path/depth は render 出力＝ケース 16, 17, 18。edge（edges_traversed 内容）は構築結果の直接 assert＝ケース 23（render 出力には現れない内部保持フィールド。DD-006 §2.1） |
| TP-005 §2.9 観点 20 suspicious/drift 出力 | 観測 | 逆方向 investigate（TS-LGX-005 / SPEC-006）へ委譲 |
| TP-005 §2.9 観点 21 未解決エッジ可視化 | 観測 | SPEC-LGX-002 REQ.11 / check へ委譲（観点 7 同根）|
| TP-005 §2.10 観点 22 global オプション受理 | 境界 API | cli 層 / LGX-COMPAT-001 §3 へ委譲 |
| TP-005 §2.10 観点 23 不在/空結果 exit 0 | 境界 API | ケース 4, 21 |
| TP-005 §2.10 観点 24 MCP 非公開 | 境界 API | MCP ツール一覧検証（SPEC-LGX-009 系 TP-009）へ委譲（REQ.10、走査は Admin 限定）|
| TP-005 §2.11 観点 25 IndexMap 順・決定論 | 領域 | ケース 7, 8, 19 |
| TP-005 §2.11 観点 26 種別混在エッジ整列 | 領域 | SPEC-LGX-002 REQ.08（IndexMap 挿入順）へ委譲 / ケース 12（消費側）|
| TP-005 §2.11 観点 27 custom from→to 方向 | 領域 | ケース 12 |
| TP-005 §2.11 観点 28 parent_child 順方向 | 領域 | ケース 12 |
| TP-005 §2.11 観点 29 DAG 破れ停止 | 領域 | ケース 11 |
| TP-005 §2.11 観点 30 self-loop 停止 | 領域 | ケース 11（境界条件で self-loop 言及）|
| TP-005 §2.11 観点 31 エッジ種別フィルタ非提供 | 領域 | cli 互換契約（--kind 等なし、LGX-COMPAT-001 §4）へ委譲・意図的 |

### 3.2 TP-LGX-016（TP[UC] UC-006 順方向探索フロー、22 観点）

| TP § | 観点 | カバーする TS ケース |
|---|---|---|
| TP-016 §2.1 BF1 ステップ連鎖（Step4→事後条件 出力）| UC フロー | ケース 16, 22 |
| TP-016 §2.1 BF2 入力検証の段階区分 | UC フロー | ケース 4（意味層不在）/ 構文層 exit 2 は cli 委譲 |
| TP-016 §2.1 BF3 成功時 exit 0 | UC フロー | ケース 21 |
| TP-016 §2.1 BF4 start_ids 複数起点セマンティクス | UC フロー | ケース 8, 9 |
| TP-016 §2.2 AF1 --max-depth 不正値パス | UC フロー | cli 引数パーサ層（exit 2）へ委譲 |
| TP-016 §2.2 AF2 2a の exit コード（空結果=exit 0）| UC フロー | ケース 4, 18, 21 |
| TP-016 §2.2 AF3 代替フロー遷移条件 | UC フロー | ケース 2（1a 無制限）, 4（2a 不在）|
| TP-016 §2.3 EF1 graph.toml 不在/破損パス | UC フロー | cli 層（exit 1、GAP-235）へ委譲（impact は Result 契約まで、§1 委譲）|
| TP-016 §2.3 EF2 エラー時状態の read-only | UC フロー | ケース 20 |
| TP-016 §2.3 EF3 エラー通知の報告先 | UC フロー | ケース 18（空結果=正常出力）/ stderr Info はケース 15 |
| TP-016 §2.4 AT1 アクター権限一貫 | UC フロー | ケース 20（read-only で両アクター同一権限）|
| TP-016 §2.4 AT2 責任境界（探索 vs 是正）| UC フロー | N/A（システム=特定のみ、是正はアクター責務 — TS 検証対象外の設計原則）|
| TP-016 §2.4 AT3 実行中外部更新整合性 | UC フロー | ケース 19, 20（メモリ内読み取り）/ 並行整合性は NFR 委譲 |
| TP-016 §2.5 DF1 stderr 出力言及 | UC フロー | ケース 15（打ち切り Info stderr）, 22（stdout/stderr 分離）|
| TP-016 §2.5 DF2 出力フォーマット多態性 --json | UC フロー | ケース 16, 17, 22 |
| TP-016 §2.5 DF3 エラー時データ解放 | UC フロー | ケース 20（メモリ上のみ、解放 N/A）|
| TP-016 §2.6 R1 engine.db 非依存 | 領域 | ケース 20（engine.db 非依存）/ DD-006 §4 依存方向で構造保証 |
| TP-016 §2.6 R2 終了コード契約 | 領域 | ケース 21（AF2/BF3 同根）|
| TP-016 §2.6 R3 打ち切り stderr Info | 領域 | ケース 13, 15, 22 |
| TP-016 §2.6 R4 使用エッジ情報の出力 | 領域 | edges_traversed の内容（初訪問 spanning-tree エッジ・グラフ向き (from,to)・多起点蓄積）の直接 assert＝ケース 23。**edges_traversed は内部保持で render_multi（Text/JsonLines）出力には現れない**（DD-006 §2.1）ため、render 委譲（ケース 16/17）ではなく構築結果を直接検証する |
| TP-016 §2.6 R5 global オプション受理 | 領域 | cli 層 / LGX-COMPAT-001 §3 へ委譲 |

> 継承 TP 観点（TP-005 31 件 + TP-016 22 件）はすべて本テーブルで TS ケースまたは明示委譲先に mapping 済み（人間ゲート判断対象）。本 TS は impact 固有（`impact` / `detect_truncation` / `emit_truncation_info` / `TraversalResult` / `TruncationInfo` / 順方向多起点マージ）の finding 化に集中し、共有 BFS 走査ロジック（純アルゴリズムの決定論・停止・visited）の正典検証は TS-LGX-005 へ、書式・共有型の正典は DD-005 由来として TS-LGX-005 へ、性能は NFR / bench へ、CLI 引数構文・global フラグ・終了コード変換は `legixy-cli` 層 / LGX-COMPAT-001 へ委譲する。

## 4. テスト技法選択

- 同値分割: 起点（到達ノードあり / 不在 / 孤立 / 空 start_ids）、max_depth（0 / 有限 / None / 巨大値）、エッジ種別（chain / custom / parent_child / 逆向き）
- 境界値分析: max_depth=0（下限）/ max_depth=境界深度（含む）/ max_depth+1（除外）/ None=無制限。excluded_count=0（打ち切りなし）/ >0（あり）
- Property-based: 同一入力 → 同一 visited 順・depth_map（決定論性、ケース 19）、impact 前後の graph 不変（read-only、ケース 20）
- 状態遷移: BFS レベル進行 = depth_map（ケース 3）、多起点マージの既出判定（ケース 8）
- 委譲明示: 純 BFS 走査・共有型・書式の正典は TS-LGX-005、CLI 構文/exit 変換は cli 層、性能は NFR/bench

## 5. テスト基盤

- 言語: Rust（CLI 本体、crate `legixy-nav`）
- フレームワーク: cargo test
- Property-based: proptest（ケース 19 決定論性 / ケース 20 read-only）
- モック: なし（`TraceGraph` は in-memory fixture を直接構築。E2E は graph.toml fixture をロード）

## 6. 関連 TC

| TS ケース | 対応 TC | 場所 |
|---|---|---|
| ケース 1〜12, 23 | TC-LGX-006（impact / traverse_forward_multi Unit。ケース 23 = edges_traversed 内容 assert）| crates/legixy-nav/src/impact.rs（#[cfg(test)]）/ tests/ |
| ケース 13, 14 | TC-LGX-006（detect_truncation Unit）| crates/legixy-nav/src/reporter.rs（#[cfg(test)]）|
| ケース 15 | TC-LGX-006（emit_truncation_info Integration）| crates/legixy-nav/tests/truncation.rs |
| ケース 16, 17, 18 | TC-LGX-006（render_multi Integration）| crates/legixy-nav/tests/render.rs |
| ケース 19, 20 | TC-LGX-006（property）| crates/legixy-nav/tests/prop_impact.rs |
| ケース 21, 22 | TC-LGX-006（contract / E2E）| crates/legixy-cli/tests/impact_e2e.rs |
