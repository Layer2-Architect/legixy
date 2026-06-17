# Document ID: SPP-LGX-004

**親 QSET**: QSET-LGX-004
**対象 SPEC**: SPEC-LGX-004（v0.5.0 → v0.6.0）
**作成日**: 2026-06-07
**作成者**: AI (designer)
**承認状態**: 承認済（2026-06-07 by 開発者。一括承認 — QSET 対応分として全差分を承認）

---

## 概要

QSET-LGX-004 への開発者回答（2026-06-07 確定）を反映した SPEC 差分案。中心は終了コード規約の整合（Q1、互換契約側の追記を伴う）、check と SPEC-LGX-010 系コマンドの責務境界の明文化（Q2/Q3/Q4）。あわせて QSET-LGX-002 Q2 回答で確定した SubnodeIdCollision 検査（check 側の責務）を新設する。

**ハードルール 1**: 本 SPP は人間が承認するまで SPEC に反映されない。

---

## 差分一覧

### 差分 1: 終了コード規約の整合（矛盾解消）

**対応 QSET 質問**: Q1

**SPEC 修正前**（§3 REQ.04）:

```
### SPEC-LGX-004.REQ.04: 終了コード

**内容:** `check` および `check --formal` は以下の終了コードを返す:
- `0`: Error 件数が 0
- `1`: Error 件数が 1 以上
- `2`: 使用法誤り（不正な引数等）

**根拠:** NFR-LGX-001.OBS.05
**検証方法:** E2E テスト
```

**SPEC 修正後**:

```
### SPEC-LGX-004.REQ.04: 終了コード

**内容:** `check` および `check --formal` は以下の終了コードを返す:
- `0`: Error 件数が 0
- `1`: Error 件数が 1 以上
- `2`: 使用法誤り（引数パーサ層〔clap〕が検出する構文レベルの誤り: 未知フラグ・必須引数欠落・型不正等）

exit 2 は check 固有ではなく**全サブコマンド共通の規約**であり、LGX-COMPAT-001 v1.0.1 でグローバル規約として契約化する（本 SPP 付随変更）。受理済みフラグ・引数の値の意味的不正は exit 1（実行エラー）に分類する。

**根拠:** NFR-LGX-001.OBS.05、QSET-LGX-004 Q1 回答（2026-06-07。v3 実測: check 本体は 0/1〔`crates/te-check/src/reporter.rs:50-57`〕+ clap 既定の exit 2 が全コマンドに存在）
**検証方法:** E2E テスト（0/1/2 の 3 値）
```

**付随変更（LGX-COMPAT-001 v1.0.0 → v1.0.1、本 SPP 承認と同時に適用）**: §3 末尾に以下を追記する。

```
> **グローバル規約（終了コード）**: 使用法誤り（引数パーサ層が検出する構文レベルの誤り）は全サブコマンドで exit 2 を返す（clap 既定動作の契約化）。受理済み引数の値の意味的不正・実行時失敗は exit 1。検証結果に基づく終了コード（check の Error>0 → 1 等）は各サブコマンドの規定に従う。
```

**根拠**: COMPAT §4 #3 の「0/1」は check の検証結果に基づく規定であり、exit 2 は記述漏れ（v3 バイナリの実挙動として既に存在）。SPEC-004 REQ.04 を正とする。

---

### 差分 2: check と SPEC-LGX-010 系コマンドの責務境界（境界確定）

**対応 QSET 質問**: Q2, Q3, Q4（QSET-LGX-001 Q1 の SPEC-LGX-010 新設に連動）

**SPEC 修正前**（§3 REQ.02 の実装行）:

```
**実装:** legixy-check の `SemanticChecker` が legixy-embed の bulk similarity API（SPEC-LGX-006.REQ.09）を呼び出して検出する。
```

**SPEC 修正後**:

```
**実装:** legixy-check の `SemanticChecker` が legixy-embed の bulk similarity API（SPEC-LGX-006.REQ.11）を呼び出して検出する（crate 名は例示であり DD で凍結、SPEC-LGX-001.REQ.03）。

**standalone コマンドとの責務境界（前段ループ反復 1 で確定）:** `report`（計測レポート）・standalone `drift`（ベースライン対比）・`snapshot`・`calibrate` は **SPEC-LGX-010（embedding 運用・監査）** が規定する。本 REQ の SemanticSimilarity / LinkCandidate / Drift は check 内の**判定**（閾値判定の結果のみを severity 付き findings として報告し、生スコア一覧は出力しない）であり、SPEC-LGX-010 の**計測**（判定なしの生スコア + 候補 + 統計の出力）と出力責務は重複しない。また本 REQ の Drift（content_hash 比較の Warning）と standalone `drift`（embedding 対比の定量値）は**同名だが別機能**である。
```

**根拠**: QSET-LGX-004 Q2/Q3/Q4 回答（2026-06-07）。「check = 判定（judgement）/ report = 計測（measurement）」は v3 実装の境界（`crates/te-check/src/reporter.rs:59-80` の severity 構造 vs `report.rs:31-44` の数値出力）の正準化。なお参照訂正について: SPEC-LGX-006 の REQ.09 は「サブノード対応（Phase 2 embedding 生成側）」、REQ.11 は「Bulk similarity API（SEM Block 新設、check / report / calibrate の共通基盤）」であり、SemanticChecker が呼び出す実体は REQ.11。修正前の REQ.09 参照は採番乱れ（QSET-LGX-006 Q3 で確認済み、物理順序 09→12→10→11）に起因する取り違えのため REQ.11 に訂正する。

---

### 差分 3: SubnodeIdCollision 検査の新設（QSET-LGX-002 Q2 連動）

**対応 QSET 質問**: （QSET-LGX-002 Q2 回答の check 側責務。SPP-LGX-002 差分 2 と対）

**SPEC 修正前**: （該当 REQ なし — 新設）

**SPEC 修正後**（§3 末尾、REQ.13 の後に追加）:

```
### SPEC-LGX-004.REQ.14: サブノード ID 衝突検査（SubnodeIdCollision、前段ループ反復 1 新設）

**内容:** `check --formal` は、同一ドキュメント内で正規化後 heading_path が一致する複数の見出し（SPEC-LGX-002.REQ.05 の生成式により同一 ID に縮退するもの、SPEC-LGX-002.REQ.12）を検出し、Warning として報告する:
- severity: **Warning**（Error にしない理由: v3 で check が通っていた既存プロジェクトが移行直後に G1 ゲート〔Error=0〕で fail する互換リスクの回避）
- category: `SubnodeIdCollision`（新規追加）
- message: 親ドキュメント・衝突した見出しテキスト・縮退先 ID を明示
- **検出対象は自動生成サブノード同士の縮退のみ**。graph.toml の明示ノードと衝突して生成スキップされた自動 ID（SPEC-LGX-002.REQ.12 の明示優先スキップ）は本検査の対象外とする（QSET-LGX-002 Q2 回答どおり、明示宣言は意図的な上書きとみなす）
- G1 ゲート: Warning のみのため阻害しない

【v3 差分】v3 は無言で後勝ち縮退し検出機構を持たなかった（`crates/te-graph/src/parser.rs:126-145`）。本検査は v3 サブノード仕様 TE-NEXT-EXT-001（line 725「`check --formal` で検出される」）が約束しながら実装が果たさなかった検出の回復である。

**根拠:** QSET-LGX-002 Q2 回答（2026-06-07）、SUBNODE-INV-3
**検証方法:** 同名見出し 2 つの fixture で Warning 1 件が報告されるテスト
```

あわせて §4 の SUBNODE-INV-3 行を以下に更新する:

修正前:
```
| SUBNODE-INV-3（ID 一意性） | 検証 | REQ.01, REQ.07（SubnodeIdUniqueness） |
```
修正後:
```
| SUBNODE-INV-3（ID 一意性） | 検証 | REQ.01, REQ.07（SubnodeIdUniqueness）, REQ.14（SubnodeIdCollision、自動生成の縮退検出） |
```

---

### 差分 4: バージョンと変更履歴（機械的）

```
ヘッダ表: | Version | 0.5.0 | → | Version | 0.6.0 |
```

§5 変更履歴に追加:

```
| 2026-06-07 | 0.6.0 | 前段ループ反復 1（QSET-LGX-004 回答 → SPP-LGX-004 承認）対応: REQ.04 の exit 2 を全コマンド共通規約として明確化（LGX-COMPAT-001 v1.0.1 と連動）。REQ.02 に SPEC-LGX-010 との責務境界（check=判定 / report=計測、check 内 Drift と standalone drift の書き分け）を明文化、bulk API 参照を REQ.09→REQ.11 に訂正。REQ.14 SubnodeIdCollision 検査（Warning、QSET-LGX-002 Q2 連動【v3 差分】）を新設 |
```

---

## 影響範囲

| 成果物 | 影響内容 | 再評価必要性 |
|---|---|---|
| LGX-COMPAT-001 | v1.0.1 改訂（グローバル終了コード規約の追記）。本 SPP 承認と同時に適用 | なし（付随変更） |
| SPEC-LGX-010 | 責務境界の対側（REQ.04 の check/report 境界）は記載済み | なし |
| SPEC-LGX-002 | REQ.14 の生成側挙動は SPP-LGX-002 差分 2 が対応（同時承認が望ましい） | あり（SPP-LGX-002） |
| SPEC-LGX-009 | exit 1/2 の MCP 表現は SPP-LGX-009 差分 2 が対応 | あり（SPP-LGX-009） |
| TP / GAP / RBA 以降 | 未生成のため影響なし | なし |

## 承認手順 / 却下時の手順

SPP-LGX-001 と同一（承認 → SPEC 反映 → FCR-LGX-004 発行。却下 → 次の空き連番で QSET 再発行）。
