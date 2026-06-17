Document ID: GAP-LGX-191

# GAP-LGX-191: UC-LGX-002 の `--command` フラグが SPEC-LGX-003 に未定義

**親 TP**: TP-LGX-012
**観点**: §2.1 BF2 / §2.4 AT3（統合）
**ステータス**: closed (2026-06-13)
**起票日**: 2026-06-13
**分類（暫定）**: GENUINE

## 1. 観点

UC-LGX-002 Step1 に `legixy context <files> [--command <intent>]` として `--command <intent>` フラグが登場するが、SPEC-LGX-003.REQ.01〜REQ.20 のいずれにもこのフラグの定義・挙動・制約が存在しない。フロー記述が親 SPEC にない要素を先行して具体化しており、「フロー記述が SPEC を忠実に具体化しているか」が検証不能。また `--command` による intent がシステムの上流解決ロジック（Step2〜5）に影響するかどうかも UC フローで不明であり、アクターの入力とシステムの処理範囲の対応が観察不能。

## 2. 現状の UC / SPEC

- **UC-LGX-002 基本フロー Step1**: 「アクターが `legixy context <files> [--command <intent>]` を実行する」
- **SPEC-LGX-003.REQ.01**: 入力として `target_files`（必須）と `granularity`（任意）および「その他オプション: v0.1.0 を継承」を規定。`--command` は列挙されていない
- **SPEC-LGX-003 全 REQ**: `--command` に対する要求・挙動・CLI フラグ定義なし
- **LGX-COMPAT-001**: `--command` に関する凍結契約の記述なし（現時点で確認できる範囲）

## 3. 推奨対応（人間裁定）

**(A) SPEC-LGX-003 へ `--command` の定義を追記する**
`--command <intent>` が intent ヒントとして渡され、システムがそれをどう扱うか（ログ記録のみ / 上流フィルタリングに使う / 将来拡張として無視する等）を REQ として追加する。UC-LGX-002 Step1 のフラグ記述はその後に正当化される。

**(B) UC-LGX-002 から `--command <intent>` の記述を削除する**
もし `--command` が実装済みの別フラグの誤記録・将来拡張の先行記述に過ぎないなら、現行 SPEC の範囲で UC フローを記述し、`--command` は将来の UC 拡張時に追加する。

## 4. 影響範囲

- UC-LGX-002 基本フロー Step1（記述の修正または削除）
- SPEC-LGX-003 REQ.01 への追加（案 A の場合）
- TP-LGX-012 BF2 / AT3（解消後 GREEN 化）
- LGX-COMPAT-001: `--command` が実装済みフラグの場合、凍結契約リストへの追加要否を確認

## 5. 解消（2026-06-13）

敵対的精査裁定: **REFUTED / OUT_OF_SCOPE**（棄却）。実 SPEC / LGX-COMPAT-001 照合により本 GAP の前提が成立しないことを確認した（サブエージェントの過剰検出）。詳細: docs/spec-change-proposals/2026-06-13_uc-gap-adversarial-review.md §E。
