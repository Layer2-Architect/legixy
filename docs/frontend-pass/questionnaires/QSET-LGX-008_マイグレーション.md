# Document ID: QSET-LGX-008

**親 SPEC**: SPEC-LGX-008
**反復回数**: 1
**作成日**: 2026-06-04
**作成者**: AI (designer)

---

## 概要

このドキュメントは前段ループの反復 1 回目で発行された質問票である。SPEC-LGX-008（マイグレーション）に対してフロントエンド検査器が検出した境界不明・複数解釈を、開発者が回答可能な形に変換したもの。本 SPEC は詳細だが、init の生成構造と設定ファイル名の二層構造に確認点がある。

---

## Q1: 境界不明 — init が生成するディレクトリ構造（単段 ICONIX vs 二段化）

**質問**: REQ.07 の `init` は ICONIX 標準 8 ディレクトリとして `docs/robustness/`, `docs/sequence/`, `docs/detailed-design/` 等（**単段**）を生成します。一方、本プロジェクト自身の DevProc_V4.1 運用は ICONIX **二段化**（`robustness-abstract` / `robustness-detail`、`sequence-abstract` / `sequence-detail`）を採用しています。この差は意図的ですか?

- 「init は legixy-the-tool 利用者向けの既定で、二段化は本プロジェクト固有の運用」という別レイヤとして確定するのか
- それとも init を二段化対応に更新するのか

UC-LGX-009 のシナリオ・TS-LGX-007（init 直後 check）の期待値に直結します。

**SPEC 上の該当箇所**: SPEC-LGX-008 §3 REQ.07

**選択肢**:

- [x] 選択肢 A: 単段 ICONIX を init 既定として維持（二段化は本プロジェクト固有の上書き運用と明記）
- [ ] 選択肢 B: init を二段化（abstract/detail 分離）対応に更新
- [ ] その他: <自由記述>

**回答**:

**選択肢 A を採用**（2026-06-07 開発者決定・AI 起草）。

- init は単段 ICONIX（`docs/robustness/`、`docs/sequence/` 等）と chain order `UC→RB→SEQ→DD→TS→TC→SRC` を legixy-the-tool の既定として維持する。
- 根拠: v3 実測（`te-mig/src/initializer.rs:56-68` のディレクトリ生成、`te-core/src/config/loader.rs:369-371` の chain 既定値）。既定変更は init 直後 check（TS-LGX-007）の期待値と既存利用者の体験を変える。
- REQ.07 に注記を追加: 「ICONIX 二段化（RBA/RBD・SEQA/SEQD 分離）は本プロジェクトが DevProc_V4.1 運用として設定を**上書き**している別レイヤであり、init の既定生成物ではない」。二段化テンプレートの同梱は将来要求とし本反復のスコープ外。

---

## Q2: 複数解釈 — 設定ファイル名の二層構造の SPEC 記述範囲

**質問**: REQ.13 は legixy が読む設定ファイルを `.legixy.toml`（正式名）→ `.trace-engine.toml`（旧名フォールバック）の順とします。一方、本プロジェクトの**開発ツール実体**（`traceability-engine` バイナリ）は `.trace-engine.toml` を既定で読みます（CLAUDE.md 補足）。つまり「legixy-the-tool が生成・読む設定」と「本プロジェクトの開発ツールが読む設定」が名前空間として逆転して見えます。本 SPEC REQ.13 が規定する対象は **「legixy-the-tool（成果物）が生成・読む設定ファイル」** で正しいですか? 両層の混同を防ぐため SPEC に対象の明記を追加すべきですか?

**SPEC 上の該当箇所**: SPEC-LGX-008 §3 REQ.04, REQ.13、LGX-COMPAT-001 §6

**回答**:

（2026-06-07 開発者決定・AI 起草）

**はい、REQ.13 の規定対象は「legixy-the-tool（成果物）が生成・読む設定ファイル」で正しい。対象の明記を SPEC に追加する。**

- 本リポジトリの開発運用が読む `.trace-engine.toml`（旧 `traceability-engine` バイナリの設定）は開発プロセス側の**別レイヤ**であり、REQ.13 の規定対象外。
- REQ.04/REQ.13 に注記を追加: 「本 REQ は成果物 legixy の設定探索（`.legixy.toml` → `.trace-engine.toml` フォールバック）を規定する。本リポジトリ自身の開発運用が旧バイナリで読む設定とは独立」。CLAUDE.md の補足記載（二層の区別）と整合。

---

## 検出元検査の集計

| 検査カテゴリ | 検出件数 |
|---|---|
| 未定義語 | 0 |
| 複数解釈 | 1 |
| 例外未定義 | 0 |
| 境界不明 | 1 |
| 矛盾 | 0 |
| 非機能不足 | 0 |
| 合計 | 2 |

## メモ

- Q1 は本プロジェクトの DevProc 運用と legixy-the-tool の既定生成の関係整理であり、UC-LGX-009 の境界に影響する。
- 回答が確定したら SPP-LGX-008 として SPEC 差分案を発行する。
