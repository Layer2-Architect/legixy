Document ID: GAP-LGX-241

# GAP-LGX-241: UC-LGX-007 embed 代替フロー 2a の終了コード・通知先が未定義

**親 TP**: TP-LGX-017
**観点**: §2.2 AF1 / §2.3 EF4（統合）
**ステータス**: closed (2026-06-13)
**起票日**: 2026-06-13
**分類（暫定）**: WEAK

## 1. 観点

UC-LGX-007 の代替フロー 2a「ONNX モデルが存在しない場合、ERROR を報告する」は出力先（stdout/stderr）・終了コードを記述しない。UC 全体として embed/drift のエラー通知規約（終了コード 0/1/2）が UC フロー・事後条件から観察不能。

## 2. 現状の UC / SPEC

**UC-LGX-007 代替フロー 2a（現行）:**
> 2a. ONNX モデルが存在しない場合、ERROR を報告する

出力先・終了コードに関する記述が無い。

**SPEC-LGX-006.REQ.02 の規定（既存）:**
> モデルの解決・読込に失敗した場合は **Error + exit 1** とし、試行したパスを stderr に通知する

SPEC は exit 1 + stderr 通知を明確に規定している。UC フロー記述への反映が欠落しているが、SPEC が既に答えている。

## 3. 推奨対応（人間裁定）

**(A) UC へ追記案:**
代替フロー 2a を以下に展開する:
> 2a. ONNX モデルが存在しない場合（解決・読込失敗）:
>   1. 試行したモデルパスを stderr に出力する
>   2. exit 1 で終了する

**(B) drop（委譲容認）案:**
SPEC-LGX-006.REQ.02 が exit 1 + stderr 通知を明示規定しており、UC フローでの再記述は冗長。「ERROR を報告する」という記述で SPEC-006.REQ.02 への委譲が暗示されているとみなし、UC フローへの追記は不要と裁定する。

## 4. 影響範囲

- UC-LGX-007 代替フロー 2a の記述明確化
- 後続の RBA/DD での終了コード設計根拠（SPEC-006.REQ.02 への参照が明示化されれば追跡容易になる）
- WEAK 確定の場合は UC フロー変更不要

## 5. 解消（2026-06-13）

敵対的精査裁定: **方針B（委譲容認）**。本観点は親 SPEC（対応 TP[SPEC] green 確定済）が回答しており、UC フローへの明示列挙は任意。GREEN-by-delegation として close。詳細: docs/spec-change-proposals/2026-06-13_uc-gap-adversarial-review.md §F。
