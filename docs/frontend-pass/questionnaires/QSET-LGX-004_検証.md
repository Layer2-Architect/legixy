# Document ID: QSET-LGX-004

**親 SPEC**: SPEC-LGX-004
**反復回数**: 1
**作成日**: 2026-06-04
**作成者**: AI (designer)

---

## 概要

このドキュメントは前段ループの反復 1 回目で発行された質問票である。SPEC-LGX-004（検証）に対してフロントエンド検査器が検出した矛盾・責務範囲不明・複数解釈を、開発者が回答可能な形に変換したもの。

---

## Q1: 矛盾 — check の終了コード 2 と互換契約

**質問**: REQ.04 は `check` / `check --formal` の終了コードを「0: Error 0 件 / 1: Error 1 件以上 / **2: 使用法誤り**」と 3 値で規定します。しかし LGX-COMPAT-001 §4 #3 は check の終了コードを「**Error 件数>0 で 1、それ以外 0**」と 2 値でのみ規定し、exit 2 に言及がありません。凍結済み境界契約との整合をどう取りますか?

**SPEC 上の該当箇所**: SPEC-LGX-004 §3 REQ.04、LGX-COMPAT-001 §4

**選択肢**:

- [x] 選択肢 A: exit 2（使用法誤り）は全サブコマンド共通の clap 規約として互換契約に追記し、SPEC-004 REQ.04 を正とする
- [ ] 選択肢 B: check は 0/1 のみとし、REQ.04 から exit 2 を削除（使用法誤りも 1 に集約）
- [ ] その他: <自由記述>

**回答**:

**選択肢 A を採用**（2026-06-07 開発者決定・AI 起草）。

- 根拠: v3 実測 — check 本体の判定は 0/1（`te-check/src/reporter.rs:50-57`）だが、**使用法誤りの exit 2 は clap 4.x の既定動作として v3 バイナリの全サブコマンドに既に存在する**（例: `te-cli/src/commands/embed.rs:28` は使用法誤りで明示的に exit 2）。SPEC-004 REQ.04 の 3 値規定は v3 実態の正確な記述であり、矛盾ではなく COMPAT 側の記述漏れ。
- 対応: LGX-COMPAT-001 §3 または §4 冒頭に「グローバル規約: 使用法誤りは全コマンドで exit 2」を追記（COMPAT v1.0.1 改訂）。§4 #3 の記述は「check の検証結果に基づく終了コード」の規定としてそのまま整合。
- QSET-LGX-009 Q2（MCP での exit 1/2 区別）と連動。

---

## Q2: 責務範囲 — report コマンド（UC-010）の SPEC オーナー

**質問**: 凍結済みコマンド `report`（全リンク類似度＋候補一覧、UC-LGX-010 健全性監査）の出力仕様・要求が本 SPEC にも SPEC-LGX-006 にもありません。REQ.02 の SemanticSimilarity / LinkCandidate は `check` 内で報告されますが、独立した `report` コマンドの出力内容・形式は未規定です。report の責務はどこに置きますか?（SPEC-LGX-001 QSET Q1/Q2 と連動）

**SPEC 上の該当箇所**: SPEC-LGX-004 §3 REQ.02、SPEC-LGX-006 REQ.11

**回答**:

（2026-06-07 開発者決定・AI 起草）

**新設 SPEC-LGX-010（embedding 運用・監査）に置く**（QSET-LGX-001 Q1 の決定に従う）。report の出力仕様は v3 実測を正準化して SPEC-010 に規定する:

- `links`: `[{from, to, score, kind}]` — 全エッジの類似度実数値
- `candidates`: `[{from, to, score}]` — リンク漏れ候補
- `summary`: `{total_links, total_candidates, min_link_score, max_link_score, mean_link_score}`
- 根拠: `te-cli/src/commands/report.rs:31-44`。check との責務境界は Q4 参照。

---

## Q3: 責務範囲 — drift コマンドと snapshot 機能の SPEC オーナー

**質問**: 凍結済みコマンド `drift <artifact_id> --against <snapshot:LABEL|snapshot:ID>` と `snapshot create/list/delete`（LGX-COMPAT-001 §4 #5, #8）の要求がどの SPEC にもありません。本 SPEC REQ.02 の Drift は「check 内で content_hash 比較を Warning 報告」する機能で、**standalone の drift コマンド（特定 artifact をスナップショットと対比）** とは別物です。drift / snapshot の SPEC オーナーをどこに置きますか?

**SPEC 上の該当箇所**: SPEC-LGX-004 §3 REQ.02、SPEC-LGX-006 REQ.05

**回答**:

（2026-06-07 開発者決定・AI 起草）

**新設 SPEC-LGX-010 に置く**（QSET-LGX-001 Q1 の決定に従う）。

- standalone `drift <artifact_id> [--against snapshot:LABEL|snapshot:ID]` と `snapshot create/list/delete` を SPEC-010 の REQ として規定。両者は engine.db の `embedding_snapshots` テーブルを共有する（`te-cli/src/commands/{drift,snapshot}.rs`）。
- 位置づけの書き分け: **check 内 Drift**（SPEC-004 REQ.02 / SPEC-006 REQ.05）は「検証層の警告」（content_hash 比較で Warning 報告）、**standalone drift** は「運用層の対比ツール」（特定 artifact を現行 embedding またはスナップショットと対比）。同名だが別機能である旨を両 SPEC に相互参照として明記する。

---

## Q4: 複数解釈 — check（全層）と report の出力責務境界

**質問**: Q2/Q3 で report / drift の所在が決まった後、`check`（全層）と `report` が双方とも「リンク類似度・候補」を出力すると責務が重複します。両者の出力責務境界（check は何を出し、report は何を出すか）はどう切り分けますか?

**SPEC 上の該当箇所**: SPEC-LGX-004 §3 REQ.02

**回答**:

（2026-06-07 開発者決定・AI 起草）v3 実装の境界をそのまま正準化する:

- **check = 判定（judgement）**: 閾値判定の**結果のみ**を severity 付き findings（SemanticSimilarity / LinkCandidate / Drift の Warning/Info）として報告する。生の類似度スコア一覧は出さない。
- **report = 計測（measurement）**: 閾値判定を行わず、全エッジの生スコア + リンク候補 + 統計を出力する。severity 概念を持たない。
- この境界を SPEC-004（check 側）と SPEC-LGX-010（report 側）の双方に明記し、出力責務の重複を禁止する。
- 根拠: v3 でも両者の出力は重複しない（`te-check/src/reporter.rs:59-80` の severity/category 構造 vs `report.rs:31-44` の数値出力）。

---

## 検出元検査の集計

| 検査カテゴリ | 検出件数 |
|---|---|
| 未定義語 | 0 |
| 複数解釈 | 1 |
| 例外未定義 | 0 |
| 境界不明 | 2 |
| 矛盾 | 1 |
| 非機能不足 | 0 |
| 合計 | 4 |

## メモ

- Q2/Q3 は SPEC-LGX-001 QSET の Q1（未割当サブコマンド）と一体で判断する必要がある。
- 回答が確定したら SPP-LGX-004 として SPEC 差分案を発行する。
