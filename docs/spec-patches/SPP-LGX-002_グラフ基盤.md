# Document ID: SPP-LGX-002

**親 QSET**: QSET-LGX-002
**対象 SPEC**: SPEC-LGX-002（v0.3.0 → v0.4.0）
**作成日**: 2026-06-07
**作成者**: AI (designer)
**承認状態**: 承認済（2026-06-07 by 開発者。一括承認 — QSET 対応分として全差分を承認）

---

## 概要

QSET-LGX-002 への開発者回答（2026-06-07 確定）を反映した SPEC 差分案。中心はサブノード ID 生成式の v3 実測による精密化（Q1）、衝突時挙動の正準化（Q2）、`refresh-subnodes` のオーナー REQ 新設（Q3）、入力耐性の上限なし明文化（Q4）。

**ハードルール 1**: 本 SPP は人間が承認するまで SPEC に反映されない。

---

## 差分一覧

### 差分 1: サブノード ID 生成式の精密化（複数解釈の解消）

**対応 QSET 質問**: Q1

**SPEC 修正前**（§3 REQ.05）:

```
### SPEC-LGX-002.REQ.05: サブノードの 2 方式

**内容:** サブノードは以下 2 方式で生成される:
- **自動生成:** Markdown の h2/h3 見出しから抽出。ID は `{親ID}#{SHA-256 の先頭16文字}`
- **明示:** graph.toml に明示的にノード宣言。ID は `{親ID}#s:{任意名}`

h1 と h4 以降は抽出対象外。
**根拠:** LGX-EXT-001 §3.3, §3.6, §4.5
**検証方法:** SUBNODE-INV-5, SUBNODE-INV-6 の遵守
```

**SPEC 修正後**:

```
### SPEC-LGX-002.REQ.05: サブノードの 2 方式

**内容:** サブノードは以下 2 方式で生成される:
- **自動生成:** Markdown の h2/h3 見出しから抽出。ID は以下の式で生成する:

  `{親ID}#{hex(SHA-256({親ID} + "|" + heading_path.join("|"))) の先頭 16 文字}`

  - **heading_path**: h2 見出しは `[正規化済み自見出し]`、h3 見出しは `[正規化済み h2 コンテキスト, 正規化済み自見出し]` の配列。h1 の出現で h2 コンテキストはリセットされる（h1 自体は heading_path に含まれない）
  - 見出しの正規化は REQ.06 に従う
- **明示:** graph.toml に明示的にノード宣言。ID は `{親ID}#s:{任意名}`

h1 と h4 以降は抽出対象外。
**根拠:** LGX-EXT-001 §3.3, §3.6, §4.5、QSET-LGX-002 Q1 回答（2026-06-07。既存プロジェクトの graph.toml に永続化済みの ID と一致させるため、v3 実測の生成式〔`crates/te-graph/src/subnode/id_gen.rs`〕を凍結する）
**検証方法:** SUBNODE-INV-5, SUBNODE-INV-6 の遵守。既知の heading_path に対する生成 ID が v3 生成 ID と一致する互換テスト
```

**根拠**: QSET 選択肢 B（heading_path）を v3 実測で精密化（ハッシュ入力に親 ID プレフィクスを含む）。DD のハッシュ実装と SUBNODE-INV-3/5 の検証可能性を確立する。

**代替案**: 選択肢 A（見出し単体）/ C（本文バイト列）— 既存 graph.toml の永続 ID と不一致になり移行（SPEC-LGX-008）を壊すため不採用。

---

### 差分 2: ID 衝突時の生成挙動の正準化（例外未定義の解消）

**対応 QSET 質問**: Q2

**SPEC 修正前**: （該当 REQ なし — 新設）

**SPEC 修正後**（§3 末尾に追加）:

```
### SPEC-LGX-002.REQ.12: 同一 heading_path の衝突時挙動（前段ループ反復 1 新設）

**内容:** 同一ドキュメント内に正規化後 heading_path が一致する見出しが複数存在する場合（REQ.05 の生成式により同一 ID となる）、生成挙動は以下とする:
- 自動生成サブノード同士の衝突: 同一 ID に**縮退**する（v3 実測の正準化。グラフ上は 1 ノードとして扱われる）
- graph.toml の明示ノードと衝突する自動生成 ID: 自動生成側を**生成スキップ**する（明示宣言を優先）
- 生成段階ではエラー・Warning を発しない。**検出と可視化は check が担う**（SPEC-LGX-004 REQ.14〔SubnodeIdCollision Warning、SPP-LGX-004 で新設。検出機構の新設は【v3 差分】 — v3 は無言縮退のみ〕）

**根拠:** QSET-LGX-002 Q2 回答（2026-06-07）。v3 実測（`crates/te-graph/src/parser.rs:126-145`）。なお v3 のサブノード仕様 TE-NEXT-EXT-001（line 725）は check による衝突検出を規定しており、SPEC-LGX-004 REQ.14 はその回復である
**検証方法:** 同名見出し 2 つの fixture で縮退動作を確認するテスト
```

あわせて §4 の SUBNODE-INV-3 行を以下に更新する:

修正前:
```
| SUBNODE-INV-3（ID 一意性） | 実装 | REQ.05（ID 生成が一意性を保証、違反検出は SPEC-LGX-004） |
```
修正後:
```
| SUBNODE-INV-3（ID 一意性） | 実装 | REQ.05, REQ.12（明示ノードとの衝突は生成スキップ、自動同士の同一 heading_path は同一 ID に縮退。違反の検出〔SubnodeIdCollision Warning〕は SPEC-LGX-004.REQ.14） |
```

**根拠**: ID 生成式を不変に保ったまま（移行互換）、無言縮退を check の Warning で可視化する。Error にしない理由: v3 で check が通っていた既存プロジェクトが移行直後に G1 ゲートで fail する互換リスクの回避。

---

### 差分 3: refresh-subnodes コマンドの要求新設（責務所在の確定）

**対応 QSET 質問**: Q3（QSET-LGX-001 Q1 と連動）

**SPEC 修正前**: （該当 REQ なし — 新設）

**SPEC 修正後**(§3 末尾、REQ.12 の後に追加):

```
### SPEC-LGX-002.REQ.13: refresh-subnodes コマンド（サブノード ID 連鎖反映、前段ループ反復 1 新設）

**内容:** `refresh-subnodes [--dry-run] | [--apply]`（排他、既定 dry-run）は、見出しリネームにより未解決化したサブノード ID を検出し、graph.toml へ連鎖反映する:
- **検出**: 現行の見出しから再生成した ID と graph.toml 上の既存サブノード ID を突合し、リネームによる新旧 ID 対応を提示する（dry-run 既定）
- **書き換え（--apply 時）**: graph.toml の `[[nodes]]` の `id`・`parent`、`[[edges]]` の `from`・`to` を新 ID へ書き換える
- **バックアップ**: 書き換え前に `graph.toml.refresh-bak.{unix epoch 秒}` を作成する
- 本コマンドは Admin Surface 専用（MCP 非公開、MCP-INV-1）

**根拠:** LGX-COMPAT-001 §4 #9（凍結契約）、QSET-LGX-002 Q3 / QSET-LGX-001 Q1 回答（2026-06-07）、v3 実測（`crates/te-cli/src/commands/refresh_subnodes.rs:285-360`）
**検証方法:** リネーム fixture での dry-run / --apply / バックアップ生成の E2E テスト
```

**根拠**: サブノード ID は heading 依存のため、リネーム時の連鎖反映は graph.toml 構造操作 = グラフ基盤の責務。embedding 非依存のため SPEC-LGX-010 ではなく本 SPEC が負う。

---

### 差分 4: 入力耐性の上限なし明文化（例外未定義の解消）

**対応 QSET 質問**: Q4

**SPEC 修正前**（§3 REQ.10）:

```
### SPEC-LGX-002.REQ.10: 入力耐性

**内容:** graph.toml パーサは以下の悪条件でクラッシュしてはならない:
- 大きすぎる Markdown ファイル
- 深いネスト見出し
- 不正な TOML
- project_root 外への参照パス

**根拠:** NFR-LGX-001.SEC.03, SEC.04, SEC.06
**検証方法:** ファズテスト（NFR-LGX-001.HARDEN.03）
```

**SPEC 修正後**:

```
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
```

---

### 差分 5: バージョンと変更履歴（機械的）

```
ヘッダ表: | Version | 0.3.0 | → | Version | 0.4.0 |
```

§5 変更履歴に追加:

```
| 2026-06-07 | 0.4.0 | 前段ループ反復 1（QSET-LGX-002 回答 → SPP-LGX-002 承認）対応: REQ.05 のサブノード ID 生成式を v3 実測で精密化（parent_id + heading_path のハッシュ、凍結）。REQ.12 衝突時挙動（縮退 + 検出は SPEC-LGX-004.REQ.14）を新設。REQ.13 refresh-subnodes（dry-run/--apply/バックアップ）を新設。REQ.10 入力耐性に「明示上限なし・線形走査」を明文化。§4 SUBNODE-INV-3 行を精密化 |
```

---

## 影響範囲

| 成果物 | 影響内容 | 再評価必要性 |
|---|---|---|
| SPEC-LGX-004 | SubnodeIdCollision 検査（Warning）の新設は SPP-LGX-004 差分 3 が対応（本 SPP と同時承認が望ましい） | あり（SPP-LGX-004） |
| UC | refresh-subnodes の UC は未存在。UC フェーズで生成要否を判断 | あり（UC フェーズ） |
| TP / GAP / RBA 以降 | 未生成のため影響なし | なし |

## 承認手順 / 却下時の手順

SPP-LGX-001 と同一（承認 → SPEC 反映 → FCR-LGX-002 発行。却下 → 次の空き連番で QSET 再発行）。
