# Document ID: SPP-LGX-006

**親 QSET**: QSET-LGX-006
**対象 SPEC**: SPEC-LGX-006（v0.3.1〔ヘッダ表記〕→ v0.5.0）
**作成日**: 2026-06-07
**作成者**: AI (designer)
**承認状態**: 承認済（2026-06-07 by 開発者。一括承認 — QSET 対応分として全差分を承認）

---

## 概要

QSET-LGX-006 への開発者回答（2026-06-07 確定）を反映した SPEC 差分案。特徴: 差分 1 は **SPEC 無変更 + 運用ファイル整合の指示**（Q1）、差分 3 は版数・採番の整理（Q3）。Q2 の calibrate 仕様本体は SPEC-LGX-010 に規定済みのため、本 SPP は境界面（REQ.11）への相互参照追記のみを行う。

**ハードルール 1**: 本 SPP は人間が承認するまで SPEC / 運用ファイルに反映されない。

---

## 差分一覧

### 差分 1: 既定 embedding モデルの運用整合（SPEC 無変更、運用ファイル修正）

**対応 QSET 質問**: Q1

**SPEC 変更**: なし（REQ.01 の `paraphrase-multilingual-MiniLM-L12-v2` 既定を正準として維持）。

**運用ファイル修正（本 SPP 承認と同時に適用）**:

1. `CLAUDE.md` プロジェクト固有の補足:
```
修正前: - ONNX モデル: `models/all-MiniLM-L6-v2/`（第 2 層 semantic を有効化する場合のみ）
修正後: - ONNX モデル: `models/paraphrase-multilingual-MiniLM-L12-v2/`（第 2 層 semantic を有効化する場合のみ。配置済み）
```
2. `scripts/trace-check.sh` [2/5] のモデル探索パス:
```
修正前: models/all-MiniLM-L6-v2/model.onnx を探索
修正後: models/paraphrase-multilingual-MiniLM-L12-v2/model.onnx を探索（案内メッセージの URL も sentence-transformers/paraphrase-multilingual-MiniLM-L12-v2 に変更）
```

**根拠**: QSET-LGX-006 Q1 回答（選択肢 A）。LGX-COMPAT-001 §6 が既に「legixy 既定 = 多言語」と宣言済み、`.trace-engine.toml` は同モデルを採用済み、`models/paraphrase-multilingual-MiniLM-L12-v2/` は配置済み。不一致は運用スクリプト側の更新漏れ。

---

### 差分 2: REQ.11 への SPEC-LGX-010 相互参照と crate 名例示化（境界確定）

**対応 QSET 質問**: Q2（QSET-LGX-001 Q1/Q3 と連動）

**SPEC 修正前**（§3 REQ.11 の冒頭行と末尾）:

```
**内容:** legixy-embed は以下の bulk similarity API を公開する（CheckChecker / report コマンド / calibrate コマンド共通基盤）:
```

**SPEC 修正後**:

```
**内容:** legixy-embed（crate 名は例示であり DD で凍結、SPEC-LGX-001.REQ.03）は以下の bulk similarity API を公開する（legixy-check SemanticChecker〔SPEC-LGX-004.REQ.02〕/ `report`・`calibrate` コマンド〔SPEC-LGX-010.REQ.04/REQ.05〕の共通基盤）:
```

また REQ.11 本文末尾の「戻り値は SCORE-INV-1（決定性）を保証する順序で返却する。」の行の直後（`**根拠:**` 行の直前）に以下を追加:

```
**consumer 側の仕様所在（前段ループ反復 1 で確定）:** 本 API を消費するコマンド群の出力仕様・引数・終了コードは SPEC-LGX-010（embedding 運用・監査）が規定する。`calibrate --recommend` の推奨閾値ロジック（p25 / 1.0−p90 / p75 のパーセンタイル方式）も SPEC-LGX-010.REQ.05 に正準定義がある。本 SPEC はエンジン（生成・検出・bulk API）に責務を限定する。
```

**根拠**: QSET-LGX-006 Q2 回答（SPEC オーナー = SPEC-LGX-010）、QSET-LGX-001 Q3 回答（crate 名例示化の個別 SPEC 側注記）。

---

### 差分 3: 版数ヘッダの整合と REQ 採番順の物理整列（メタデータ整合）

**対応 QSET 質問**: Q3

**(a) ヘッダ Version**: Q3 回答の 2 段階を 1 回の表記更新で適用する — (1) Q3 回答どおりヘッダ（0.3.1）を変更履歴の最新（0.4.0、2026-04-28）に訂正し、(2) その上で本 SPP 反映分の改訂として 0.5.0 へ進める。結果としてヘッダ表記は 0.5.0 となる（0.4.0 は経過状態のため表記しない。変更履歴の 0.5.0 行に両段階を記録する）:

```
修正前: | Version | 0.3.1 |
修正後: | Version | 0.5.0 |
```

**(b) §3 の REQ 物理順序**: 現状 `…REQ.09 → REQ.12 → REQ.10 → REQ.11` と乱れている節順を、**REQ-id は一切変更せず**（下流参照の安定性優先、リナンバリング禁止）、物理順序のみ `REQ.09 → REQ.10 → REQ.11 → REQ.12` に並べ替える（節本文は無変更の移動のみ）。

**根拠**: QSET-LGX-006 Q3 回答（2026-06-07）。不整合の起源は v3 SPEC-TE-006 の変更履歴（0.4.0-draft の REQ.11 後付け新設）を引き写した際の取り込み漏れ。

---

### 差分 4: 次元不一致時の挙動確定（例外未定義の解消）

**対応 QSET 質問**: Q4

**SPEC 修正前**（§3 REQ.04 の末尾 2 行）:

```
しきい値は `.legixy.toml` の `[semantic]` セクションで設定。
**根拠:** v0.1.0 継承, SPEC-LGX-004.REQ.02
```

**SPEC 修正後**:

```
しきい値は `.legixy.toml` の `[semantic]` セクションで設定。

**次元不一致時の挙動（前段ループ反復 1 で確定）:** モデル切替等により次元数の異なる embedding が混在する場合（REQ.10 の全再生成が完了するまでの遷移期）、類似度計算は次元不一致ペアを skip し、**集約 Warning 1 件**（skip 件数 + `embed --all` 誘導。ペア毎ではなく集約）を報告する。【v3 差分】v3 は無言 skip（`crates/te-embed/src/similarity.rs:84-86` 等）であり、semantic 検証の静かな無効化 → check の偽 green を防ぐための可視化である。standalone `drift` の次元不一致は Error を維持する（SPEC-LGX-010.REQ.03。明示指定の対比は失敗を隠さない）。

**根拠:** v0.1.0 継承, SPEC-LGX-004.REQ.02、QSET-LGX-006 Q4 回答（2026-06-07）
```

また REQ.10 の末尾に 1 文を追加:

```
再生成完了までの遷移期における類似度計算の挙動（次元不一致ペアの skip + 集約 Warning）は REQ.04 を参照。
```

---

### 差分 5: 変更履歴（機械的）

§5 変更履歴に追加:

```
| 2026-06-07 | 0.5.0 | 前段ループ反復 1（QSET-LGX-006 回答 → SPP-LGX-006 承認）対応: ヘッダ Version の不整合（0.3.1 表記 vs 履歴 0.4.0）を解消し 0.5.0 へ。§3 の REQ 物理順序を ID 順（09→10→11→12）に整列（REQ-id 不変）。REQ.04 に次元不一致時の「skip + 集約 Warning【v3 差分】」を確定（drift の Error は SPEC-LGX-010 で維持）。REQ.11 に SPEC-LGX-010 相互参照と crate 名例示化注記を追加。既定モデルの運用整合（CLAUDE.md / trace-check.sh → paraphrase-multilingual-MiniLM-L12-v2）は SPEC 無変更の付随修正として実施 |
```

---

## 影響範囲

| 成果物 | 影響内容 | 再評価必要性 |
|---|---|---|
| CLAUDE.md / scripts/trace-check.sh | 差分 1 の運用整合修正（SPEC 外） | なし（付随変更） |
| SPEC-LGX-010 | 相互参照の対側は記載済み（REQ.05/REQ.08） | なし |
| SPEC-LGX-004 | REQ.02 の bulk API 参照訂正は SPP-LGX-004 差分 2 が対応 | あり（SPP-LGX-004） |
| 下流成果物の REQ-id 引用 | リナンバリングを行わないため影響なし（物理順序のみ変更） | なし |
| TP / GAP / RBA 以降 | 未生成のため影響なし | なし |

## 承認手順 / 却下時の手順

SPP-LGX-001 と同一（承認 → SPEC 反映 + 運用ファイル修正 → FCR-LGX-006 発行。却下 → 次の空き連番で QSET 再発行）。
