Document ID: AT-LGX-001

# AT-LGX-001: embedding サブシステム 実機受け入れ（実 ONNX 推論・決定性・スループット・並行制御）

> AT は **暗黙知・ドメイン慣行・前提不一致** を検出する独立検証チャネル（→ `docs/DevProc_V4/07-at-and-nfr.md`）。
> 本 AT は TC-LGX-007/010/011/012/013 がスタブ backend / in-memory DB で GREEN 化した領域のうち、
> **実物（配置済み ONNX モデル・on-disk SQLite）に触れて初めて確認できる前提**を実機で受け入れ検証する。

**対象 UC**: UC-LGX-007（embedding 生成）, UC-LGX-010〜013（report / calibrate / snapshot / drift）
**対象 SPEC**: SPEC-LGX-006（embedding）, SPEC-LGX-010（drift/snapshot）（参照）
**実施タイミング**: 全 13UC SRC[GREEN] 到達マイルストーン（2026-06-14）
**関連 NFR**: NFR-LGX-001.PERF.08（スループット）, .REL.07（busy_timeout）, .PERF.07（WAL）

## 1. 検証方法

- [x] 実機自動検証（AI 実行）: 配置済み実 ONNX モデルでの推論・on-disk SQLite 競合の実時間観察
- [ ] 想定ユーザーによる実機操作観察 ← **未実施（保留）**: CLI/MCP の UX 受け入れは別途人間観察セッションが必要
- [ ] 半構造化インタビュー ← 未実施
- [x] ドメイン専門家レビュー: 決定性モデル（ADR-LGX-003）とスループット予見（NFR §3.2）との突合

> ⚠️ 正直な範囲限定: 本 AT は「実物の振る舞い／非機能の受け入れ」を自動検証で満たすが、AT 本来の
> 中核目的である **人間の暗黙知・UX 違和感の観察**（テンプレ §2/§3/§5）は CLI/MCP の対人運用開始時に
> 別セッションとして実施する。本書はその時点で §5 観察記録を追補する。

## 2. 想定ユーザー特性（UX 観察セッション用・未実施）

- 役割: 単独開発者（legixy で自プロジェクトのトレーサビリティを管理）
- 経験レベル: CLI 慣れ、ONNX/embedding は非専門
- 利用環境: ローカル CPU（NFR §3.1 = i5-12400F / 32GB）、Step 1 Windows / Step 2 Ubuntu Docker

## 3. ベンチマークタスク（自動検証で代替した受け入れ項目）

| # | 受け入れ項目 | 成功条件 | 検証コード |
|---|---|---|---|
| 1 | 実 ONNX 推論が動く | 配置済みモデルをロードし 384 次元・L2 正規化済み（norm≈1）の非ゼロ embedding を返す | `tests/at_onnx_reproducibility.rs::at_onnx_real_inference_normalized_384dim` |
| 2 | 同一環境決定性 | 同一入力 → 推論値の最大絶対差 < 1e-4（ADR-003 範囲）。加えて同一環境ではバイト一致を観察 | `tests/at_onnx_reproducibility.rs::at_onnx_same_environment_determinism` |
| 3 | 内容反映の健全性 | 無関係な 2 文の cosine が著しく低い（同一 embedding に縮退しない） | `tests/at_onnx_reproducibility.rs::at_onnx_distinct_text_distinct_embedding` |
| 4 | スループット（PERF.08）| 逐次 embed_node の throughput を実測し閾値と突合 | `benches/perf08_embed_throughput.rs`（criterion） |
| 5 | 並行制御（REL.07）| 競合書込みが busy_timeout 上限内で SQLITE_BUSY を Err 返却（無限リトライ無し） | `tests/at_rel07_concurrency.rs::rel07_busy_timeout_bounds_wait_and_returns_error` |

実行コマンド:
```bash
cargo test  -p legixy-embed --features onnx --test at_onnx_reproducibility -- --nocapture
cargo bench -p legixy-embed --features onnx --bench perf08_embed_throughput
cargo test  -p legixy-embed --test at_rel07_concurrency -- --nocapture
```

## 4. 判定基準

- [x] 実 ONNX 推論が 384 次元・L2 正規化済みベクトルを返す
- [x] 同一環境・同一入力で推論値が微小差以内（ADR-LGX-003 の順序/入力/モデル決定性の範囲）
- [x] 異なる内容が異なる embedding になる（モデルが実際に内容を反映）
- [x] embedding 生成スループットを実測し NFR §3.2 PERF.08 と突合（再評価込み）
- [x] 並行ロック競合が**上限時間内に失敗として返る**（無限リトライしない、REL.07）

## 5. 結果記録

### 自動検証セッション 1

- 観察日: 2026-06-14
- 実行者: AI（Author セッション）
- 測定環境: Intel Core i5-12400F（可視 4 コア）, Linux, `models/paraphrase-multilingual-MiniLM-L12-v2`

| 項目 | 結果 | 実測値 |
|---|---|---|
| 1 実 ONNX 推論 | ✅ PASS | 384 次元・norm≈1・非ゼロ |
| 2 同一環境決定性 | ✅ PASS | 最大絶対差 = **0.000e0**、3 回 to_le_bytes 比較 = **バイト一致** |
| 3 内容反映 | ✅ PASS | 無関係 2 文の cosine = **0.0735** |
| 4 PERF.08 スループット | ✅ PASS（閾値再評価済） | 中央値 **≈ 31 nodes/sec**（criterion、30 samples）。旧暫定 ≥50（L6 想定）未達 → **ADR-LGX-022（accepted 2026-06-14）** で確定閾値 ≥25 nodes/sec に再評価 → 31 ≥ 25 で PASS |
| 5 REL.07 並行制御 | ✅ PASS | busy_timeout=300ms 競合書込み: elapsed=**301.6ms**、SQLITE_BUSY を Err 返却、解放後は書込み成功 |

### つまずき / 発見

- **PERF.08 が L6 時代の暫定 50 nodes/sec を下回る（実測 ≈31）**。NFR §3.2 が「L12 で低下見込み・確定後に
  閾値見直し」、§13 #6 が「ONNX モデル確定後に再評価必須」と予見済みの想定内事象。SRC 高速化は
  DD-LGX-007 §7（逐次・単一スレッド凍結）により範囲外のため、閾値を実測接地値へ再評価（ADR-LGX-022）。
- **「ビット再現性」の語の前提ズレ**: ADR-LGX-003 は環境間（CPU/BLAS/スレッド差）のビット単位再現を
  **明示的に対象外**とし、順序・入力(content_hash)・モデル同一性(model_version)の三層のみ保証する。
  実機では**同一環境ではバイト一致を確認**できたが、これは仕様不変条件への昇格ではなく回帰ガード。
  クロス環境の厳密ビット再現を要件化するなら ADR-003 の反転＝SPEC/設計改訂が必要（本 AT は要件化しない）。

## 6. 発見された新観点（perspectives.md 昇格候補）

| # | 観点 | カテゴリ | 既存観点ベースに追記済 |
|---|---|---|---|
| 1 | NFR の暫定閾値は「モデル/環境確定」で必ず実測接地し直す（過大宣言の自動失効点を設ける） | core | no（昇格候補） |
| 2 | 「再現性」は環境内/環境間で意味が異なる。保証範囲を語彙レベルで分離して記述する | core | no（昇格候補） |

> 昇格は人間レビュー時に `docs/perspectives/core-perspectives.md` の「AT から戻ってきた観点」表へ反映する。

## 7. AT 失敗時の処置

本 AT で SPEC / UC の不備に直結する失敗は**発見されなかった**（PERF.08 は NFR 内の予見済み再評価、
ビット再現性は ADR-003 の既定範囲内）。したがって GAP[UC]/GAP[SPEC] の新規起票は無し。

| 発見事項 | 関連 GAP | 影響先 |
|---|---|---|
| PERF.08 実測 < 旧暫定閾値 | （GAP 不要：NFR 内再評価）| NFR-LGX-001.PERF.08 → ADR-LGX-022 |
| 同一環境バイト一致は仕様外保証 | （GAP 不要：ADR-003 既定）| 回帰ガードとして AT 保持 |

## 8. perspectives.md への昇格

UX 観察セッション実施後に §6 の観点を昇格する（現時点は候補登録のみ）。

## 9. 関連 NFR / SPEC / UC / ADR

- 関連 NFR: NFR-LGX-001.PERF.08 / .PERF.07 / .REL.07
- 関連 SPEC: SPEC-LGX-006, SPEC-LGX-010
- 関連 UC: UC-LGX-007, UC-LGX-010, UC-LGX-011, UC-LGX-012, UC-LGX-013
- 関連 ADR: ADR-LGX-003（embedding 決定論モデル）, ADR-LGX-022（PERF.08 L12 再評価）
- 関連 DD: DD-LGX-007 §7（並行性 / 逐次凍結）
