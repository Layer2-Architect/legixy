Document ID: GAP-LGX-282

# GAP-LGX-282: calibrate --recommend フラグの分岐が UC に未記載

**親 TP**: TP-LGX-021
**観点**: §2.2 AF2 --recommend 分岐の非網羅（§2.6 R5 --json スキーマ整合と連動）
**ステータス**: closed (2026-06-13)
**起票日**: 2026-06-13
**分類（暫定）**: GENUINE

## 1. 観点

UC-LGX-011 の基本フロー・代替フローに `--recommend` フラグの使用パターンが存在しない。SPEC-LGX-010 REQ.05 が規定する推奨閾値算出（パーセンタイル方式 p25/1.0−p90/p75）は calibrate の主要機能であり、QA リードが false positive / false negative トレードオフ判断に使用する用途が UC 概要で示されているにもかかわらず、フロー記述にこの分岐が欠落している。

## 2. 現状の UC / SPEC

**UC-LGX-011 の記述:**
- アクター: QA リード（false positive / false negative トレードオフ判断）
- 基本フロー: `legixy calibrate [--buckets N] [--json]` — `--recommend` の言及なし
- 代替フロー: 2a / 1a / 3a のいずれも `--recommend` を扱わない
- JSON スキーマ（Step6）: `{"pairs": N, "min", "max", "mean", "distribution": [...], "thresholds": {...}}` — `recommended_thresholds` キーなし

**SPEC-LGX-010 REQ.05 の規定:**
- `[--recommend]` が引数体系に含まれる（LGX-COMPAT-001 §4 #7）
- `--recommend` 指定時: `recommended_thresholds: {similarity_threshold, drift_threshold, link_candidate_threshold}` + p10/25/50/75/90 を追加出力
- `--recommend` 指定かつ pairs=0: stderr に INFO 1 件 + `recommended_thresholds` 非出力（`--json` は stdout を汚さない）
- 推奨閾値算出方式: パーセンタイル方式（SPEC 凍結）、補間式は DD

`--recommend` フラグは LGX-COMPAT-001 §4 #7 で凍結済みの引数契約であり、キャリブレーション UC の中核機能。

## 3. 推奨対応（人間裁定）

**(A) UC へ追記案**: 以下を UC-LGX-011 に追加する:
- 基本フローのコマンド記述を `legixy calibrate [--buckets N] [--json] [--recommend]` に拡張
- 代替フロー `1b. --recommend 指定時: パーセンタイル方式推奨閾値（similarity_threshold / drift_threshold / link_candidate_threshold）を追加出力する。pairs=0 の場合は推奨値を算出せず stderr に INFO を出力して exit 0`
- JSON スキーマ（Step6）に `recommended_thresholds` キーを追記

**(B) drop（委譲容認）案**: `--recommend` の出力詳細は SPEC-LGX-010 REQ.05 が規定しており、UC フローは主要制御フロー（成功 / 空ストア / バリデーション失敗 / 全体失敗）に集中すればよい。`--recommend` は引数オプションの一つに過ぎず、有効時の出力増分は SPEC 委譲として UC への明示は不要と判断する。

## 4. 影響範囲

- UC-LGX-011 §基本フロー・§代替フロー・§基本フロー Step6 JSON スキーマ（追記案 A の場合）
- TP-LGX-021 §2.6 R5（--json スキーマ整合）: 本 GAP close により R5 も GREEN 化
- 下流成果物（RBA 以降）: `--recommend` フローが認識される場合、推奨閾値算出パスの RBA 記述が必要になる
- QA リードのユースケース（false positive / false negative トレードオフ判断）の中核フローに直結

## 5. 解消（2026-06-13）

敵対的精査裁定: **GENUINE**（実 SPEC 照合で確定）。UC 修正で解消（A4: UC-LGX-011 に --recommend フラグ・推奨閾値/pairs=0 代替フローを追加）。人間承認 2026-06-13（A2/C2/C3 は AskUserQuestion 裁定、推奨案採用）。詳細: docs/spec-change-proposals/2026-06-13_uc-gap-adversarial-review.md §A。
