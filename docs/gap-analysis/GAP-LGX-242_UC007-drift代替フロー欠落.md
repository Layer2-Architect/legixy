Document ID: GAP-LGX-242

# GAP-LGX-242: UC-LGX-007 drift フローの代替フロー・失敗パスが全欠落

**親 TP**: TP-LGX-017
**観点**: §2.2 AF3 / §2.3 EF2（統合）
**ステータス**: closed (2026-06-13)
**起票日**: 2026-06-13
**分類（暫定）**: GENUINE

## 1. 観点

UC-LGX-007 は embed コマンドの代替フロー（2a / 3b）を記述するが、drift コマンドに対応する代替フローが UC に一切存在しない。SPEC-LGX-010.REQ.03 が詳細を規定するが、drift フローの分岐・失敗パスが UC フロー上から観察不能であり、UC レベルの記述として構造的に欠落している。

## 2. 現状の UC / SPEC

**UC-LGX-007 代替フロー（現行）:**
> - 2a. ONNX モデルが存在しない場合、ERROR を報告する
> - 3b. `--all` の場合、ハッシュ比較をスキップして全ノードを再生成する

drift コマンドに対応する代替フロー・例外フローが存在しない。

**SPEC-LGX-010.REQ.03 が規定する drift の分岐（既存）:**
- ベースライン不在（未 embed ノード・スナップショット行なし）: stderr へ INFO 通知 + exit 0。`--json` 時は `{"drift": null, "baseline_available": false}` を stdout、INFO は stderr に併出
- `--against snapshot:<token>` 指定: label 解決（最新 1 件）→ snapshot_id フォールバック
- `snapshot:` プレフィクス欠如の `--against` 値: exit 1（アプリ層 reject）
- 現行ファイル欠落: ERROR (stderr) + exit 1
- モデル解決全失敗・読込失敗: exit 1 + 試行内容を stderr 通知
- model_version 不一致（同一次元で別バージョン）: exit 1（GAP-LGX-186 対応）
- 非有限スコア（NaN/±Inf）: exit 1

これらの重要な分岐が UC-LGX-007 の drift フローに記述されていない。

## 3. 推奨対応（人間裁定）

**(A) UC へ追記案:**
drift フローの代替フロー節を新設し、最低限の分岐を列挙する:
> （drift コマンド代替フロー）
> - D-1a. モデルが存在しない場合（解決失敗）: stderr に試行パスを通知し exit 1
> - D-2a. ベースライン（既存 embedding）が存在しない場合: INFO を stderr に通知し exit 0
> - D-3a. 現行ファイルが欠落している場合: ERROR を stderr に通知し exit 1
> - D-4a. model_version 不一致（GAP-LGX-186）: exit 1

**(B) drop（委譲容認）案:**
SPEC-LGX-010.REQ.03 が drift の全分岐・終了コード・JSON スキーマを完全に規定しており、UC-LGX-007 はフロー概要の記述に留める設計とみなす。drift の代替フローは UC-LGX-013（standalone ドリフト対比、SPEC-LGX-010.REQ.03 が対応）が分離して詳細記述するため、UC-LGX-007 での列挙は不要と裁定する。

※ GENUINE 判断の根拠: SPEC-LGX-010 の対応 UC は UC-LGX-013（`drift = standalone 対比`）であり、UC-LGX-007 は embed の UC として定義されている。drift フローが UC-LGX-007 に組み込まれていること自体が UC 粒度の設計判断であり、組み込まれる以上は代替フロー記述も必要と判断した（GENUINE 寄り）。ただし「drift の詳細は UC-LGX-013 で扱う」という分割設計が確定するなら OUT_OF_SCOPE となる可能性がある。

## 4. 影響範囲

- UC-LGX-007 drift フロー節の代替フロー記述追加（案 A の場合）
- または UC-LGX-007 から drift フロー節を削除し UC-LGX-013 に移譲する設計変更の検討（案 B の応用）
- 後続 RBA/DD での drift 失敗パス設計根拠に影響
- GAP-LGX-244（drift Step2 の観察可能性欠如）と関連（両 GAP とも drift フロー記述の構造的欠落に起因）

## 5. 解消（2026-06-13）

敵対的精査裁定: **GENUINE**（実 SPEC 照合で確定）。UC 修正で解消（C3: UC-LGX-007 を embed 専念化し drift を UC-LGX-013 へ委譲）。人間承認 2026-06-13（A2/C2/C3 は AskUserQuestion 裁定、推奨案採用）。詳細: docs/spec-change-proposals/2026-06-13_uc-gap-adversarial-review.md §C。
