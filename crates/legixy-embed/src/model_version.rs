// (module of SRC-LGX-007; anchor: orchestrator.rs)
// compute_model_version / PreprocessProfile / ShapeValidation（DD-LGX-007 §2.2・§3）。
//
// model_version 複合キー生成・ONNX shape 検証。

use std::path::Path;

use sha2::{Digest, Sha256};

use crate::error::EmbedError;

/// model_version 複合キーの前処理プロファイル識別子（DD-LGX-007 §2.2、REQ.10）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PreprocessProfile {
    /// prefix なし（paraphrase-multilingual-MiniLM-L12-v2 等 BERT 系）。
    Plain,
    /// "query:" / "passage:" prefix 付与（intfloat/multilingual-e5 系）。
    E5Prefix,
}

impl PreprocessProfile {
    fn as_str(&self) -> &'static str {
        match self {
            PreprocessProfile::Plain => "plain",
            PreprocessProfile::E5Prefix => "e5",
        }
    }
}

/// ONNX モデル出力 shape 検証結果（DD-LGX-007 §2.2、REQ.01 GAP-LGX-103）。
#[derive(Debug, Clone, PartialEq)]
pub enum ShapeValidation {
    Ok { hidden_dim: usize },
    Invalid { reason: String },
}

/// `{model_name}:{onnx_sha256_8hex}:{profile}:{dim}` 複合キー文字列を返す（DD-LGX-007 §3、REQ.10）。
/// 同一 ONNX ファイルなら同一 model_version。ファイル読込不能時はパス文字列でハッシュ代替し、
/// 別パス → 別 model_version を保証する（SCORE-INV-2）。
pub fn compute_model_version(
    model_name: &str,
    onnx_path: &Path,
    profile: PreprocessProfile,
    dim: usize,
) -> Result<String, EmbedError> {
    let mut hasher = Sha256::new();
    match std::fs::read(onnx_path) {
        Ok(bytes) => hasher.update(&bytes),
        Err(_) => hasher.update(onnx_path.to_string_lossy().as_bytes()),
    }
    let full = format!("{:x}", hasher.finalize());
    let onnx_sha8 = &full[..8];
    Ok(format!(
        "{}:{}:{}:{}",
        model_name,
        onnx_sha8,
        profile.as_str(),
        dim
    ))
}
