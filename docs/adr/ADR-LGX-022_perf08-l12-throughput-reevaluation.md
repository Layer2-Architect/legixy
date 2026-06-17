Document ID: ADR-LGX-022

# ADR-LGX-022: PERF.08 embedding スループットの L12 モデル確定後再評価

**ステータス**: accepted
**起票日**: 2026-06-14
**承認日**: 2026-06-14
**承認者**: 開発者（人間 ratification 2026-06-14）
**対象**: NFR-LGX-001.PERF.08（embedding 生成スループット）、§13 再評価トリガ #6

## 1. 文脈（Context）

- NFR-LGX-001.PERF.08 は embedding 生成スループットの暫定閾値を **≥ 50 nodes/sec** とし、
  「【暫定・要再評価】」「L12 は旧 L6 比で層数約 2 倍 → スループット低下見込み、確定後に閾値見直し」
  と明記していた。§13 再評価トリガ表 #6 も「**ONNX モデル確定後に再評価必須**」としていた。
- 採用モデルは `paraphrase-multilingual-MiniLM-L12-v2`（12 層、384 次元、mean pooling）で確定し、
  `models/` に配置済み。SRC-LGX-007 の ONNX backend（ort 2.0.0-rc.12 + tokenizers 0.22、
  `--features onnx`）が実推論を担う。これにより PERF.08 の実測が可能になった。
- DD-LGX-007 §7 は embedding 生成を **同期・単一スレッド・逐次**と凍結しており、ノード間並列化は
  将来最適化として明示的に範囲外（本 ADR は逐次前提で評価する）。ONNX intra-op 並列は ort 既定で
  物理コア数を使用する。

## 2. 実測（Measurement）

- 測定環境: NFR §3.1 の正準測定機（**Intel Core i5-12400F**、可視 4 コア、Linux、release profile）。
- 測定手段: `cargo bench -p legixy-embed --features onnx --bench perf08_embed_throughput`
  （criterion、`Throughput::Elements(1)` で elem/s = nodes/sec を報告、SAMPLE_NODE = JA/EN 混在 ~300 字）。
- 結果（30 samples / 465 iterations、外れ値 1/30 high mild）:
  - latency: **[31.93 ms, 32.27 ms, 32.69 ms]**（lower / median / upper）
  - throughput: **[30.59, 30.99, 31.32] nodes/sec**

→ 実測中央値 **≈ 31 nodes/sec**。暫定閾値 50 nodes/sec を下回る（≈ 0.62×）。これは L6→L12 で
  予見された低下（§3.2 PERF.08 注記）と整合し、§13 #6 の再評価トリガが予定通り発火した。

## 3. 検討した選択肢（Options）

### 選択肢 A: SRC 最適化でスループットを 50 まで引き上げる
- ノード間並列化（embed_all のマルチスレッド化）/ バッチ推論。
- 欠点: **DD-LGX-007 §7 がノード間並列・バッチを将来最適化として凍結範囲外**にしている。導入は
  境界設計の変更＝次バージョンの設計事項。ONNX intra-op 並列は既定で適用済みで追加余地は小さい。

### 選択肢 B: 閾値を実測接地値へ再評価する（採用）
- L12 確定後の実測（≈ 31 nodes/sec @ i5-12400F）に基づき、暫定閾値を **≥ 25 nodes/sec
  （NFR §3.1 正準測定機基準）** に改訂する。実測値に対し約 20% のヘッドルームを確保した床値。
- 根拠: ①§13 #6 が「モデル確定後に再評価必須」と本ケースを事前承認済み ②embedding 生成は
  `embed --all` のバッチ索引化工程であり check/compile_context のホットパスではない（数百ノード規模で
  も 25 nodes/sec なら十数秒で完了し実用上問題なし）③DD-007 §7 の逐次設計を尊重。

### 選択肢 C: 閾値を撤廃する
- 欠点: 回帰検知の基準を失う（大幅劣化を見逃す）。過小宣言。不採用。

## 4. 判断（Decision）

選択肢 B を採用する（**accepted**：人間 ratification 2026-06-14）。

- PERF.08 の暫定閾値を **≥ 50 nodes/sec → ≥ 25 nodes/sec（NFR §3.1 正準測定機基準、CPU-only、逐次）**
  へ再評価する。
- 「緩和は累積させない」（07-at-and-nfr.md §3）原則に従い、本 ADR を唯一の根拠として 1 回のみ改訂し、
  以後の更なる緩和は新たな実測と ADR を要する。
- ノード間並列化・バッチ推論による高速化は DD-007 §7 凍結事項のため、必要時は次バージョンの設計
  改訂（DD 改訂 + SPEC 整合）として扱う。

## 5. 結果（Consequences）

### 期待される効果
- L12 確定後の現実的スループットに接地した回帰基準を持てる。過大宣言（達成不能な ≥50）を排除。

### 受け入れる代償
- 旧 L6 想定（≥50）より低速を許容。バッチ索引化の体感待ち時間は数百ノードで十数秒オーダ。

### 残存リスク / 申し送り
- 閾値はコア数依存。高コア機ではスループット向上が見込まれるため、本 ADR の床値は保守的。
- 将来 embed_all を並列化する場合は DD-007 §7 を改訂し PERF.08 を再々評価する。
- 本 ADR の ratification（2026-06-14）により NFR PERF.08 の確定閾値は **≥ 25 nodes/sec**（NFR §3.1 正準測定機基準）。実測値（≈31）は §14 に記録済み。

## 6. 関連

- 対象: NFR-LGX-001.PERF.08、NFR-LGX-001 §13 再評価トリガ #6
- 関連: DD-LGX-007 §7（逐次・単一スレッド凍結）、SRC-LGX-007（ONNX backend）、ADR-LGX-003（embedding 決定論モデル）
- ベンチ: `crates/legixy-embed/benches/perf08_embed_throughput.rs`
