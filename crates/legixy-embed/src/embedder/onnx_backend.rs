// (module of SRC-LGX-007; anchor: embedder.rs)
// 実 ONNX backend（ort + tokenizers + ndarray）。cfg(feature = "onnx") でのみコンパイル。
// mean pooling → L2 正規化（v3 embedder.rs 底本）。

use std::cell::RefCell;
use std::path::Path;

use ndarray::Array2;
use ort::session::builder::GraphOptimizationLevel;
use ort::session::Session;
use ort::value::Tensor;

use crate::error::EmbedError;

pub struct OnnxBackend {
    session: RefCell<Session>,
    tokenizer: tokenizers::Tokenizer,
    dim: usize,
}

impl OnnxBackend {
    pub fn load(onnx_path: &Path, tok_path: &Path) -> Result<Self, EmbedError> {
        let tokenizer =
            tokenizers::Tokenizer::from_file(tok_path).map_err(|e| EmbedError::TokenizerError {
                reason: e.to_string(),
            })?;

        let session = Session::builder()
            .map_err(|e| EmbedError::ModelLoadFailed {
                path: onnx_path.to_path_buf(),
                reason: format!("session builder: {}", e),
            })?
            .with_optimization_level(GraphOptimizationLevel::Level3)
            .map_err(|e| EmbedError::ModelLoadFailed {
                path: onnx_path.to_path_buf(),
                reason: format!("optimization level: {}", e),
            })?
            .commit_from_file(onnx_path)
            .map_err(|e| EmbedError::ModelLoadFailed {
                path: onnx_path.to_path_buf(),
                reason: format!("commit_from_file: {}", e),
            })?;

        Ok(Self {
            session: RefCell::new(session),
            tokenizer,
            dim: 384,
        })
    }

    pub fn dim(&self) -> usize {
        self.dim
    }

    /// mean pooling → L2 正規化。出力 (embedding, output_dim)。
    pub fn embed(&self, text: &str) -> Result<(Vec<f32>, usize), EmbedError> {
        let encoding =
            self.tokenizer
                .encode(text, true)
                .map_err(|e| EmbedError::TokenizerError {
                    reason: e.to_string(),
                })?;
        let input_ids: Vec<i64> = encoding.get_ids().iter().map(|&id| id as i64).collect();
        let attention_mask: Vec<i64> = encoding
            .get_attention_mask()
            .iter()
            .map(|&m| m as i64)
            .collect();
        let token_type_ids: Vec<i64> =
            encoding.get_type_ids().iter().map(|&t| t as i64).collect();
        let seq_len = input_ids.len();

        let input_ids_array = Array2::<i64>::from_shape_vec((1, seq_len), input_ids)
            .map_err(|e| EmbedError::OnnxInferenceError {
                reason: e.to_string(),
            })?;
        let attention_mask_array =
            Array2::<i64>::from_shape_vec((1, seq_len), attention_mask.clone()).map_err(|e| {
                EmbedError::OnnxInferenceError {
                    reason: e.to_string(),
                }
            })?;
        let token_type_ids_array = Array2::<i64>::from_shape_vec((1, seq_len), token_type_ids)
            .map_err(|e| EmbedError::OnnxInferenceError {
                reason: e.to_string(),
            })?;

        let input_ids_val = Tensor::from_array(input_ids_array).map_err(|e| {
            EmbedError::OnnxInferenceError {
                reason: e.to_string(),
            }
        })?;
        let attention_mask_val = Tensor::from_array(attention_mask_array).map_err(|e| {
            EmbedError::OnnxInferenceError {
                reason: e.to_string(),
            }
        })?;
        let token_type_ids_val = Tensor::from_array(token_type_ids_array).map_err(|e| {
            EmbedError::OnnxInferenceError {
                reason: e.to_string(),
            }
        })?;

        let mut session_mut = self.session.borrow_mut();
        let outputs = session_mut
            .run(ort::inputs![
                "input_ids" => input_ids_val,
                "attention_mask" => attention_mask_val,
                "token_type_ids" => token_type_ids_val,
            ])
            .map_err(|e| EmbedError::OnnxInferenceError {
                reason: e.to_string(),
            })?;

        let (shape, data) = outputs[0]
            .try_extract_tensor::<f32>()
            .map_err(|e| EmbedError::OnnxInferenceError {
                reason: e.to_string(),
            })?;
        let output_dim = *shape.last().unwrap_or(&(self.dim as i64)) as usize;

        let mask_f32: Vec<f32> = attention_mask.iter().map(|&m| m as f32).collect();
        let mask_sum: f32 = mask_f32.iter().sum();
        let mut pooled = vec![0.0_f32; output_dim];
        if mask_sum > 0.0 {
            for (token_idx, &mask_val) in mask_f32.iter().enumerate().take(seq_len) {
                if mask_val > 0.0 {
                    let offset = token_idx * output_dim;
                    for dim_idx in 0..output_dim {
                        pooled[dim_idx] += data[offset + dim_idx] * mask_val;
                    }
                }
            }
            for val in pooled.iter_mut() {
                *val /= mask_sum;
            }
        }

        let norm: f32 = pooled.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 0.0 {
            for val in pooled.iter_mut() {
                *val /= norm;
            }
        }

        Ok((pooled, output_dim))
    }
}
