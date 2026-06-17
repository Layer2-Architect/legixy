# Document ID: SPP-LGX-010

**親 QSET**: QSET-LGX-010
**対象 SPEC**: SPEC-LGX-010
**作成日**: 2026-06-08
**作成者**: AI (designer)
**承認状態**: 承認済（2026-06-08 by 開発者。差分 1〜9 を一括承認）

---

## 概要

QSET-LGX-010 への開発者回答（2026-06-08 確定、10 サブ決定）を反映した SPEC 差分案。SPEC-LGX-010（embedding 運用・監査）の前段ループ反復 1 を閉じる。

本 SPP の承認をもって SPEC-LGX-010 は Raw SPEC（v0.1.1-draft）から受理版（v0.2.0）へ昇格し、§5「前段ループへの引き継ぎ事項」を削除する（SPEC §5 自身の規定: 「FCR が ACCEPTED となった時点で削除する」）。

回答 10 件のうち 8 件は v3 実測の正準化または高度規律に基づく DD 委任で、本 SPP 起草前に v3 実物（`traceability-engine.v3/`）と照合済み（§検証記録）。新規に導入される v3 差分は 2 件（Q2-a の `LGX_MODELS_DIR` リブランド、Q3-b の pairs=0 INFO）。

**ハードルール 1**: 本 SPP は人間が承認するまで SPEC に反映されない。AI は提案する、人間が決定する。

---

## 差分一覧

### 差分 1: snapshot_id の凍結範囲の確定（Q1-b、§5 項目 2 解決）

**対応 QSET 質問**: Q1-b（選択肢 B）

**SPEC 修正前**（§3 REQ.02 第 1 bullet）:

> - snapshot_id は `snap-` プレフィクスを持つ（確定）。プレフィクス以降の形式は §5 項目 2 の確定まで v3 実測（epoch ミリ秒 + 乱数）の記述的例示とする

**SPEC 修正後**:

> - snapshot_id は `snap-` プレフィクスを持ち（SPEC 凍結）、スナップショット領域内で**一意**である（SPEC 要求）。プレフィクス以降の内部形式（v3 実測: epoch ミリ秒 + 乱数）は consumer から不透明トークンとして扱い、生成方式は **DD で凍結**する。delete target が snapshot_id を受理する外部契約（LGX-COMPAT-001 §4 #8）は内部形式に依存しない

**根拠**: snapshot_id を不透明トークン化し、consumer が内部構造（時刻抽出等）に依存する隠れ契約を防ぐ。crate 名・parser 詳細を DD に置く本プロジェクトの高度規律（QSET-LGX-001 Q3）と同型。

### 差分 2: drift のモデル解決順とリブランドの精密化（Q2-a、§5 項目 4 解決）【v3 差分】

**対応 QSET 質問**: Q2-a（選択肢 A）

**SPEC 修正前**（§3 REQ.03「実行時依存」bullet）:

> モデルは `--models-dir` → 環境変数 → 設定ファイルの順で解決し（優先順の詳細は DD で凍結）、全解決失敗・モデル読込失敗は実行エラー（exit 1）として試行内容を stderr に通知する。環境変数名のリブランド要否は §5 項目 4

**SPEC 修正後**:

> モデルは **`--models-dir` フラグ ＞ 環境変数 `LGX_MODELS_DIR` ＞ 環境変数 `TE_MODELS_DIR`（旧名フォールバック。使用時は stderr に Info で新名を案内）＞ 設定ファイル** の順で解決する。全解決失敗・モデル読込失敗は実行エラー（exit 1）として試行内容を stderr に通知する。
> **【v3 差分】** v3 は環境変数 `TE_MODELS_DIR` のみを参照する（`crates/te-cli/src/model_dir.rs:34,67`）。legixy は `LGX_MODELS_DIR` を正準とし `TE_MODELS_DIR` を旧名フォールバックとして受理する。LGX-COMPAT-001 §3 が凍結するのは `--models-dir` フラグのみで環境変数名は凍結対象外（実機確認済み）。既存の `TE_MODELS_DIR` 設定は引き続き機能するため正当な入力空間の挙動は不変（互換安全）。両変数が同時設定された場合は `LGX_MODELS_DIR` を優先する（v3 に存在しなかった状況のため互換破壊にあたらない）

**根拠**: 他識別子の LGX 化との整合、既存スクリプト・CI の `TE_MODELS_DIR` 設定の非破壊移行。旧名フォールバックの撤去時期は将来 SPEC 改訂事項として DD へ申し送る。

### 差分 3: drift の現行ファイル欠落時の挙動（Q2-c、§5 項目 8 解決）

**対応 QSET 質問**: Q2-c（選択肢 A）

**SPEC 修正前**（§3 REQ.03、`<artifact_id>` 不在 bullet の直後に追加）:

> - `<artifact_id>` が graph.toml に存在しない場合: ERROR（stderr）+ exit 1

**SPEC 修正後**（bullet を追加）:

> - `<artifact_id>` が graph.toml に存在しない場合: ERROR（stderr）+ exit 1
> - `<artifact_id>` は graph.toml に存在するが**現行ファイルが欠落**している場合（embedding 生成のためのファイル読込失敗）: ERROR（stderr）+ exit 1。これは v3 正準でもある（`crates/te-embed/src/orchestrator.rs:160` `read_current_content_for_node` の `read_to_string(...)?` が伝播、`compute_node_drift` doc 「Err: …ファイル読込失敗…」）。
>   **非対称性の注記**: ベースライン不在（未 embed / 未 snapshot）は正常なライフサイクル状態として exit 0 で扱う一方、現行ファイル欠落は graph.toml が存在を主張するファイルが消えた**壊れた状態**として exit 1 とする。この非対称は意図的であり、明示指定の対比は壊れた状態を隠さない（REQ.03 冒頭の原則および次元不一致 Error と同列）

**根拠**: 壊れたリポジトリ状態の隠蔽防止。「baseline 不在が exit 0 なら現行不在も対称に exit 0」という反論に対し、両者は状態の性質が異なる（正常 vs 破損）ため非対称が正しい、と先回りで明記する。

### 差分 4: calibrate の pairs=0 時 INFO 追加（Q3-b、§5 項目 7 解決）【v3 差分】

**対応 QSET 質問**: Q3-b（選択肢 A）

**SPEC 修正前**（§3 REQ.05、`--recommend` bullet 内の最終サブ項目）:

> - ペア数 0（空ストア、またはノード 1 件等）の場合、`recommended_thresholds` は出力されない（v3 実測。可視化要否は §5 項目 7）

**SPEC 修正後**:

> - ペア数 0（空ストア、ノード 1 件、または全ペア次元不一致 skip 等）の場合、`recommended_thresholds` は出力されない（v3 実測）。**【v3 差分】** `--recommend` 指定かつ pairs=0 のときは stderr に INFO 1 件（「ペア数 0 のため推奨値は算出されません」）を出力する。`--json` の stdout は汚さない。v3 は無言省略していたが、`--recommend` 指定に対し無言で何も返さないのは沈黙的な機能無効化にあたるため可視化する（QSET-LGX-006 Q4 の集約 Warning と同趣旨）

**根拠**: pairs=0 は空ストアだけでなくノード 1 件・全ペア次元不一致 skip（ストア非空）でも発生し、後者は空ストア INFO（REQ.05 末尾）では捕捉できない。`--recommend` 経路に INFO を置くことで全 pairs=0 ケースを拾う。

### 差分 5: engine.db 不在時の snapshot delete 挙動の明記（Q1-d、追加検出解決）

**対応 QSET 質問**: Q1-d（選択肢 A）

**SPEC 修正前**（§3 REQ.07「engine.db 不在時【v3 差分】」bullet 末尾に追記）:

> - **engine.db 不在時【v3 差分】**: 読取専用コマンドおよび `snapshot delete` は **DB ファイルを新規作成せず**、空ストア相当の挙動（REQ.02〜05 の空ストア時挙動）で正常終了する。〔…既存文…〕

**SPEC 修正後**（同 bullet 末尾に文を追加）:

> 〔…既存文…〕**DB 不在 ≡ 空ストアであり、`snapshot delete` の挙動は REQ.02 から導出される（新たな特例ではない）: `delete label:<L>` は label 解決 0 件で ERROR + exit 1、`delete <snapshot_id>` は該当 0 行で WARNING + exit 0（`--json` 時は `{"deleted_rows": 0}`）。** DB の物理存在の有無を exit code に露出させない（露出させると REQ.07 自身の「観測可能な差は副作用ファイルの有無のみ」に反する）

**根拠**: 本設問は v3 が DB を自動作成していたため対応状況が存在しない新規確定。`label:<L>` は名前参照であり、名前解決失敗を DB 欠落理由で WARNING に格下げ（選択肢 B）すると project-root 誤り等を覆い隠すため、A で「大声で失敗」させる。

### 差分 6: 確認・委任の追認（Q1-a / Q1-c / Q2-b / Q3-a — 本文変更なし）

以下 4 決定は SPEC-LGX-010 v0.1.1-draft の現行記述を**追認**するもので REQ 本文の変更を伴わない。SPP としては「現記述で確定」を記録し、DD への申し送りのみ付す:

| 質問 | 決定 | 現 SPEC 記述（確定） | DD 申し送り |
|---|---|---|---|
| Q1-a（§5-1） | 空ストア snapshot 非永続（v3 正準） | REQ.02「複製行 0 件のため永続化されず…WARNING + exit 0」 | create 時 `warning` 文言に「ストアが空のため永続化されません」を明示 |
| Q1-c（§5-9） | 同時刻 label タイブレークは DD | REQ.02/06「同時刻タイブレークは DD で確定」 | label 解決（REQ.02）と list 降順安定出力（REQ.06）に**同一**タイブレーク規則（例: snapshot_id 全順序）を適用 |
| Q2-b（§5-5） | `--json` 時 INFO の stderr 併出（v3 正準） | REQ.03「INFO は stderr に併出する」 | （変更なし） |
| Q3-a（§5-6） | パーセンタイル式詳細は DD | REQ.05「算出式は v3 実測（nearest-rank 変種）を正準とし詳細は DD で凍結」 | nearest-rank 変種 `sorted[round((n−1)·frac)]`。検証は REQ.05「既知分布 fixture に対する推奨値一致テスト」で再現性を担保 |

### 差分 7: 対応 UC の確定と SPEC-LGX-001 連動の予告（Q4、§5 項目 3 解決）

**対応 QSET 質問**: Q4（選択肢 A）

**SPEC 修正前**（ヘッダ表「対応 UC」欄）:

> UC-LGX-010（report）, UC-LGX-011（calibrate）。snapshot / drift 系 UC は本 SPEC 受理後の UC フェーズで生成する

**SPEC 修正後**:

> UC-LGX-010（report = 健全性監査）, UC-LGX-011（calibrate = キャリブレーション）, **UC-LGX-012（snapshot = ベースライン凍結ライフサイクル）, UC-LGX-013（drift = standalone 対比）**。UC-012/013 は本 SPEC 受理後の UC フェーズで新規生成する（運用者アクター。粒度は §下記注記）

**追加注記**（§1.2 スコープ直後に小節を追加、または §3 冒頭に注記）:

> **SPEC-LGX-001 との連動**: UC-LGX-012/013 の新規生成に伴い、SPEC-LGX-001 REQ.01/REQ.02 の網羅宣言「UC-LGX-001〜011」は「001〜013」へ再改訂する必要がある。これは UC フェーズ着手時（本 SPEC FCR ACCEPTED 後、ハードルール 9）に SPP-LGX-001（次反復）として処理する。SPEC-LGX-001 v0.5.0 変更履歴は既に「snapshot/drift 系 UC は本 SPEC 受理後の UC フェーズで生成」と予告済みであり、本再改訂は網羅宣言を誠実に保つもの。
> **UC 粒度**: UC-012（snapshot）= create / list / delete ＋ 復旧フロー（誤 snapshot 削除、空ストア create の非永続）。UC-013（drift）= baseline 有無 × `--against` 3 形式 × モデル解決失敗 × 次元不一致 × 現行ファイル欠落（差分 3）を代替フローとする。基本は UC-010/011 と同粒度で AI が起草し UC レビューで調整する。

### 差分 8: §5 の削除と参照クリーンアップ（前段ループ完了処理）

- **§5「前段ループへの引き継ぎ事項」を全削除**する（SPEC §5 自身の規定: 「本セクションは…FCR が ACCEPTED となった時点で削除する」）。9 項目はそれぞれ差分 1〜7 で REQ 本文へ昇格または DD 申し送りとして処理済み。
- 本文中に残る §5 への前方参照（REQ.02「§5 項目 1」、REQ.03「§5 項目 4」「§5 項目 5」、REQ.05「§5 項目 6」「§5 項目 7」、§4 等）を該当差分の確定文言へ置換する（差分 1〜5 で実施済みのものを除く）。
- 参照文書（§2）の「QSET-LGX-001 / 004 / 006 の回答」に **QSET-LGX-010 の回答（2026-06-08）** を追加する。

### 差分 9: ヘッダと変更履歴

- ヘッダ表 `| Version | 0.1.1-draft |` → `| Version | 0.2.0 |`
- ヘッダ表 `| Status | Draft（Raw SPEC、前段ループ未完） |` → `| Status | Accepted（前段ループ反復 1 完了） |`
- §6 変更履歴に追記:

> | 2026-06-08 | 0.2.0 | 前段ループ反復 1 完了（QSET-LGX-010 回答 → SPP-LGX-010 承認 → FCR-LGX-010 ACCEPTED）。§5 の 9 引き継ぎ項目を全て確定し §5 を削除。確定: snapshot_id 不透明トークン化（Q1-b）、drift モデル解決の `LGX_MODELS_DIR` リブランド【v3 差分】（Q2-a）、現行ファイル欠落 drift = exit 1 + 非対称性注記（Q2-c）、calibrate pairs=0 INFO【v3 差分】（Q3-b）、DB 不在時 delete 挙動の REQ.02 導出明記（Q1-d）、UC-012/013 新規化と SPEC-001 連動予告（Q4）。追認: 空ストア非永続（Q1-a）/ 同時刻タイブレーク DD 委任（Q1-c）/ `--json` INFO 併出（Q2-b）/ パーセンタイル式 DD 委任（Q3-a） |

---

## 波及分析

| 対象 | 影響 | 対応要否 |
|---|---|---|
| SPEC-LGX-001（REQ.01/REQ.02 網羅宣言） | UC-012/013 新規化により「001〜011」→「001〜013」再改訂が必要 | あり（UC フェーズ着手時に SPP-LGX-001 次反復で処理。本 SPP では予告のみ） |
| LGX-COMPAT-001 | 環境変数名は §3 凍結対象外のため改訂不要。`LGX_MODELS_DIR` 導入は契約非該当 | なし（注記の追加は任意） |
| CLAUDE.md / scripts | `TE_MODELS_DIR` を使う運用箇所があれば `LGX_MODELS_DIR` への移行を案内（フォールバックがあるため緊急性なし） | 低（DD/実装フェーズで確認） |
| SPEC-LGX-006 | bulk similarity API 境界（REQ.08）は不変 | なし |
| DD フェーズ | snapshot_id 生成方式・同時刻タイブレーカー・パーセンタイル補間式・create warning 文言・env 解決実装の凍結対象が確定 | あり（DD で凍結） |
| UC-LGX-012/013（未生成） | 本 SPP 承認 + FCR ACCEPTED 後の UC フェーズで新規起草 | あり（次フェーズ） |

## 検証記録

本 SPP 起草前に、回答が引用する v3 実測主張を実物照合（2026-06-08、`traceability-engine.v3/`）:

- **Q2-a**: `crates/te-cli/src/model_dir.rs:34,67` — env 名は `TE_MODELS_DIR`、解決順 `--models-dir` → `TE_MODELS_DIR` → 既定。リブランドは新規導入。✅
- **Q2-c**: `crates/te-embed/src/orchestrator.rs:160` `read_current_content_for_node` の `std::fs::read_to_string(&file_path)?` がエラー伝播、`compute_node_drift` の doc 「Err: node 不在・ファイル読込失敗・次元不一致等」。現行ファイル欠落 = exit 1 は v3 正準。✅
- **Q3-a**: `crates/te-cli/src/commands/calibrate.rs:216-242` `compute_recommended` の `sorted[round((n−1)·frac)]`、p25 / 1.0−p90 / p75 の写像。✅
- **Q1-a / Q1-b / snapshot label 解決**: `crates/te-cli/src/commands/snapshot.rs` 行複製方式・label 非一意・最新優先（反復 1 起草時に照合済み）。✅

教訓（[[legixy-project-status]] 反映済み）: 反復 1 で exit code 誤認が Adversary 検証をすり抜けた事例に鑑み、v3 実測を根拠とする決定は実物照合を必須とした。
