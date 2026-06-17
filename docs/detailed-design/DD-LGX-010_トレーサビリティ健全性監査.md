Document ID: DD-LGX-010

# DD-LGX-010: トレーサビリティ健全性監査（report）の詳細設計

**親 SEQD**: SEQD-LGX-010
**親 RBD**: RBD-LGX-010 / **親 UC**: UC-LGX-010
**対象言語**: Rust（CLI 本体）
**境界 API 凍結**: yes（本ドキュメント確定後、crate 間公開 API は変更不可。追加は許容、削除・改名・型変更は次版 SPEC 改訂。HR7）

> 言語別規律は `guides/language-stacks/rust.md`。crate 境界・共有型は **ADR-LGX-020** で凍結済（本 DD は参照する）。型は v3 実装（traceability-engine.v3.chg_to_lexigy `crates/lx-embed/src/similarity.rs`、`crates/lx-cli/src/commands/report.rs`）に整合させ引数互換を保つ。

## 1. 対象範囲

- **主 crate**: `legixy-embed`（`compute_edge_scores` / `compute_link_candidates` / `EmbeddingStore::load_all` による計測レポート生成・リンク漏れ候補抽出・summary 集計）
- **依存 crate（共有型は ADR-LGX-020、再定義しない）**:
  - `legixy-graph`（`TraceGraph` / `Node` / `Edge`）
  - `legixy-db`（`EmbeddingStore` — embeddings テーブルアクセス）
  - `legixy-core`（`Id` / 共通エラー / 設定型 `Config`）
  - `legixy-cli`（コマンドディスパッチ・グローバルオプション受理）
- **公開 API surface**: 本 DD §3（`legixy-embed` の crate 公開関数 — bulk similarity API の report 用サブセット。SPEC-LGX-006.REQ.11 に対応）
- **関連 SEQD**: SEQD-LGX-010

`report` は **計測専用**（閾値判定なし・severity 概念なし）。`check` コマンドの判定（severity 付き findings）とは責務が重複しない（SPEC-LGX-010.REQ.04、QSET-LGX-004 Q4 回答）。MCP 非公開（MCP-INV-1、REQ.01）。

## 2. 型定義

### 2.1 主要データ型

```rust
// ── legixy-embed bulk similarity エンジン型（DD-LGX-007 所有・参照、ADR-LGX-021 §2.3）──
// EdgeScore / CandidateScore は legixy-embed の bulk similarity エンジンが所有する共有型で、
// DD-LGX-007 §2 で凍結済。本 DD（report、consumer）は再定義せず参照する。
// 正準定義（v3 lx-embed/similarity.rs 準拠、NodeId = String）:
//   pub struct EdgeScore { from: String, to: String, score: f32, edge_kind: legixy_graph::EdgeKind }
//   pub struct CandidateScore { from: String, to: String, score: f32 }

// ── legixy-embed（report コマンド層、本 DD で凍結）──

/// report コマンドの算出結果集約
/// v3 実測: report.rs L165-186 の統計集約に対応
pub struct ReportOutput {
    pub links: Vec<EdgeScore>,          // 算出対象エッジのスコア一覧（skip 後）
    pub candidates: Vec<CandidateScore>, // リンク漏れ候補一覧
    pub summary: ReportSummary,
    pub skipped: usize,                 // スキップ件数 = 試行エッジ数 − links.len()（report 層で算出可能）。
                                        // 理由別内訳（missing/dim/non_finite）と stderr 集約 Warning の出力は
                                        // bulk エンジン（DD-LGX-007 §6）が所有する。ADR-LGX-021 §2.3 / 後述 §6 / TS レビュー B-1
}

/// links の統計サマリ
/// v3 実測: report.rs L165-186 の集計ロジックに対応
pub struct ReportSummary {
    pub total_links: usize,         // 算出対象エッジ数（スキップ後、REQ.04）
    pub total_candidates: usize,
    pub min_link_score: Option<f32>, // links 0 件時 None（REQ.04）
    pub max_link_score: Option<f32>,
    pub mean_link_score: Option<f32>,
}

/// スキップ集約 Warning エントリ（v3 差分: v3 は無言スキップ）。
/// **bulk エンジン（DD-LGX-007）所有の型**。エンジンが検出・理由別集計し stderr へ 1 件まとめ出力する。
/// report（本 DD）は再定義・再集計せず、`ReportOutput.skipped`（件数のみ）を保持する（ADR-LGX-021 §2.3）。
/// SPEC-LGX-010.REQ.04【v3 差分】/ SUPP-010 C-7
pub struct SkipWarning {            // legixy-embed（DD-007 §6 のエンジン側集約。本 DD は参照）
    pub skip_count: usize,
    pub reasons: SkipReasonSummary,
}

pub struct SkipReasonSummary {     // legixy-embed（DD-007 所有。理由別内訳はエンジン stderr のみ）
    pub missing_embedding: usize,   // 端点 embedding 不在
    pub dim_mismatch: usize,        // 次元不一致
    pub non_finite_score: usize,    // 非有限スコア（NaN/±Inf、REQ.09）
}
```

### 2.2 列挙 / Sum 型

```rust
// ── legixy-graph（共有、ADR-LGX-020 参照）──
// EdgeKind は legixy-graph で定義。再定義しない。
// pub enum EdgeKind { Chain, Custom, ParentChild }

// ── legixy-embed（本 DD で凍結）──

/// --json 出力時の links.kind 文字列
/// v3 実測: report.rs L157-163 の kind 文字列と一致
/// "chain" | "custom" | "parent_child"
impl EdgeKind {
    pub fn to_json_str(&self) -> &'static str { ... }
}

/// report の出力形式
pub enum ReportFormat {
    Text,
    Json,
}
```

### 2.3 エラー型

```rust
// ── legixy-embed（本 DD で凍結）──

/// report 実行時失敗（exit 1。計測スキップとは別概念）
pub enum ReportError {
    /// graph.toml 破損・パース不能（legixy-graph::GraphError を包む）
    GraphLoad(legixy_graph::GraphError),
    /// .legixy.toml 不在/破損（legixy-core::ConfigError を包む）
    ConfigLoad(legixy_core::ConfigError),
    /// engine.db open 失敗等
    Db(legixy_db::DbError),
}

// 終了コード規約（NFR-LGX-001.OBS.05、LGX-COMPAT-001 §3）:
//   ReportError  → exit 1
//   引数構文誤り（clap）→ exit 2
//   空ストア・正常計測完了 → exit 0
// スキップ（端点 embedding 不在・次元不一致・非有限スコア）は ReportError に昇格しない
// → SkipWarning として集約し計測継続（SPEC-LGX-010.REQ.04）
```

## 3. 公開 API surface（凍結、HR7）

> **bulk similarity エンジン関数は DD-LGX-007 所有（ADR-LGX-021 §2.3）**。本 DD は再凍結せず参照する。正準シグネチャ（DD-007 §3 凍結）: `compute_edge_scores(graph, store) -> Result<Vec<EdgeScore>, EmbedError>` / `compute_link_candidates(graph, store, threshold) -> Result<Vec<CandidateScore>, EmbedError>`。スキップは DD-007 §6 に従いエンジンが集約 Warning として **stderr へ 1 件まとめ出力**する（report 側で別途出力しない）。`run_report` はこれらを呼び出し ReportOutput を組み立てる report コマンド層の入口。

| 関数 | シグネチャ | 不変条件 | 冪等性 | 同期/非同期 |
|---|---|---|---|---|
| `legixy_embed::run_report` | `fn run_report(graph: &TraceGraph, store: &EmbeddingStore, config: &Config) -> Result<ReportOutput, ReportError>` | DD-007 の `compute_edge_scores`/`compute_link_candidates` を呼び ReportOutput へ集約。`skipped = 試行エッジ数 − links.len()`（report 層で算出）。理由別内訳と stderr 集約 Warning はエンジンが担う（DD-007 §6）。空ストア時は `ReportOutput{ links: [], candidates: [], summary: zero, skipped: 0 }`（exit 0）。read-only | yes | 同期 |
| `legixy_embed::ReportOutput::to_text` | `fn to_text(&self) -> String` | v3 実測の text フォーマット（SUPP-010 R-4）に準拠。links / candidates / summary の 3 セクション。診断 Warning は含まない（stderr 出力はコマンド層が担う） | yes | 同期 |
| `legixy_embed::ReportOutput::to_json` | `fn to_json(&self) -> String` | `{"links":[...], "candidates":[...], "summary":{...}}` の構造化 JSON（pretty print）。非有限値非出力（REQ.09）。warn フィールドは任意付加（C-7、本 DD で確定: `--json` の stdout に `"warnings":[{...}]` を付加しない — stderr 集約 Warning のみ） | yes | 同期 |
| `legixy_embed::EmbeddingStore::load_all` | `fn load_all(&self) -> Result<Vec<EmbeddingRow>, legixy_db::DbError>` | node_id 昇順でロード（SPEC-LGX-006.REQ.11 / SCORE-INV-1。legixy-db 所管だが report の決定性の基盤として本 DD でも明記） | yes | 同期 |

### 3.1 引数 / オプション（`legixy-cli` 層）

`report` コマンドの CLI 引数契約は LGX-COMPAT-001 §4 #6 で凍結済み:

```
legixy [--project-root <PATH>] [--json] [--models-dir <PATH>] report
```

- フラグ一切なし（`report` サブコマンド自体に固有オプションは無い）
- `--models-dir` は受理するがモデル不要なため無視（SPEC-LGX-010.REQ.01）
- `--json`: `ReportFormat::Json` を選択（既定: `ReportFormat::Text`）

### 3.2 出力先と終了コード

| ケース | stdout | stderr | exit |
|---|---|---|---|
| 正常計測（links あり／なし） | 監査報告（text or JSON） | スキップ集約 Warning（件数 > 0 時、v3 差分） | 0 |
| 空ストア | INFO 文（text） / 空構造 JSON | — | 0 |
| 空ストア（--json） | `{"links":[],"candidates":[],"summary":{"total_links":0,"total_candidates":0,"min_link_score":null,"max_link_score":null,"mean_link_score":null}}` | — | 0 |
| ReportError | — | ERROR メッセージ | 1 |

スキップ集約 Warning の stderr 文言（C-7 確定）:
```
WARNING: N ペアをスキップしました（embedding 不在: X 件 / 次元不一致: Y 件 / 非有限スコア: Z 件）
```
v3 は `--json` stdout に warning フィールドを持たないため、本 DD でも `--json` stdout には warning を含めない（v3 正準化 + 機械可読性の保全）。`--json` 時も集約 Warning は stderr に出力する（NFR-LGX-001.OBS.02）。

## 4. module / package 構成

```
legixy-embed/
├── src/
│   ├── lib.rs                // Document ID: SRC-LGX-010（run_report / compute_edge_scores /
│   │                         //   compute_link_candidates / 再エクスポート）
│   ├── report.rs             // ReportOutput / ReportSummary / SkipWarning / to_text / to_json
│   │                         //   v3 実測 report.rs の text フォーマット・summary 集計・JSON
│   ├── similarity.rs         // compute_edge_scores / compute_link_candidates /
│   │                         //   compute_all_pair_scores（calibrate 共用）/ SkipReasonSummary
│   │                         //   v3 実測 similarity.rs L66-141 の移植先
│   ├── store.rs              // EmbeddingStore::load_all / load_snapshot_embedding /
│   │                         //   snapshot_create / snapshot_list / snapshot_delete /
│   │                         //   resolve_snapshot_id_by_label（snapshot 系コマンド共用）
│   │                         //   v3 実測 store.rs の移植先
│   ├── orchestrator.rs       // run_report（統括制御）/ compute_node_drift_at /
│   │                         //   compute_node_drift_against_snapshot /
│   │                         //   read_current_content_for_node（drift 共用）
│   ├── drift.rs              // DriftCalculator / cosine_similarity（値域 [-1,1]）
│   │                         //   v3 実測 drift.rs L24-43 の移植先
│   │                         //   ゼロベクトル skip 保証（SPEC-LGX-006.REQ.04）
│   ├── calibrate.rs          // histogram / compute_recommended（calibrate 専用）
│   │                         //   v3 実測 calibrate.rs の移植先
│   └── error.rs              // ReportError（+ DriftError / CalibrateError）
└── Cargo.toml
```

依存方向（DAG、ADR-LGX-020）:
```
legixy-embed → legixy-graph / legixy-db / legixy-core
legixy-cli   → legixy-embed（run_report 呼び出し）
```

`report.rs` は `similarity.rs` の `compute_edge_scores` / `compute_link_candidates` を利用し、`orchestrator.rs` の `run_report` が全体を統括する（SEQD-LGX-010 §1 の Control 層に対応）。循環なし。

## 5. ライフタイム / 所有権 / 借用 方針

- `run_report` / `compute_edge_scores` / `compute_link_candidates` は `&TraceGraph` / `&EmbeddingStore` / `&Config` を**借用**（所有権を取らない。read-only 計測、複数呼び出し間で共有可）。
- `ReportOutput` は所有を返す（呼び出し側が stdout 出力・JSON 直列化に使う）。
- `Id` は `legixy-core` 所有。`EdgeScore` / `CandidateScore` は `Id` をクローンして所有（アロケーション許容 — 計測フローは性能クリティカルではない）。
- `Vec<EdgeScore>` / `Vec<CandidateScore>` の所有は `ReportOutput` が保持（`Arc`/`Mutex` 不要 — 単一スレッド逐次、§7）。
- `EmbeddingRow`（`load_all` の戻り値）は一時的に所有するが、スコア算出後は解放（ベクトルデータは大きいため不要になったら早期 drop）。

## 6. エラー伝播戦略

- **内部処理**: `compute_edge_scores` / `compute_link_candidates` は端点 embedding 不在・次元不一致・非有限スコアを `SkipWarning` に記録して継続（`Err` に昇格しない。SPEC-LGX-010.REQ.04 の部分成功継続）。
- **実行時失敗**: graph.toml 破損・engine.db open 失敗は `ReportError` として `run_report` が `Err` を返す（exit 1）。
- **部分成功**: read-only のためロールバック不要。算出可能なエッジのみ `links` に含める。スキップ件数は `ReportOutput.skipped`（試行 − 算出）で report が保持する。
- **panic 禁止**: `unwrap` / `expect` を内部処理内で使用しない（rust.md §4）。`load_all` の DB エラーは `?` で伝播させ `ReportError::Db` に変換。
- **非有限スコア**: `cosine_similarity` の `f32::is_finite()` 検査・非有限 skip・`non_finite_score` 計上は **bulk エンジン（DD-007 §6 / compute_edge_scores 内）が担う**（理由別内訳はエンジン所有、stderr へ集約 Warning として 1 件出力）。report の `links` には算出済み有限スコアのみが含まれるため、`--json`/text stdout に NaN/Inf は原理的に現れない（REQ.09、serde_json 動作に非依存）。report 層は理由別検査を二重実行しない。
- **ユーザ通知**: 計測結果 = stdout、スキップ集約 Warning / ERROR = stderr（REQ.04、NFR-LGX-001.OBS.02）。

## 7. 並行性 / 非同期境界

- `report` は **同期・単一スレッド・read-only**。async なし（v3 実測の計算モデルと同一）。
- `compute_edge_scores` / `compute_link_candidates` は O(N²) だが現状は逐次実装（将来の Rayon 並列化は別 ADR）。NFR-LGX-001.PERF.07（WAL モード）は `legixy-db` が担保。
- 並行アクセス（外部からの `embed --all` 同時実行）整合性は対象外（read-only のため write-read 競合はデータ破損に至らない。STATE-INV-1）。

## 8. テスト分類

| 分類 | 内容 | 対応 TP |
|---|---|---|
| Unit | `compute_edge_scores`: 端点 embedding 不在 skip / 次元不一致 skip / 非有限スコア skip + 集約 Warning 正確性 | TP-LGX-010 |
| Unit | `compute_link_candidates`: 無向除外（from↔to 両方向 HashSet）/ 閾値境界（≥ / <）/ 算出不能ペア skip | TP-LGX-010 |
| Unit | `ReportSummary` 集計: links 0 件時 min/max/mean = null、links ありで統計値正確性 | TP-LGX-010 |
| Unit | `to_text`: v3 実測の text フォーマット（R-4）への準拠確認（ヘッダ行・`score={:.4}`・統計行） | TP-LGX-010 |
| Unit | `to_json`: 3 キー構造・links.kind 文字列（"chain"/"custom"/"parent_child"）・非有限値非出力 | TP-LGX-010 |
| Integration | `run_report` の正常系: embed 済み graph での links / candidates / summary の E2E 出力検証 | TP-LGX-010 |
| Integration | 空ストア時の早期終了（text INFO / JSON 空構造 / exit 0 / engine.db 不変） | TP-LGX-010 |
| Integration | スキップ発生時の集約 Warning stderr 出力と stdout クリーン（v3 差分） | TP-LGX-010 |
| Integration | `--json` 出力の stdin バイト一致（同一入力での決定性 REQ.06）| TP-LGX-010 |
| Property-based | 同一入力での `to_json` / `to_text` 出力バイト一致（決定性、REQ.06 / proptest） | TP-LGX-010 |
| Property-based | 非有限スコア注入 fixture で `to_json` に NaN/Inf が現れないこと | TP-LGX-010 |
| Bench | N ノード × E エッジでの `run_report` 応答時間（NFR PERF.02 目安、criterion） | NFR-LGX-001 |

## 9. ADR への参照

- **ADR-LGX-020**: crate 分割・共有型凍結（本 DD の crate 境界）
- ADR-LGX-003: embedding 決定論モデル（`compute_edge_scores` / `load_all` の再現性）
- ADR-LGX-007: 非有限スコア・model_version ポリシー（NaN/±Inf の防御層設計根拠）
- ADR-LGX-014: SPEC 準拠原則
- ADR-LGX-015: DB パス（`.legixy/engine.db` の物理パス凍結）

## 10. 関連 NFR

- NFR-LGX-001.OBS.02: 出力先（計測結果 = stdout / ログ・Warning = stderr）
- NFR-LGX-001.OBS.04: エラーメッセージ日本語（primary）。WARNING 文言・空ストア INFO も日本語（SUPP-010 C-8 確定: v3 混在文言を日本語統一）
- NFR-LGX-001.OBS.05: 終了コード（0=正常・空ストア / 1=ReportError / 2=引数構文誤り）
- NFR-LGX-001.PERF.07: WAL モード（legixy-db が担保、`report` の read 整合性に寄与）
- NFR-LGX-001.REL.06: engine.db アクセス（busy_timeout 等は legixy-db 所管）

## 11. 凍結履歴

| Date | Version | Note |
|---|---|---|
| 2026-06-13 | 1.0 | 初版凍結。`legixy-embed` の `EdgeScore` / `CandidateScore` / `ReportOutput` / `ReportSummary` / `SkipWarning` / `ReportError` 型と `compute_edge_scores` / `compute_link_candidates` / `run_report` / `to_text` / `to_json` 公開 API を確定（v3 lx-embed/similarity.rs・report.rs 整合）。SUPP-010 [要決定] C-7 確定（`--json` stdout に warnings フィールドなし、集約 Warning は stderr のみ）/ C-8 確定（診断メッセージ日本語統一）。crate 境界は ADR-LGX-020。HR7 凍結 |
| 2026-06-13 | 1.1 | cross-DD 整合（ADR-LGX-021 §2.3）: `EdgeScore` / `CandidateScore` と bulk similarity エンジン関数（`compute_edge_scores` / `compute_link_candidates`）の所有を **DD-LGX-007 に一本化**。本 DD はローカル定義（`from: Id`/`to: Id`/`kind`・tuple 戻り）を撤去し DD-007 正準定義（`from: String`/`to: String`/`edge_kind`・`Result<_, EmbedError>`）を参照。本 DD 所有は report コマンド層（`ReportOutput`/`ReportSummary`/`SkipWarning`/`SkipReasonSummary`/`ReportError`/`run_report`/`to_text`/`to_json`）に限定。HR7 境界契約は不変（report コマンドの CLI/出力契約は不変） |
| 2026-06-14 | 1.2 | TS フェーズ敵対的レビューで顕在化した skip 集約の責務矛盾を解消（§3 「エンジンが stderr 出力」と §6 「run_report が is_finite 検査」の二重所有）。bulk エンジン（DD-007、frozen signature `Result<Vec<EdgeScore>, EmbedError>`）が skip 検出・理由別集計・stderr 集約 Warning を**単独所有**。`ReportOutput.skip_warnings: Vec<SkipWarning>` を `skipped: usize`（試行 − 算出、report 算出可能）へ簡素化。`SkipWarning`/`SkipReasonSummary` はエンジン側集約型（参照）と明記。report は理由別検査を二重実行しない。report の stdout/json 出力契約（C-7、warnings 欄なし）は不変 |
