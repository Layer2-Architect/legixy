Document ID: GAP-LGX-162

# GAP-LGX-162: サイズ超過・エラー応答時の `_meta` 付与可否が未定義

**親 TP**: TP-LGX-009
**観点出典**: TP-LGX-009 §2.3 観点 19（エラーハンドリング）
**ステータス**: closed (2026-06-10)
**起票日**: 2026-06-08
**敵対的精査（2026-06-09）**: minor (verification: low-value, 人間判断で drop 可)。`_meta["anthropic/maxResultSizeChars"]` は永続化ヒントであり、`isError:true` の短いエラー本文に付与しても意味的には no-op。サイズ超過エラー自体の本文形式は cache spec §4.3 で確定、転送は REQ.07 で確定済み。残る「エラー時 `_meta` を付けるか」は実装が一意に選べばよい分岐で SPEC レベルの決定相関性は低い。

## 1. 観点

`compile_context` が Rust CLI の非ゼロ終了（特に CACHE-INV-3 の 500,000 文字超過エラー, SPEC-LGX-003 REQ.13）を `isError: true` で返すとき、返却ペイロードに `_meta["anthropic/maxResultSizeChars"]` を付与するか否かが未定義。成功時は REQ.13 で付与が確定しているが、エラー時の扱いが規定にない。

## 2. 現状の SPEC

SPEC-LGX-009 §3 REQ.13 は **成功返却への `_meta` 付与（compile_context / get_compile_audit 適用）** に触れているが、**エラー応答（`isError: true`）のペイロードに `_meta` を付与するか** は未定義。REQ.07 はエラー本文の形式（`Rust CLI failed (exit N):`）を規定するが `_meta` の有無に言及しない。

## 3. 期待される情報

SPEC に追加されるべき記述:

- エラー応答（`isError: true`）時に `_meta` を付与するか／しないかの明示
- 付与する場合、エラー本文（短い）に対し `maxResultSizeChars` を宣言する意味があるかの根拠
- 付与しない場合、成功/失敗で応答構造が分岐することの規定（実装の分岐点として）

## 4. 影響範囲

この GAP がクローズされないと以下に影響:

- UC-LGX-002 / UC-LGX-004（MCP 経由）: サイズ超過エラー時の Agent 受信ペイロード形が不定
- 下流の DD/TS: `_meta` 付与ロジックの分岐条件が書けず、CACHE-INV-4 の検証ケースに穴
- 関連: GAP-LGX-163（部分出力との合成）

## 5. 解決経緯（クローズ時に記入）

SPEC-LGX-009 v0.6.0（人間裁定 fix・承認 2026-06-10）: REQ.13 にエラー応答（isError: true）への _meta 非付与を確定（v3 実測: compile-context.ts は成功経路のみ付与）。成功/失敗で応答構造が分岐することを正準化。

## 6. 関連 ADR

なし
