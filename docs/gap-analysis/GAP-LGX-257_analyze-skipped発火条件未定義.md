Document ID: GAP-LGX-257

# GAP-LGX-257: analyze の Pessimistic Claim パターン「skipped」終端の発火条件が UC フローに未定義

**親 TP**: TP-LGX-018
**観点**: §2.6 R2「analyze の skipped パスの発火条件」
**ステータス**: closed (2026-06-13)
**起票日**: 2026-06-13
**分類（暫定）**: GENUINE

## 1. 観点

UC-LGX-008 §基本フロー「Proposal 生成」Step2 は「Pessimistic Claim パターン: pending → analyzing → proposed/skipped」と記述し、`skipped` という終端状態を示す。しかし、どの条件で `proposed` ではなく `skipped` になるかが UC フローに定義されていない。SPEC-LGX-007 も skipped の発火条件を明示していない（v3 実装では「不正 category → skipped」があったが category が 3 値凍結後の skipped 発生条件は未規定）。

## 2. 現状の UC / SPEC

**UC-LGX-008 §Proposal 生成 Step2:**
```
Pessimistic Claim パターン: pending → analyzing → proposed/skipped
```

**SPEC-LGX-007.REQ.03:**
```
`legixy analyze` は observations を集約・分析し、対応する proposal を生成する。
```

`skipped` 状態への遷移条件は REQ.03 に記述なし。

**SPEC-LGX-007.REQ.01（v3 差分参照）:**
```
v3 の CLI 経路は無検証（`category: String`）で、不正 category は保存後 analyze で「その他 → skipped」となり observation が死蔵されていた。正当な 3 値の挙動は完全不変
```

SPEC の変更履歴（v0.4.0）によると v3 では「不正 category → skipped」という経路があったが、category が 3 値凍結（`compile_miss` / `review_correction` / `manual_note`）された legixy では不正 category が UC/CLI/MCP レベルで reject されるため、この経路は正常フローでは発生しない。しかし skipped 状態は Pessimistic Claim パターンの一部として残留しており、その発火条件が未定義のまま。

**skipped 状態の問題:**
- skipped に遷移した observation は observation の状態モデル（SPEC-LGX-007.REQ.08: pending/analyzing/resolved の 3 値）に収まらない（resolved でも analyzing でもない。skipped は observation status ではなく observation の「処理結果」かもしれないが不明）
- skipped のまま残った observation は再分析対象（pending）に戻るのか、永続的に skipped に留まるのかが不明

## 3. 推奨対応（人間裁定）

### (A) UC に skipped 発火条件を追記

category 凍結後の legixy における skipped 発生条件を明示する。候補:

```
Pessimistic Claim パターン: pending → analyzing → {proposed | skipped}
  - proposed: 対応する Proposal 種別へ変換成功
  - skipped: 対応する Proposal 種別が存在しない（例: orphan_file category への未実装変換先）
             または analyze 時に同一 semantic_key の pending Proposal が既存（FB-INV-5 重複抑止）
```

また、skipped 後の observation の状態遷移（再 pending 化するか、終端 skipped 状態として残るか）を UC §Observation 生成または事後条件に追記する。

### (B) SPEC-LGX-007 への追記案

analyze の skipped 条件を SPEC-LGX-007.REQ.03 または REQ.08 の observation 状態モデルに追記し、UC はそれへの参照として委譲する。この場合は SPEC 変更であり `/spec-change` プロセスを経る必要がある。

### (C) drop（実装詳細として委譲）案

「skipped の発火条件は DD での実装詳細」として UC フロー記述には現状のまま残し、GAP をこの粒度で close する。ただし DD での skipped 条件の明示と observation の状態モデル接続（REQ.08 の 3 値モデルとの整合）を確保することを条件とする。

## 4. 影響範囲

- SPEC-LGX-007.REQ.03/REQ.08: analyze の skipped 条件と observation 状態モデルへの接続
- GAP-LGX-256（R1 orphan_file 変換先欠落）との関連: orphan_file が skipped の主要な発生源になり得る
- RBA/SEQA: analyze の skipped パスのアクター責務（skipped observation を人間が手動 resolve/再観測するか）
- DD: Pessimistic Claim の skipped 終端処理の実装
- TS: analyze の skipped パスを検証するテストケース（どの category/条件で skipped になるかの fixture が必要）

## 5. 解消（2026-06-13）

敵対的精査裁定: **方針B（委譲容認）**。本観点は親 SPEC（対応 TP[SPEC] green 確定済）が回答しており、UC フローへの明示列挙は任意。GREEN-by-delegation として close。詳細: docs/spec-change-proposals/2026-06-13_uc-gap-adversarial-review.md §F。
