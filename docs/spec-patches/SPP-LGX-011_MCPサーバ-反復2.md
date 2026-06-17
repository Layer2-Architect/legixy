# Document ID: SPP-LGX-011

**親 QSET**: QSET-LGX-011
**対象 SPEC**: SPEC-LGX-009
**作成日**: 2026-06-07
**作成者**: AI (designer)
**承認状態**: 承認済（2026-06-07 by 開発者。差分 1・2 を一括承認）

---

## 概要

QSET-LGX-011 への開発者回答（2026-06-07 確定、選択肢 A = v3 実在形式の正準化）を反映した SPEC 差分案。前段ループ反復 2。

SPEC-LGX-009 v0.5.0 REQ.07 が虚偽の根拠（「v3 は exit code を区別不能だった」）に基づいて規定した `[exit N]` 新フォーマットを撤回し、v3 実在形式 `Rust CLI failed (exit N): <stderr 本文>` の正準化に差し替える。実装変更は不要となる（v3 の ts-mcp 挙動をそのまま契約化）。

**ハードルール 1**: 本 SPP は人間が承認するまで SPEC に反映されない。AI は提案する、人間が決定する。

---

## 差分一覧

### 差分 1: REQ.07 の exit code 区別ブロックの差し替え（矛盾解消）

**対応 QSET 質問**: QSET-LGX-011 Q1

**SPEC 修正前**（§3 REQ.07、v0.5.0）:

> **exit code の区別（前段ループ反復 1 で確定、【v3 差分】）:** エラー応答のメッセージ先頭に exit code を `[exit N]` 形式で含める（例: `[exit 2] <stderr 本文>`）。これにより Agent は「検証 Error（exit 1 → 成果物を修正すべき）」と「呼び出しミス（exit 2 → 自らの引数を修正すべき）」を判別し、誤った修正ループを回避できる。
> - v3 実測: exitCode を内部（RustCliError）に保持しながら、応答は一律 `isError: true` + stderr 本文のみで区別不能だった（`ts-mcp/src/engine.ts:54-78`）
> - 互換性: MCP 応答本文は凍結対象 (a)〜(f) 外。stderr 本文の忠実転送（MCP-INV-2）は `[exit N]` プレフィクスの後に原文のまま維持される
> - `[exit N]` のフォーマットは本 REQ で固定し、Agent 側パースの安定性を保証する
>
> **根拠:** NFR-LGX-001.USE.02, MCP プロトコル、QSET-LGX-009 Q2 回答（2026-06-07）、SPEC-LGX-004 REQ.04（終了コード 3 値）
> **検証方法:** エラーシナリオテスト（exit 1 / exit 2 それぞれの `[exit N]` プレフィクス検証）

**SPEC 修正後**:

> **exit code の区別（前段ループ反復 2 で訂正確定、v3 既存挙動の正準化）:** エラー応答のメッセージ先頭に exit code を `Rust CLI failed (exit N): <stderr 本文>` 形式（v3 実在形式）で含める。これにより Agent は「検証 Error（exit 1 → 成果物を修正すべき）」と「呼び出しミス（exit 2 → 自らの引数を修正すべき）」を判別し、誤った修正ループを回避できる。
> - v3 実測: 全 3 MCP ツールが既に本形式で exit code を出力している（`ts-mcp/src/tools/compile-context.ts:98`、`observe.ts:96`、`get-compile-audit.ts:60`。exitCode は `engine.ts:54-78` の `RustCliError` が保持）。本 REQ はその正準化であり実装変更を要しない。反復 1 の「v3 は区別不能」という根拠記述は誤認であり本反復で訂正（QSET-LGX-011 Q1）
> - プロセス起動不能・シグナル等で数値 exit code が無い場合は v3 同様 `exit -1` として同形式で報告する（`engine.ts:67` のフォールバック）
> - 互換性: MCP 応答本文は凍結対象 (a)〜(f) 外だが、本形式の維持により v3 からの観測可能差分も生じない。stderr 本文の忠実転送（MCP-INV-2）はプレフィクスの後に原文のまま維持される
> - 本フォーマットは本 REQ で固定し、Agent 側パースの安定性を保証する
>
> **根拠:** NFR-LGX-001.USE.02, MCP プロトコル、QSET-LGX-009 Q2 回答（2026-06-07 訂正版）、QSET-LGX-011 Q1 回答、SPEC-LGX-004 REQ.04（終了コード 3 値）
> **検証方法:** エラーシナリオテスト（exit 1 / exit 2 それぞれの `Rust CLI failed (exit N):` プレフィクス検証）

**影響**: ts-mcp の実装要件から「エラー整形の変更」が消える（v3 コードのまま REQ.07 を満たす）。SPP-LGX-009 の波及分析表「ts-mcp（将来 SRC）: エラー整形の 1 行変更」は本差分により無効化される。

### 差分 2: ヘッダ Version と変更履歴

- ヘッダ表 `| Version | 0.5.0 |` → `| Version | 0.5.1 |`
- §5 変更履歴に追記:

> | 2026-06-07 | 0.5.1 | 前段ループ反復 2（QSET-LGX-011 回答 → SPP-LGX-011 承認）対応: REQ.07 の根拠記述の事実誤認（「v3 は exit code を区別不能」→ 実際は全 3 ツールが `Rust CLI failed (exit N):` 形式で出力済み）を訂正し、`[exit N]` 新フォーマット【v3 差分】を撤回。v3 実在形式の正準化に差し替え（実装変更不要化） |

---

## 波及分析

| 対象 | 影響 | 対応要否 |
|---|---|---|
| SPP-LGX-009 | 履歴として保存（書き換えない）。本 SPP が差分 2 を上書き訂正 | なし（03a §7 規律） |
| FCR-LGX-009 | v0.5.0 に対する ACCEPTED として履歴保存。v0.5.1 反映後に FCR-LGX-011 を新規発行し、SPEC-LGX-009 の「現在の状態」は FCR-LGX-011 が引き継ぐ | あり（FCR-LGX-011 発行） |
| SPEC-LGX-004 / LGX-COMPAT-001 | exit 2 グローバル規約（COMPAT v1.0.1）は本訂正と独立に有効 | なし |
| ts-mcp（将来 SRC） | エラー整形の実装変更が不要になる（v3 挙動のまま適合） | なし（要件の削減） |
| UC / TP[SPEC]（未着手） | 着手前の訂正のため下流影響なし | なし |

## 検証記録

- v3 実装の実物照合（2026-06-07）: `ts-mcp/src/tools/compile-context.ts:98`、`observe.ts:96`、`get-compile-audit.ts:60` の 3 箇所全てで `Rust CLI failed (exit ${err.exitCode}): ${err.message}` を確認。`engine.ts:54-78` の `toRustCliError` が数値 exit code を保持し、非数値（ENOENT 等）は `-1` にフォールバックすることを確認。
