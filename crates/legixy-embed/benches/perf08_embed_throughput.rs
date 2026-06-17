// NFR-LGX-001.PERF.08 — embedding 生成スループット (nodes/sec) ベンチ。
//
// 検証対象: PERF.08「embedding 生成スループット ≥ 50 nodes/sec【暫定・要再評価】」
//   （CPU only、多言語モデル paraphrase-multilingual-MiniLM-L12-v2。NFR-LGX-001 §3.2）。
//
// 実行: cargo bench -p legixy-embed --features onnx --bench perf08_embed_throughput
//   required-features = ["onnx"] のため onnx 無効ビルドでは本ベンチはビルド対象外（スキップ）。
//   モデル不在環境では計測をスキップ（panic しない）。
//
// criterion は単一 embed_node 呼出しの latency を計測し、Throughput::Elements(1) により
// elem/s = nodes/sec を直接報告する（逐次・単一スレッド前提＝ DD-007 §7）。

use std::path::PathBuf;
use std::time::Duration;

use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use legixy_embed::Embedder;

fn model_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../models/paraphrase-multilingual-MiniLM-L12-v2")
}

/// 代表的なノード本文（SPEC/UC の 1 ノード相当、JA/EN 混在 ~300 字）。
const SAMPLE_NODE: &str = "legixy は有向グラフを主体とするトレーサビリティエンジンであり、\
SPEC・UC・RBA・SEQA・RBD・SEQD・DD・TS・TC・SRC を一本の連鎖として接続する。\
check は chain 整合と DAG 健全性を検証し、compile_context は上流成果物を文脈として合成する。\
embedding は paraphrase-multilingual-MiniLM-L12-v2 で生成し、mean pooling 後に L2 正規化する。\
This node represents a realistic mixed Japanese/English artifact body for throughput measurement.";

fn bench_throughput(c: &mut Criterion) {
    let dir = model_dir();
    if !(dir.join("model.onnx").exists() && dir.join("tokenizer.json").exists()) {
        eprintln!("SKIP PERF.08 bench: model not present at {dir:?}");
        return;
    }
    let embedder =
        Embedder::new(&dir, "perf08:bench:plain:384").expect("実 ONNX モデルのロード");
    assert_eq!(embedder.dim(), 384);

    let mut group = c.benchmark_group("PERF.08_embed_throughput");
    // 1 call = 1 node。criterion はこれを elem/s（= nodes/sec）として報告する。
    group.throughput(Throughput::Elements(1));
    group.sample_size(30);
    group.warm_up_time(Duration::from_secs(2));
    group.measurement_time(Duration::from_secs(10));
    group.bench_function("embed_node_single", |b| {
        b.iter(|| {
            let r = embedder
                .embed_node(black_box(SAMPLE_NODE), None, "bench")
                .expect("embed_node");
            black_box(r.embedding.len())
        })
    });
    group.finish();
}

criterion_group!(benches, bench_throughput);
criterion_main!(benches);
