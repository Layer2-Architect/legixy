Document ID: GAP-LGX-262

# GAP-LGX-262: migrate Step 7 移行レポートの UC 観察可能性の欠如

**親 TP**: TP-LGX-019
**観点**: §2.1 BF4
**ステータス**: closed (2026-06-13)
**起票日**: 2026-06-13
**分類（暫定）**: WEAK

## 1. 観点

migrate Step 7「移行レポートを出力する」が SPEC-LGX-008.REQ.08 の成功時変更サマリ要件（stdout/stderr 分離・`--format json` 対応・STATE-INV-2 運用支援）を UC フロー記述として観察可能に具体化しているか。

## 2. 現状の UC / SPEC

UC-LGX-009 migrate の Step 7 は「移行レポートを出力する」の 1 行のみ。

SPEC-LGX-008.REQ.08 は成功時変更サマリとして以下を規定する:
- stdout に変更サマリを出力（生成/更新ファイル一覧・書き換え ID 件数と id-map への参照・バックアップ場所）
- `--format json` 時は構造化出力
- 診断・進捗は stderr（NFR OBS.02）
- STATE-INV-2「ユーザが内容を確認して Git commit するまでが完了」という運用を支える事後条件

「移行レポートを出力する」は意図を包んでいるが、stdout/stderr 分離・出力内容の具体性・`--format` 対応が UC フロー記述として観察可能でない。

## 3. 推奨対応（人間裁定）

**(A) UC へ追記案:**
Step 7 を以下のように具体化する:
- 7. システムが移行レポートを stdout に出力する（生成/更新ファイル一覧・書き換え ID 件数・バックアップ場所を含む。`--format json` 指定時は構造化出力）

**(B) drop（委譲容認）案:**
SPEC-LGX-008.REQ.08 + TP-LGX-008 O-3/O-4 が所有する観点として委譲を確定し、「移行レポートを出力する」の記述で十分とする。出力の具体的内容は DD/TS レベルで定義。

## 4. 影響範囲

- 下流 TS: migrate 成功時の出力内容検証テスト（TP-LGX-008 O-3/O-4 と重複しない範囲）
- STATE-INV-2 の運用支援を UC 事後条件として扱うか否かの判断

## 5. 解消（2026-06-13）

敵対的精査裁定: **方針B（委譲容認）**。本観点は親 SPEC（対応 TP[SPEC] green 確定済）が回答しており、UC フローへの明示列挙は任意。GREEN-by-delegation として close。詳細: docs/spec-change-proposals/2026-06-13_uc-gap-adversarial-review.md §F。
