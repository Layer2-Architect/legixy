Document ID: DD-LGX-006

# DD-LGX-006: 順方向探索（impact）の詳細設計

**親 SEQD**: SEQD-LGX-006
**親 RBD**: RBD-LGX-006 / **親 UC**: UC-LGX-006
**対象言語**: Rust（CLI 本体）
**境界 API 凍結**: yes（本ドキュメント確定後、crate 間公開 API は変更不可。追加は許容、削除・改名・型変更は次版 SPEC 改訂。HR7）

> 言語別規律は `guides/language-stacks/rust.md`。crate 境界・共有型は **ADR-LGX-020** で凍結済（本 DD は参照する）。型・シグネチャは v3 実装（`traceability-engine.v3.chg_to_lexigy` の `crates/lx-nav/`、`crates/lx-graph/src/traversal.rs`）に整合させ引数互換を保つ。

> **DD-005（UC-LGX-005 逆方向探索）との関係**: 本 DD は `legixy-nav` crate の共有型（`VisitedNode` / `MultiTraversalResult` / `NavError` / `ReportFormat` / `MultiTraverser` / `render_multi`）を含む。DD-LGX-005 が先に起草された場合、§2.1〜§2.3・§3・§4 の共有型・関数については DD-LGX-005 を正典とし本 DD は参照とする。本 DD を先に起草するため完全定義を記載する。

## 1. 対象範囲

- **主 crate**: `legixy-nav`（BFS 走査・多起点ラッパー・打ち切り判定・Reporter）
- **依存 crate（共有型は ADR-LGX-020、再定義しない）**: `legixy-graph`（`TraceGraph` / `Node` / `Edge`）, `legixy-core`（`Id` / 共通エラー / 設定型）
- **公開 API surface**: 本 DD §3（`legixy-nav` の `impact` 関数・Reporter・結果型）
- **関連 SEQD**: SEQD-LGX-006
- **本 DD が扱う機能**: `impact <node-id> [--max-depth <N>]`（順方向 BFS 走査）。`legixy-cli` 層のディスパッチ・引数パースは `legixy-cli` が担う（本 DD の境界外）

## 2. 型定義

### 2.1 主要データ型

```rust
// legixy-graph（共有、ADR-LGX-020）— 再定義しない
// pub struct TraceGraph { ... }  // IndexMap<NodeId, Node> + edges Vec<Edge>
// pub type NodeId = String;

// legixy-nav — impact 専用結果型
/// 単起点 BFS の生結果（legixy-graph 層）。
/// legixy-graph::traversal が返す内部型。
pub struct TraversalResult {
    pub visited: Vec<NodeId>,                 // BFS 発見順（起点含む）
    pub edges_traversed: Vec<(NodeId, NodeId)>, // spanning tree エッジ (from, to) グラフ向き
    pub depth_map: HashMap<NodeId, usize>,    // 起点からの BFS 距離（起点=0）
}

/// visited ノードの要約情報（出力用）。
/// v3 対応: lx-nav/src/result.rs VisitedNode と 1:1。
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct VisitedNode {
    pub id: NodeId,
    pub type_code: String, // SPEC-LGX-002 REQ.04: ノードのタイプコード
    pub path: String,      // graph.toml 記載パス
    pub depth: usize,      // 起点からの BFS 距離
}

/// 多起点走査の集約結果（impact の公開出力型）。
/// v3 対応: lx-nav/src/result.rs MultiTraversalResult と 1:1。
/// serde 対応（--json 出力、SPEC-LGX-005.REQ.09）。
#[derive(Debug, Clone, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct MultiTraversalResult {
    /// BFS 発見順の visited ノード一覧（REQ.03 決定論保証）。
    /// 多起点: start_ids の入力順 × 各起点の BFS 順でマージ（既出は追加しない）。
    pub visited: Vec<VisitedNode>,
    /// spanning tree エッジ (from, to) グラフ向き。初訪問を生んだエッジのみ。
    /// v3 実測: Text/JSON 出力には含まれない（内部保持のみ）。
    pub edges_traversed: Vec<(NodeId, NodeId)>,
    /// 各ノードの起点からの最短距離。多起点では min(depth) で記録。
    /// IndexMap（挿入順保持）で決定論的出力を保証（REQ.03）。
    pub depth_map: IndexMap<NodeId, usize>,
    /// 入力起点 ID 列（Reporter 出力・serde 用）。
    pub start_ids: Vec<NodeId>,
}

/// 打ち切り情報（REQ.04 打ち切り可観測性、GAP-LGX-085 対応、v3 差分）。
/// --max-depth 指定かつ深度超過ノードが 1 件以上ある場合のみ生成される。
#[derive(Debug, Clone)]
pub struct TruncationInfo {
    /// 打ち切り発生フラグ（true = 除外ノードあり）。
    pub truncated: bool,
    /// --max-depth 境界ノードから出た未訪問隣接ノード数（近似値、算定法は §6 参照）。
    pub excluded_count: usize,
    /// 使用した max_depth 値（stderr Info メッセージ生成用）。
    pub max_depth: usize,
}
```

### 2.2 列挙 / Sum 型

```rust
// legixy-nav

/// Reporter の出力形式。
/// v3 対応: lx-nav/src/reporter.rs ReportFormat と 1:1。
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ReportFormat {
    #[default]
    Text,       // 既定。人間可読テキスト（v3 互換書式）
    JsonLines,  // --json 指定時。JSON Lines（SPEC-LGX-005.REQ.09 機能化、v3 差分）
}
```

### 2.3 エラー型

```rust
// legixy-nav（走査実行時エラー。起点不在・空グラフは NavError ではなく空結果）
#[derive(Debug, thiserror::Error)]
pub enum NavError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("invalid input: {0}")]
    InvalidInput(String),
}
```

- 終了コードの対応は `legixy-cli` 層が担う（LGX-COMPAT-001 §3）:
  - 引数構文誤り（clap） → exit 2
  - `NavError` 発生（グラフロード失敗等） → exit 1
  - 正常完了（起点不在 = 空結果含む） → exit 0（REQ.05）
  - 打ち切り Info 出力時も exit 0（REQ.04 終了コード不変）
- 起点ノードが存在しない場合は `NavError` に昇格しない → 空 `MultiTraversalResult` を返す（REQ.05、SPEC-LGX-005.REQ.05、ADR-019/GAP-234 で確定）。
- グラフロード失敗（graph.toml 不在・破損）は `legixy-cli` 層が `NavError::Io` / `legixy-graph::GraphError` を受け exit 1 として処理する（GAP-LGX-235 の DD 対処）。

## 3. 公開 API surface（凍結、HR7）

| 関数 | シグネチャ | 不変条件 | 冪等性 | 同期/非同期 |
|---|---|---|---|---|
| `legixy_nav::impact` | `fn impact(graph: &TraceGraph, start_ids: &[String], max_depth: Option<usize>) -> Result<MultiTraversalResult, NavError>` | 同一入力 → 同一 `visited` 順・`depth_map`（REQ.03 決定論性）。read-only（graph を変更しない）。`start_ids` に不在 ID があれば読み飛ばし（REQ.05）。空 `start_ids` は空結果 | yes | 同期 |
| `legixy_nav::render_multi` | `fn render_multi(result: &MultiTraversalResult, format: ReportFormat) -> String` | Text: `{id} (type={t}, depth={d}, path={p})` 各行 + `Summary: visited={n}` 末尾（v3 互換）。JsonLines: visited 各行 `{"id","type","depth","path"}` + `{"summary":{"visited":n}}` 末尾（v3 reporter.rs と 1:1） | yes | 同期 |
| `legixy_nav::emit_truncation_info` | `fn emit_truncation_info(info: &TruncationInfo)` | `info.truncated` のとき stderr へ `[nav] info: max-depth {N} truncated traversal; {k} reachable node(s) excluded` を出力（REQ.04 打ち切り可観測性、v3 差分）。stdout・終了コード不変 | yes（副作用: stderr） | 同期 |
| `legixy_nav::detect_truncation` | `fn detect_truncation(graph: &TraceGraph, result: &MultiTraversalResult, max_depth: usize) -> TruncationInfo` | `result` の境界深度ノード（`depth_map[id] == max_depth`）の未訪問隣接ノード数を集計して `excluded_count` とする（§6 参照）。stdout 不変・走査コスト小 | yes | 同期 |
| `MultiTraverser::traverse_forward_multi` | `fn traverse_forward_multi(graph: &TraceGraph, start_ids: &[String], max_depth: Option<usize>) -> MultiTraversalResult` | `impact` が委譲する内部ラッパー。start_ids 入力順 × BFS 順マージ。不在起点は読み飛ばし | yes | 同期 |

- `start_ids` は CLI からは常に 1 要素（位置引数 `<start>` が 1 個）。複数起点はライブラリ層 API として許容（SUPP §2.1 互換制約確認）。CLI インターフェース（引数名・位置引数個数）は LGX-COMPAT-001 §4 #11 で凍結済。
- `--json` 対応は `legixy-cli` 層が `ReportFormat` を選択して `render_multi` へ渡す（v3 では CLI 層で未伝播 → legixy で機能化、SPEC-LGX-005.REQ.09 v3 差分）。

## 4. module / package 構成

```
legixy-nav/
├── src/
│   ├── lib.rs              // 公開 re-export（impact / investigate / MultiTraverser /
│   │                       //   render_multi / render_pruned / ReportFormat /
│   │                       //   MultiTraversalResult / PrunedTraversalResult /
│   │                       //   VisitedNode / SuspiciousNode / NavError /
│   │                       //   emit_truncation_info / detect_truncation / TruncationInfo）
│   ├── impact.rs           // Document ID: SRC-LGX-006（impact 関数）。traverse_forward_multi 委譲
│   ├── investigate.rs      // UC-LGX-005（逆方向探索）の実装（DD-LGX-005 管轄）
│   ├── multi_traverser.rs  // MultiTraverser（traverse_forward_multi / traverse_reverse_multi）
│   ├── result.rs           // VisitedNode / MultiTraversalResult / SuspiciousNode /
│   │                       //   PrunedTraversalResult（UC-005/006 共有）
│   ├── reporter.rs         // render_multi / render_pruned / ReportFormat /
│   │                       //   emit_truncation_info / detect_truncation（UC-005/006 共有）
│   ├── drift_pruner.rs     // DriftPruner（逆方向専用、DD-LGX-005 管轄）
│   └── error.rs            // NavError
└── Cargo.toml
```

依存方向（DAG、ADR-LGX-020）: `legixy-nav` → `legixy-graph` / `legixy-core`。`legixy-db` は `investigate` の DriftPruner のみが使用（`impact` は engine.db 非依存、TP-016 観点 R1）。循環禁止。`legixy-cli` → `legixy-nav`（ディスパッチ層）。

## 5. ライフタイム / 所有権 / 借用 方針

- `impact` は `&TraceGraph` を**借用**（所有権を取らない。read-only、複数コマンドで共有可）。
- `MultiTraversalResult` は所有を返す（呼び出し側が Reporter・exit 判定に使う）。
- `NodeId`（= `String`）は clone を許容。`MultiTraversalResult.visited` は `VisitedNode` を所有。`depth_map` の `IndexMap` も所有。
- `TruncationInfo` は `detect_truncation` が所有を返し、`emit_truncation_info` が `&TruncationInfo` で借用。
- `'static` バウンド不要（全処理は呼び出しスコープ内で完結）。`Arc` / `Mutex` 不要（単一スレッド逐次、§7）。
- `start_ids: &[String]`（スライス借用）。`to_vec()` で内部保持（`MultiTraversalResult.start_ids`）。

## 6. エラー伝播戦略

- `impact` が `Result<MultiTraversalResult, NavError>` を返す。現 v3 実装では `NavError` を返す経路は事実上ない（空起点・不在起点は空結果扱い）が、将来の I/O 起因エラーのために `Result` を維持する。
- グラフロード失敗は `legixy-cli` 層の `run` 関数で捕捉し exit 1 → stderr エラー出力（GAP-LGX-235 対処。`legixy-graph::GraphError` を `NavError::Io` に変換しない — crate 境界で `anyhow` 経由処理）。
- panic 禁止（rust.md §4）: `multi_traverser.rs:67` の `expect("lx_graph visited IDs are guaranteed to exist in graph")` は `legixy-graph` の不変条件（走査した ID は graph に存在する）に依存する内部 unwrap。この 1 箇所のみ例外（前提違反は実装バグで不変条件破れ → panic が適切）。
- 打ち切り情報（`TruncationInfo`）はエラーではなく副産物。`detect_truncation` は panic なし・純関数。

**打ち切り除外ノード件数の算定法（SUPP §2.5 [要決定] 解決）**: 境界深度ノード（`depth_map[id] == max_depth`）の各未訪問隣接ノード数を集計する（近似値、オプション(b) 採用）。走査コストは境界ノード数 × 平均出次数であり、「打ち切らない場合」の全探索コストより大幅に小さい。`excluded_count` の意味は「境界から見えた未訪問隣接ノード数（到達集合の下界推定）」と文書化する。

## 7. 並行性 / 非同期境界

- `impact` は**同期・単一スレッド・read-only**。async なし。
- 多起点マージは逐次ループ（`for start in start_ids`）。並列化は将来最適化。
- `emit_truncation_info` は stderr への同期書き込み（`eprintln!`）。
- 並行アクセス（graph.toml の外部同時更新）整合性は対象外（UC-006 事後条件「グラフの状態は変更されない（読み取り専用操作）」）。

## 8. テスト分類

| 分類 | 内容 | 対応 TP |
|---|---|---|
| Unit | `traverse_forward` の単起点 BFS: 線形チェーン順（T-GT-001）、max_depth 境界（T-GT-003）、不在起点=空結果（T-GT-004、REQ.05）、循環グラフ停止（REQ.06）、custom/parent_child エッジ from→to 方向のみ（REQ.01 一般則）| TP-LGX-005 T-GT-001/003/004/005 |
| Unit | `MultiTraverser::traverse_forward_multi`: 単起点委譲・多起点マージ（入力順 × BFS 順）・min(depth) 記録・不在起点読み飛ばし | TP-LGX-005 |
| Unit | `detect_truncation`: 境界ノード隣接カウント・`truncated=false`（max_depth 未指定 or 超過ノードなし） | TP-LGX-005 REQ.04 |
| Unit | `render_multi`: Text 書式（v3 互換）・JsonLines 書式（`{"id","type","depth","path"}` 各行 + `{"summary":...}` 末尾） | TP-LGX-005 REQ.09 |
| Integration | `impact` E2E: `graph.toml` fixture → `impact(&graph, &["UC-LGX-001"], None)` → visited 正確 / `--max-depth 2` → 打ち切り発生時 stderr Info / `--json` → JsonLines スキーマ | TP-LGX-016 R3/DF2 |
| Integration | 存在しない起点 → visited 空・exit 0（REQ.05、GAP-234 確定）| TP-LGX-016 AF2 |
| Integration | graph.toml 不在 → exit 1（GAP-235 確定） | TP-LGX-016 EF1 |
| Property-based | `MultiTraversalResult.visited` の決定論性: 同一 `(&graph, start_ids, max_depth)` を 2 回呼び出して `visited` 順・`depth_map` が一致（REQ.03、proptest） | TP-LGX-005 T-GT-005 |
| Bench | グラフ ノード1,000+エッジ2,000 での `impact` 応答時間（NFR PERF.02、criterion） | NFR-LGX-001 |

## 9. ADR への参照

- **ADR-LGX-020**: crate 分割・共有型凍結（本 DD の crate 境界。`legixy-nav` = ADR-020 §2.1 表 6 行目）
- ADR-LGX-014: SPEC 準拠原則
- ADR-LGX-019: UC-LGX-006 GAP 裁定（起点不在=空結果・exit 0 確定。GAP-LGX-234 解消）
- ADR-LGX-002: embed `--node`/`--force` フラグ追加前例（legixy で新フラグを追加する場合の手順参照）
- （参考）GAP-LGX-234: 起点不在 exit 0 確定（SPEC-LGX-005.REQ.05 準拠、本 DD §2.3 に反映）
- （参考）GAP-LGX-235: graph.toml 不在 → exit 1（本 DD §6 に反映）
- （参考）GAP-LGX-236: 打ち切り stderr Info（本 DD §2.1 TruncationInfo・§3 emit_truncation_info に反映）

## 10. 関連 NFR

- NFR-LGX-001.PERF.02: impact 性能予算（ノード1,000+エッジ2,000で<500ms）
- NFR-LGX-001.OBS.02: 出力先（visited 結果=stdout / ログ・Info=stderr）
- NFR-LGX-001.OBS.05: 終了コード（0/1/2）
- NFR-LGX-001.REL.05: BFS 走査決定性（CTX-INV-1: 同一入力→同一 visited 順。T-GT-005）

## 11. 凍結履歴

| Date | Version | Note |
|---|---|---|
| 2026-06-13 | 1.0 | 初版凍結。`legixy-nav` の `impact` / `render_multi` / `emit_truncation_info` / `detect_truncation` 公開 API と `VisitedNode` / `MultiTraversalResult` / `TruncationInfo` / `NavError` / `ReportFormat` 型を確定（v3 lx-nav 整合）。打ち切り可観測性（TruncationInfo・emit_truncation_info、v3 差分）と --json 機能化（JsonLines、v3 差分）を新設。起点不在=空結果・exit 0 確定（GAP-234/SPEC-005.REQ.05）。crate 境界は ADR-LGX-020。HR7 凍結 |
