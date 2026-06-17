Document ID: SPEC-LGX-005

# SPEC-LGX-005: グラフ走査

| 項目 | 内容 |
|------|------|
| Document ID | SPEC-LGX-005 |
| Version | 0.4.0 |
| Status | Approved（人間査読済） |
| Date | 2026-04-17 |
| Classification | CONFIDENTIAL |
| 親文書 | SPEC-LGX-001, SPEC-LGX-002 |
| 対応 NFR | NFR-LGX-001.REL.05 |
| 対応 UC | UC-LGX-005（逆方向探索）, UC-LGX-006（順方向探索） |

---

## 1. 本文書の位置づけ

### 1.1 目的

グラフ走査機能（順方向・逆方向）の要求を定義する。**本 SPEC は legixy で新規に要求を明文化する**（LEGIXY-SPEC-001 および LGX-EXT-001 では明示的な SPEC 記述がない領域）。

### 1.2 スコープ

**含む:** 順方向/逆方向走査の挙動、最大深度制御、CLI コマンドインターフェース
**含まない:** 走査の実装詳細（対応 DD で定義）、MCP 経由のアクセス（Agent Surface では走査を直接提供しない）

---

## 2. 参照文書

- UC-LGX-005 逆方向探索
- UC-LGX-006 順方向探索
- SPEC-LGX-002 グラフ基盤（決定論保証、ノード/エッジ構造）

---

## 3. 要求事項

### SPEC-LGX-005.REQ.01: 順方向走査

**内容:** 指定したノード（起点）から chain / custom / parent_child エッジを**順方向**に辿り、到達可能な全ノードを返す。

**方向の一般則（GAP-LGX-081 対応）:** 全エッジ種別（chain / custom / parent_child）で「**順方向 = エッジの `from`→`to`、逆方向 = `to`→`from`**」と統一する。REQ.08 の「親ドキュメント → サブノードが順方向」はこの一般則の具体化である。custom エッジも同一規則に従い、双方向の特例は設けない。なお CTX-INV-3（カスタムエッジ独立性）は compile_context の意味的制約（所有: SPEC-LGX-003.REQ.05）であり、本 SPEC の走査到達性とは別概念で矛盾しない。

**根拠:** UC-LGX-006、SPEC-LGX-002.REQ.04（3 種別とも `from`/`to` を持つ有向エッジ）、GAP-LGX-081
**検証方法:** TS-LGX-001 T-GT-001（custom エッジを含む fixture で from→to 方向のみ到達することの確認を含む）

### SPEC-LGX-005.REQ.02: 逆方向走査

**内容:** 指定したノード（起点）から chain / custom / parent_child エッジを**逆方向**に辿り、到達可能な全ノードを返す。
**根拠:** UC-LGX-005
**検証方法:** TS-LGX-001 T-GT-002

### SPEC-LGX-005.REQ.03: BFS と決定論性

**内容:** 走査は幅優先探索（BFS）で実施する。隣接ノードの処理順は IndexMap の挿入順に従い、同一入力からは常に同一の visited 順・depth_map を返す。
**根拠:** LEGIXY-SPEC-001 §10 CTX-INV-1, SPEC-LGX-002.REQ.08
**検証方法:** TS-LGX-001 T-GT-005

### SPEC-LGX-005.REQ.04: 最大深度制御

**内容:** 走査には `max_depth` パラメータを指定できる。深度は起点を 0 として、BFS のレベルに対応する。`max_depth` を超えるノードは返さない。

**省略時の既定（前段ループ反復 1 で確定）:** `max_depth` 省略時（CLI で `--max-depth` 未指定）は**無制限**とし、到達可能な全ノードを返す。停止性は REQ.06 の visited 制御により保証される（DAG 保証下では有限）。省略時挙動は LGX-COMPAT-001 の互換対象 (d) 既定値として維持する。

**打ち切りの可観測性（GAP-LGX-085 対応、人間裁定 2026-06-10）:** `--max-depth` 指定時に深度超過で除外された到達可能ノードが 1 件以上存在する場合、**stderr に Info 1 件**（打ち切り発生 + 除外ノード件数）を出力する【v3 差分: v3 は無言打ち切り（multi_traverser.rs）。結果が部分集合であることの黙殺を可視化】。stdout の到達集合・depth_map・終了コードは不変（決定論保全）。`--json` 出力への truncated フラグの追加は REQ.09 の DD 凍結対象へ申し送る。

**根拠:** UC-LGX-005, UC-LGX-006、QSET-LGX-005 Q1 回答（2026-06-07。v3 実測〔`max_depth: Option<usize>` の `None` = 無制限、`crates/te-graph/src/traversal.rs:54-58`〕の正準化）、GAP-LGX-085
**検証方法:** TS-LGX-001 T-GT-003、--max-depth なし E2E テスト、打ち切り発生/非発生での stderr Info 有無テスト

### SPEC-LGX-005.REQ.05: 存在しない起点

**内容:** 起点ノードがグラフに存在しない場合、空の結果を返す（エラーではない）。呼出し側は事前に `contains_node` 等で確認すること。
**根拠:** 堅牢性
**検証方法:** TS-LGX-001 T-GT-004

### SPEC-LGX-005.REQ.06: 循環検出と safety

**内容:** グラフが DAG 制約を満たさない場合でも、走査は無限ループに陥ってはならない。visited セットで循環を防止する。
**根拠:** 堅牢性（SUBNODE-INV-4 が破れた状態でも走査が動作すること）
**検証方法:** サイクルあり入力での走査テスト

### SPEC-LGX-005.REQ.07: CLI インターフェース

**内容:** 以下 CLI サブコマンドを提供する:
- `legixy impact <node-id>`: 順方向走査（変更影響分析）
- `legixy investigate <node-id>`: 逆方向走査（要求追跡）

どちらも `--max-depth N` オプションを受け付ける。出力は visited ノードの一覧（ID + タイプ + パス）。
**根拠:** UC-LGX-005, UC-LGX-006
**検証方法:** CLI E2E テスト

### SPEC-LGX-005.REQ.08: サブノード対応

**内容:** 走査はドキュメントノードとサブノードの両方を対象とする。`parent_child` エッジも走査対象に含める（例: 親ドキュメント → サブノード方向が順方向）。
**根拠:** SPEC-LGX-002.REQ.04
**検証方法:** サブノード含むグラフでの走査テスト

### SPEC-LGX-005.REQ.09: 走査結果の情報と出力フォーマット

**内容:** 走査結果には少なくとも以下の情報を含む:
- 到達したノードの ID 一覧（走査順）
- 各ノードのタイプ・パス
- 走査で使用したエッジの情報
- 起点からの深度情報

**出力フォーマット（前段ループ反復 1 で確定）:**
- 既定は人間可読 Text（v3 互換: id / type / depth / path。investigate の suspicious nodes は drift 値を含む）
- グローバル `--json` 指定時は同情報の構造化 JSON を標準出力へ返す。【v3 差分】v3 は `--json` を受理するが無視して Text 固定だった（`crates/te-cli/src/main.rs:334-339` で未伝播）。受理済みフラグの機能化であり引数体系 (a)〜(f) は不変。Admin Surface の出力一貫性（SPEC-LGX-004 REQ.08 の check JSON 対応、SPEC-LGX-010 REQ.01 の 4 コマンド JSON 対応）に整合させる

具体的なフィールド名やデータ構造は DD で定義する（JSON スキーマは DD 凍結対象）。
**根拠:** UC-LGX-005, UC-LGX-006、QSET-LGX-005 Q2 回答（2026-06-07）
**検証方法:** 結果情報のシリアライズテスト、--json 出力スキーマテスト

### SPEC-LGX-005.REQ.10: Admin Surface 限定

**内容:** 走査機能は Admin Surface（CLI）でのみ提供する。MCP ツールとしては提供しない（MCP-INV-1 維持）。
**根拠:** LGX-EXT-001 §6.1, SPEC-LGX-001.REQ.08
**検証方法:** MCP ツール一覧の検証

---

## 4. 不変条件との関係

| 不変条件 | 役割 | 対応要求 |
|---------|------|---------|
| CTX-INV-1（決定論保証） | 実装 | REQ.03（BFS 順の決定論性） |
| MCP-INV-1（Agent Surface 限定） | 実装 | REQ.10（走査は Admin Surface のみ） |
| SUBNODE-INV-4（DAG） | 関連 | REQ.06（DAG 破れでも安全に走査可能） |

**本 SPEC が関与しない不変条件:** CTX-INV-2/3/4, MCP-INV-2〜4、SUBNODE-INV-1/2/3/5/6

---

## 5. 変更履歴

| 日付 | バージョン | 変更内容 |
|------|----------|---------|
| 2026-04-17 | 0.1.0-draft | 初版（AI 起草、legixy で新規に要求を明文化） |
| 2026-04-17 | 0.1.1-draft | F-03 修正: REQ.09 から DD-LGX-001 §3.9 への逆参照を削除し内容を抽象化、§2 参照文書からも DD 項目を削除 |
| 2026-04-17 | 0.1.2-draft | F-04 修正: §4 表に「役割」列を追加、対象外不変条件を明記、CTX-INV-1 名称を「決定論的順序」→「決定論保証」、MCP-INV-1 名称を「3ツール限定」→「Agent Surface 限定」に訂正 |
| 2026-04-17 | 0.2.0 | 人間査読完了により承認 |
| 2026-06-07 | 0.3.0 | 前段ループ反復 1（QSET-LGX-005 回答 → SPP-LGX-005 承認）対応: REQ.04 に max_depth 省略時 = 無制限（v3 既定の正準化、互換対象 (d)）を明記。REQ.09 に出力フォーマット（Text 既定 + --json 機能化【v3 差分】）を確定 |
| 2026-06-10 | 0.4.0 | weak GAP 解消（人間裁定 fix・承認 2026-06-10）: GAP-LGX-081 対応で REQ.01 に方向の一般則（全種別で順方向 = from→to、REQ.08 は具体化、CTX-INV-3 との無矛盾注記）を明記。GAP-LGX-085 対応（人間裁定: Info 追加）で REQ.04 に打ち切り発生時の stderr Info【v3 差分】を新設（stdout・終了コード不変、--json truncated フラグは DD 申し送り） |
