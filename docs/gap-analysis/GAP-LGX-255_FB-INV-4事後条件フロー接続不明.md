Document ID: GAP-LGX-255

# GAP-LGX-255: UC 事後条件の FB-INV-4 記述がフィードバックループフローと接続していない

**親 TP**: TP-LGX-018
**観点**: §2.5 DF2「事後条件 FB-INV-4 とフロー記述の接続」
**ステータス**: closed (2026-06-13)
**起票日**: 2026-06-13
**分類（暫定）**: WEAK

## 1. 観点

UC-LGX-008 §事後条件の一文「engine.db がなくてもグラフ上流は正常に返される（FB-INV-4）」は、フィードバックループ UC の基本フロー・代替フローのどのステップとも接続していない。本 UC の事前条件は「engine.db が存在する」であり、engine.db 不在のケースはフロー記述に登場しない。FB-INV-4 は本 UC の動作結果を述べる事後条件としてフロー記述から導出できない。

## 2. 現状の UC / SPEC

**UC-LGX-008 §事後条件:**
```
- pending の Proposal は context 結果に影響しない（FB-INV-3）
- engine.db がなくてもグラフ上流は正常に返される（FB-INV-4）
```

**SPEC-LGX-007 §4（FB-INV-4）:**
```
| FB-INV-4（DB 不在時安全性） | 関連 | SPEC-LGX-003 主導。本 SPEC は DB 前提のため DB 不在時は observation/proposal 機能が無効化される設計。
```

SPEC-LGX-007 §4 自体が「FB-INV-4 は SPEC-LGX-003 主導であり、本 SPEC は DB 前提」と明示している。つまり FB-INV-4 の主語はコンテキスト解決機能（UC-LGX-003/004 系）であり、フィードバックループ UC の事後条件として記述するのは範囲外の性質を持つ。

**UC-LGX-008 §事前条件（矛盾）:**
```
- engine.db が存在する
```

事前条件で engine.db 存在を必須としながら、事後条件で engine.db 不在時の挙動を述べており整合が取れていない。

## 3. 推奨対応（人間裁定）

### (A) 事後条件から FB-INV-4 を削除（スコープ外として drop）

FB-INV-4 はフィードバックループ UC のスコープ外（SPEC-LGX-003 主導）として、UC-LGX-008 §事後条件から削除する。フィードバックループ UC の適切な事後条件は「FB-INV-3（pending が context に影響しない）」のみとする。

### (B) 事後条件の文言を修正して接続を明示

「本 UC が関与する feedback.db（engine.db）は、グラフ上流の compile_context に影響しない（FB-INV-4 の一側面）」という接続を明示する文言に修正する:

```
- フィードバックループ機能（observations / proposals）の状態は、コンテキスト解決（compile_context）の結果に影響しない
  （engine.db が存在しない環境では observation/proposal 機能は無効化されるが、グラフ上流は正常動作する: FB-INV-4）
```

## 4. 影響範囲

- UC-LGX-008 §事後条件: 修正の場合は本 UC ファイルの更新（人間裁定後に Author が実施）
- SPEC-LGX-007 §4: 変更不要（FB-INV-4 の主導が SPEC-LGX-003 であることは既に明記済み）
- TS: フィードバックループ UC のテストシナリオ設計（engine.db 不在時のシナリオが本 UC に含まれるかどうかの確定）

## 5. 解消（2026-06-13）

敵対的精査裁定: **方針B（委譲容認）**。本観点は親 SPEC（対応 TP[SPEC] green 確定済）が回答しており、UC フローへの明示列挙は任意。GREEN-by-delegation として close。詳細: docs/spec-change-proposals/2026-06-13_uc-gap-adversarial-review.md §F。
