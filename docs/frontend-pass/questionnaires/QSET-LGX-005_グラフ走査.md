# Document ID: QSET-LGX-005

**親 SPEC**: SPEC-LGX-005
**反復回数**: 1
**作成日**: 2026-06-04
**作成者**: AI (designer)

---

## 概要

このドキュメントは前段ループの反復 1 回目で発行された質問票である。SPEC-LGX-005（グラフ走査）に対してフロントエンド検査器が検出した既定値の未定義・出力形式の境界不明を、開発者が回答可能な形に変換したもの。本 SPEC は比較的閉じており、検出は 2 件にとどまる。

---

## Q1: 例外未定義 / 既定値 — max_depth 省略時の挙動

**質問**: REQ.04 は `max_depth` を「指定できる」とするのみで **省略時の既定値が未定義** です。LGX-COMPAT-001 §4 #11/#12 も `[--max-depth <N>]` を任意とするだけです。`impact` / `investigate` を `--max-depth` なしで実行した場合の挙動を確定してください（CLI E2E テストと DD の BFS 制御に必須）。

**SPEC 上の該当箇所**: SPEC-LGX-005 §3 REQ.04, REQ.07

**選択肢**:

- [x] 選択肢 A: **無制限**（到達可能な全ノードを返す）
- [ ] 選択肢 B: 既定値 N（具体値を指定。例: 7 = SPEC→...→SRC の全 chain 段数）
- [ ] その他: <自由記述>

**回答**:

**選択肢 A（無制限）を採用**（2026-06-07 開発者決定・AI 起草）。

- 根拠: v3 実測 — `max_depth: Option<usize>` で既定 `None` = 無制限（`te-nav/src/impact.rs:14-21`、BFS 制御は `te-graph/src/traversal.rs:54-58` で `Some(limit)` 時のみ打切り）。省略時挙動は互換契約 (d) 既定値の一部とみなし維持する。
- REQ.04 に「省略時は到達可能な全ノードを返す（グラフは DAG 保証があるため有限・停止保証あり）」と明記する。

---

## Q2: 境界不明 — 走査結果の出力フォーマット

**質問**: REQ.09 は走査結果に「ID + タイプ + パス、走査順、エッジ情報、深度」を含むとしますが、**出力フォーマット**が未定義です。SPEC-LGX-004 REQ.08 の check は `--log-format=json` で JSON Lines 出力に対応します。`impact` / `investigate` も同様に機械可読出力（JSON）に対応しますか、それとも人間可読のみですか? Admin Surface 全体の出力一貫性に関わります。

**SPEC 上の該当箇所**: SPEC-LGX-005 §3 REQ.07, REQ.09、SPEC-LGX-004 REQ.08

**回答**:

（2026-06-07 開発者決定・AI 起草）

**JSON 出力対応を追加する（改善上積み、v3 差分）**。

- v3 実態: impact/investigate はグローバル `--json` を**受理するが無視**し Text 固定（`te-cli/src/commands/impact.rs:21`、`main.rs:334-339` で cli.json 未伝播）。
- 「受理して無視 → 受理して機能」は引数体系 (a)〜(f) を変えない互換安全な拡張であり、Admin Surface の出力一貫性（SPEC-004 REQ.08 の check JSON 対応）に揃える。
- REQ.09 改訂: 既定は人間可読 Text。グローバル `--json` 指定時は `id / type / path / depth / エッジ情報（走査順）` を JSON で出力（investigate の suspicious nodes は `drift` 値を含む）。フィールド集合は v3 Text 出力（`te-nav/src/reporter.rs:87-102`）を基準に定義する。

---

## 検出元検査の集計

| 検査カテゴリ | 検出件数 |
|---|---|
| 未定義語 | 0 |
| 複数解釈 | 0 |
| 例外未定義 | 1 |
| 境界不明 | 1 |
| 矛盾 | 0 |
| 非機能不足 | 0 |
| 合計 | 2 |

## メモ

- 回答が確定したら SPP-LGX-005 として SPEC 差分案を発行する。
