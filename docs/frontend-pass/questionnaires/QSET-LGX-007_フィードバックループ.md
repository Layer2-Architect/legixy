# Document ID: QSET-LGX-007

**親 SPEC**: SPEC-LGX-007
**反復回数**: 1
**作成日**: 2026-06-04
**作成者**: AI (designer)

---

## 概要

このドキュメントは前段ループの反復 1 回目で発行された質問票である。SPEC-LGX-007（フィードバックループ）に対してフロントエンド検査器が検出した用語定義不足・境界不明を、開発者が回答可能な形に変換したもの。

---

## Q1: 用語定義 — 重複排除キーの正準定義

**質問**: REQ.11 は observation の重複排除を「content_hash 等のキーで識別」、FB-INV-5 は proposal の重複排除を「semantic_key 相当のキー」と抽象的に記述しています。これらのキーの**正準定義**が未確定です。DD で重複排除ロジックを実装するために、以下を確定してください:

- observation 重複排除キー: 何のハッシュか?（例: `category + message` / `+ related_node_id` / `+ target_file`）
- proposal 重複排除キー（semantic_key）: 何で同一性を判定するか?（例: 対象ノード ID + 提案種別）

**SPEC 上の該当箇所**: SPEC-LGX-007 §3 REQ.11、§4 FB-INV-1, FB-INV-5

**回答**:

（2026-06-07 開発者決定・AI 起草）v3 実装を正準定義として SPEC に明記する:

- **observation 重複排除キー** = `(category, related_ids)` の複合一意制約。related_ids は**昇順ソート後 JSON 文字列化**して比較。適用範囲は `status IN ('pending', 'analyzing')` のみ（解決済み observation は再観測可能）。根拠: `te-db/src/schema.rs:83-85` の partial UNIQUE INDEX、`te-feedback/src/recorder.rs:35-98`。
  - **注意（意図的選択として明記）**: message はキーに**含まれない**。同一 category + 同一 related_ids なら異なる message でも重複扱いとなる。これは「同一対象への同種観測の重複蓄積を防ぐ」v3 の設計であり、legixy でも維持する。
  - REQ.11 の「content_hash 等のキー」という表現は実装と乖離しているため「(category, related_ids) 複合キー」に修正する（content_hash は embeddings テーブルの概念で observation には存在しない）。
- **proposal 重複排除キー（semantic_key）** = kind 別の文字列: `add_chain_entry:{missing_id}` / `add_link:{source_id}:{target_id}`（ソート済みペア）/ `update_doc:{changed_id}`。`status='pending'` の既存と一致時は INSERT 抑止（FB-INV-5）。根拠: `te-feedback/src/analyzer.rs:275-312, 167-181`。

---

## Q2: 境界不明 — observe の category 列挙値を SPEC で凍結するか

**質問**: `observe` の CATEGORY 列挙値（`compile_miss` / `review_correction` / `manual_note`）は LGX-COMPAT-001 §4.1 と MCP zod スキーマには定義されていますが、本 SPEC REQ.01 には列挙されていません。この列挙値は凍結済み境界契約の一部として SPEC-007 にも明記し凍結しますか? それとも将来の category 追加余地を残しますか?（MCP zod の enum 制約と RBD の入力検証に影響）

**SPEC 上の該当箇所**: SPEC-LGX-007 §3 REQ.01、LGX-COMPAT-001 §4.1

**回答**:

（2026-06-07 開発者決定・AI 起草）

**SPEC-007 にも 3 値を明記して凍結する**。

- CATEGORY 列挙値（`compile_miss` / `review_correction` / `manual_note`）は MCP zod enum として既に凍結済み境界契約の一部（LGX-COMPAT-001 §4.1）。SPEC-007 REQ.01 に列挙を明記して契約と整合させる。
- 将来の category 追加は MCP スキーマ変更（= 凍結境界の変更）を必然的に伴うため、いずれにせよ次バージョンの SPEC 改訂として扱われる（ハードルール 7）。「SPEC に書かず追加余地を残す」ことに実益はなく、未記載はかえって RBD 入力検証の根拠を欠く。
- 追記（2026-06-07、整合性・堅牢性・実現性の 3 軸比較で採用確定、**v3 差分**）: **CLI 層でも 3 値を検証し、不正値は exit 2（使用法誤り）で reject する**。
  - v3 実態: CLI 経路は `category: String` で無検証（`te-cli/src/main.rs:197`、recorder まで検証なし）。不正 category は保存後 analyze で「その他 → skipped」（`te-feedback/src/analyzer.rs:270`）となり observation が**死蔵**される（フィードバックループの黙殺）。3 値を強制するのは MCP zod 経由のみで、契約文書（LGX-COMPAT-001 §4.1 の列挙明記）と CLI 実装が乖離していた。
  - 互換性判定: 正当な入力空間（3 値）の挙動は完全不変。変わるのは契約外の不正入力のみ（exit 0+死蔵 → exit 2 fail fast）であり互換破壊とみなさない。この判定基準を COMPAT 改訂時に注記する。MCP zod との二重検証で層間整合も回復する。

---

## 検出元検査の集計

| 検査カテゴリ | 検出件数 |
|---|---|
| 未定義語 | 1 |
| 複数解釈 | 0 |
| 例外未定義 | 0 |
| 境界不明 | 1 |
| 矛盾 | 0 |
| 非機能不足 | 0 |
| 合計 | 2 |

## メモ

- 回答が確定したら SPP-LGX-007 として SPEC 差分案を発行する。
