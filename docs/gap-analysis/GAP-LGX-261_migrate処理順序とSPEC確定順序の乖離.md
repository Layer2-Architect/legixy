Document ID: GAP-LGX-261

# GAP-LGX-261: migrate フロー処理順序と SPEC-LGX-008.REQ.02 確定順序の乖離

**親 TP**: TP-LGX-019
**観点**: §2.1 BF3
**ステータス**: closed (2026-06-13)
**起票日**: 2026-06-13
**分類（暫定）**: GENUINE

## 1. 観点

migrate フローのステップ順序と SPEC-LGX-008.REQ.02 が要求する「DB コミット先行 → 平文ファイル（graph.toml 等）atomic 確定」という確定順序が一致しているか。

## 2. 現状の UC / SPEC

UC-LGX-009 の migrate 基本フローは以下の順で記述されている:

1. アクターが `legixy migrate --from <v01_project_root>` を実行する
2. システムが v0.1.0 プロジェクトを読み込む（a. `.legixy.toml` 解析 / b. matrix.md パース）
3. **graph.toml のノードとエッジを生成する**（平文ファイル操作）
4. v0.1.0 の `.legixy.toml` を legixy 形式に変換する
5. **feedback.db を engine.db に移行する**（DB 操作）
6. vectors.bin があれば embeddings テーブルにインポートする
7. 移行レポートを出力する

SPEC-LGX-008.REQ.02 は以下を規定する:
- **確定順序**: engine.db のトランザクションコミットを**先行**させ、その後に graph.toml / id-map を atomic に確定する
- **根拠**: DB コミット後・平文確定前の中断は再実行（冪等）で回復できるが、平文先行では中断時に不整合中間状態が残る可能性がある

UC の Step 3（graph.toml 生成 = 平文ファイル）が Step 5（engine.db 移行 = DB 操作）より前に来ており、SPEC が要求する「DB 先行」の確定順序と逆転している可能性がある。

## 3. 推奨対応（人間裁定）

**(A) UC へ追記案（UC フロー順序修正）:**
UC の migrate 基本フローを SPEC-LGX-008.REQ.02 確定順序に合わせて並べ替える:

1. v0.1.0 プロジェクト読み込み（Step 2: `.legixy.toml` 解析 / matrix.md パース）
2. feedback.db を engine.db に移行する（DB トランザクション、Step 5 を前倒し）
3. vectors.bin があれば embeddings テーブルにインポートする（Step 6 を前倒し）
4. graph.toml のノードとエッジを生成する（DB コミット後、atomic 確定）
5. v0.1.0 の `.legixy.toml` を legixy 形式に変換する（atomic 確定）
6. 移行レポートを出力する

**(B) drop（委譲容認）案:**
SPEC-LGX-008.REQ.02 確定順序は実装詳細（DD レベル）であり、UC フローは論理的ステップ記述で十分とする。TP-LGX-008 P-2 が所有する観点として委譲を確定し、UC の順序記述は「論理処理順」として扱う。

## 4. 影響範囲

- 下流 RBD/SEQD: migrate シーケンス図の処理順序に直接影響
- 下流 DD: DB トランザクション境界とファイル書き込みの実装順序
- TP-LGX-008 P-2（graph/id-map の atomic 書込）との連携

## 5. 解消（2026-06-13）

敵対的精査裁定: **方針B（委譲容認）**。本観点は親 SPEC（対応 TP[SPEC] green 確定済）が回答しており、UC フローへの明示列挙は任意。GREEN-by-delegation として close。詳細: docs/spec-change-proposals/2026-06-13_uc-gap-adversarial-review.md §F。
