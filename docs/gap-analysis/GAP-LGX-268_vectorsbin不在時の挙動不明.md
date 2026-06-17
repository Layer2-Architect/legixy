Document ID: GAP-LGX-268

# GAP-LGX-268: migrate Step 6 vectors.bin 不在時の事後状態が不明

**親 TP**: TP-LGX-019
**観点**: §2.5 DF2
**ステータス**: closed (2026-06-13)
**起票日**: 2026-06-13
**分類（暫定）**: WEAK

## 1. 観点

UC-LGX-009 migrate Step 6「vectors.bin があれば embeddings テーブルにインポートする」は不在時の分岐を「あれば」という条件で示しているが、不在時の挙動（スキップして正常終了 / 警告発行 / 別フロー）が UC に観察可能でない。SPEC-LGX-008.REQ.05（Phase 1 はドキュメントノードのみ・サブノード情報は含めない）との整合も UC フロー記述から判断できない。

## 2. 現状の UC / SPEC

UC の Step 6: `vectors.bin があれば embeddings テーブルにインポートする`

「あれば」という条件分岐の記述で不在ケースの存在を示唆しているが、不在時の事後状態（embeddings テーブルの初期状態・正常 exit・警告の有無）が UC に記述されていない。

SPEC-LGX-008.REQ.05: 移行直後 graph.toml にサブノードを含まずドキュメントノードのみ。ユーザが明示的に `embed --all` 等を実行しない限り graph.toml にはドキュメントノードのみ。

vectors.bin は v0.1.0 での embedding 保存形式（embedding はサブノード情報と関連）。vectors.bin 不在 = embedding なし → REQ.05 のドキュメントノードのみ状態と整合するはずだが、UC フロー記述からこの整合が観察可能でない。

## 3. 推奨対応（人間裁定）

**(A) UC へ追記案:**
Step 6 に代替フローまたは注記を追加する:
- **6a.** vectors.bin が存在しない場合は Step 6 をスキップし正常に Step 7 へ進む（embeddings テーブルは空の初期状態のまま。REQ.05 と整合）

**(B) drop（委譲容認）案:**
「あれば」の記述で不在時スキップが自明であり、SPEC-LGX-008.REQ.05 + TP-LGX-008 L-3 へ委譲確定とする。不在時の正常終了は合理的な帰結として委譲で十分。

## 4. 影響範囲

- 下流 RBA: migrate フローの vectors.bin 判定分岐
- 下流 TS: vectors.bin なしでの migrate テスト（embeddings テーブルの初期状態確認）
- SPEC-LGX-008.REQ.05（Phase 1 ドキュメントノードのみ）との整合確認

## 5. 解消（2026-06-13）

敵対的精査裁定: **方針B（委譲容認）**。本観点は親 SPEC（対応 TP[SPEC] green 確定済）が回答しており、UC フローへの明示列挙は任意。GREEN-by-delegation として close。詳細: docs/spec-change-proposals/2026-06-13_uc-gap-adversarial-review.md §F。
