# Document ID: QSET-LGX-006

**親 SPEC**: SPEC-LGX-006
**反復回数**: 1
**作成日**: 2026-06-04
**作成者**: AI (designer)

---

## 概要

このドキュメントは前段ループの反復 1 回目で発行された質問票である。SPEC-LGX-006（embedding とドリフト検出）に対してフロントエンド検査器が検出した運用上の矛盾・責務範囲不明・メタデータ不整合・例外未定義を、開発者が回答可能な形に変換したもの。

---

## Q1: 矛盾（運用） — 既定 embedding モデルの不一致

**質問**: REQ.01 は既定モデルを `paraphrase-multilingual-MiniLM-L12-v2`（多言語、384 次元）と定めます。しかし運用側は `all-MiniLM-L6-v2`（英語中心）を参照しています:

- CLAUDE.md「ONNX モデル: `models/all-MiniLM-L6-v2/`」
- `scripts/trace-check.sh` [2/5] が `models/all-MiniLM-L6-v2/model.onnx` を探してスキップ
- `.trace-engine.toml` のコメントは `paraphrase-multilingual-MiniLM-L12-v2` を採用と記載（SPEC と一致、運用スクリプトと不一致）

正準の既定モデルはどちらで、不一致を解消するのはどちら側ですか?（第 2 層 semantic 検証の有効化手順に直結）

**SPEC 上の該当箇所**: SPEC-LGX-006 §3 REQ.01

**選択肢**:

- [x] 選択肢 A: `paraphrase-multilingual-MiniLM-L12-v2` を正準とし、CLAUDE.md / trace-check.sh / models ディレクトリを更新
- [ ] 選択肢 B: `all-MiniLM-L6-v2` を正準とし、REQ.01 と .trace-engine.toml を更新
- [ ] その他（用途で使い分け 等）: <自由記述>

**回答**:

**選択肢 A を採用**（2026-06-07 開発者決定・AI 起草）。`paraphrase-multilingual-MiniLM-L12-v2` を legixy の正準既定とし、不一致は**運用側を修正**する。

- 根拠: (1) LGX-COMPAT-001 §6 が既に「legixy 既定: 多言語（旧バイナリ実測値は all-MiniLM-L6-v2）」と宣言済み。(2) 本プロジェクトの成果物は日本語主体で多言語モデルが適合。(3) `models/paraphrase-multilingual-MiniLM-L12-v2/` は本リポジトリに配置済み。
- 修正対象（SPEC 無変更）: CLAUDE.md の「ONNX モデル: `models/all-MiniLM-L6-v2/`」記載、`scripts/trace-check.sh` [2/5] の探索パス。（2026-06-07 レビュー注記: 両修正とも SPP-LGX-006 の付随修正として同日実施済みであることをリポジトリ実物で確認。）
- SPP-LGX-006 は「SPEC-006 REQ.01 無変更 + 運用整合の指示」を含む形で発行する。

---

## Q2: 責務範囲 — calibrate コマンド（UC-011）の要求未規定

**質問**: 凍結済みコマンド `calibrate [--buckets <N>] [--recommend]`（UC-LGX-011 閾値キャリブレーション）の要求がありません。REQ.11 は calibrate を bulk similarity API の consumer として言及するのみで、**`--recommend` の推奨閾値算出ロジック**（ヒストグラムのどの分位点／谷を採るか）と calibrate コマンド自体の出力仕様が未規定です。calibrate の SPEC オーナーと推奨閾値ロジックをどう確定しますか?（SPEC-LGX-001 QSET Q1/Q2 と連動）

**SPEC 上の該当箇所**: SPEC-LGX-006 §3 REQ.11、SPEC-LGX-004 REQ.02

**回答**:

（2026-06-07 開発者決定・AI 起草）

**SPEC オーナーは新設 SPEC-LGX-010**（QSET-LGX-001 Q1 の決定に従う）。`--recommend` の推奨閾値ロジックは v3 実測を正準化して SPEC-010 に規定する:

- 全ペア類似度のパーセンタイル（p10/p25/p50/p75/p90）を算出し、
  - `similarity_threshold` 推奨値 = **p25**
  - `drift_threshold` 推奨値 = **1.0 − p90**
  - `link_candidate_threshold` 推奨値 = **p75**
- 出力仕様: `pairs`（ペア数）/ `min`・`max`・`mean` / `distribution`（ヒストグラム、`--buckets` 既定 10）/ `thresholds`（現在値）/ `recommended_thresholds`（`--recommend` 時）。
- 根拠: `te-cli/src/commands/calibrate.rs:216-242`（推奨ロジック）、同 31-40, 128-148（出力構造）。SPEC-006 REQ.11 の bulk similarity API（`compute_all_pair_scores` / `histogram`）の consumer として SPEC-010 から参照する。

---

## Q3: 整合（メタデータ） — バージョンヘッダと REQ 採番の乱れ

**質問**: 本 SPEC のヘッダ Version は `0.3.1` ですが、§5 変更履歴には後続日付の `0.4.0`（2026-04-28）と `0.4.0-draft`（2026-04-20）のエントリがあり、ヘッダと履歴が不整合です。また REQ 採番が `…REQ.09 → REQ.12 → REQ.10 → REQ.11` と物理順序で乱れています。下流成果物が本 SPEC の REQ-id を安定的に引用できるよう、正準バージョンと REQ 採番順を確定してください。

**SPEC 上の該当箇所**: SPEC-LGX-006 ヘッダ表、§3（REQ.09〜REQ.12）、§5 変更履歴

**回答**:

（2026-06-07 開発者決定・AI 起草）

- **正準バージョンは 0.4.0**。ヘッダ Version を変更履歴の最新エントリ（0.4.0、2026-04-28）に合わせて修正する。
- **REQ-id は不変**とする（リナンバリング禁止 — 下流成果物・本 QSET 群が既に REQ-id で引用しており、参照安定性を最優先）。§3 の物理順序のみ ID 順（REQ.09 → 10 → 11 → 12）に並べ替える。
- SPP-LGX-006 で機械的差分として処理する。

---

## Q4: 例外未定義 — モデル次元変更時の既存 embedding 取扱い

**質問**: REQ.01 は出力次元をモデル shape から動的確定（384/768 対応）とし、REQ.10 は model_version 変化時の全再生成を定めます。しかし **次元数が異なるモデルに切替えた場合**（384→768 等）、再生成前の既存 embeddings と次元不一致になります。この遷移期の挙動（コサイン類似度計算が次元不一致で失敗しうる）はどう扱いますか?

**SPEC 上の該当箇所**: SPEC-LGX-006 §3 REQ.01, REQ.10、REQ.04（コサイン類似度）

**選択肢**:

- [ ] 選択肢 A: model_version 変化を検出した時点で旧 embeddings を無効化し、再生成まで類似度計算をスキップ（Info 通知）
- [x] 選択肢 B: 次元不一致ペアを検出して Warning + 当該ペアをスキップ
- [ ] その他: <自由記述>

**回答**:

**選択肢 B を採用し、v3 の混在挙動を統一・可視化する**（2026-06-07 開発者決定・AI 起草）。

- v3 実態は混在: 類似度計算（edge/candidate/all-pair）は次元不一致ペアを**無言 skip**（`te-embed/src/similarity.rs:84-86, 124-126, 155-157` の continue）、standalone drift は **Error**（`te-embed/src/drift.rs:11-17` の DimensionMismatch）。
- 確定仕様: check 内・bulk API の類似度計算は次元不一致ペアを skip し、**Warning（DimensionMismatch）を報告**する（**上積み、v3 差分**: v3 は無言 skip）。standalone drift の Error は v3 どおり維持。
- model_version 変化の検出（`te-embed/src/store.rs:35-53` の (content_hash, model_version) 比較）→ REQ.10 の全再生成が完了するまでの遷移期にこの skip+Warning が適用される旨を REQ.01/REQ.04 に明記する。
- 精密化（2026-06-07 追記、整合性・堅牢性・実現性の 3 軸比較で採用確定）: Warning は**ペア毎ではなく集約 1 件**（例:「次元不一致により N ペアの類似度計算を skip（model_version 遷移中。`embed --all` で再生成してください）」）として報告する。大規模グラフでの Warning 洪水を避けつつ、無言 skip による「semantic 検証の静かな無効化 → check の偽 green」（品質偏向防止に反する）を可視化する。v3 の無言 skip 維持は、同一条件で similarity=無言/drift=Error という v3 内の自己不整合の温存でもあり、3 軸全てで劣後。

---

## 検出元検査の集計

| 検査カテゴリ | 検出件数 |
|---|---|
| 未定義語 | 0 |
| 複数解釈 | 0 |
| 例外未定義 | 1 |
| 境界不明 | 1 |
| 矛盾 | 2 |
| 非機能不足 | 0 |
| 合計 | 4 |

## メモ

- Q1 は SPEC ではなく運用ファイル（CLAUDE.md / scripts / models）側の修正で解決する可能性が高く、SPP では「SPEC 無変更 + 運用整合の指示」となりうる。
- Q2 は SPEC-LGX-001 QSET の未割当サブコマンド判断と一体。
- 回答が確定したら SPP-LGX-006 として SPEC 差分案を発行する。
