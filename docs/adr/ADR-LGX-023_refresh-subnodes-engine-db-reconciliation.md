Document ID: ADR-LGX-023

# ADR-LGX-023: refresh-subnodes の実装方式 — engine.db 照合・凍結 Node 保持

**ステータス**: accepted
**起票日**: 2026-06-14
**承認日**: 2026-06-14
**承認者**: 開発者（19/19 完成のためグラフモデル拡張を承認 2026-06-14）
**対象**: LGX-COMPAT-001 §3 #9（refresh-subnodes）、ADR-LGX-020（共有型凍結）、ADR-LGX-021（cross-DD 型）

## 1. 文脈（Context）

CLI 統合層の最後の未配線サブコマンド `refresh-subnodes`（見出しリネーム時のサブノード ID 連鎖反映、
LGX-COMPAT-001 §3 #9）を配線するにあたり、v3 底本（`te-cli/commands/refresh_subnodes.rs`、657 行）が
依存する **サブノードメタデータ（anchor / heading_levels / subnode_kind）が legixy の現グラフモデルに
存在しない**ことが判明した:

- legixy `Node` = `{id, type_code, path, parent_id}` のみ（**ADR-LGX-020 で凍結**）。
- `docs/traceability/graph.toml` はドキュメントノードのみを保持し、サブノードを **graph.toml に永続化しない**
  （v3 はサブノードを graph ノードとして persist していた = モデル差）。
- 一方、**legixy はサブノードを engine.db の `embeddings` テーブルに永続化する**
  （`parent_id` / `anchor` / `is_subnode` 列、DD-LGX-007 §2.1）。`embed` 実行後に subnode 行が入る。

## 2. 検討した選択肢（Options）

### 選択肢 A: 凍結 Node を subnode メタで拡張し graph.toml にサブノードを persist
- `Node` に anchor/heading_levels/subnode_kind を追加 + graph.toml format/loader/writer 拡張。
- 欠点: **ADR-LGX-020 凍結共有型 Node の破壊的拡張**（全 TC の `Node{...}` 構造体リテラルが破綻）。
  legixy はそもそもサブノードを graph.toml に持たない設計のため、persist 方式の再設計まで波及する。過大。

### 選択肢 B（採用）: engine.db `embeddings` の subnode 行を照合元にする
- 凍結 `Node` は不変。サブノードの永続状態（id + anchor）は engine.db から取得する（legixy の実際の
  永続先）。グラフ側は `TraceGraph::document_nodes()`（親ドキュメント列挙、**非破壊な追加メソッド**）のみ拡張。
- `EmbeddingStore` に `list_subnodes()` / `rename_subnode()` を**加算的に追加**（ADR-021 §2.3 所有内）。
- refresh ロジック（再抽出 → 差分 → anchor Levenshtein で rename 対応付け → dry-run/apply）は legixy-cli
  の `refresh` モジュールに実装（v3 アルゴリズムを legixy 永続モデルへ移植）。

### 選択肢 C: refresh-subnodes を未配線のまま据え置き
- 18/19 で完了とする。欠点: LGX-COMPAT-001 §3 の凍結契約 #9 を満たさない。

## 3. 判断（Decision）

選択肢 B を採用する。

- **凍結 Node（ADR-LGX-020）は変更しない**。subnode メタは legixy の実際の永続先 = engine.db `embeddings`
  から取得する。これにより全 TC の `Node{...}` リテラルを壊さず、persist 方式の再設計を回避する。
- グラフ側拡張は `TraceGraph::document_nodes()` の**加算的追加**のみ（既存 API 不変）。
- `EmbeddingStore::list_subnodes()` / `rename_subnode()` を加算的に追加（DD-LGX-007 所有、ADR-021 §2.3）。
- refresh は engine.db に subnode 行が存在する場合（= `embed` 実行後）に意味を持つ。未生成時は空レポート
  （graceful）。`--apply` 前に engine.db を `.refresh-bak.{epoch}` へバックアップする（LGX-COMPAT-001 #9）。

## 4. 結果（Consequences）

### 期待される効果
- 凍結共有型を壊さず 19/19 サブコマンドを完成。legixy の永続モデル（engine.db）に忠実。

### 受け入れる代償
- v3 と照合元が異なる（graph.toml → engine.db embeddings）。refresh は `embed` 後にのみ実効。
- heading_levels を Node に持たないため、再抽出は既定 `[2,3]`（h2/h3）を用いる（v3 の per-node 既定と同値）。

### 残存リスク / 申し送り
- engine.db に subnode 行が無い（embed 未実行）プロジェクトでは refresh は no-op レポート。
- custom_edges がサブノード ID を参照する場合の更新は本 ADR 範囲では rename_subnode の embeddings 更新に
  限定し、custom_edges 連鎖更新は次段（必要時）に拡張する。

## 5. 関連

- 対象: LGX-COMPAT-001 §3 #9、ADR-LGX-020（Node 凍結、本 ADR は Node 不変を確認）、ADR-LGX-021（型所有）
- 底本: `traceability-engine.v3/crates/te-cli/src/commands/refresh_subnodes.rs`（アルゴリズム移植元）
- 実装: `crates/legixy-cli/src/refresh.rs`、`legixy-graph::TraceGraph::document_nodes`、
  `legixy-embed::EmbeddingStore::{list_subnodes, rename_subnode}`
