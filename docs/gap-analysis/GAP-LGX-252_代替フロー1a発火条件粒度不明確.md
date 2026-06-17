Document ID: GAP-LGX-252

# GAP-LGX-252: 代替フロー 1a「check 結果に該当カテゴリがない場合」の発火粒度が不明確

**親 TP**: TP-LGX-018
**観点**: §2.2 AF1「代替フロー 1a の発火条件の明示」
**ステータス**: closed (2026-06-13)
**起票日**: 2026-06-13
**分類（暫定）**: WEAK

## 1. 観点

UC-LGX-008 §代替フロー「1a. check 結果に該当カテゴリがない場合、Observation は生成されない」は発火条件の粒度が不明確。4 観測カテゴリ（ChainIntegrity / LinkCandidate / Drift / OrphanFile）すべてが 0 件の場合のみ 1a が発火するのか、各カテゴリ個別に「そのカテゴリの Observation を生成しない」という per-category の縮退として発火するのかが UC フローから判別できない。

## 2. 現状の UC / SPEC

**UC-LGX-008 §代替フロー:**
```
1a. check 結果に該当カテゴリがない場合、Observation は生成されない
```

**SPEC-LGX-007.REQ.02:**
```
`legixy feedback` は check の結果や embedding から未対応の observation を生成する。
```

SPEC は「該当カテゴリがない場合」の粒度（全件 vs per-category）を規定していない。実装上は per-category に自然に処理される（各カテゴリの結果件数が 0 なら当該カテゴリの observation を生成しないだけ）と想定されるが、UC フロー記述の曖昧さとして記録する。

## 3. 推奨対応（人間裁定）

### (A) UC に追記案

代替フロー 1a の発火条件を per-category として明示する:

```
1a. check 結果に該当カテゴリの finding が 0 件の場合、そのカテゴリの Observation は生成されない
    （チェック 4 カテゴリすべてが 0 件の場合、Observation は一切生成されない）
```

### (B) drop（委譲容認）案

「per-category の自然な縮退であり明示不要」として UC フロー記述には「check 結果に該当カテゴリがない場合」のままで close する。SPEC-LGX-007.REQ.02 の機械的帰結として処理可。

## 4. 影響範囲

- SEQA/DD: feedback コマンドの空ループ時の出力フォーマット（0 件時の stdout/stderr）
- TS: 各カテゴリ 0 件・全カテゴリ 0 件の境界値テストケース設計

## 5. 解消（2026-06-13）

敵対的精査裁定: **方針B（委譲容認）**。本観点は親 SPEC（対応 TP[SPEC] green 確定済）が回答しており、UC フローへの明示列挙は任意。GREEN-by-delegation として close。詳細: docs/spec-change-proposals/2026-06-13_uc-gap-adversarial-review.md §F。
