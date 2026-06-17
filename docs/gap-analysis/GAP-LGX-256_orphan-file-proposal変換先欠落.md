Document ID: GAP-LGX-256

# GAP-LGX-256: orphan_file カテゴリの observation に対応する Proposal 種別が UC フローに未記述

**親 TP**: TP-LGX-018
**観点**: §2.6 R1「Proposal 生成カテゴリ別変換の網羅性」
**ステータス**: closed (2026-06-13)
**起票日**: 2026-06-13
**分類（暫定）**: GENUINE

## 1. 観点

UC-LGX-008 §基本フロー「Observation 生成」Step1 は 4 観測カテゴリ（ChainIntegrity / LinkCandidate / Drift / OrphanFile）を定義するが、§基本フロー「Proposal 生成」Step3 のカテゴリ別変換は以下 3 つのみを記述する:

```
chain_integrity → add_chain_entry
link_candidate → add_link
drift → update_doc
```

4 観測カテゴリのうち `orphan_file` → 何の Proposal 種別 に変換されるかが UC フローに記述されていない。

## 2. 現状の UC / SPEC

**UC-LGX-008 §Proposal 生成 Step3:**
```
カテゴリ別の変換:
  - chain_integrity → add_chain_entry
  - link_candidate → add_link
  - drift → update_doc
```

**UC-LGX-008 §Observation 生成 Step1:**
```
- OrphanFile → orphan_file カテゴリ
```

**SPEC-LGX-007.REQ.02 / REQ.03:**

REQ.02 は `feedback` コマンドが check 結果から observation を生成することを述べるが、orphan_file category 対応 proposal 種別を明示していない。REQ.03 は analyze が observations → proposal を生成すると述べるが、カテゴリ別変換先は規定していない。

v3 実測（SPEC-LGX-007 §3 変更履歴参照）では `crates/te-feedback/src/analyzer.rs` の変換ロジックが正準化の根拠として使用されているが、orphan_file の扱いは明記されていない。

orphan_file observation が analyze の「skipped」パスに転落する可能性（対応する proposal 種別がなく変換できないため skipped 扱い）も UC フローから判別できない。

## 3. 推奨対応（人間裁定）

### (A) UC Proposal 生成 Step3 に orphan_file 変換先を追記

v3 実装を確認し、orphan_file カテゴリに対応する proposal 種別（例: `remove_file`, `register_file`, または skipped 扱い）を UC フローに追記する:

```
カテゴリ別の変換:
  - chain_integrity → add_chain_entry
  - link_candidate → add_link
  - drift → update_doc
  - orphan_file → remove_file（または: skipped — 対応 proposal なし）
```

### (B) drop（SPEC への委譲）案

「orphan_file の変換先は SPEC-LGX-007.REQ.03 または DD でのみ定義すれば足りる」として UC フロー記述への追記は不要と判断し close する。ただし SPEC-LGX-007.REQ.03 へ orphan_file → proposal 変換先の明記を求める SPEC 改訂を起案することを条件とする。

## 4. 影響範囲

- UC-LGX-008 §Proposal 生成 Step3: orphan_file 変換先の追記
- SPEC-LGX-007.REQ.03: analyze のカテゴリ別変換先の完全列挙（現在は規定なし）
- GAP-LGX-257（R2 skipped 発火条件）との関連: orphan_file が skipped になる場合は skipped 条件定義の一部となる
- RBA/DD: orphan_file observation に対応する Proposal 種別のアクション定義
- TS: orphan_file category を含む feedback → analyze パイプラインの E2E テスト

## 5. 解消（2026-06-13）

敵対的精査裁定: **方針B（委譲容認）**。本観点は親 SPEC（対応 TP[SPEC] green 確定済）が回答しており、UC フローへの明示列挙は任意。GREEN-by-delegation として close。詳細: docs/spec-change-proposals/2026-06-13_uc-gap-adversarial-review.md §F。
