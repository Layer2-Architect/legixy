Document ID: GAP-LGX-202

# GAP-LGX-202: UC-LGX-003 Step4 ID 一意性検証の実施主体が SPEC 分担と矛盾する可能性

**親 TP**: TP-LGX-013
**観点**: §2.1 BF4、§2.6 R4（同根）
**ステータス**: closed (2026-06-13)
**起票日**: 2026-06-13
**分類（暫定）**: GENUINE

## 1. 観点

UC-LGX-003 基本フロー Step4「全サブノードの ID 一意性を検証する（SUBNODE-INV-3）」は、graph.toml 読み込みフローの一ステップとして「システム（graph.toml 読み込み時に自動実行）」が ID 一意性検証を実施することを示唆している。

しかし SPEC-LGX-002.REQ.12 は「生成段階ではエラー・Warning を発しない」と規定し、衝突の「検出と可視化は check が担う」と明記している。また SPEC-LGX-004.REQ.14 は SubnodeIdCollision Warning の検出を check コマンドが行うと規定している。

UC の Step4 記述が「graph.toml 読み込み時に自動実行」のアクターのもとに一意性検証を置くことは、上記 SPEC の分担（生成フロー＝無言処理、check コマンド＝検出）と矛盾する可能性がある。

## 2. 現状の UC / SPEC

- UC-LGX-003 Step4: 「全サブノードの ID 一意性を検証する（SUBNODE-INV-3）」——アクターが「システム（graph.toml 読み込み時に自動実行）」であるため、読み込みフロー内で検証が行われるように読める。
- SPEC-LGX-002.REQ.12: 「生成段階ではエラー・Warning を発しない。**検出と可視化は check が担う**（SPEC-LGX-004 REQ.14〔SubnodeIdCollision Warning〕）」
- SPEC-LGX-004.REQ.14（引用は SPEC-LGX-002.REQ.12 から間接参照）: check コマンドが SubnodeIdCollision Warning を検出。
- TP-LGX-002 G-LC-2（GREEN 確立済）: 衝突縮退/明示優先はREQ.12 で規定。

2 つの解釈が競合する:
1. **解釈 A**: Step4 は「読み込みフロー内でメモリ上の内部整合性確認を行う」（エラーを発しない形式の検証）
2. **解釈 B**: Step4 は「check コマンドに委譲される検証のことをフローとして記述している」（位置づけが不正確）

## 3. 推奨対応（人間裁定）

(A) **UC へ修正案（解釈 A を採用する場合）**: Step4 を「生成フロー内でのメモリ上 ID 一意性確認（エラーを発しない、外部通知なし）を行う。衝突の検出と外部通知は SPEC-LGX-002.REQ.12 / SPEC-LGX-004.REQ.14 に従い check コマンドが担う」と修正し、分担を明示する。

(B) **UC へ修正案（Step4 を削除する場合）**: Step4 は graph.toml 読み込みフローに属さないとして削除し、ID 一意性検証は「チェック観点: SUBNODE-INV-3」として別管理する（check コマンドの UC に委ねる）。

(C) **drop（委譲容認）案**: SPEC-LGX-002.REQ.12 が分担を明記しており、UC の Step4 を「読み込みフロー内で縮退・スキップ処理が行われる内部整合確認」として解釈することで矛盾なしとし drop する。ただし下流成果物での誤解リスクを memo として残す。

## 4. 影響範囲

- UC-LGX-003 §基本フロー Step4
- 下流成果物（RBA/SEQA/DD）でサブノード生成処理の責務を設計する際に、ID 一意性検証の位置づけを誤る可能性がある。GENUINE 判定相当の影響（設計誤りを下流に波及させるリスク）。
- SPEC-LGX-004（検証）との境界設計にも影響する。

## 5. 解消（2026-06-13）

敵対的精査裁定: **GENUINE**（実 SPEC 照合で確定）。UC 修正で解消（C1: UC-LGX-003 Step4 を「一意性を担保する（生成段階は無エラー、検出は check）」へ表現修正）。人間承認 2026-06-13（A2/C2/C3 は AskUserQuestion 裁定、推奨案採用）。詳細: docs/spec-change-proposals/2026-06-13_uc-gap-adversarial-review.md §C。
