// (module of SRC-LGX-007; anchor: orchestrator.rs)
// Embedder / EmbedResult / embed_node（DD-LGX-007 §2.1・§3）。
//
// Embedder は ONNX セッション + tokenizer を保持する（DD-007 §2.1）。実 ONNX 推論は
// `onnx` backend（ort + tokenizers + ndarray、cfg(feature = "onnx")）で行う。feature 無効時／
// テストは決定的スタブ backend を用いる。`new` はモデルファイル欠落で ModelLoadFailed（exit 1）。
//
// SHARED-NEED: なし（Embedder / EmbedResult は DD-LGX-007 所有 = 本 crate ローカル）。

use std::path::Path;

use sha2::{Digest, Sha256};

use crate::error::EmbedError;

#[cfg(feature = "onnx")]
mod onnx_backend;

/// ONNX モデル + tokenizer を保持し、1 ノード分の embedding 生成を担う（DD-LGX-007 §2.1）。
pub struct Embedder {
    backend: Backend,
    model_version: String,
    dim: usize,
}

enum Backend {
    /// 決定的スタブ backend（実 ONNX セッションを持たない、TS-LGX-007 §5）。
    Stub,
    #[cfg(feature = "onnx")]
    Onnx(onnx_backend::OnnxBackend),
}

/// embed_node の出力。1 ノード分の embedding 生成結果（DD-LGX-007 §2.1）。
#[derive(Debug, Clone, PartialEq)]
pub struct EmbedResult {
    pub embedding: Vec<f32>,
    pub dim: usize,
    pub model_version: String,
    pub content_hash: String,
    pub context: Option<String>,
    pub context_hash: Option<String>,
}

/// SHA-256（hex 小文字 64 字）を返す（DD-LGX-007 §3 step 3、v3 embedder.rs 底本）。
pub(crate) fn sha256_hex(s: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(s.as_bytes());
    format!("{:x}", hasher.finalize())
}

impl Embedder {
    /// model.onnx + tokenizer.json 存在 + shape 検証合格の場合のみ Ok（DD-LGX-007 §3、REQ.01）。
    /// 欠落ファイルは ModelLoadFailed（exit 1）。
    pub fn new(model_dir: &Path, model_version: &str) -> Result<Self, EmbedError> {
        let onnx_path = model_dir.join("model.onnx");
        if !onnx_path.exists() {
            return Err(EmbedError::ModelLoadFailed {
                path: onnx_path,
                reason: "model.onnx not found".to_string(),
            });
        }
        let tok_path = model_dir.join("tokenizer.json");
        if !tok_path.exists() {
            return Err(EmbedError::ModelLoadFailed {
                path: tok_path,
                reason: "tokenizer.json not found".to_string(),
            });
        }

        #[cfg(feature = "onnx")]
        {
            let backend = onnx_backend::OnnxBackend::load(&onnx_path, &tok_path)?;
            let dim = backend.dim();
            Ok(Embedder {
                backend: Backend::Onnx(backend),
                model_version: model_version.to_string(),
                dim,
            })
        }
        #[cfg(not(feature = "onnx"))]
        {
            // ONNX backend 無効ビルド: ファイルは存在するがランタイムが無いため shape 検証不能。
            let _ = model_version;
            Err(EmbedError::ModelShapeInvalid {
                reason: "onnx backend not compiled (feature \"onnx\" disabled)".to_string(),
            })
        }
    }

    /// テスト用 test-double 構築子（実 ONNX セッションを持たない決定的スタブ Embedder）。
    pub fn stub(model_version: &str, dim: usize) -> Self {
        Embedder {
            backend: Backend::Stub,
            model_version: model_version.to_string(),
            dim,
        }
    }

    /// 1 ノード分の embedding を生成する（DD-LGX-007 §3）。空テキストでも Err を返さない。
    ///
    /// `context` が `Some` のとき（Contextual Retrieval 有効、CACHE-CR-002）:
    ///  - embedding 対象テキストへ context を前置する（LGX-EXT-001 §5.8「embedding 対象テキストの前に付加」）。
    ///  - `EmbedResult.context` / `context_hash`(= SHA-256(context)) を埋める。
    ///  - `content_hash` は**素テキスト基準**のまま（ADR-LGX-009: freshness は content_hash のみが寄与、
    ///    context は freshness に関与せずキャッシュされる）。
    pub fn embed_node(
        &self,
        text: &str,
        context: Option<&str>,
        _node_id: &str,
    ) -> Result<EmbedResult, EmbedError> {
        let content_hash = sha256_hex(text);
        // context 列値（freshness 非関与、ADR-LGX-009）。embedding 入力への前置は backend 側で行う。
        let (ctx_val, ctx_hash) = match context {
            Some(ctx) if !ctx.is_empty() => (Some(ctx.to_string()), Some(sha256_hex(ctx))),
            _ => (None, None),
        };
        match &self.backend {
            Backend::Stub => {
                // スタブはテキスト非依存（決定的固定ベクトル）。CR の有無は context 列にのみ反映。
                let mut v: Vec<f32> = (0..self.dim).map(|i| ((i % 7) as f32) + 1.0).collect();
                let norm: f32 = v.iter().map(|x| x * x).sum::<f32>().sqrt();
                if norm > 0.0 {
                    for x in v.iter_mut() {
                        *x /= norm;
                    }
                }
                Ok(EmbedResult {
                    embedding: v,
                    dim: self.dim,
                    model_version: self.model_version.clone(),
                    content_hash,
                    context: ctx_val,
                    context_hash: ctx_hash,
                })
            }
            #[cfg(feature = "onnx")]
            Backend::Onnx(backend) => {
                // context を前置した入力で埋め込む（LGX-EXT-001 §5.8「embedding 対象テキストの前に付加」）。
                let embed_input = match context {
                    Some(ctx) if !ctx.is_empty() => format!("{ctx}\n\n{text}"),
                    _ => text.to_string(),
                };
                let (embedding, output_dim) = backend.embed(&embed_input)?;
                Ok(EmbedResult {
                    embedding,
                    dim: output_dim,
                    model_version: self.model_version.clone(),
                    content_hash,
                    context: ctx_val,
                    context_hash: ctx_hash,
                })
            }
        }
    }

    pub fn dim(&self) -> usize {
        self.dim
    }

    pub fn model_version(&self) -> &str {
        &self.model_version
    }
}
