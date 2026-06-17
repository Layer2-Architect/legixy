Document ID: GAP-LGX-106

# GAP-LGX-106: embed 実行時の ONNX モデル不在・読込失敗時の挙動と終了コードが未定義

**親 TP**: TP-LGX-006
**観点出典**: TP-LGX-006 §2.2 観点 E-01
**ステータス**: closed (2026-06-10)
**起票日**: 2026-06-08
**重要度**: minor (verification: low-value, 人間判断で drop 可) — 敵対的精査パス 2026-06-09: 終了コードは NFR OBS.05 + SPEC-LGX-010 REQ.03（drift のモデル解決失敗 = exit 1）の前例で類推可能。embed 固有の明文化は望ましいが大半は既答。GAP-LGX-107（解決順序）は本 GAP の DUPLICATE として削除済み。WEAK_OR_PADDED として維持。

## 1. 観点

`embed` は embedding 生成のため ONNX モデルの解決・読込が必須だが、モデルが不在・破損・権限不足で読み込めない場合の挙動（即時 Error 終了か、Warning + スキップか）と終了コードが未定義。SPEC-LGX-010 REQ.03 は `drift` のモデル解決失敗を exit 1 と規定しているが、`embed` コマンド本体は SPEC-LGX-006 の owner であり、SPEC-006 側に同等の規定が無い。

## 2. 現状の SPEC / UC

SPEC-LGX-006 §3 REQ.01 はモデルパスを `.legixy.toml [semantic]` で指定するとし、検証方法に「モデル読み込みテスト」を挙げるが、**読込失敗時の挙動・終了コード・エラーメッセージ**は未定義。SPEC-LGX-010 REQ.03 のモデル解決失敗（exit 1）は明示的に「drift のみ」を対象とし、`embed` は範囲外。

## 3. 期待される情報

- `embed` 実行時のモデル読込失敗: Error + 終了コード（NFR-LGX-001.OBS.05 の exit 1 想定。明文化が必要）
- 試行したモデルパスの stderr 通知（SPEC-LGX-010 REQ.03 と同等の診断）
- `[semantic] enabled = false` 時に `embed` を実行した場合の挙動（no-op / Error / Warning）

## 4. 影響範囲

- UC-LGX-007: embedding 生成の例外フロー（モデル不在）が定義できない
- 下流 DD / TS: embed コマンドの初期化失敗パスとテスト
- SPEC-LGX-010 との整合: drift と embed のモデル解決失敗挙動を揃えるべきか（GAP-LGX-107 と連携）

## 5. 解決経緯（クローズ時に記入）

SPEC-LGX-006 v0.7.0（人間裁定 fix・承認 2026-06-10）: REQ.02 にモデル解決・読込失敗 = Error exit 1 + 試行パス stderr 通知（SPEC-LGX-010.REQ.03 と同一解決順）を確定（v3 実測 embed.rs:48-54 の正準化）。[semantic] enabled は check の意味層専用で embed の実行可否に影響しない（v3 実挙動）。

## 6. 関連 ADR

該当なし（embed と drift のモデル失敗挙動の統一は実装方針）。
