Document ID: GAP-LGX-251

# GAP-LGX-251: feedback コマンドの入力（check 結果受け取り方）が UC フローで観察不能

**親 TP**: TP-LGX-018
**観点**: §2.1 BF2「feedback コマンドの入力受け取り方の観察可能性」
**ステータス**: closed (2026-06-13)
**起票日**: 2026-06-13
**分類（暫定）**: GENUINE

## 1. 観点

UC-LGX-008 §基本フロー「Observation 生成」Step1 は「`feedback` コマンド: check 結果から自動で Observation を生成する」と記述するが、check 結果をどのように受け取るか（直接引数として渡す / 自動的に check を実行して取得する / 既存ファイルから読み取る）が観察可能なステップとして記述されていない。

## 2. 現状の UC / SPEC

**UC-LGX-008 §基本フロー（Observation 生成 Step1）:**
```
`feedback` コマンド: check 結果から自動で Observation を生成する
  - ChainIntegrity → chain_integrity カテゴリ
  - LinkCandidate → link_candidate カテゴリ
  - Drift → drift カテゴリ
  - OrphanFile → orphan_file カテゴリ
```

**SPEC-LGX-007.REQ.02:**
```
`legixy feedback` は check の結果や embedding から未対応の observation を生成する。
人間のみが CLI で実行する。
```

SPEC も「check の結果や embedding から」と入力源を示すが、具体的な受け取り方（コマンド引数 / 自動実行 / ファイルパイプ）を規定していない。UC も SPEC も入力インターフェースを未定義のまま残している。

## 3. 推奨対応（人間裁定）

### (A) UC に追記案

§基本フロー「Observation 生成」Step1 の `feedback` コマンド記述に、check 結果の受け取り方を明示する:

```
`feedback` コマンド: check 結果から自動で Observation を生成する
  - feedback 実行時に内部で check を呼び出し CheckReport を取得する
    （または: 引数として check 出力ファイルパスを受け取る）
```

どちらの方式か、または両対応かは設計判断（LGX-COMPAT-001 §4 との整合も確認要）。

### (B) drop（委譲容認）案

「feedback コマンドの入力インターフェースは DD で確定する実装詳細」として UC フロー記述には不要と判断し、GAP を close する。ただし RBA/SEQA で観察可能なアクター間データフローを明示することを条件とする。

## 4. 影響範囲

- RBA/SEQA: feedback コマンドのアクター間データフロー（check 結果の流れ）が設計の起点になるため、UC 未定義のまま下流に進むと設計根拠が失われる
- DD: feedback コマンドの引数定義（LGX-COMPAT-001 §4 との整合）
- 下流: TS（フィードバックループ統合テスト）の入力 fixture 設計

## 5. 解消（2026-06-13）

敵対的精査裁定: **方針B（委譲容認）**。本観点は親 SPEC（対応 TP[SPEC] green 確定済）が回答しており、UC フローへの明示列挙は任意。GREEN-by-delegation として close。詳細: docs/spec-change-proposals/2026-06-13_uc-gap-adversarial-review.md §F。
