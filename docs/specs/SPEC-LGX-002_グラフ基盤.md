Document ID: SPEC-LGX-002

# SPEC-LGX-002: グラフ基盤

| 項目 | 内容 |
|------|------|
| Document ID | SPEC-LGX-002 |
| Version | 0.4.3 |
| Status | Approved（人間査読済） |
| Date | 2026-04-17 |
| Classification | CONFIDENTIAL |
| 親文書 | SPEC-LGX-001, LGX-EXT-001 §3, §4 |
| 対応 NFR | NFR-LGX-001.PERF.04, PERF.05, SEC.03, REL.04 |
| 対応 UC | UC-LGX-001, UC-LGX-003 |

---

## 1. 本文書の位置づけ

### 1.1 目的

legixy の一次データ構造である**有向グラフ**と**サブノード**の要求を定義する。具体的なデータスキーマは LGX-EXT-001 §4 に委ねる。

### 1.2 スコープ

**含む:** graph.toml のデータモデル要求、サブノード化方針、DAG 制約
**含まない:** 実装クラス構造、テスト手順（それぞれ対応する DD / TS で定義）

---

## 2. 参照文書

- LGX-EXT-001 §3 サブノード化の基本概念
- LGX-EXT-001 §4 データ構造の変更
- LGX-EXT-001 §7.1 追加不変条件 SUBNODE-INV-1〜6

---

## 3. 要求事項

### SPEC-LGX-002.REQ.01: graph.toml を一次データとする

**内容:** legixy はトレーサビリティの一次データとして `docs/traceability/graph.toml` を採用する。全てのノード・エッジ情報はここに一元化される。
**根拠:** LGX-EXT-001 §4.1
**検証方法:** legixy `check` が graph.toml を読み込み構造検証できること

### SPEC-LGX-002.REQ.02: matrix.md は読み取り専用ビュー

**内容:** `docs/traceability/matrix.md` は graph.toml から自動生成される人間可読ビューに位置づけられる。手動編集は禁止する。
**根拠:** LGX-EXT-001 §4.4
**検証方法:** matrix.md 冒頭に生成元情報を明記、編集検知は CI で実施

### SPEC-LGX-002.REQ.03: ノードのスキーマ

**内容:** 各ノードは以下のフィールドを持つ:
- id（必須）: `{type}-{area}-{seq}` または サブノード ID 形式
- type（必須）: 成果物タイプコード（SPEC, UC, RB, SEQ, DD, TS, TC, SRC 等）
- path（必須）: 対応ファイルの相対パス
- parent（サブノードのみ）: 親ノード ID
- anchor（サブノードのみ）: 見出しテキスト
- content_range（サブノードのみ、自動生成時）: 本文のバイト範囲

**根拠:** LGX-EXT-001 §4.1
**検証方法:** 対応実装が全フィールドを正しく扱うこと（テストは TS-LGX-001 T-GP-001〜003）

### SPEC-LGX-002.REQ.04: エッジの種類

**内容:** エッジは以下 3 種とする:
- `chain`: 成果物連鎖（UC→RB→SEQ→DD→TS→TC→SRC）
- `custom`: 任意の参照関係（人間が graph.toml に明示）
- `parent_child`: 親ドキュメント → サブノードの暗黙エッジ（システム生成）

**根拠:** LGX-EXT-001 §4.2
**検証方法:** サブノードに対して ParentChild エッジが自動生成されること（TS-LGX-001 T-GP-003, T-GP-005）

### SPEC-LGX-002.REQ.05: サブノードの 2 方式

**内容:** サブノードは以下 2 方式で生成される:
- **自動生成:** Markdown の h2/h3 見出しから抽出。ID は以下の式で生成する:

  `{親ID}#{hex(SHA-256({親ID} + "|" + heading_path.join("|"))) の先頭 16 文字}`

  - **heading_path**: h2 見出しは `[正規化済み自見出し]`、h3 見出しは `[正規化済み h2 コンテキスト, 正規化済み自見出し]` の配列。h1 の出現で h2 コンテキストはリセットされる（h1 自体は heading_path に含まれない）
  - 見出しの正規化は REQ.06 に従う
- **明示:** graph.toml に明示的にノード宣言。ID は `{親ID}#s:{任意名}`

h1 と h4 以降は抽出対象外。

- **コードフェンス内 `#` 行の扱い（v3 実測凍結）:** Markdown コードフェンス（` ``` ` / `~~~` ブロック）内の `#` 始まり行も見出しとして抽出される（フェンスを認識しない）。これは v3 実装の挙動であり、既存 graph.toml に永続化済みのサブノード ID との一致（本 REQ 根拠）を保つため凍結する。フェンスを認識して除外する変更はサブノード ID を変える破壊的変更であり、次版 SPEC 改訂として扱う。
- **見出しレベル（heading_levels）内部属性:** 自動生成サブノードの見出しレベル（h2/h3）は、v3 踏襲の**内部属性**として保持してよい（DD-LGX-003 `extract_subnodes_with_levels`）。本属性を消費・公開する要件は現時点で存在せず、公開仕様化は consumer REQ の新設を伴う次版 SPEC 改訂とする。ID 生成（本 REQ 生成式）には影響しない。

**根拠:** LGX-EXT-001 §3.3, §3.6, §4.5、QSET-LGX-002 Q1 回答（2026-06-07。既存プロジェクトの graph.toml に永続化済みの ID と一致させるため、v3 実測の生成式〔`crates/te-graph/src/subnode/id_gen.rs`〕を凍結する）
**検証方法:** SUBNODE-INV-5, SUBNODE-INV-6 の遵守。既知の heading_path に対する生成 ID が v3 生成 ID と一致する互換テスト

### SPEC-LGX-002.REQ.06: 見出しテキストの正規化

**内容:** 自動生成サブノードの heading_path には正規化された見出しテキストを用いる:
- 前後空白削除
- 連続空白を 1 つに統合
- **Markdown 装飾文字の除去**: `**`, `__`, `*`, `_`, `` ` ``, `~~`（強調・斜体・インラインコード・打消線マーカー）
- **全角・半角スペースの正規化**: 全角スペース（U+3000）を半角スペース 1 つに変換
- **Unicode 正規化**: NFC（Canonical Composition）を適用

これらの正規化により、Markdown フォーマッタ（Prettier 等）による微細な文字列変化が ID 不安定性を引き起こさないことを保証する。

**根拠:** LGX-EXT-001 §4.6、VAL-LGX-001 Finding E-06
**検証方法:** TS-LGX-001 T-HN-001〜003 + 装飾文字・全角スペース・Unicode 異形に対するテスト追加

### SPEC-LGX-002.REQ.07: DAG 制約

**内容:** graph.toml で表現されるグラフはサイクルを含んではならない（SUBNODE-INV-4）。
- サイクル検出は Kahn's algorithm を用いる
- 対象エッジ種別: **Chain / Custom / ParentChild の全種別**を対象とする（サイクルは種別を問わず禁止）
- Custom エッジが Chain 上流に影響しないこと（CTX-INV-3）は意味的制約として別途 SPEC-LGX-003 で扱う

**根拠:** LGX-EXT-001 §7.1 SUBNODE-INV-4、VAL-LGX-001 Finding P-02
**検証方法:** TS-LGX-001 T-VL-001, T-VL-002

### SPEC-LGX-002.REQ.08: 決定論的順序（CTX-INV-1）

**内容:** ノードの格納順は graph.toml 記載順（IndexMap）に従い、同一入力からは常に同一順序で走査結果を返す。
- 使用する TOML パーサは**ノード定義順を保持する実装**でなければならない（Rust の場合は `toml` crate の `preserve_order` / `indexmap` feature 等）
- 内部データ構造は `IndexMap<NodeId, Node>` を推奨

**根拠:** LEGIXY-SPEC-001 §10 CTX-INV-1、VAL-LGX-001 Finding P-03
**検証方法:** TS-LGX-001 T-GT-005、および同一 graph.toml の複数回パースで同一順序が得られることを確認するテスト

### SPEC-LGX-002.REQ.09: サブノード化の任意性

**内容:** プロジェクトはサブノード化を必須としてはならない。全ドキュメントノードのみで運用することも可能（v0.1.0 互換）。
**根拠:** LGX-EXT-001 §3.5
**検証方法:** サブノードゼロのテストケース（TS-LGX-001 T-GP-002）

### SPEC-LGX-002.REQ.10: 入力耐性

**内容:** graph.toml パーサおよび Markdown 見出し抽出は、以下の悪条件でクラッシュ（panic / abort）してはならない:
- 大きすぎる Markdown ファイル
- 深いネスト見出し
- 不正な TOML
- project_root 外への参照パス

入力サイズ・見出しネスト深さに**明示上限は設けない**:
- 見出し抽出は行単位走査で入力サイズに対し線形に動作する
- 対象外レベルの見出し（h1、h4 以深）は無視する（エラーにしない）
- メモリ不足等の OS 起因の失敗は通常のエラー経路（exit 1）で報告する
- 出力側の上限は SPEC-LGX-003.REQ.13（500,000 文字）が独立に規定する（入力側とは無関係）
- 巨大入力・深ネストはファズテスト境界（TS の検証観点）とし、SPEC のしきい値とはしない

**根拠:** NFR-LGX-001.SEC.03, SEC.04, SEC.06、QSET-LGX-002 Q4 回答（2026-06-07。新規上限の導入は「v3 で処理できた入力が legixy でエラー」となる互換破壊のため、v3 実測〔上限なし・線形走査〕を明文化）
**検証方法:** ファズテスト（NFR-LGX-001.HARDEN.03）

### SPEC-LGX-002.REQ.11: 未解決エッジの許容性（CTX-INV-5 実装）

**内容:** エッジの from/to に指定されたノード ID がグラフに存在しない場合、以下の動作を行う:
- **グラフ構築は継続する**: 未解決エッジを除外した部分グラフとして TraceGraph を構築する
- **未解決エッジの記録**: 除外したエッジは `unresolved_edges` として保持し、SPEC-LGX-004 の検証で Warning として報告される
- **クラッシュ・例外伝播の禁止**: パースエラーや IndexOutOfBounds 等の実行時エラーを発生させない
- **DAG 検証との独立**: 未解決エッジは DAG 検証（REQ.07）の対象外とする（循環判定に影響しない）

この挙動により、Markdown 見出しの一時的な削除・リネームで発生するダングリング参照がシステム全体の動作を停止させないことを保証する。
**根拠:** LEGIXY-SPEC-001 §10 CTX-INV-5、VAL-LGX-001 Finding E-01
**検証方法:** 存在しないノード ID を参照する graph.toml のパース + 除外エッジの記録確認テスト

### SPEC-LGX-002.REQ.12: 同一 heading_path の衝突時挙動（前段ループ反復 1 新設）

**内容:** 同一ドキュメント内に正規化後 heading_path が一致する見出しが複数存在する場合（REQ.05 の生成式により同一 ID となる）、生成挙動は以下とする:
- 自動生成サブノード同士の衝突: 同一 ID に**縮退**する（v3 実測の正準化。グラフ上は 1 ノードとして扱われる）
- graph.toml の明示ノードと衝突する自動生成 ID: 自動生成側を**生成スキップ**する（明示宣言を優先）
- 生成段階ではエラー・Warning を発しない。**検出と可視化は check が担う**（SPEC-LGX-004 REQ.14〔SubnodeIdCollision Warning、SPP-LGX-004 で新設。検出機構の新設は【v3 差分】 — v3 は無言縮退のみ〕）

**根拠:** QSET-LGX-002 Q2 回答（2026-06-07）。v3 実測（`crates/te-graph/src/parser.rs:126-145`）。なお v3 のサブノード仕様 TE-NEXT-EXT-001（line 725）は check による衝突検出を規定しており、SPEC-LGX-004 REQ.14 はその回復である
**検証方法:** 同名見出し 2 つの fixture で縮退動作を確認するテスト

### SPEC-LGX-002.REQ.13: refresh-subnodes コマンド（サブノード ID 連鎖反映、前段ループ反復 1 新設）

**内容:** `refresh-subnodes [--dry-run] | [--apply]`（排他、既定 dry-run）は、見出しリネームにより未解決化したサブノード ID を検出し、graph.toml へ連鎖反映する:
- **検出**: 現行の見出しから再生成した ID と graph.toml 上の既存サブノード ID を突合し、リネームによる新旧 ID 対応を提示する（dry-run 既定）
- **書き換え（--apply 時）**: graph.toml の `[[nodes]]` の `id`・`parent`、`[[edges]]` の `from`・`to` を新 ID へ書き換える
- **バックアップと書き換えの atomicity（GAP-LGX-023 対応）**:
  1. 書き換え前に `graph.toml.refresh-bak.{unix epoch 秒}` を作成する。**バックアップ作成に失敗した場合は本体書き換えに進まない**（exit 1）
  1a. **命名衝突と保持（GAP-LGX-024 対応）**: 同一秒内の再実行で退避名が衝突する場合は連番サフィックスを付与し**既存バックアップを上書きしない**（SPEC-LGX-008.REQ.02a と同一規約）。バックアップは累積し legixy が機械的に削除することはない（手動掃除前提）。配置は graph.toml と同一ディレクトリ。バックアップファイルは VCS 追跡対象外とすることを**推奨**する（.gitignore 例: `*.refresh-bak.*`。規範ではなく運用ガイダンス）
  2. graph.toml の置換は**同一ディレクトリの一時ファイルへ全量書き出し → fsync → アトミック rename** で行う（直接上書き禁止。【v3 差分】 — v3 は `std::fs::write` による直接上書き〔`refresh_subnodes.rs:357`〕で、中断時に graph.toml が破損し得た）
  3. 書き出し・rename のいずれかが失敗した場合は exit 1 とし、元の graph.toml は無傷のまま残す
  4. 順序不変条件: バックアップ作成 → 一時ファイル書き出し + fsync → rename の順を守る
- 本コマンドは Admin Surface 専用（MCP 非公開、MCP-INV-1）

**根拠:** LGX-COMPAT-001 §4 #9（凍結契約）、QSET-LGX-002 Q3 / QSET-LGX-001 Q1 回答（2026-06-07）、v3 実測（`crates/te-cli/src/commands/refresh_subnodes.rs:285-360`）、GAP-LGX-023（NFR REL.01 は engine.db/WAL 限定で平文 graph.toml を保護しない）
**検証方法:** リネーム fixture での dry-run / --apply / バックアップ生成の E2E テスト。加えて①書き換え中断シナリオで元 graph.toml が無傷であること、②一時ファイルが graph.toml と同一ディレクトリに作成されること、③バックアップ作成失敗時に本体が書き換えられないこと、のテスト

---

## 4. 不変条件との関係

| 不変条件 | 役割 | 対応要求 |
|---------|------|---------|
| CTX-INV-1（決定論保証） | 実装 | REQ.08（IndexMap 挿入順） |
| CTX-INV-2（グラフ整合性） | 実装 | REQ.03（ノードスキーマの強制） |
| CTX-INV-3（カスタムエッジ独立性） | 関連 | REQ.04（Chain/Custom/ParentChild の種別区別） |
| CTX-INV-4（DAG 制約） | 実装 | REQ.07（Kahn's algorithm、全エッジ種別対象） |
| CTX-INV-5（未解決エッジの許容性） | 実装 | REQ.11（部分グラフ構築 + unresolved_edges 記録） |
| SUBNODE-INV-1（親存在） | 実装 | REQ.03（parent_id フィールド、違反検出は SPEC-LGX-004） |
| SUBNODE-INV-2（パス整合性） | 実装 | REQ.03（path フィールド、違反検出は SPEC-LGX-004） |
| SUBNODE-INV-3（ID 一意性） | 実装 | REQ.05, REQ.12（明示ノードとの衝突は生成スキップ、自動同士の同一 heading_path は同一 ID に縮退。違反の検出〔SubnodeIdCollision Warning〕は SPEC-LGX-004.REQ.14） |
| SUBNODE-INV-4（DAG） | 実装 | REQ.07 |
| SUBNODE-INV-5（ID 決定性） | 実装 | REQ.05（SHA-256 ベース） |
| SUBNODE-INV-6（ID フォーマット） | 実装 | REQ.05（hex16 / s:name の形式） |

**本 SPEC が関与しない不変条件:** MCP-INV-1〜4（SPEC-LGX-007, SPEC-LGX-009 で扱う）

---

## 5. 変更履歴

| 日付 | バージョン | 変更内容 |
|------|----------|---------|
| 2026-04-17 | 0.1.0-draft | 初版（AI 起草） |
| 2026-04-17 | 0.1.1-draft | F-03 修正: REQ.03/REQ.04 の検証方法から DD-LGX-001 への逆参照を削除し、TS 参照に置き換え |
| 2026-04-17 | 0.1.2-draft | F-04 修正: §4 表に「役割」列（実装/検証/関連）を追加、CTX-INV-2/3/4 と対象外不変条件を追記、CTX-INV-1 名称を「決定論的順序」→「決定論保証」に訂正 |
| 2026-04-17 | 0.2.0 | 人間査読完了により承認 |
| 2026-04-17 | 0.3.0 | S1-c 対応: REQ.06 に Markdown 装飾文字・全角空白・NFC の正規化を追加（E-06）、REQ.07 に DAG 検証対象エッジ種別を明記（P-02）、REQ.08 に順序保持 TOML パーサの要件を明記（P-03）、REQ.11 未解決エッジの許容性（CTX-INV-5 実装）を新設（E-01）、§4 マトリクスに CTX-INV-5 行を追加 |
| 2026-06-07 | 0.4.0 | 前段ループ反復 1（QSET-LGX-002 回答 → SPP-LGX-002 承認）対応: REQ.05 のサブノード ID 生成式を v3 実測で精密化（parent_id + heading_path のハッシュ、凍結）。REQ.12 衝突時挙動（縮退 + 検出は SPEC-LGX-004.REQ.14）を新設。REQ.13 refresh-subnodes（dry-run/--apply/バックアップ）を新設。REQ.10 入力耐性に「明示上限なし・線形走査」を明文化。§4 SUBNODE-INV-3 行を精密化 |
| 2026-06-10 | 0.4.1 | TP[SPEC] GAP 解消（人間承認 2026-06-10）: GAP-LGX-023 対応で REQ.13 のバックアップ箇条を atomicity 規定に拡張（バックアップ失敗時非書換・temp+fsync+rename・失敗時 exit 1 で元ファイル無傷・順序不変条件）。v3 の直接上書き方式からの【v3 差分】を明記。CLI 引数・バックアップ命名は不変（凍結契約 §4 #9 維持） |
| 2026-06-10 | 0.4.2 | weak GAP 解消（人間裁定 fix・承認 2026-06-10）: GAP-LGX-024 対応で REQ.13 に命名衝突回避（同一秒は連番・上書き禁止、SPEC-LGX-008.REQ.02a と統一）・累積保持（機械削除なし）・同一ディレクトリ配置・VCS 追跡対象外推奨を追記 |
| 2026-06-13 | 0.4.3 | DD 整合（人間承認 2026-06-13、spec-change 提案 2026-06-13_dd-freeze-spec-alignment M-1/M-2）: REQ.05 にコードフェンス内 `#` 行の非認識（v3 実測凍結・永続化サブノード ID 安定のため、フェンス認識化は次版破壊的改訂）と heading_levels 内部属性の v3 踏襲（公開仕様化は consumer REQ 新設待ち、ID 生成に無影響）を明文化。DD-LGX-003 整合。生成式・既存挙動は不変 |
