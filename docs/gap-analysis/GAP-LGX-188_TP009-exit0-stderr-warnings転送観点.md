Document ID: GAP-LGX-188

# GAP-LGX-188: TP-LGX-009 への観点追加（exit 0 時の非空 stderr → `_meta["legixy/warnings"]` 転送）

**親 TP**: TP-LGX-009
**観点出典**: SPEC-LGX-009.REQ.03 新 bullet（spec-change 2026-06-12）
**ステータス**: closed (2026-06-12)
**起票日**: 2026-06-12
**起票理由**: spec-change 2026-06-12（ADR-LGX-004 可観測性強化）により SPEC-LGX-009.REQ.03 に新要件が追加されたが、TP-LGX-009 に対応観点が存在しない

## 1. 観点

SPEC-LGX-009.REQ.03 に以下の新 bullet が追加された（spec-change 2026-06-12）:

> Rust CLI が exit 0 で終了しても stderr が非空の場合、MCP サーバは成功応答の `_meta["legixy/warnings"]` フィールドに stderr 本文を格納して Agent に転送する。

TP-LGX-009 の既存 46 観点はすべて GREEN（2026-06-10）だが、この新要件をカバーする観点が存在しない。

## 2. 現状の SPEC / TP

- SPEC-LGX-009.REQ.03 の `_meta["legixy/warnings"]` 転送は spec-change 2026-06-12 で新設。
- TP-LGX-009 §2.8「永続化（CACHE-INV-4 / `_meta` 非改変）」の観点 40〜42 は `_meta["anthropic/maxResultSizeChars"]` を対象としており、`_meta["legixy/warnings"]` は対象外。
- 既存の観点 5「忠実転送（MCP-INV-2）」も exit 0 stderr の `_meta` 変換は対象外。

## 3. 推奨対応

TP-LGX-009 に以下の観点を追加し、対応する RED 判定行を追記する:

**観点 47（§2.11 可観測性）**:
- (a) Rust CLI が exit 0 + stderr 非空の場合、MCP 成功応答の `_meta["legixy/warnings"]` に stderr 本文が格納されること
- (b) stderr が空の場合、`_meta["legixy/warnings"]` フィールドが省略される（空文字列でなくフィールド自体が存在しない）こと
- (c) 適用対象ツールの明示: `compile_context` のみか `get_compile_audit` / `observe` も含むか（REQ.13 表の対象範囲との整合）

(c) は SPEC.REQ.03 の「例: context_log 書込失敗」が `compile_context` 起点の Warning を指しており、他ツールへの適用可否が未明記。TP 対応前に REQ.03 の適用対象を明確化するか、TP 観点として問いかけとして立てる。

## 4. 影響範囲

- TP-LGX-009: 観点 47 追加 → RED（本 GAP close 後に GREEN 化）
- TP ステータス: green → red（本 GAP open により変更済み）
