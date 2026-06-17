# Document ID: QSET-LGX-009

**親 SPEC**: SPEC-LGX-009
**反復回数**: 1
**作成日**: 2026-06-04
**作成者**: AI (designer)

---

## 概要

このドキュメントは前段ループの反復 1 回目で発行された質問票である。SPEC-LGX-009（MCP サーバ）に対してフロントエンド検査器が検出した用語整合・例外未定義を、開発者が回答可能な形に変換したもの。本 SPEC は転送層として比較的閉じており、検出は 2 件。

---

## Q1: 用語整合 / 責務 — maxResultSizeChars の単位とサイズ判定の所在

**質問**: REQ.13 は `_meta["anthropic/maxResultSizeChars"] = 500000` を付与します。この `Chars` の単位は SPEC-LGX-003 REQ.13 の「500,000 **文字**」と同一の概念のはずですが、その単位（コードポイント / UTF-8 バイト / UTF-16）が両 SPEC で未定義です（QSET-LGX-003 Q1 と連動）。あわせて、**500,000 超過のサイズ判定とエラー生成を行う層**は Rust CLI 側（SPEC-003 REQ.13）でよいですか? MCP サーバは `_meta` 付与のみで本文サイズ判定は行わない、という責務分担で確定しますか?

**SPEC 上の該当箇所**: SPEC-LGX-009 §3 REQ.13、SPEC-LGX-003 REQ.13

**回答**:

（2026-06-07 開発者決定・AI 起草）

- **単位は Unicode コードポイント数で確定**（QSET-LGX-003 Q1 と同一決定）。根拠: v3 の判定実装は `te-ctx/src/section_formatter.rs:129-144` の `.chars().count()`。
- **責務分担も本問の提案どおり確定**: 500,000 超過のサイズ判定とエラー生成は **Rust CLI 側**（SPEC-003 REQ.13、te-ctx 層）。MCP サーバは本文サイズ判定を行わず、`_meta["anthropic/maxResultSizeChars"] = 500000` の**宣言のみ**を行う。根拠: v3 実測 — `ts-mcp/src/tools/compile-context.ts:12, 90` はメタデータ付与のみで切り捨て・判定を実装していない。
- 両 SPEC（003/009）に単位と責務所在を明記する。

---

## Q2: 例外未定義 — Rust CLI の exit 1 と exit 2 の MCP 表現

**質問**: REQ.07 は Rust CLI の非ゼロ終了コードを MCP エラー応答として転送するとします。しかし SPEC-LGX-004 REQ.04（QSET-LGX-004 Q1 で確認中）では check が exit 1（Error 検出）と exit 2（使用法誤り）を区別しています。MCP エラー応答でこの 2 つを区別しますか?（例: exit code を error message に含める / `isError: true` に一律集約）。Agent 側が「検証 Error」と「呼び出しミス」を判別できるかに関わります。

**SPEC 上の該当箇所**: SPEC-LGX-009 §3 REQ.07、SPEC-LGX-004 REQ.04

**選択肢**:

- [x] 選択肢 A: exit code を MCP エラーメッセージ／構造化フィールドに含めて区別可能にする
- [ ] 選択肢 B: 一律 `isError: true` + stderr 転送のみ（区別しない）
- [ ] その他: <自由記述>

**回答**:

**選択肢 A を採用（v3 既存挙動の正準化）**（2026-06-07 開発者決定・AI 起草。同日レビューで事実関係を訂正: 当初「v3 は区別不能」とした前提は誤認であり、「改善上積み、v3 差分」ラベルを撤回）。

- v3 実態（訂正済み）: **全 3 MCP ツールが既にエラーメッセージへ exit code を含めている** — `Rust CLI failed (exit N): <stderr 本文>` 形式（`ts-mcp/src/tools/compile-context.ts:98`、`observe.ts:96`、`get-compile-audit.ts:60`。exitCode は `engine.ts:54-78` の `RustCliError` が保持。プロセス起動不能・シグナル等で数値 exit code が無い場合は `-1`）。
- 確定仕様: この v3 実在形式 `Rust CLI failed (exit N): <stderr 本文>` を SPEC-009 REQ.07 に明記して正準化する。当初案の新フォーマット `[exit N]` は不要な v3 差分となるため採用しない。`isError: true` と stderr 転送も v3 どおり維持。
- Agent が「検証 Error（exit 1）」と「呼び出しミス（exit 2 → 自らの引数を修正すべき）」を判別できる性質は v3 で既に成立しており、MCP-INV-2（忠実転送）と整合する。
- SPEC-LGX-004 REQ.04 の exit 2 確定（QSET-LGX-004 Q1 = 選択肢 A）と連動。

---

## 検出元検査の集計

| 検査カテゴリ | 検出件数 |
|---|---|
| 未定義語 | 1 |
| 複数解釈 | 0 |
| 例外未定義 | 1 |
| 境界不明 | 0 |
| 矛盾 | 0 |
| 非機能不足 | 0 |
| 合計 | 2 |

## メモ

- Q1 は SPEC-LGX-003 QSET Q1 と一体で回答する。
- 回答が確定したら SPP-LGX-009 として SPEC 差分案を発行する。
