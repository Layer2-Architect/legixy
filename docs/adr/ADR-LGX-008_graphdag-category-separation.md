Document ID: ADR-LGX-008

# ADR-LGX-008: グラフ全体 DAG 検証を新カテゴリ GraphDag に分離する

**ステータス**: accepted
**起票日**: 2026-06-10
**承認日**: 2026-06-10
**承認者**: 開発者（人間裁定、AskUserQuestion 経由）
**対象**: SPEC-LGX-004 §3 REQ.01 / REQ.15、§4 CTX-INV-4

## 1. 文脈（Context）

- 背景: GAP-LGX-064。グラフ全体（Chain/Custom/ParentChild 全エッジ種別）のサイクル検出（CTX-INV-4）が、v3 実装ではサブノード用カテゴリ `SubnodeDag` の名前で報告されていた（`traceability-engine.v3/crates/te-graph/src/validation.rs:52-65`。`CheckCategory` enum にグラフ全体用カテゴリが存在しない）。カテゴリ名が適用範囲を誤って示しており、severity 割当表（REQ.15）の正確性にも影響する。
- 制約: LGX-COMPAT-001 の凍結対象は引数・既定値・終了コード・MCP 3 ツールであり、check 出力のカテゴリ名は凍結対象外。ただし v3 出力を解析する既存スクリプトがあれば影響し得る。

## 2. 検討した選択肢（Options）

### 選択肢 A: 新カテゴリ GraphDag（採用）

- 概要: グラフ全体サイクルを `GraphDag`【Error】として報告。サブノード関与エッジの DAG は `SubnodeDag` に残る。【v3 差分】注記必須。
- 利点: カテゴリ名が適用範囲を正確に表す。CTX-INV-4（グラフ全体）と SUBNODE-INV-4（サブノード）の検証カテゴリが 1:1 対応。
- 欠点: v3 の JSON 出力からカテゴリ名が変わる（出力フォーマットは凍結対象外だが既存パーサには差分）。

### 選択肢 B: v3 のまま SubnodeDag 名を維持

- 利点: 出力完全互換。欠点: 「Subnode」と名乗りながらグラフ全体に作用する誤誘導が恒久化。

### 選択肢 C: ChainIntegrity に統合

- 欠点: v3 実挙動とも新設案とも異なる第 3 の出力変更で、エッジ整合性（参照解決）とサイクル検出という別概念を混ぜる。

## 3. 判断（Decision）

選択肢 A を採用する（人間裁定 2026-06-10）。

理由:

- カテゴリは不変条件の検証チャネルであり、名前の正確性が偽 green/red の解析容易性に直結する。
- 出力フォーマットは凍結契約外であり、【v3 差分】注記 + 本 ADR で変更を追跡可能にする。

## 4. 結果（Consequences）

### 期待される効果
- REQ.15 severity 割当表で CTX-INV-4 と SUBNODE-INV-4 が別行として完全に定義される。

### 受け入れる代償
- v3 出力のカテゴリ名を前提とする既存ツールがあれば追従が必要（【v3 差分】として文書化済み）。

### 残存リスク
- 既存パーサの破損 → カテゴリ名は --json 出力に現れるため、DD/TS で GraphDag/SubnodeDag 両方の fixture を用意し回帰を検出する。

## 5. 関連

- closes: GAP-LGX-064（旧 073/074 吸収）の人間判断点 (a)。判断点 (b)（DocumentId 行欠落 = Error）も同裁定で確定（v3 実挙動と一致のため ADR 不要、SPEC-LGX-004 REQ.15 に記録）
