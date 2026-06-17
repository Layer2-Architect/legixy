# Document ID: QSET-LGX-003

**親 SPEC**: SPEC-LGX-003
**反復回数**: 1
**作成日**: 2026-06-04
**作成者**: AI (designer)

---

## 概要

このドキュメントは前段ループの反復 1 回目で発行された質問票である。SPEC-LGX-003（コンテキスト解決）に対してフロントエンド検査器が検出した用語定義不足・例外未定義・組合せ境界不明を、開発者が回答可能な形に変換したもの。

---

## Q1: 用語定義 / 矛盾 — 「文字」のカウント単位（重大）

**質問**: REQ.13 は「返却本文が **500,000 文字** を超える場合エラー」とし、REQ.14 は「バイト単位で同一」を求め、SPEC-LGX-009 REQ.13 は `_meta["anthropic/maxResultSizeChars"] = 500000` を付与します。この **「文字」のカウント単位** が未定義です。日本語成果物では文字数とバイト数が大きく乖離するため、しきい値判定が単位により最大 3 倍変わります。どの単位ですか?

**SPEC 上の該当箇所**: SPEC-LGX-003 §3 REQ.13, REQ.14、SPEC-LGX-009 REQ.13

**選択肢**:

- [x] 選択肢 A: **Unicode コードポイント数**（`.chars().count()` 相当）
- [ ] 選択肢 B: **UTF-8 バイト数**（`.len()` 相当。CACHE-INV-1 の「バイト単位決定論」と整合）
- [ ] 選択肢 C: **UTF-16 コードユニット数**（`maxResultSizeChars` の "Chars" を Claude 側 UTF-16 換算と解釈）
- [ ] その他: <自由記述>

**回答**:

**選択肢 A（Unicode コードポイント数）を採用**（2026-06-07 開発者決定・AI 起草）。

- 根拠: v3 実測 — `te-ctx/src/section_formatter.rs:129-144` の `enforce_size_limit` が `rendered.chars().count()` で判定（= Unicode スカラ値単位）。既存挙動の正準化。
- REQ.14 の「バイト単位で同一」は**出力の決定論**（同一入力 → 同一バイト列）の要求であり、サイズ判定のカウント単位とは独立。矛盾しない旨を REQ.13/14 に注記する。
- SPEC-LGX-009 REQ.13 の `maxResultSizeChars` も同一単位（コードポイント）と明記する（QSET-LGX-009 Q1 と連動、同一決定）。

---

## Q2: 例外未定義 / 境界 — 「未定義」と明記された 2 挙動の確定

**質問**: 以下 2 つが SPEC 本文で「挙動は未定義」と明記されています。凍結済み境界契約の一部であるため、UC/DD を確定する前に形式仕様として動作を確定すべきと考えます。それぞれどう確定しますか?

1. REQ.16: `--sections` に **親ドキュメントノード ID**（`#` を含まない ID）を指定した場合
2. REQ.17: `--depth 0` を指定した場合（現状「CLI 入力検証に委ね、内部的には walker が空集合」）

**SPEC 上の該当箇所**: SPEC-LGX-003 §3 REQ.16, REQ.17

**選択肢**（各々について）:

- [ ] 選択肢 A: CLI 層で **入力エラー（exit 2）** として reject する
- [x] 選択肢 B: **空集合 / 無視**として正常終了させる（現状の暗黙挙動を正式化）
- [ ] その他: <自由記述>

**回答**:

**両ケースとも選択肢 B（空集合/無視で正常終了）を正式化**（2026-06-07 開発者決定・AI 起草）。

- 根拠: v3 実測 — (1) `--sections` に親 ID: sections フィルタは subnode id とのみ照合するため何にもマッチしない（`te-ctx/src/compiler.rs:235-238`）。(2) `--depth 0`: walker が `depth >= limit` で全 continue し空集合（`te-ctx/src/upstream_walker.rs:49-55`）。exit 2 reject に変えると v3 で成功していた呼び出しが失敗する互換破壊。
- MCP 層は zod `depth: min1` により depth 0 を既に reject しており、これも現状維持（CLI と MCP で受理範囲が異なることを SPEC に注記）。
- **上積み（v3 差分）**: 両ケースで stderr に Info 診断を追加する（例:「`--sections` に親ドキュメント ID が指定されました。サブノード ID（`#` 付き）を指定してください」）。出力本文・終了コードは不変のため互換安全。

---

## Q3: 境界不明 — フラグ組合せの優先順位マトリクス

**質問**: `--granularity`（document/subnode）, `--outline-only`, `--sections`, `--depth` は相互に組合せ可能ですが、組合せ時の出力が一部不明確です。特に:

- `--outline-only --granularity document`: 各 artifact は「見出し階層リスト」か「全文」か?（REQ.15 は subnode 組合せ時の挙動のみ明記）
- `--sections --granularity document`: REQ.16 は「document 粒度時は無視」と明記 → これは確定。
- `--outline-only --sections`: outline 化と section 抽出の適用順は?

後段（UC-LGX-004 粒度別テスト、DD の walker 制御）に組合せマトリクスの確定が必要です。

**SPEC 上の該当箇所**: SPEC-LGX-003 §3 REQ.03, REQ.15, REQ.16, REQ.17

**回答**:

（2026-06-07 開発者決定・AI 起草）v3 実測に基づき以下のマトリクスで確定し、REQ.15/16/17 に表形式で追記する:

| 組合せ | 確定挙動 | v3 根拠 |
|---|---|---|
| `--outline-only --granularity document` | 各 artifact の本文を**見出し階層リストに置換**して返す（全文ではない） | `te-ctx/src/compiler.rs:201-202`（build_outline） |
| `--sections --granularity document` | `--sections` は**無視**（REQ.16 既定どおり確定） | REQ.16 明記済み |
| `--outline-only --sections`（subnode 粒度） | **sections フィルタが先、outline 化が後**。sections で絞り込んだ後、残った subnode の body を outline/anchor に置換 | `te-ctx/src/compiler.rs:234-253` |
| `--depth` × 上記 | 直交（walker の探索深度のみ制御し、粒度・outline と干渉しない） | `upstream_walker.rs` |

---

## 検出元検査の集計

| 検査カテゴリ | 検出件数 |
|---|---|
| 未定義語 | 1 |
| 複数解釈 | 0 |
| 例外未定義 | 1 |
| 境界不明 | 1 |
| 矛盾 | 1 |
| 非機能不足 | 0 |
| 合計 | 3 |（Q1 は用語定義 + 矛盾の両面）

## メモ

- Q1 は SPEC-LGX-009 QSET Q1 と連動する。回答は両 SPEC に反映する。
- 回答が確定したら SPP-LGX-003 として SPEC 差分案を発行する。
