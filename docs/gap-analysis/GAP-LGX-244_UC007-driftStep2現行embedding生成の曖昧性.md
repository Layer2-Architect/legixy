Document ID: GAP-LGX-244

# GAP-LGX-244: UC-LGX-007 drift Step2「現在の embedding を生成する」の観察可能性欠如

**親 TP**: TP-LGX-017
**観点**: §2.6 R3
**ステータス**: closed (2026-06-13)
**起票日**: 2026-06-13
**分類（暫定）**: GENUINE

## 1. 観点

UC-LGX-007 の drift フロー Step2「システムが対象成果物の現在の embedding を生成する」は、「現行ファイル内容からその場で embedding を生成する（engine.db への永続化なし）」という drift の本質的動作が UC フロー記述から読み取れない。「既存 embedding を engine.db から読み取る」という解釈も可能であり、UC レベルで観察可能な動作として不分明。

## 2. 現状の UC / SPEC

**UC-LGX-007 drift フロー Step2（現行）:**
> 2. システムが対象成果物の現在の embedding を生成する

「現行ファイル内容から on-the-fly 生成する（engine.db に保存しない）」なのか「engine.db から既存 embedding を読み取る」なのかが一読では判断できない。

**SPEC-LGX-010.REQ.03 の規定（既存）:**
> `drift <artifact_id>` は、指定成果物の**現行ファイル内容から生成した embedding** とベースラインとの乖離を報告する

「現行ファイル内容から生成」と明示しており、on-the-fly 生成（engine.db への永続化なし）が正解。さらに SPEC-LGX-010.REQ.03 は「4 コマンド中 drift のみが ONNX モデルの解決が必要」とも明示している（report/calibrate/snapshot は保存済みベクトルのみを使用しモデル不要）。

UC フロー Step2 はこの重要な区別（on-the-fly 生成 vs. 読取）を伝達できていない。

## 3. 推奨対応（人間裁定）

**(A) UC へ追記案:**
Step2 の記述を以下のように明確化する:
> 2. システムが対象成果物の**現行ファイル内容から** embedding をその場で生成する（ONNX モデルが必要。engine.db への永続化は行わない）

これにより:
- drift 実行にモデルが必要な理由が観察可能になる
- embed コマンド（永続化する）との動作差が明確になる
- 代替フロー「2a. モデルが存在しない場合」の発火理由が明確になる

**(B) drop（委譲容認）案:**
SPEC-LGX-010.REQ.03 に正準定義があり、UC の「現在の embedding を生成する」という記述は on-the-fly 生成を意図していると解釈可能（生成 ≠ 読取）。UC フローへの追記は不要と裁定する。

※ GENUINE 判断の根拠: drift の on-the-fly 生成という特性は embed との本質的な違いであり、「モデルが必要な唯一のコマンド」という前提が代替フローの発火条件（モデル不在 = 失敗）の根拠となる。この因果連鎖が UC フロー記述から観察不能であることは、後続 RBA での設計誤解（「drift も engine.db から読む」との誤解）につながる可能性がある。委譲で解決する性質ではなく、UC フロー記述の観察可能性の問題として GENUINE 寄りに判断した。

## 4. 影響範囲

- UC-LGX-007 drift フロー Step2 の記述明確化
- 後続 RBA/SEQA での drift の処理フロー設計（on-the-fly 生成 vs. 読取の誤解防止）
- GAP-LGX-242（drift 代替フロー欠落）と根が同じ「drift フロー記述の観察可能性不足」の構造的問題

## 5. 解消（2026-06-13）

敵対的精査裁定: **GENUINE**（実 SPEC 照合で確定）。UC 修正で解消（C3: UC-LGX-007 を embed 専念化し drift を UC-LGX-013 へ委譲）。人間承認 2026-06-13（A2/C2/C3 は AskUserQuestion 裁定、推奨案採用）。詳細: docs/spec-change-proposals/2026-06-13_uc-gap-adversarial-review.md §C。
