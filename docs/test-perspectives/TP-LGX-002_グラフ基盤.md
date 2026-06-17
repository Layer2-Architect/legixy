Document ID: TP-LGX-002

# TP-LGX-002: グラフ基盤

> TP は **テストケース** ではなく **観点リスト**。「仕様文書に問いかける質問のリスト」として書く。具体化（テストデータ・期待値）は TS 層で行う。

**親**: SPEC-LGX-002
**ステータス**: green
**最終更新**: 2026-06-08

## 1. 対象スコープ

この TP は SPEC-LGX-002（グラフ基盤 — 有向グラフ + サブノード + DAG 制約）の全要求 REQ.01〜REQ.13 を対象とする。

- 対象: SPEC-LGX-002 §3 REQ.01〜REQ.13、§4 不変条件マトリクス
- 関連文書: LGX-EXT-001 §3, §4, §7.1（サブノード仕様）、LGX-COMPAT-001 §4 #9（refresh-subnodes 凍結契約）、NFR-LGX-001（PERF.04/05, SEC.03/04/06, REL.04, COMPAT.07/08, HARDEN.03）
- 委譲先: SPEC-LGX-003（出力側上限・CTX-INV-3 意味制約）、SPEC-LGX-004（検証・衝突検出・未解決エッジ Warning・ID 一意性強制）

> **判定方針（品質偏向防止）**: GREEN は SPEC-LGX-002 が当該観点に明示的に答えている、または名前付き SPEC/文書へ正しく委譲している場合に限る。SPEC が沈黙・曖昧な場合のみ RED とし、他 SPEC が所有する観点は委譲として GREEN 扱いする（捏造 RED を避ける）。

## 2. 観点リスト

### 2.1 境界値

- [ ] G-BV-1: 空 graph.toml（ノード 0 / エッジ 0）の構築
- [ ] G-BV-2: サブノードゼロ運用（ドキュメントノードのみ、v0.1.0 互換）
- [ ] G-BV-3: 単一見出しのみのドキュメント / 見出しゼロのドキュメント
- [ ] G-BV-4: 巨大 Markdown ファイル・深いネスト見出しの上限挙動
- [ ] G-BV-5: 対象外見出しレベル（h1 / h4 以深）の扱い
- [ ] G-BV-6: 見出しテキストが空・空白のみ・装飾文字のみ（正規化後に空文字列となるケース）
- [ ] G-BV-7: SHA-256 ハッシュの 16 文字切り詰めによる衝突境界

### 2.2 エラーハンドリング

- [ ] G-EH-1: 不正な TOML 入力時の挙動（クラッシュ禁止）
- [ ] G-EH-2: project_root 外への参照パス（パストラバーサル）
- [ ] G-EH-3: 未解決エッジ（from/to が存在しないノード ID）の扱い
- [ ] G-EH-4: メモリ不足等 OS 起因失敗の報告経路（exit 1）
- [ ] G-EH-5: DAG 違反（サイクル検出）時の報告

### 2.3 状態遷移

- [ ] G-ST-1: ノードタイプ（ドキュメント / 自動生成サブノード / 明示サブノード）の区別が型・スキーマで表現されているか
- [ ] G-ST-2: 自動生成サブノードと明示サブノードの形式的区別（衝突しない設計）

### 2.4 並行性

- [ ] G-CC-1: refresh-subnodes --apply による graph.toml 書き換え中の同時アクセス
- [ ] G-CC-2: グラフ構築・走査の決定論性が並行実行下でも保たれるか

### 2.5 バージョニング・互換性

- [ ] G-VC-1: 既存 graph.toml（サブノード定義なし）の無変更動作（後方互換）
- [ ] G-VC-2: サブノード ID 生成式の v3 互換（凍結契約）
- [ ] G-VC-3: refresh-subnodes コマンドの引数互換（LGX-COMPAT-001 #9）
- [ ] G-VC-4: 入力エンコーディング（BOM 付き UTF-8）・改行コード（CRLF/LF）が見出し正規化・ID 生成に与える影響

### 2.6 永続化

- [ ] G-PS-1: refresh-subnodes --apply の graph.toml 書き換えの atomicity（書き込み途中のクラッシュ・部分書き込み）
- [ ] G-PS-2: バックアップ `graph.toml.refresh-bak.{epoch}` の生成保証・命名衝突・保持/掃除方針
- [ ] G-PS-3: matrix.md の読み取り専用ビュー（手動編集禁止）

### 2.7 入力検証

- [ ] G-IV-1: 見出しテキスト正規化（前後空白・連続空白・Markdown 装飾・全角スペース・NFC）
- [ ] G-IV-2: 明示サブノード ID（`s:` 接頭辞）の文字制約検証
- [ ] G-IV-3: ノード必須フィールド（id/type/path）の存在検証

### 2.8 ライフサイクル

- [ ] G-LC-1: 見出しリネーム時のサブノード ID 連鎖変化（正しい挙動として）
- [ ] G-LC-2: 同一 heading_path 衝突時の縮退・明示優先スキップ
- [ ] G-LC-3: 見出し削除によりダングリング化したエッジの扱い

### 2.9 ロギング・観測性

- [ ] G-OB-1: 生成段階で無言処理（REQ.12 の縮退、REQ.11 の未解決エッジ除外）が起きたとき、診断可能なログ/トレースが残るか

### 2.10 FFI / 境界 API（CLI/MCP 凍結契約）

- [ ] G-FA-1: refresh-subnodes が Admin Surface 専用（MCP 非公開、MCP-INV-1）であること
- [ ] G-FA-2: refresh-subnodes の dry-run / --apply 排他・既定 dry-run の維持

### 2.11 領域固有観点（content-addressing / Unicode / DAG / graph-VCS）

- [ ] G-DM-1: コンテンツアドレッシングの決定論（同一 parent_id + heading_path → 常に同一 ID）
- [ ] G-DM-2: heading_path 構築規則（h2 は自見出しのみ / h3 は h2 コンテキスト + 自見出し / h1 で h2 コンテキストリセット）
- [ ] G-DM-3: Unicode 正規化エッジケース（合成済み vs 分解、互換等価、サロゲートペア、ZWJ）
- [ ] G-DM-4: DAG サイクル検出が全エッジ種別（Chain/Custom/ParentChild）横断で行われるか
- [ ] G-DM-5: 未解決エッジが DAG 検証から除外されること（循環判定への非影響）
- [ ] G-DM-6: 決定論的格納順（IndexMap、TOML パーサの順序保持）

## 3. RED / GREEN 判定

| 観点 | 判定 | SPEC §X.Y で回答 / 委譲先 | 関連 GAP |
|---|---|---|---|
| G-BV-1 空グラフ | GREEN | REQ.09（サブノードゼロ運用）+ REQ.01（構造検証）。空ノード/エッジは線形走査の自明系 | — |
| G-BV-2 サブノードゼロ | GREEN | REQ.09（任意性、v0.1.0 互換） | — |
| G-BV-3 単一/ゼロ見出し | GREEN | REQ.05（h2/h3 抽出、h1/h4 対象外）+ REQ.10（対象外レベルは無視） | — |
| G-BV-4 巨大/深ネスト | GREEN | REQ.10（明示上限なし・線形走査、ファズ境界へ委譲）+ NFR HARDEN.03 | — |
| G-BV-5 対象外見出し | GREEN | REQ.05（h1/h4 以降は抽出対象外）+ REQ.10（エラーにしない） | — |
| G-BV-6 空/装飾のみ見出し | GREEN | REQ.06（正規化規則が決定的に適用される）。正規化後文字列がそのままハッシュ入力 | — |
| G-BV-7 ハッシュ切り詰め衝突 | GREEN | REQ.05（生成式）+ SUBNODE-INV-3 → SPEC-LGX-004.REQ.01/07（SubnodeIdUniqueness で異親同値の切り詰め衝突も ID 一意性違反として検出） | — |
| G-EH-1 不正 TOML | GREEN | REQ.10（クラッシュ禁止）+ NFR SEC.03 | — |
| G-EH-2 パストラバーサル | GREEN | REQ.10（project_root 外参照でクラッシュ禁止）+ NFR SEC.06 | — |
| G-EH-3 未解決エッジ | GREEN | REQ.11（部分グラフ構築 + unresolved_edges 記録、例外伝播禁止） | — |
| G-EH-4 OS 起因失敗 | GREEN | REQ.10（通常エラー経路 exit 1 で報告） | — |
| G-EH-5 DAG 違反報告 | GREEN | REQ.07（Kahn）+ 報告は SPEC-LGX-004（検証）へ委譲 | — |
| G-ST-1 ノード種別区別 | GREEN | REQ.03（スキーマ）+ REQ.05（自動/明示の ID 形式）+ §4 CTX-INV-2 | — |
| G-ST-2 自動/明示の形式的区別 | GREEN | REQ.05（hex16 vs s:name）+ SUBNODE-INV-6 | — |
| G-CC-1 書き換え中の同時アクセス | GREEN | scope: NFR-LGX-001（SEC.08 単独開発者前提 + §11 非目標「マルチユーザ同時編集」非対象）+ 基盤 §11.1（graph.toml は Git 管理）。並行性脅威モデルは NFR が所有し対象外宣言済（敵対的精査 2026-06-09: GAP-LGX-021 OUT_OF_SCOPE 削除） | — |
| G-CC-2 並行下の決定論 | GREEN | REQ.08（決定論的順序、同一入力→同一順序）+ NFR REL.05。並行は読み取り主体で共有状態を持たない | — |
| G-VC-1 既存 graph.toml 後方互換 | GREEN | REQ.09 + LGX-EXT-001 §4.4（後方互換性） | — |
| G-VC-2 ID 生成式 v3 互換 | GREEN | REQ.05（v3 実測の生成式を凍結、QSET-LGX-002 Q1） | — |
| G-VC-3 refresh-subnodes 引数互換 | GREEN | REQ.13 + LGX-COMPAT-001 §4 #9（凍結契約） | — |
| G-VC-4 BOM/改行コードの ID 影響 | GREEN | scope: NFR-LGX-001.COMPAT.07（BOM 付き UTF-8 受容）+ COMPAT.08（LF/CRLF 両受容、出力 LF 統一）がファイル IO 層で BOM/改行を正規化。見出し抽出は正規化済み入力に対し走査するため REQ.06 の対象外（敵対的精査 2026-06-09: GAP-LGX-022 ALREADY_ANSWERED〔NFR 層〕削除） | — |
| G-PS-1 書き換え atomicity | GREEN | (該当なし。REQ.13 はバックアップ生成のみ、temp+rename 等の atomicity は沈黙。REL.01 は engine.db 限定)【2026-06-10 解消: SPEC 改訂適用（人間承認）により GAP closed】 | GAP-LGX-023（GENUINE / minor。敵対的精査 2026-06-09 維持: 一次データ完全性、ただしバックアップで復旧可のため minor） |
| G-PS-2 バックアップ命名衝突/保持 | GREEN | (REQ.13 は生成のみ規定、命名衝突・掃除は沈黙)【2026-06-10 解消: weak GAP fix 適用（人間裁定）により GAP closed】 | GAP-LGX-024（WEAK_OR_PADDED / minor。敵対的精査 2026-06-09 維持: 低 decision-relevance、人間判断で drop 可） |
| G-PS-3 matrix.md 読み取り専用 | GREEN | REQ.02（自動生成ビュー、手動編集禁止、CI 検知） | — |
| G-IV-1 見出し正規化 | GREEN | REQ.06（前後空白/連続空白/装飾/全角/NFC を明示列挙） | — |
| G-IV-2 明示 ID 文字制約 | GREEN | REQ.05 + LGX-EXT-001 §4.5.2（英数字/ハイフン/_、1〜63 字、英数字始終）。強制は SPEC-LGX-004.REQ.07 | — |
| G-IV-3 必須フィールド検証 | GREEN | REQ.03（id/type/path 必須）。強制検証は SPEC-LGX-004.REQ.01/07 へ委譲 | — |
| G-LC-1 ID 連鎖変化 | GREEN | REQ.05 + SUBNODE-INV-5 + LGX-EXT-001 §4.5.1 連鎖変化、REQ.13 が連鎖反映を支援 | — |
| G-LC-2 衝突縮退/明示優先 | GREEN | REQ.12（自動同士は同一 ID に縮退、明示衝突は生成スキップ） | — |
| G-LC-3 ダングリングエッジ | GREEN | REQ.11（未解決エッジとして部分グラフ構築・記録、Warning は SPEC-LGX-004.REQ.10） | — |
| G-OB-1 無言処理の診断可能性 | GREEN | scope: SPEC-LGX-004.REQ.10（UnresolvedEdge Warning）+ REQ.14（SubnodeIdCollision Warning）が検出・可視化を所有。ログレベル方針は NFR-LGX-001.OBS.01（RUST_LOG/tracing）が所有。REQ.11/REQ.12 は明示的に検出を check へ委譲済（敵対的精査 2026-06-09: GAP-LGX-025 OUT_OF_SCOPE 削除） | — |
| G-FA-1 Admin Surface 専用 | GREEN | REQ.13（MCP 非公開、MCP-INV-1）+ LGX-COMPAT-001 §5（MCP 3 ツールに refresh なし） | — |
| G-FA-2 dry-run/apply 排他 | GREEN | REQ.13（排他、既定 dry-run）+ LGX-COMPAT-001 §4 #9 | — |
| G-DM-1 アドレッシング決定論 | GREEN | REQ.05 + SUBNODE-INV-5 + NFR REL.04 | — |
| G-DM-2 heading_path 構築規則 | GREEN | REQ.05（h2 は自見出し、h3 は h2 コンテキスト+自見出し、h1 でリセット） | — |
| G-DM-3 Unicode 正規化エッジ | GREEN | REQ.06（NFC 適用を明示）。検証は TS T-HN-001〜003 + 異形テスト | — |
| G-DM-4 全エッジ種別横断 DAG | GREEN | REQ.07（Chain/Custom/ParentChild 全種別を対象と明示） | — |
| G-DM-5 未解決エッジ DAG 非影響 | GREEN | REQ.11（DAG 検証との独立を明示） | — |
| G-DM-6 決定論的格納順 | GREEN | REQ.08（IndexMap、順序保持 TOML パーサ要件）+ NFR REL.05 | — |

## 4. ステータスの決定

- 敵対的精査パス（2026-06-09）後、RED 観点は 2 件（G-PS-1, G-PS-2）に縮小。いずれも GAP-LGX-023（GENUINE/minor）・GAP-LGX-024（WEAK/minor）に対応。両者とも minor のため人間判断で一括 drop すれば本 TP は green に到達可能。
- 残 RED 2 件が存在するため、本 TP のステータスは `**ステータス**: green`。

> 2026-06-10 追記（weak GAP fix 適用後）: 残存していた weak/minor GAP も SPEC 改訂（人間裁定 fix・承認 2026-06-10）で全件 closed。全観点 GREEN のためステータスを green に更新。

> 2026-06-10 追記: GENUINE GAP は SPEC 改訂（人間承認 2026-06-10）で全件 closed（本 TP の該当観点を GREEN 化）。残る RED は weak/minor（人間判断で drop 可）のみであり、weak 裁定が完了するまでステータスは red を維持する。
- 残 GAP（GAP-LGX-023, GAP-LGX-024）が close または drop されたら全観点を再評価し、`green` に更新する。

## 5. 観点ナレッジベース参照

この TP の生成時に参照したナレッジベース章節:

- `docs/perspectives/core-perspectives.md` §境界値, §エラーハンドリング, §状態遷移, §並行性, §バージョニング・互換性, §永続化, §入力検証, §ライフサイクル, §ロギング・観測性, §領域固有観点（バージョン管理系 Git 由来）
- `docs/perspectives/core-perspectives.md` §FFI / 境界 API 観点（ABI 互換性・境界契約）
- 領域固有観点（コンテンツアドレッシング決定論 / 切り詰め衝突 / Unicode 正規化 / DAG 全種別横断）は本 SPEC のドメインから導出

## 6. 履歴

| 日付 | 変更内容 |
|---|---|
| 2026-06-08 | 初版作成。観点 41 件（10 カテゴリ + 領域固有）追加。GREEN 36 / RED 5。RED 観点に GAP-LGX-021〜025 を起票 |
| 2026-06-09 | 敵対的精査パス: 削除 3 件 / 維持 2 件 |
| 2026-06-10 | SPEC 改訂適用（人間承認 2026-06-10、spec-change-proposals/2026-06-09_genuine-gap-resolution-proposals.md）: GENUINE GAP に対応する観点を GREEN 化。GAP-157 は人間裁定・案A、GAP-064 は GraphDag 新設 + DocumentId 行欠落 Error、GAP-120 は凍結契約への加算的拡張承認。ADR-LGX-001〜008 起票 |
| 2026-06-10 | weak GAP 解消適用（人間裁定 fix・承認 2026-06-10、spec-change-proposals/2026-06-10_weak-gap-resolution-proposals.md）: 残存 RED 観点（weak/minor）を全て GREEN 化。個別裁定: GAP-085=打ち切り Info 追加 / GAP-135=永続保持 / GAP-169=タイムアウト導入【v3 差分】。ADR-LGX-009〜011 起票。open GAP 0 となり本 TP は green |
