# Document ID: SPP-LGX-009

**親 QSET**: QSET-LGX-009
**対象 SPEC**: SPEC-LGX-009（v0.4.0 → v0.5.0）
**作成日**: 2026-06-07
**作成者**: AI (designer)
**承認状態**: 承認済（2026-06-07 by 開発者。一括承認 — QSET 対応分として全差分を承認）

---

## 概要

QSET-LGX-009 への開発者回答（2026-06-07 確定）を反映した SPEC 差分案。maxResultSizeChars の単位とサイズ判定の責務所在の確定（Q1、QSET-LGX-003 Q1 と同一決定）、Rust CLI の exit 1/2 の MCP 表現（Q2、【v3 差分】）。

**ハードルール 1**: 本 SPP は人間が承認するまで SPEC に反映されない。

---

## 差分一覧

### 差分 1: maxResultSizeChars の単位とサイズ判定の責務所在（用語整合）

**対応 QSET 質問**: Q1（QSET-LGX-003 Q1 と連動 — SPP-LGX-003 差分 1 が対側）

**SPEC 修正前**（§3 REQ.13 の MCP-INV-2 整合段落の直前、適用対象表の直後）:

```
**MCP-INV-2 との整合:** `_meta` フィールドの付与は Rust CLI の出力本文（`content` フィールド）に対する変更ではなく、MCP プロトコルレベルでのメタデータ追加である。本文は従来通り改変なく転送される。したがって MCP-INV-2（忠実な転送）は維持される。
```

**SPEC 修正後**:

```
**単位と責務所在（前段ループ反復 1 で確定）:**
- `maxResultSizeChars` の単位は **Unicode コードポイント数**であり、SPEC-LGX-003 REQ.13 の「500,000 文字」と同一概念・同一単位である（QSET-LGX-009 Q1 / QSET-LGX-003 Q1 回答 2026-06-07。v3 実測 = Rust `.chars().count()`）
- 500,000 超過の**サイズ判定とエラー生成は Rust CLI 側**（SPEC-LGX-003 REQ.13）が行う。MCP サーバは本文サイズの判定・切り捨てを行わず、`_meta` の宣言のみを担う（v3 実測の正準化: `ts-mcp/src/tools/compile-context.ts` はメタデータ付与のみ）

**MCP-INV-2 との整合:** `_meta` フィールドの付与は Rust CLI の出力本文（`content` フィールド）に対する変更ではなく、MCP プロトコルレベルでのメタデータ追加である。本文は従来通り改変なく転送される。したがって MCP-INV-2（忠実な転送）は維持される。
```

---

### 差分 2: エラー転送における exit code の区別（例外未定義の解消）

**対応 QSET 質問**: Q2（QSET-LGX-004 Q1 の終了コード 3 値確定と連動）

**SPEC 修正前**（§3 REQ.07）:

```
### SPEC-LGX-009.REQ.07: エラー転送

**内容:** Rust CLI の非ゼロ終了コードは MCP エラー応答として転送する。stderr の内容も Agent が参照可能にする。
**根拠:** NFR-LGX-001.USE.02, MCP プロトコル
**検証方法:** エラーシナリオテスト
```

**SPEC 修正後**:

```
### SPEC-LGX-009.REQ.07: エラー転送

**内容:** Rust CLI の非ゼロ終了コードは MCP エラー応答（`isError: true`）として転送する。stderr の内容も Agent が参照可能にする。

**exit code の区別（前段ループ反復 1 で確定、【v3 差分】）:** エラー応答のメッセージ先頭に exit code を `[exit N]` 形式で含める（例: `[exit 2] <stderr 本文>`）。これにより Agent は「検証 Error（exit 1 → 成果物を修正すべき）」と「呼び出しミス（exit 2 → 自らの引数を修正すべき）」を判別し、誤った修正ループを回避できる。
- v3 実測: exitCode を内部（RustCliError）に保持しながら、応答は一律 `isError: true` + stderr 本文のみで区別不能だった（`ts-mcp/src/engine.ts:54-78`）
- 互換性: MCP 応答本文は凍結対象 (a)〜(f) 外。stderr 本文の忠実転送（MCP-INV-2）は `[exit N]` プレフィクスの後に原文のまま維持される
- `[exit N]` のフォーマットは本 REQ で固定し、Agent 側パースの安定性を保証する

**根拠:** NFR-LGX-001.USE.02, MCP プロトコル、QSET-LGX-009 Q2 回答（2026-06-07）、SPEC-LGX-004 REQ.04（終了コード 3 値）
**検証方法:** エラーシナリオテスト（exit 1 / exit 2 それぞれの `[exit N]` プレフィクス検証）
```

---

### 差分 3: バージョンと変更履歴（機械的）

```
ヘッダ表: | Version | 0.4.0 | → | Version | 0.5.0 |
```

§5 変更履歴に追加:

```
| 2026-06-07 | 0.5.0 | 前段ループ反復 1（QSET-LGX-009 回答 → SPP-LGX-009 承認）対応: REQ.13 に maxResultSizeChars の単位（Unicode コードポイント、SPEC-LGX-003 REQ.13 と同一）とサイズ判定の責務所在（Rust CLI 側判定、MCP は _meta 宣言のみ）を確定。REQ.07 にエラー応答への `[exit N]` プレフィクス埋め込み【v3 差分】を新設（Agent の検証 Error / 呼び出しミス判別） |
```

---

## 影響範囲

| 成果物 | 影響内容 | 再評価必要性 |
|---|---|---|
| SPEC-LGX-003 | 単位確定の対側は SPP-LGX-003 差分 1 が反映（同時承認が望ましい） | あり（SPP-LGX-003） |
| SPEC-LGX-004 | exit 2 のグローバル規約化は SPP-LGX-004 差分 1 が対応 | あり（SPP-LGX-004） |
| ts-mcp（将来 SRC） | エラー整形の 1 行変更（`[exit N]` プレフィクス）が実装要件に加わる | あり（DD/TS フェーズ） |
| TP / GAP / RBA 以降 | 未生成のため影響なし | なし |

## 承認手順 / 却下時の手順

SPP-LGX-001 と同一（承認 → SPEC 反映 → FCR-LGX-009 発行。却下 → 次の空き連番で QSET 再発行）。
