# Document ID: SPP-LGX-007

**親 QSET**: QSET-LGX-007
**対象 SPEC**: SPEC-LGX-007（v0.3.0 → v0.4.0）
**作成日**: 2026-06-07
**作成者**: AI (designer)
**承認状態**: 承認済（2026-06-07 by 開発者。一括承認 — QSET 対応分として全差分を承認）

---

## 概要

QSET-LGX-007 への開発者回答（2026-06-07 確定）を反映した SPEC 差分案。重複排除キーの正準定義（Q1、v3 実測の正準化 — 既存記述「content_hash 等」は実装と乖離していたため訂正）と、observe の CATEGORY 列挙値の凍結 + CLI 層検証（Q2、【v3 差分】を含む）。

**ハードルール 1**: 本 SPP は人間が承認するまで SPEC に反映されない。

---

## 差分一覧

### 差分 1: observation 重複排除キーの正準定義（用語定義の確定）

**対応 QSET 質問**: Q1

**SPEC 修正前**（§3 REQ.11 の箇条書き 2 行目）:

```
- 重複排除は content_hash 等のキーで識別
```

**SPEC 修正後**:

```
- 重複排除キーの正準定義（前段ループ反復 1 で確定、v3 実測の正準化）: `(category, related_ids)` の複合一意キー。related_ids は**昇順ソート後に JSON 文字列化**して比較する。適用範囲は `status IN ('pending', 'analyzing')` のみ（解決済み observation は同一キーで再観測可能）
- **message は重複排除キーに含まれない**（同一 category + 同一 related_ids であれば異なる message でも重複扱い）。「同一対象への同種観測の重複蓄積を防ぐ」v3 の設計を意図的選択として維持する
```

**根拠**: QSET-LGX-007 Q1 回答（2026-06-07）。v3 実測（`crates/te-db/src/schema.rs:83-85` の partial UNIQUE INDEX、`crates/te-feedback/src/recorder.rs:35-98`）。修正前の「content_hash 等」は実装と乖離した記述（content_hash は embeddings テーブルの概念で observations には存在しない）であり訂正する。

---

### 差分 2: proposal semantic_key の正準定義（用語定義の確定）

**対応 QSET 質問**: Q1

**SPEC 修正前**（§3 REQ.09）:

```
### SPEC-LGX-007.REQ.09: proposals テーブル

**内容:** proposals テーブルは v0.1.0 スキーマを継承。status（pending / approved / rejected）、approved_by、approved_at、reject_reason 等を持つ。
**根拠:** v0.1.0 継承
**検証方法:** DB スキーマ検証
```

**SPEC 修正後**:

```
### SPEC-LGX-007.REQ.09: proposals テーブル

**内容:** proposals テーブルは v0.1.0 スキーマを継承。status（pending / approved / rejected）、approved_by、approved_at、reject_reason 等を持つ。

**semantic_key の正準定義（前段ループ反復 1 で確定、FB-INV-5 の実装キー）:** proposal の重複排除は kind 別の文字列キーで判定する:
- `add_chain_entry:{missing_id}`
- `add_link:{from_id}:{to_id}`（ID ペアは辞書順ソート済み）
- `update_doc:{changed_id}`

`status = 'pending'` の既存 proposal と semantic_key が一致する場合、新規 INSERT を抑止する。

**根拠:** v0.1.0 継承、QSET-LGX-007 Q1 回答（2026-06-07。v3 実測 `crates/te-feedback/src/analyzer.rs:275-312, 167-181` の正準化）
**検証方法:** DB スキーマ検証、同一 semantic_key の重複 analyze で INSERT 抑止を確認するテスト
```

---

### 差分 3: CATEGORY 列挙値の凍結と CLI 層検証（境界確定）

**対応 QSET 質問**: Q2

**SPEC 修正前**（§3 REQ.01）:

```
### SPEC-LGX-007.REQ.01: observation（Agent Surface）

**内容:** Claude Code 等の Agent は `observe` MCP ツールで気づき（ガイドラインの不足、見落とし、矛盾等）を記録できる。
- 入力: 気づきのテキスト、関連ノード ID（任意）
- 格納先: engine.db の `observations` テーブル

**根拠:** LEGIXY-SPEC-001 §2, CLAUDE.md MCP ツール使用ルール
**検証方法:** MCP スキーマテスト
```

**SPEC 修正後**:

```
### SPEC-LGX-007.REQ.01: observation（Agent Surface）

**内容:** Claude Code 等の Agent は `observe` MCP ツールで気づき（ガイドラインの不足、見落とし、矛盾等）を記録できる。
- 入力: category（必須・位置引数）、気づきのテキスト（message、必須・位置引数）、関連ノード ID（任意）
- 格納先: engine.db の `observations` テーブル

**CATEGORY 列挙値（前段ループ反復 1 で凍結）:** category は以下の 3 値に凍結する（LGX-COMPAT-001 §4.1 の凍結契約と整合）:
- `compile_miss` / `review_correction` / `manual_note`

将来の category 追加は MCP zod スキーマ変更（= 凍結境界の変更）を伴うため、次バージョンの SPEC 改訂として扱う（ハードルール 7）。

**検証の層:** MCP 層（zod enum、SPEC-LGX-009）と CLI 層の双方で 3 値を検証する。CLI 層は引数パーサの値域検証（ValueEnum 相当）として実装し、不正値は使用法誤り（exit 2）で reject する。【v3 差分】v3 の CLI 経路は無検証（`category: String`）で、不正 category は保存後 analyze で「その他 → skipped」となり observation が死蔵されていた。正当な 3 値の挙動は完全不変であり、契約文書（COMPAT §4.1）が列挙を明記済みのため互換破壊とみなさない（QSET-LGX-007 Q2 回答 2026-06-07）。

**根拠:** LEGIXY-SPEC-001 §2, CLAUDE.md MCP ツール使用ルール、LGX-COMPAT-001 §4.1
**検証方法:** MCP スキーマテスト、CLI 不正 category の exit 2 テスト、正当 3 値の受理テスト
```

---

### 差分 4: バージョンと変更履歴（機械的）

```
ヘッダ表: | Version | 0.3.0 | → | Version | 0.4.0 |
```

§6 変更履歴に追加:

```
| 2026-06-07 | 0.4.0 | 前段ループ反復 1（QSET-LGX-007 回答 → SPP-LGX-007 承認）対応: REQ.11 の重複排除キーを正準定義（(category, related_ids 昇順 JSON) 複合キー・pending/analyzing 限定・message 非包含。「content_hash 等」の乖離記述を訂正）。REQ.09 に proposal semantic_key の kind 別正準定義を追加。REQ.01 に CATEGORY 3 値凍結と CLI 層検証（不正値 exit 2【v3 差分】）を明記 |
```

---

## 影響範囲

| 成果物 | 影響内容 | 再評価必要性 |
|---|---|---|
| SPEC-LGX-009 | MCP zod enum は既存のまま（変更なし）。CLI 検証は Rust 側実装 | なし |
| DD（将来） | observations / proposals の重複排除実装が正準定義に拘束される | あり（DD フェーズ） |
| TP / GAP / RBA 以降 | 未生成のため影響なし | なし |

## 承認手順 / 却下時の手順

SPP-LGX-001 と同一（承認 → SPEC 反映 → FCR-LGX-007 発行。却下 → 次の空き連番で QSET 再発行）。
