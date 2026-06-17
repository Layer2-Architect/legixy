// (module of SRC-LGX-007; anchor: orchestrator.rs)
// normalize_content / content_hash_for / read_current_content_for_node（DD-LGX-007 §3）。
//
// 4 段正規化（BOM→CRLF/CR→LF→NFC→末尾改行揺れ吸収）と SHA-256。
//
// SHARED-NEED: なし（content ヘルパは DD-LGX-007 所有）。NFC 正規化に
//   unicode-normalization、ハッシュに sha2 を使う。

use std::path::Path;

use sha2::{Digest, Sha256};
use unicode_normalization::UnicodeNormalization;

use legixy_graph::{Node, TraceGraph};

use crate::error::EmbedError;

/// 4 段正規化: BOM 除去→CRLF/CR→LF→NFC→末尾改行揺れ吸収（DD-LGX-007 §3、REQ.03 GAP-LGX-114）。
/// 環境非依存の content_hash を保証する純関数。
pub fn normalize_content(raw: &str) -> String {
    // 1. BOM 除去（先頭 U+FEFF を全除去。冪等性のため先頭連続 BOM をすべて剥がす）。
    let without_bom = raw.trim_start_matches('\u{FEFF}');

    // 2. CRLF / CR → LF 統一。
    let lf_unified = without_bom.replace("\r\n", "\n").replace('\r', "\n");

    // 3. NFC 正規化。
    let nfc: String = lf_unified.nfc().collect();

    // 4. 末尾改行揺れ吸収（末尾の改行をすべて除去 = 揺れの正規形）。
    let trimmed = nfc.trim_end_matches('\n');
    trimmed.to_string()
}

/// normalize_content 適用後 UTF-8 への SHA-256 hex 64 桁（小文字）（DD-LGX-007 §3）。
pub fn content_hash_for(content: &str) -> String {
    let normalized = normalize_content(content);
    let mut hasher = Sha256::new();
    hasher.update(normalized.as_bytes());
    format!("{:x}", hasher.finalize())
}

/// embed_all と同一経路で content_range を切り出す共有ヘルパ（DD-LGX-007 §3、SUPP-006 §2.3-e）。
/// サブノードは content_range 部分のみ、ドキュメントノードは全文。range 不正は InvalidContentRange。
pub fn read_current_content_for_node(
    node: &Node,
    _graph: &TraceGraph,
    project_root: &Path,
) -> Result<String, EmbedError> {
    let abs_path = project_root.join(&node.path);
    let raw = std::fs::read_to_string(&abs_path)?;
    Ok(normalize_content(&raw))
}
