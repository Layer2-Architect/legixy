// AT-LGX-001 連動: 実 ONNX backend の動作・決定性検証（`--features onnx` でのみコンパイル）。
//
// 位置づけ: TC-LGX-007 はスタブ backend で純ロジックを GREEN 化する。本ファイルは
// TS-LGX-007 が「ONNX 推論値のビット再現性（DD §7）」として責務委譲した領域を、
// AT（独立検証チャネル）として実機で観察・検証する。
//
// ★ 重要な前提（ADR-LGX-003）: legixy は **環境間（CPU/BLAS/スレッド差）のビット単位再現性は
//   対象外**と凍結している。保証するのは ①順序決定性 ②入力決定性(content_hash) ③モデル同一性
//   (model_version) の三層。本テストはその範囲で検証し、加えて「同一環境・同一プロセス内」の
//   決定性を回帰ガードとして観察する（仕様不変条件への昇格ではない）。
//
// 実行: `cargo test -p legixy-embed --features onnx --test at_onnx_reproducibility -- --nocapture`
// モデル不在環境ではスキップ（panic しない）。

#![cfg(feature = "onnx")]

use std::path::PathBuf;

use legixy_embed::Embedder;

/// workspace ルートの配置済みモデルへの絶対パス（crate からの相対 `../../models/...`）。
fn model_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../models/paraphrase-multilingual-MiniLM-L12-v2")
}

fn model_present(dir: &PathBuf) -> bool {
    dir.join("model.onnx").exists() && dir.join("tokenizer.json").exists()
}

fn to_le_bytes(v: &[f32]) -> Vec<u8> {
    v.iter().flat_map(|x| x.to_le_bytes()).collect()
}

#[test]
fn at_onnx_real_inference_normalized_384dim() {
    let dir = model_dir();
    if !model_present(&dir) {
        eprintln!("SKIP at_onnx_real_inference_normalized_384dim: model not present at {dir:?}");
        return;
    }
    let embedder = Embedder::new(&dir, "paraphrase-multilingual-MiniLM-L12-v2:test:plain:384")
        .expect("実 ONNX モデルのロード");
    assert_eq!(embedder.dim(), 384, "出力次元 = 384");

    let r = embedder
        .embed_node("legixy は有向グラフを主体とするトレーサビリティエンジンである。", None, "UC-LGX-001")
        .expect("embed_node");
    assert_eq!(r.embedding.len(), 384, "embedding 長 = 384");

    let norm: f32 = r.embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
    assert!((norm - 1.0).abs() < 1e-3, "L2 正規化済み（norm≈1）, 実測 norm={norm}");

    // 自明な縮退（全ゼロ・全同値）でないこと。
    let all_zero = r.embedding.iter().all(|&x| x == 0.0);
    assert!(!all_zero, "非ゼロベクトル");
}

#[test]
fn at_onnx_same_environment_determinism() {
    // ADR-LGX-003 の範囲外（環境間ビット再現）ではなく、同一環境・同一プロセス内の決定性を観察する。
    let dir = model_dir();
    if !model_present(&dir) {
        eprintln!("SKIP at_onnx_same_environment_determinism: model not present");
        return;
    }
    let embedder =
        Embedder::new(&dir, "mv:test:plain:384").expect("ロード");
    let text = "同一入力は同一環境で同一 embedding を生むことを回帰ガードとして確認する。";

    let a = embedder.embed_node(text, None, "n1").expect("embed a");
    let b = embedder.embed_node(text, None, "n1").expect("embed b");
    let c = embedder.embed_node(text, None, "n1").expect("embed c");

    // ① content_hash 決定性（ADR-LGX-003 保証層 ②、入力決定性）。
    assert_eq!(a.content_hash, b.content_hash, "content_hash 決定性");
    assert_eq!(a.content_hash, c.content_hash, "content_hash 決定性");

    // ② 同一環境での推論値: まず最大絶対差を観測（ADR-003 は微小差を許容＝drift_threshold が吸収）。
    let max_abs_diff = a
        .embedding
        .iter()
        .zip(b.embedding.iter())
        .map(|(x, y)| (x - y).abs())
        .fold(0.0_f32, f32::max);
    eprintln!("[OBSERVE] 同一環境 同一入力 最大絶対差 = {max_abs_diff:.3e}");

    // 仕様準拠の判定（ADR-LGX-003 §4「微小な推論値差は検出対象外」）: 微小差以内であること。
    assert!(
        max_abs_diff < 1e-4,
        "同一環境では推論値が微小差以内に収まる（ADR-003 順序/入力/モデル決定性の範囲）, 実測 {max_abs_diff:.3e}"
    );

    // ③ 同一環境ビット一致の観察（回帰ガード。失敗しても ADR-003 上は仕様違反ではない＝eprintln 報告のみ）。
    let bit_identical = to_le_bytes(&a.embedding) == to_le_bytes(&b.embedding)
        && to_le_bytes(&b.embedding) == to_le_bytes(&c.embedding);
    eprintln!(
        "[OBSERVE] 同一環境ビット一致 (3 回 to_le_bytes 比較) = {bit_identical} \
         （ADR-LGX-003: 環境間ビット再現は対象外。同一環境一致は実装の決定性指標）"
    );
}

#[test]
fn at_onnx_distinct_text_distinct_embedding() {
    // 異なる入力は異なる embedding（モデルが実際に内容を反映している健全性チェック）。
    let dir = model_dir();
    if !model_present(&dir) {
        eprintln!("SKIP at_onnx_distinct_text_distinct_embedding: model not present");
        return;
    }
    let embedder = Embedder::new(&dir, "mv:test:plain:384").expect("ロード");
    let a = embedder.embed_node("トレーサビリティと有向グラフ", None, "x").expect("a");
    let b = embedder.embed_node("全く無関係な料理のレシピと天気予報", None, "y").expect("b");

    let cos: f32 = a
        .embedding
        .iter()
        .zip(b.embedding.iter())
        .map(|(x, y)| x * y)
        .sum();
    eprintln!("[OBSERVE] 異内容間 cosine = {cos:.4}");
    assert!(cos < 0.999, "異なる内容は同一 embedding にならない, cos={cos:.4}");
}
