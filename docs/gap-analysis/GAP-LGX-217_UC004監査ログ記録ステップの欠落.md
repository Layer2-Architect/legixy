Document ID: GAP-LGX-217

# GAP-LGX-217: UC-LGX-004 基本フローに監査ログ記録ステップが存在しない

**親 TP**: TP-LGX-014
**観点**: §2.5 DF3（監査ログ記録ステップの欠落）
**ステータス**: closed (2026-06-13)
**起票日**: 2026-06-13
**分類（暫定）**: WEAK

## 1. 観点

UC-LGX-002 の基本フローには Step 7「システムが context_log に監査ログを記録する」が明示されているが、UC-LGX-004 の基本フローに相当するステップが存在しない。UC-LGX-004 の事後条件には「UC-LGX-002 の事後条件と同じ（決定論保証、監査ログ）」と記述されているが、`--granularity subnode` 指定時に granularity カラムの記録（SPEC-LGX-003.REQ.07）がフロー記述として観察可能でない。

## 2. 現状の UC / SPEC

- **UC-LGX-002 基本フロー Step 7**: 「システムが context_log に監査ログを記録する」と明示。
- **UC-LGX-004 基本フロー**: Step 4 で返却して終了。監査ログ記録ステップなし。
- **UC-LGX-004 事後条件**: 「UC-LGX-002 の事後条件と同じ（決定論保証、監査ログ）」と記述。
- **SPEC-LGX-003.REQ.07**: 「compile_context の全呼出しは context_log テーブルに記録される。legixy では granularity カラムを追加する」。
- **問題点**: granularity=subnode の呼出記録が context_log の granularity カラムに正しく記録されるかどうかが、UC フロー記述から観察可能でない。「事後条件の同一」という委譲記述のみでは、granularity カラム追加（legixy 固有の拡張）が subnode 呼出時にも記録されることが UC フローとして示されない。

## 3. 推奨対応（人間裁定）

**(A) UC への追記案**

基本フロー末尾に「5. システムが granularity=subnode を含む呼出情報を context_log に記録する（SPEC-LGX-003.REQ.07 / UC-LGX-002 Step 7 と同様）」を追記する。これにより granularity 拡張カラムの記録がフローに観察可能になる。

**(B) drop（委譲容認）案**

事後条件「UC-LGX-002 の事後条件と同じ（…監査ログ）」の委譲で十分と裁定し、UC フローの修正は不要とする。「UC-LGX-002 と同じ事後条件」にはStep 7 の監査ログ記録が内包されており、granularity カラムの詳細は SPEC-LGX-003.REQ.07 に委譲する。

## 4. 影響範囲

- UC-LGX-004 §基本フロー（追記案の場合のみ）
- 下流への影響: 軽微（監査ログ記録は SPEC 規定で確立済み。UC への追記は可読性向上目的）

## 5. 解消（2026-06-13）

敵対的精査裁定: **方針B（委譲容認）**。本観点は親 SPEC（対応 TP[SPEC] green 確定済）が回答しており、UC フローへの明示列挙は任意。GREEN-by-delegation として close。詳細: docs/spec-change-proposals/2026-06-13_uc-gap-adversarial-review.md §F。
