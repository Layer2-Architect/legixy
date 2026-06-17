Document ID: DD-LGX-005

# DD-LGX-005: 逆方向探索（investigate）の詳細設計

**親 SEQD**: SEQD-LGX-005
**親 RBD**: RBD-LGX-005 / **親 UC**: UC-LGX-005
**対象言語**: Rust（CLI 本体）
**境界 API 凍結**: yes（本ドキュメント確定後、crate 間公開 API は変更不可。追加は許容、削除・改名・型変更は次版 SPEC 改訂。HR7）

> 言語別規律は `guides/language-stacks/rust.md`。crate 境界・共有型は **ADR-LGX-020** で凍結済（本 DD は参照する）。型は v3 実装（traceability-engine.v3 `crates/lx-nav`）に整合させ引数互換を保つ。

## 1. 対象範囲

- **主 crate**: `legixy-nav`（逆方向 BFS 走査・ドリフトスコア評価・suspicious_nodes 整列・走査結果整形・Reporter）
- **依存 crate（共有型は ADR-LGX-020、再定義しない）**:
  - `legixy-graph`（`TraceGraph` / `Node` / `Edge` / `traverse_reverse` / `TraversalResult`）
  - `legixy-db`（`Connection` 参照 — rusqlite 経由。scores テーブル drift スコア照会）
  - `legixy-core`（`Id` / `Config` / `ConfigError` / `Severity` 基底）
- **公開 API surface**: 本 DD §3（`legixy-nav` の crate 公開関数・型）
- **関連 SEQD**: SEQD-LGX-005（逆方向探索コマンド受付窓口 → 逆方向探索統括処理 → 走査・ドリフト評価・整形の全フロー）

## 2. 型定義

### 2.1 主要データ型

```rust
// legixy-graph（共有、ADR-LGX-020）
// TraceGraph: ノード・エッジを保持する有向グラフ（ADR-LGX-020 参照、再定義しない）
// TraversalResult: 単起点 BFS の生結果 { visited: Vec<NodeId>, edges_traversed, depth_map: HashMap<NodeId,usize> }
// 注: 本 DD の String フィールドは legixy-graph の NodeId（= String、ADR-LGX-021 §2.1）と同一型。
//     legixy-nav 共有走査型（VisitedNode / MultiTraversalResult / NavError / ReportFormat）は本 DD が正典、DD-LGX-006 は参照（ADR-LGX-021 §2.2）。

// legixy-nav（本 DD で定義する型）
pub struct VisitedNode {
    pub id: String,              // ノード識別子（legixy-graph NodeId = String と同一、ADR-LGX-021）
    pub type_code: String,       // ノードタイプコード（例: "UC", "DD", "SRC"）
    pub path: String,            // ノードファイルパス（graph.toml の path 値）
    pub depth: usize,            // 起点からの BFS 距離（起点 = 0）
}

pub struct MultiTraversalResult {
    /// 走査された全ノード（visited）の要約情報。
    /// start_ids 入力順 × 各起点の BFS 順による決定論的順序（SPEC-LGX-005.REQ.03）
    pub visited: Vec<VisitedNode>,
    /// 走査で使用したエッジ (from, to) の spanning tree 表現（グラフ向き）。
    /// v3 互換: 初訪問を生んだエッジのみ記録、既訪問ノードへのエッジは含まない
    pub edges_traversed: Vec<(String, String)>,
    /// 各ノードの起点からの最短 BFS 距離。多起点では min(depth) を記録
    /// IndexMap で挿入順（発見順）を保持（REQ.03 の決定論性）
    pub depth_map: indexmap::IndexMap<String, usize>,
    /// 入力 start_ids（CLI からは常に単起点、ライブラリ API は多起点対応）
    pub start_ids: Vec<String>,
}

pub struct SuspiciousNode {
    pub id: String,
    pub drift_score: f32,        // scores テーブルの MAX(value) where score_type='drift'
    pub type_code: String,
    pub path: String,
}

pub struct PrunedTraversalResult {
    pub traversal: MultiTraversalResult,
    /// drift_threshold 以上のノード。drift_score 降順、同値は id 昇順（stable sort）
    /// db = None または DB 照会失敗時は空（REQ.09 代替 3a、NFR REL.02）
    pub suspicious_nodes: Vec<SuspiciousNode>,
    /// 閾値値（Reporter Text / JSON 出力に含まれる）
    pub drift_threshold: f32,
}

/// 打ち切り発生時の stderr Info 付き調査結果（REQ.04 GAP-LGX-085 対応）
/// truncated = true かつ excluded_count > 0 のとき呼び出し側が stderr へ Info を出力する
pub struct InvestigateOutcome {
    pub result: PrunedTraversalResult,
    pub truncated: bool,         // max_depth 打ち切りが発生したか
    pub excluded_count: usize,   // 打ち切りで除外された到達可能ノード数（近似: 境界ノードの未訪問隣接数）
}
```

### 2.2 列挙 / Sum 型

```rust
// legixy-nav
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ReportFormat {
    #[default]
    Text,       // 人間可読テキスト（v3 互換既定）
    JsonLines,  // JSON Lines（--json 機能化、SPEC-LGX-005.REQ.09【v3 差分】）
}

// legixy-core（共有、ADR-LGX-020）
// Severity: Ok / Info / Warning / Error は ADR-LGX-020 参照、再定義しない
```

### 2.3 エラー型

```rust
// legixy-nav（実行時失敗 = 上位に伝播して exit 1 または exit 2）
#[derive(Debug, thiserror::Error)]
pub enum NavError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("invalid input: {0}")]
    InvalidInput(String),
    // 注: グラフロード失敗は legixy-graph::GraphError → legixy-cli で捕捉。
    //     DB 照会失敗は NavError に昇格させずベストエフォート継続（§6 参照）
}
```

- 終了コードは LGX-COMPAT-001 §3 グローバル規約: 引数構文誤り（clap）→ exit 2、実行時失敗（NavError / GraphError）→ exit 1、正常 → exit 0。
- 打ち切り Info 出力時（REQ.04）も exit 0（stdout の結果集合・終了コードは不変、SPEC-LGX-005.REQ.04）。
- 起点ノード不在（SEQD-LGX-005 §3 例外フロー）: v3 実測は空 visited + exit 0。SPEC-LGX-005.REQ.05「空の結果を返す（エラーではない）」と一致。SUPP-005 §2.4 [要決定] の UC-LGX-005 代替フロー 2a との矛盾は SPEC が後勝ちと解釈し exit 0 で実装（人間確認推奨、NOTES 参照）。

## 3. 公開 API surface（凍結、HR7）

| 関数 | シグネチャ | 不変条件 | 冪等性 | 同期/非同期 |
|---|---|---|---|---|
| `legixy_nav::investigate` | `fn investigate(graph: &TraceGraph, start_ids: &[String], db: Option<&rusqlite::Connection>, drift_threshold: f32) -> Result<PrunedTraversalResult, NavError>` | 同一入力 → 同一 PrunedTraversalResult（visited 順・suspicious 順含む、REQ.03）。read-only（graph/db を変更しない） | yes | 同期 |
| `legixy_nav::investigate_with_depth` | `fn investigate_with_depth(graph: &TraceGraph, start_ids: &[String], db: Option<&rusqlite::Connection>, drift_threshold: f32, max_depth: Option<usize>) -> Result<InvestigateOutcome, NavError>` | max_depth=None のとき investigate と同一結果（truncated=false、excluded_count=0）。max_depth 指定時は打ち切り発生を InvestigateOutcome.truncated で表現（REQ.04 GAP-LGX-085） | yes | 同期 |
| `legixy_nav::render_pruned` | `fn render_pruned(result: &PrunedTraversalResult, format: ReportFormat) -> String` | Text: v3 互換書式（SUPP-005 §2.3）。JsonLines: v3 reporter.rs JSON Lines 実装の機能化（REQ.09【v3 差分】）。打ち切り非発生時（investigate / max_depth=None）に使用 | yes | 同期 |
| `legixy_nav::render_outcome` | `fn render_outcome(outcome: &InvestigateOutcome, format: ReportFormat) -> String` | `render_pruned(&outcome.traversal, format)` を基に、`outcome.truncated == true` のとき summary 行へ `"truncated":true,"excluded":K`（JsonLines）/ 相当の Text 注記を加える（§3 JSON Lines 書式・GAP-LGX-085）。`truncated == false` のとき出力は `render_pruned` と同一。打ち切り可観測性の整形経路（B-1 系 TS-005 レビューで顕在化した API gap を加算的に解消、HR7 追加） | yes | 同期 |
| `legixy_nav::render_multi` | `fn render_multi(result: &MultiTraversalResult, format: ReportFormat) -> String` | impact コマンド用（UC-LGX-006 対応 DD で主に使用）。本 UC から参照可 | yes | 同期 |

**追記（v3 差分・SPEC 要求）**:
- `investigate_with_depth` は v3 `investigate.rs` が `max_depth = None` 固定だった点を機能化（SUPP-005 §2.4 [要決定]A 採用）。これは SPEC-LGX-005.REQ.04/07 の素直な読みに従う。
- `render_pruned` の `JsonLines` は v3 で CLI 未配線だった JSON Lines 実装を機能化（REQ.09【v3 差分】）。フィールド名・構造は v3 reporter.rs に整合させる（§2.3 JSON Lines 書式を参照）。
- `investigate` は後方互換エントリポイントとして残す（max_depth = None 相当 = 無制限）。

**出力書式の凍結（Text）**:
```
# visited 各行（inspect コマンドと同形式）
{id} (type={type_code}, depth={depth}, path={path})
# suspicious nodes セクション
Suspicious nodes (drift_threshold={t}):
{id} (drift={score}, type={type_code}, path={path})
# サマリ行
Summary: visited={n}, suspicious={m}
```

**出力書式の凍結（JsonLines）**:
```json
{"id":"...","type":"...","depth":N,"path":"..."}
{"suspicious":{"id":"...","drift":0.3,"type":"...","path":"..."}}
{"summary":{"visited":N,"suspicious":M,"drift_threshold":0.3}}
```
- `drift` 値は `f32_to_clean_f64`（Display 経由丸め）で短い 10 進表現を保つ（v3 reporter.rs:124-126 整合）。
- `truncated` フラグ（GAP-LGX-085 申し送り）: `InvestigateOutcome.truncated = true` のとき summary 行に `"truncated":true,"excluded":K` を追加する（DD 凍結対象。JSON スキーマ変更となるため今後の削除・フィールド改名は次版 SPEC 改訂扱い）。

## 4. module / package 構成

```
legixy-nav/
├── src/
│   ├── lib.rs               // Document ID: SRC-LGX-005（公開 API 再エクスポート）
│   ├── multi_traverser.rs   // MultiTraverser: 多起点 BFS ラッパー（v3 lx-nav/src/multi_traverser.rs 整合）
│   ├── investigate.rs       // investigate / investigate_with_depth（v3 lx-nav/src/investigate.rs + max_depth 機能化）
│   ├── drift_pruner.rs      // DriftPruner::prune: scores テーブル照会 + 降順整列（v3 lx-nav/src/drift_pruner.rs 整合）
│   ├── reporter.rs          // render_pruned / render_multi / ReportFormat / f32_to_clean_f64（v3 lx-nav/src/reporter.rs 整合）
│   ├── result.rs            // VisitedNode / MultiTraversalResult / SuspiciousNode / PrunedTraversalResult / InvestigateOutcome
│   └── error.rs             // NavError
└── Cargo.toml
```

依存方向（DAG、ADR-LGX-020）:
```
legixy-nav
  ├── legixy-graph  (TraceGraph / traverse_reverse / traverse_forward / TraversalResult)
  ├── legixy-db     (rusqlite::Connection 経由 scores テーブル照会)
  └── legixy-core   (Config / Id / ConfigError)
```
循環なし。`legixy-nav` は legixy-check / legixy-ctx を依存しない。`legixy-cli` が legixy-nav を依存して dispatch する。

## 5. ライフタイム / 所有権 / 借用 方針

- `investigate` / `investigate_with_depth` は `&TraceGraph` と `Option<&rusqlite::Connection>` を**借用**（所有権を取らない。read-only 操作、複数呼び出しで共有可能）。
- `PrunedTraversalResult` / `InvestigateOutcome` は所有を返す（呼び出し側が render と exit コード判定に使う）。
- `MultiTraversalResult.visited` は `Vec<VisitedNode>`（所有）。検索の `HashMap<&str, &VisitedNode>` は `drift_pruner.rs` 内で一時借用（スコープ内完結、`'static` バウンド不要）。
- `SuspiciousNode` の `id` / `type_code` / `path` は `VisitedNode` からクローン（DB 結果と照合後に所有）。
- `Arc`/`Mutex` 不要（単一スレッド逐次実行、§7）。

## 6. エラー伝播戦略

- **内部**: `drift_pruner.rs` の SQL 照会失敗（`rusqlite::Error`）は `NavError` に昇格させず、`eprintln!("[nav] drift pruning skipped: {e}")` で stderr 警告を出して空 `suspicious_nodes` で継続する（ベストエフォート、NFR-LGX-001.REL.02、v3 drift_pruner.rs:53-56 整合）。
- **公開境界**: `investigate` / `investigate_with_depth` が返す `Err(NavError)` は呼び出し側（legixy-cli）が exit 1 に変換する。グラフロード失敗（`legixy_graph::GraphError`）は legixy-graph から legixy-cli へ伝播し exit 1。
- **部分成功**: DB 不在（`db = None`）は致命的ではなく `suspicious_nodes = []` で走査結果のみ返す（SPEC-LGX-005.REQ.05 代替フロー 3a、UC-LGX-005 代替フロー 3a）。
- **打ち切り可観測性**: `max_depth` 指定時に打ち切りが発生した場合、`InvestigateOutcome.truncated = true / excluded_count = K` を返す。legixy-cli が stderr へ `[nav] info: max-depth {N} truncated traversal; {K} reachable node(s) excluded` を出力する（REQ.04 GAP-LGX-085、NFR-LGX-001.OBS.02 = Info は stderr）。除外件数 K の算定法: 深度境界ノード（`depth == limit`）から出る未訪問隣接ノード数の合計（安価な近似、SUPP-005 §2.5）。
- **panic 禁止**: `unwrap` / `expect` は禁止（rust.md §4）。`graph.node(id)` の存在は `MultiTraverser` が `graph.contains_node` 通過後に呼ぶため `expect` 使用箇所は不変条件が成立（v3 multi_traverser.rs:66 コメント通り）。

## 7. 並行性 / 非同期境界

- `investigate` / `investigate_with_depth` は**同期・単一スレッド・read-only**。`async` なし（SEQD-LGX-005 §4 の並行性「なし」と一致）。
- BFS は逐次 `VecDeque` ループ（v3 traversal.rs 整合）。`rayon` による並列化は将来最適化（本 DD では逐次。NFR PERF 予算内で充足見込み、benches で測定）。
- 外部更新整合性は対象外（走査は snapshot 読取。concurrent write の整合は legixy-db の責務、NFR-LGX-001.REL.07/08 の射程）。

## 8. テスト分類

| 分類 | 内容 | 対応 TP |
|---|---|---|
| Unit | `MultiTraverser::traverse_reverse_multi`（線形チェーン逆走査 T-GT-002 相当）、起点不在（T-GT-004 相当）、BFS 決定論性（T-GT-005 相当）、`DriftPruner::prune`（db=None/Some/照会失敗の 3 経路）、`render_pruned` Text / JsonLines 書式 | TP-LGX-005 |
| Integration | `investigate_with_depth` の max_depth=None / Some 打ち切り・InvestigateOutcome.truncated / excluded_count、`--json` JsonLines 出力スキーマ、drift 含む E2E（engine.db fixture）、起点不在で exit 0 確認 | TP-LGX-005 |
| Property-based | suspicious_nodes の整列決定性（同一入力 → drift_score 降順・id 昇順、proptest） | TP-LGX-005 |
| Bench | `investigate_with_depth` のノード 1,000 + エッジ 2,000（NFR PERF.02、criterion） | NFR-LGX-001 |

注: legixy 版 TS-LGX-001（T-GT-001〜005）は未作成（SUPP-005 §2.6）。旧 v3 `TS-LX-004` T-MT-001〜006 / T-DP-001〜005 / T-IV-xxx が前身。

## 9. ADR への参照

- **ADR-LGX-020**: crate 分割・共有型凍結（本 DD の crate 境界）
- **ADR-LGX-018**: investigate の drift 配線（ADR-018#11 — drift スコア参照の CLI 配線方針。本 DD は legixy-cli が engine.db を開いて `Some(&conn)` を渡す方式を採用し SUPP-005 §2.4 [要決定]A を解決する）
- ADR-LGX-003: embedding 決定論モデル（drift スコアの再現性）
- ADR-LGX-014: SPEC 準拠原則
- ADR-LGX-015: DB パス（engine.db 配置。legixy-cli が `config.db.file` を参照して Optional 開き）
- ADR-LGX-016: env（バイナリ解決・モデルディレクトリ）

## 10. 関連 NFR

- NFR-LGX-001.REL.05: BFS 走査決定性（同一グラフ・同一起点から常に同一 visited 順、TS-LGX-001 T-GT-005 相当）
- NFR-LGX-001.REL.02: DB 照会失敗時のベストエフォート継続（空 suspicious_nodes）
- NFR-LGX-001.OBS.02: 出力先（走査結果 = stdout / ログ・Info = stderr）
- NFR-LGX-001.OBS.05: 終了コード（0/1/2）
- NFR-LGX-001.PERF.02: 走査性能予算（ノード 1,000 + エッジ 2,000 で < 500ms）

## 11. 凍結履歴

| Date | Version | Note |
|---|---|---|
| 2026-06-13 | 1.0 | 初版凍結。`legixy-nav` の VisitedNode / MultiTraversalResult / SuspiciousNode / PrunedTraversalResult / InvestigateOutcome / NavError / ReportFormat 型と investigate / investigate_with_depth / render_pruned / render_multi 公開 API を確定。v3 lx-nav 整合。InvestigateOutcome（打ち切り可観測性、GAP-LGX-085）と investigate_with_depth（max_depth 機能化、SUPP-005 §2.4 [要決定]A 採用）を新設。JSON Lines 書式（suspected・truncated フィールド含む）を凍結。crate 境界は ADR-LGX-020。HR7 凍結 |
| 2026-06-14 | 1.1 | TS フェーズ敵対的レビューで顕在化した API gap を加算的に解消（HR7 追加・既存シグネチャ不変）: `render_outcome(&InvestigateOutcome, ReportFormat)` を新設。§3 JSON Lines 書式が規定する truncated/excluded の summary 注記を、`InvestigateOutcome.truncated`/`excluded_count` を持たない `render_pruned(&PrunedTraversalResult)` では描けなかった整合不良を解消。`render_pruned` は打ち切り非発生時用として存置 |
