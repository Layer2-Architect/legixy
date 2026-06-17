Document ID: ADR-LGX-021

# ADR-LGX-021: 複数 DD が共有する crate の型所有権・正準表現の調停（DD フェーズ整合）

**ステータス**: accepted
**起票日**: 2026-06-13
**承認日**: 2026-06-13（DD フェーズ内整合。AI 自律修正範囲。HR11。境界 crate 公開 API の追加・削除・改名ではなく、同一 crate 内で複数 per-UC DD が重複定義した共有型の **正準化**）
**対象**: per-UC DD（DD-LGX-001〜013）が同一 crate を跨いで共有する型の単一所有権確定。ADR-LGX-020 §2.3「共有型は所有 crate で 1 回だけ定義」の per-DD 実装ガード。

## 1. 文脈（Context）

DD は per-UC（DD-LGX-001〜013、SEQD と 1:1）で編成したため、**1 つの crate に複数 UC が写像される**（`legixy-nav` ← UC-005/006、`legixy-embed` ← UC-007/010/011/012/013）。各 per-UC DD は自 UC 視点で型を完全定義したため、同一 crate 内の共有型が **DD 間で重複定義・表現乖離**を起こした。ADR-LGX-020 §2.3 は「所有 crate で 1 回だけ定義」を要求していたが、所有を担う DD の指名が per-DD レベルで未確定だった。

DD-LGX-005 の起点ノード不在=空結果・exit 0 化（GAP-234/SPEC-LGX-005.REQ.05）と並ぶ DD フェーズの未決事項処理（item E: cross-DD coordination）として、本 ADR で型所有権を確定する。

### 検出した乖離（v3 実装＝`traceability-engine.v3.chg_to_lexigy` を底本に照合）

| # | 共有型 / crate | 乖離 | v3 底本 |
|---|---|---|---|
| E-1 | `NodeId`（legixy-graph） | DD-002 が `pub type NodeId = Id`、DD-003/006 が `= String`、DD-005 が生 `String` | `lx-graph::model::NodeId = String`（型エイリアス） |
| E-2 | `EdgeScore` / `CandidateScore`（legixy-embed） | DD-007 が `{from: String, to: String, edge_kind}`・`compute_edge_scores -> Result<Vec<EdgeScore>, EmbedError>`、DD-010 が `{from: Id, to: Id, kind}`・`-> (Vec<EdgeScore>, Vec<SkipWarning>)` と二重定義・型乖離 | `lx-embed/similarity.rs` の bulk API（NodeId=String） |
| E-3 | legixy-db スキーマ | 専属 DD なし。`embeddings`（DD-007）・`observations`/`proposals`（DD-008）・`embedding_snapshots`（DD-012）がテーブル定義を各所に分散 | `lx-db` の各テーブル DDL |

## 2. 判断（Decision）

### 2.1 `NodeId` の正準表現（E-1）

`legixy-graph` が `pub type NodeId = String`（v3 `lx-graph::model` 準拠）を**唯一所有**する。グラフのノード識別子（`IndexMap<NodeId, Node>` のキー・隣接リストのキー・エッジ端点）はすべて `NodeId`（= `String`）。

- `legixy-core::Id`（newtype `Id(String)`）は **ID 書式の検証・解析**に用いる別概念であり、グラフのマップキー型 `NodeId` とは**区別する**（v3 も graph キーは raw String）。検証済み識別子を扱う文脈（`legixy-check` の `related_ids` 等）は `Id`、グラフ走査・キー参照の文脈は `NodeId` を使う。
- per-UC DD は `NodeId` を**再定義しない**（`legixy-graph` を参照）。`= Id` への別名付けは禁止（マップキー不整合を招く）。

### 2.2 `legixy-nav` 共有型の所有（既存調停の確認）

`DD-LGX-005`（investigate）を **legixy-nav 共有走査型の正典**とする（DD-LGX-006 §2 既述の調停を追認）。

- **DD-005 所有（DD-006 は参照）**: `VisitedNode` / `MultiTraversalResult` / `NavError` / `ReportFormat` / `MultiTraverser` / 多起点逆走査・`render_multi`。
- **DD-006 所有**: impact 固有型 `TraversalResult` / `TruncationInfo` と `impact` / `detect_truncation` / `emit_truncation_info`。
- §2.1 の解決により DD-005 の生 `String` と DD-006 の `NodeId` は同一型（`String`）に収束。表現乖離は解消（コード上の差異なし）。

### 2.3 `legixy-embed` 共有型の所有（E-2）

`DD-LGX-007`（embed 生成）を **legixy-embed エンジン共有型の正典**とする。

- **DD-007 所有（他 embed DD は参照）**: `EmbeddingStore` / `EmbeddingRow` / bulk similarity エンジン（`EdgeScore` / `CandidateScore` / `compute_edge_scores` / `compute_link_candidates` / `compute_all_pair_scores`）。`EdgeScore = { from: String, to: String, score: f32, edge_kind: legixy_graph::EdgeKind }`（v3 similarity.rs 準拠、フィールド名 `edge_kind`）。エンジン関数の戻り値は `Result<_, EmbedError>`（skip は集約 Warning）。
- **DD-010 所有**: report コマンド層（`ReportOutput` / `ReportSummary` / `SkipWarning` / `SkipReasonSummary` / `ReportError` / `run_report` / `to_text` / `to_json`）。bulk エンジン型・関数は DD-007 を参照し再定義しない。
- **DD-011/012/013 所有**: calibrate（`AllPairScores`/`Histogram`/…）・snapshot（`Snapshot*`）・drift（`Drift*`/`ResolvedModel`/…）の各コマンド固有型。`EmbeddingStore`/`EmbeddingRow` は DD-007 参照。

### 2.4 legixy-db スキーマの所有（E-3）

専属 DD は設けず、**テーブル単位で所有 DD を指名**する。`legixy-db` は接続・DB パス解決（ADR-LGX-015）・トランザクション基盤・WAL 設定（NFR PERF.07）を提供する横断層とし、各テーブルのスキーマは利用 DD が定義する。

| テーブル | スキーマ所有 DD | 接続/パス層 |
|---|---|---|
| `embeddings` | DD-LGX-007（`EmbeddingStore`/`EmbeddingRow`） | legixy-db + ADR-LGX-015 |
| `observations` / `proposals` | DD-LGX-008（feedback 状態モデル） | legixy-db + ADR-LGX-015 |
| `embedding_snapshots` | DD-LGX-012（snapshot 管理） | legixy-db + ADR-LGX-015 |

## 3. 結果（Consequences）

- 反映済み調停（本 ADR 採択に伴う surgical 修正、各 DD §11 に v1.1 追記）:
  - DD-LGX-002: `pub type NodeId = Id` → `= String`（legixy-graph 参照）。
  - DD-LGX-010: `EdgeScore` / `CandidateScore` のローカル定義を撤去し DD-007 参照に変更。bulk エンジン関数は DD-007 所有を明示。
  - DD-LGX-007: `EdgeScore` / `CandidateScore` / bulk API が legixy-embed 所有・DD-010 が consumer である旨を明記。
- crate 公開 API の **追加・削除・改名は行わない**（HR7 凍結境界は不変）。本 ADR は同一 crate 内の重複定義を単一所有へ収斂させる整合作業であり、境界契約・SPEC・LGX-COMPAT-001 を変更しない。
- SRC フェーズは本 ADR の所有表に従い、共有型を所有 crate/DD に 1 回だけ実装する。
- 残存（人間裁定へ送る item A/B/C とは独立）: なし（E-1/E-2/E-3 はすべて v3 底本で一意に確定）。

## 4. 関連

- 基盤: ADR-LGX-020（crate 分割・共有型境界凍結。本 ADR はその §2.3 の per-DD 実装ガード）
- DB パス: ADR-LGX-015（engine.db 正準パス・fallback）
- 統治: HR7（境界 API 凍結）, HR11（RBA 以降 AI 自律）
- 底本: `traceability-engine.v3.chg_to_lexigy`（`lx-graph::model`, `lx-nav`, `lx-embed/similarity.rs`, `lx-db`）
- 反映先: DD-LGX-002 / DD-LGX-007 / DD-LGX-010（§11 v1.1）
