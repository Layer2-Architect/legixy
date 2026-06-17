Document ID: GAP-LGX-270

# GAP-LGX-270: migrate の設定ファイル探索順（REQ.13）が UC フローで観察不可能

**親 TP**: TP-LGX-019
**観点**: §2.6 R1
**ステータス**: closed (2026-06-13)
**起票日**: 2026-06-13
**分類（暫定）**: GENUINE

## 1. 観点

UC-LGX-009 migrate Step 2a「`.legixy.toml` を解析」は単一ファイルを名指しするが、SPEC-LGX-008.REQ.13 の設定ファイル探索順（`.legixy.toml` 優先 / `.trace-engine.toml` フォールバック）が UC フロー記述として観察可能でない。`.trace-engine.toml` のみ存在する v0.1.0 プロジェクトに対する migrate の挙動が UC から不明。

## 2. 現状の UC / SPEC

UC-LGX-009 migrate Step 2:
> 2. システムが v0.1.0 プロジェクトを読み込む:
>    a. `.legixy.toml` を解析

SPEC-LGX-008.REQ.13 の設定ファイル探索順規定:
1. `.legixy.toml`（既定・正式名）
2. `.trace-engine.toml`（旧名フォールバック）

挙動の詳細:
- `.legixy.toml` が無く `.trace-engine.toml` のみ存在する場合は後者を読み、**一度だけ Info を出力**して `.legixy.toml` への移行を案内する
- `legixy migrate` 実行時は `.legixy.toml` を生成し、旧ファイルは `.bak` 退避する（REQ.04 と整合）

v0.1.0 プロジェクトの現実的な姿は `.trace-engine.toml`（旧名）を持つ可能性が高い（traceability-engine バイナリが使っていた名前）。migrate の主要シナリオで最初に読む設定ファイルが `.trace-engine.toml` であるケースが UC Step 2a に反映されていない。

UC の「`.legixy.toml` を解析」という記述は探索先が `.legixy.toml` 一択であるかのように読め、旧名フォールバックが観察不可能。

## 3. 推奨対応（人間裁定）

**(A) UC へ追記案（探索順を観察可能に）:**
Step 2a を以下のように修正する:
- 2a. 設定ファイルを解析する（`.legixy.toml` 優先、なければ `.trace-engine.toml` をフォールバックとして読む。後者のみの場合は Info を出力し `.legixy.toml` への変換が migrate の成果物となる）

または代替フローとして:
- **2c.** `.legixy.toml` が存在せず `.trace-engine.toml` のみが存在する場合は、`.trace-engine.toml` を読み込んで migrate を継続し、成果物として `.legixy.toml` を生成する（`.trace-engine.toml` は `.bak` 退避）

**(B) drop（委譲容認）案:**
SPEC-LGX-008.REQ.13 + TP-LGX-008 V-7 へ委譲確定とする。探索順の詳細はSPEC レベルで確立しており、UC の「`.legixy.toml` を解析」は「設定ファイルを解析」の略記として許容する。

## 4. 影響範囲

- 下流 RBA: migrate フローの設定ファイル読み込みパス（`.trace-engine.toml` フォールバックケース）
- 下流 TS: `.trace-engine.toml` のみを持つ v0.1.0 プロジェクトへの migrate テスト
- LGX-COMPAT-001 §6（設定ファイル探索順の凍結契約）との整合
- migrate の中核シナリオ（既存 traceability-engine プロジェクトの移行）への影響

## 5. 解消（2026-06-13）

敵対的精査裁定: **方針B（委譲容認）**。本観点は親 SPEC（対応 TP[SPEC] green 確定済）が回答しており、UC フローへの明示列挙は任意。GREEN-by-delegation として close。詳細: docs/spec-change-proposals/2026-06-13_uc-gap-adversarial-review.md §F。
