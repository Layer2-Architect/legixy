Document ID: GAP-LGX-267

# GAP-LGX-267: migrate Step 3〜6 の各段階失敗パスが UC に未列挙

**親 TP**: TP-LGX-019
**観点**: §2.3 EF2
**ステータス**: closed (2026-06-13)
**起票日**: 2026-06-13
**分類（暫定）**: WEAK

## 1. 観点

migrate Step 3〜6（graph.toml 生成・`.legixy.toml` 変換・engine.db 移行・vectors.bin インポート）のいずれも失敗パスが UC に定義されていない。SPEC-LGX-008.REQ.02（DB トランザクション・atomic rename・確定順序・中断耐性）および REQ.08（失敗段階・バックアップ場所・リカバリ手順提示）が UC フロー記述として観察可能でない。

## 2. 現状の UC / SPEC

UC-LGX-009 の migrate 基本フローは Step 1〜7 を記述するが、Step 3〜6 のいずれにも失敗パス・例外フロー・代替フローが対応していない。

SPEC-LGX-008.REQ.02 は以下を規定する:
- 失敗時は元ファイルを保持（REQ.02a 命名で退避）
- 途中中断されても DB が壊れない（SQLite トランザクション）
- 再実行可能（冪等・全やり直し方式）
- 非 DB ファイルの atomic 書込（temp+fsync+rename）
- 確定順序（DB コミット先行→平文ファイル atomic 確定）

各ステップ固有の失敗:
- Step 3（graph.toml 生成）: ディスクフル・書き込み権限なし・rename 失敗
- Step 4（`.legixy.toml` 変換）: 既存 config の read 失敗・write 失敗
- Step 5（engine.db 移行）: DB ロック・SQLite エラー・テーブル移行失敗
- Step 6（vectors.bin インポート）: embeddings テーブル書き込み失敗・vectors.bin 破損

これらの失敗パスは UC に存在せず、各失敗時の原本保護・エラー報告・中断状態からの回復が UC フロー記述として観察可能でない。

## 3. 推奨対応（人間裁定）

**(A) UC へ追記案（包括的な失敗フロー追加）:**
代替フローに包括的な「各段階失敗時」フローを追加する:
- **3a.** Step 3〜6 のいずれかで失敗した場合（ディスクフル・権限エラー・DB エラー等）、失敗した段階を報告し、原本ファイルを保持したまま終了する（部分変換の結果は破棄）

**(B) drop（委譲容認）案:**
SPEC-LGX-008.REQ.02/REQ.08 + TP-LGX-008 E-3/P-1/P-2/P-3 へ委譲確定とする。各段階の失敗パスは SPEC レベルの規定で十分であり、UC フロー記述への列挙は任意。

## 4. 影響範囲

- 下流 RBA: migrate ロバストネス図での各段階エラーパス
- 下流 DD: 各段階のエラーハンドリングと原本保護の実装
- SPEC-LGX-008.REQ.02（中断耐性・冪等性）との整合

## 5. 解消（2026-06-13）

敵対的精査裁定: **方針B（委譲容認）**。本観点は親 SPEC（対応 TP[SPEC] green 確定済）が回答しており、UC フローへの明示列挙は任意。GREEN-by-delegation として close。詳細: docs/spec-change-proposals/2026-06-13_uc-gap-adversarial-review.md §F。
