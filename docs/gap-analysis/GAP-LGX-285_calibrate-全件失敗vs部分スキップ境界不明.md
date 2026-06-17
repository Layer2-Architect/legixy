Document ID: GAP-LGX-285

# GAP-LGX-285: calibrate の全件失敗と部分スキップ（継続）の境界が UC フローで観察不能

**親 TP**: TP-LGX-021
**観点**: §2.3 EF4 全件失敗 vs 部分スキップの区別
**ステータス**: closed (2026-06-13)
**起票日**: 2026-06-13
**分類（暫定）**: GENUINE

## 1. 観点

UC-LGX-011 の代替フロー 3a は「全ペア算出失敗時: anyhow エラーコンテキスト付きで exit 1」を規定する。一方 SPEC-LGX-010 REQ.05 は次元不一致・非有限スコアの「部分スキップ＋継続（exit 0）」を規定する。UC フローがこの 2 つのパスを区別していないため、消費者が「どのような条件で exit 1 になり、どのような条件で exit 0 になるか」を UC レベルで把握できない。

## 2. 現状の UC / SPEC

**UC-LGX-011 の記述:**
- 代替フロー 3a: `全ペア算出失敗時: anyhow エラーコンテキスト付きで exit 1`
  - 「全ペア算出失敗」の定義が UC に明示されていない
  - 次元不一致スキップ（部分成功 exit 0）との区別が記述されていない

**SPEC-LGX-010 REQ.05 / REQ.09 の規定:**
- 次元不一致ペア: skip + 集約 Warning → 継続 → exit 0
- NaN/Inf ペア: skip + 集約 Warning → 継続 → exit 0（REQ.09）
- `--buckets 0`: exit 1（バリデーション失敗、1a で対応済み）
- 全ペア算出 API 自体が例外を返す（`compute_all_pair_scores` の anyhow エラー）: exit 1

**消費者への影響:**
- exit 0 と exit 1 の境界は CI/CD パイプラインにおいてキャリブレーション実行の成否判定に直結する
- 「次元不一致があっても exit 0 で終了するため、CI はヒストグラムが生成されたと判断する」という挙動が UC フローから読み取れない

## 3. 推奨対応（人間裁定）

**(A) UC へ追記案**: 以下の形で 3a を分割または注記を追加する:
- `3a. 全ペア算出 API が例外を返した場合（infrastructure 系障害）: anyhow エラーコンテキスト付きで exit 1`
- 注記: 「次元不一致ペアおよび非有限スコアのペアはスキップして継続し exit 0 とする（部分スキップは 3b/3c を参照）」

**(B) drop（委譲容認）案**: exit 0 vs exit 1 の分類規則は SPEC-LGX-010 REQ.01（「結果が空を含む正常終了 = exit 0 / 実行エラー = exit 1」）と REQ.05（次元不一致スキップ継続）が合成して答える。UC フローで逐一列挙せず SPEC 委譲として扱う。

## 4. 影響範囲

- UC-LGX-011 §代替フロー（追記案 A の場合）
- CI/CD での calibrate 実行結果判定（exit code semantics）
- 下流 RBA での「calibrate の成否判定」ロバストネスシナリオ
- GAP-LGX-283（次元不一致スキップ）/ GAP-LGX-284（非有限スコア）が close した場合、本 GAP の追記案 A を自然に包含する

## 5. 解消（2026-06-13）

敵対的精査裁定: **方針B（委譲容認）**。本観点は親 SPEC（対応 TP[SPEC] green 確定済）が回答しており、UC フローへの明示列挙は任意。GREEN-by-delegation として close。詳細: docs/spec-change-proposals/2026-06-13_uc-gap-adversarial-review.md §F。
