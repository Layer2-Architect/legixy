Document ID: GAP-LGX-212

# GAP-LGX-212: UC-LGX-004 不正 granularity 値の失敗パスがフローに列挙されていない

**親 TP**: TP-LGX-014
**観点**: §2.2 AF1（granularity 不正値フローの欠落）
**ステータス**: closed (2026-06-13)
**起票日**: 2026-06-13
**分類（暫定）**: WEAK

## 1. 観点

UC-LGX-004 の代替フロー（1a=document、基本フロー=subnode）は `granularity` の 2 値のみを記述しており、document/subnode 以外の不正値が指定された場合の失敗パスが UC フローに列挙されていない。

## 2. 現状の UC / SPEC

- **UC-LGX-004 代替フロー 1a**: 「`--granularity document`（デフォルト）の場合、UC-LGX-002 と同一の動作をする」。
- **基本フロー**: `--granularity subnode` を前提として記述。
- **SPEC-LGX-003.REQ.03**: 「document/subnode の 2 値のみサポート」と明記。
- **LGX-COMPAT-001 §1 グローバル規約**: 構文レベルの誤り（clap が検出）は exit 2、受理済み値の意味的不正は exit 1。不正値は clap の value_parser が拒否 → exit 2 として凍結済み。
- **問題点**: SPEC と凍結契約で挙動は確定しているが、UC フロー記述としての失敗パス列挙が存在しない。

## 3. 推奨対応（人間裁定）

**(A) UC への追記案**

代替フロー に「1b. `--granularity` に document / subnode 以外の値が指定された場合、引数パース層がエラーを返し exit 2 で終了する（LGX-COMPAT-001 §1 へ委譲）」を追記する。

**(B) drop（委譲容認）案**

SPEC-LGX-003.REQ.03 の「2 値のみサポート」と LGX-COMPAT-001 §1 の凍結 exit 2 規約に委譲し、UC フローの修正は不要と裁定する。代替フロー全体が 2 値前提で記述されており、不正値は「代替フロー発動前に拒否される」構造は SPEC 委譲で十分に示される。

## 4. 影響範囲

- UC-LGX-004 §代替フロー（追記案の場合のみ）
- 下流への影響: 軽微（入力検証層の実装指針に関わるが凍結契約が既存のため変更なし）

## 5. 解消（2026-06-13）

敵対的精査裁定: **方針B（委譲容認）**。本観点は親 SPEC（対応 TP[SPEC] green 確定済）が回答しており、UC フローへの明示列挙は任意。GREEN-by-delegation として close。詳細: docs/spec-change-proposals/2026-06-13_uc-gap-adversarial-review.md §F。
