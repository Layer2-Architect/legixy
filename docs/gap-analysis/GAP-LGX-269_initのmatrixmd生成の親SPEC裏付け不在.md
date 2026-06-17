Document ID: GAP-LGX-269

# GAP-LGX-269: UC init フローの matrix.md 生成が親 SPEC に裏付けなし

**親 TP**: TP-LGX-019
**観点**: §2.5 DF3
**ステータス**: closed (2026-06-13)
**起票日**: 2026-06-13
**分類（暫定）**: GENUINE

## 1. 観点

UC-LGX-009 init Step 2 に「`docs/traceability/matrix.md`（空テンプレート）」の生成が列挙されているが、親 SPEC（SPEC-LGX-008.REQ.07）には matrix.md の init 生成が規定されていない。UC が SPEC に規定のない生成物を追加しており、裏付けとなる REQ/§が確認できない。

## 2. 現状の UC / SPEC

UC-LGX-009 init Step 2 の列挙 5 項目のうち:
- `docs/traceability/matrix.md`（空テンプレート）

SPEC-LGX-008.REQ.07 の init 生成物規定:
- `.legixy.toml`（ICONIX 8 typecode + `[id.document_id]` template）
- `docs/traceability/graph.toml`（空ファイル）
- `.legixy/engine.db`（初期スキーマ）
- ICONIX 成果物用 8 ディレクトリ + `.gitkeep`

**matrix.md の init 生成は REQ.07 に登場しない。**

SPEC-LGX-008.REQ.04 は「`.legixy.toml` に `[graph]` セクションを追加する。`[matrix]` セクションは後方互換のため残す（graph.toml から matrix.md を生成する設定としての意味に変更）」と規定するが、これは migrate での設定移行を指しており、init での matrix.md 生成を規定するものではない。

UC が SPEC の規定を超えた生成物を列挙していることは、ハードルール「すべての成果物は親への参照を持つ」の精神に照らして人間裁定が必要。

## 3. 推奨対応（人間裁定）

**(A) UC へ追記案（根拠追記）:**
matrix.md 生成を UC から削除するか、または根拠 REQ/§を明示する:
- SPEC-LGX-008 に matrix.md init 生成を規定する REQ を追加してから UC に残す（spec-change イベントとして扱う）

**(B) UC から削除案:**
SPEC-LGX-008.REQ.07 に規定がない以上、UC の列挙から matrix.md を削除する。matrix.md は migrate 後の生成物または手動作成物として扱う。

**(C) 既存実装確認案:**
v3 実装の `crates/te-mig/src/initializer.rs` を確認し、init 時に matrix.md を実際に生成しているかを確認したうえで人間裁定する（実装事実があれば SPEC への追記が必要）。

## 4. 影響範囲

- SPEC-LGX-008.REQ.07 への matrix.md 追記（spec-change イベント）が必要な可能性
- 下流 TS-LGX-007（init 直後 check テスト）: matrix.md の存在が期待値に含まれるか
- DevProc ハードルール 2「成果物は親への参照を持つ」の遵守確認

## 5. 解消（2026-06-13）

敵対的精査裁定: **GENUINE**（実 SPEC 照合で確定）。UC 修正で解消（C2: UC-LGX-009 init 生成物から旧形式 matrix.md を除去）。人間承認 2026-06-13（A2/C2/C3 は AskUserQuestion 裁定、推奨案採用）。詳細: docs/spec-change-proposals/2026-06-13_uc-gap-adversarial-review.md §C。
