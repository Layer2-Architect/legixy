Document ID: GAP-LGX-253

# GAP-LGX-253: analyze の claim release（代替フロー 2a）の発火条件が UC フローに未定義

**親 TP**: TP-LGX-018
**観点**: §2.2 AF2「代替フロー 2a の失敗発火条件の明示」
**ステータス**: closed (2026-06-13)
**起票日**: 2026-06-13
**分類（暫定）**: GENUINE

## 1. 観点

UC-LGX-008 §代替フロー「2a. analyze で処理中に失敗した場合、Observation を pending に戻す（claim release）」は「失敗した場合」の発火条件が具体的に定義されていない。どのような失敗（プロセス強制終了 / DB エラー / 分析アルゴリズム例外 / タイムアウト）が claim release を起動するかが観察不能であり、RBA/DD での状態回復設計の根拠が失われる。

## 2. 現状の UC / SPEC

**UC-LGX-008 §代替フロー:**
```
2a. analyze で処理中に失敗した場合、Observation を pending に戻す（claim release）
```

**SPEC-LGX-007.REQ.03:**
```
`legixy analyze` は observations を集約・分析し、対応する proposal を生成する。
人間のみが CLI で実行する。
```

REQ.03 は失敗時の claim release を明示していない。SPEC-LGX-007.REQ.08（observation 状態モデル）は「`reject` または proposal 未生成 → pending（再分析対象に戻る）」を規定しているが、これは正常な reject 後の状態遷移であり、analyze 処理中の中断シナリオを扱っていない。

analyzing 状態にある observation がプロセス kill 等で pending に戻れない場合、当該 observation は永続的に analyzing 状態に留まり再分析対象から除外される（REQ.11 の dedup 適用範囲 `status IN ('pending', 'analyzing')` により新規観測も重複排除される）リスクがある。

## 3. 推奨対応（人間裁定）

### (A) UC に追記案

代替フロー 2a の発火条件を具体化する:

```
2a. analyze が以下の場合に Observation を pending に戻す（claim release）:
    - analyze プロセスの異常終了（SIGKILL / クラッシュ）
    - DB 書き込み失敗（observation status 更新失敗）
    - 分析アルゴリズム例外（Proposal 生成不能）
    ※ 通常終了で Proposal 未生成の場合は skipped（UC §Proposal 生成 Step2 参照）
```

加えて、analyze 再実行時（または legixy 起動時）に analyzing 残留 observation を pending に戻す「orphaned claim の回収」ステップを基本フローまたは事前条件に記述することを推奨。

### (B) drop（委譲容認）案

「claim release の機構は Pessimistic Claim パターンの実装詳細であり SPEC.REQ.08 の状態モデルが包括する」として UC フロー記述への追記は不要と判断し close する。ただし DD での orphaned analyzing 状態の回収処理の設計根拠を ADR に残すことを条件とする。

## 4. 影響範囲

- SPEC-LGX-007.REQ.03/REQ.08: 失敗時の claim release 明示（SPEC レベルの記述補完も要検討）
- RBA/SEQA: analyze の失敗回復フロー設計
- DD: Pessimistic Claim の orphaned claim 回収の実装戦略
- TS: analyze 中断（プロセス kill）後の DB 状態を検証するテストケース

## 5. 解消（2026-06-13）

敵対的精査裁定: **方針B（委譲容認）**。本観点は親 SPEC（対応 TP[SPEC] green 確定済）が回答しており、UC フローへの明示列挙は任意。GREEN-by-delegation として close。詳細: docs/spec-change-proposals/2026-06-13_uc-gap-adversarial-review.md §F。
