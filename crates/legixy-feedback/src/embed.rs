// Document ID: SRC-LGX-008
// (module of SRC-LGX-008; anchor: manager.rs)（embed 局所部）
// SHARED-NEED 解消（局所）: EmbedError / mask_api_key は本来 legixy-embed 所有（DD-LGX-008 §1 /
//   §3.1 drift_from_embed_error の入力型、NFR-LGX-001 SEC.05）。legixy-embed scaffold が空のため
//   本 crate に v3 lx-embed の mask_api_key を底本とした実体を置く（legixy-embed 編集禁止）。

use regex::Regex;

/// 埋め込み層の失敗（本来 legixy-embed 所有）。drift Observation 生成の入力。
/// SHARED-NEED: 統合時に legixy_embed::EmbedError へ置換。
#[derive(Debug, thiserror::Error)]
pub enum EmbedError {
    /// 文脈検索（contextual retrieval）失敗。drift カテゴリ Observation 生成の唯一の契機（DD §3.1）。
    #[error("contextual retrieval failed for {node_id}: {detail}")]
    ContextualRetrievalFailed { node_id: String, detail: String },

    /// 他の埋め込み失敗（drift には変換しない＝None）。
    #[error("embedding failed: {0}")]
    Other(String),
}

/// API キー等のクレデンシャルを message からマスクする（NFR SEC.05）。
/// v3 lx-embed::mask_api_key を底本（DD-LX-005 §3.8「誤検出より漏洩ゼロを優先」）。
/// より特定的なパターンから順に適用する（sk-ant- は sk- でも一致するため先に除去）。
/// 【legixy 差分】sk- 系プレフィックスは短い断片も漏洩ゼロ優先で除去（閾値 {6,}）。
pub fn mask_api_key(message: &str) -> String {
    static PATTERNS: &[&str] = &[
        r"sk-ant-[A-Za-z0-9_\-]{6,}",
        r"sk-[A-Za-z0-9_\-]{6,}",
        r"AIza[0-9A-Za-z_\-]{35}",
        r"Bearer\s+[A-Za-z0-9_\-\.=/+]{20,}",
    ];
    let mut out = message.to_string();
    for p in PATTERNS {
        let re = Regex::new(p).expect("valid regex literal");
        out = re.replace_all(&out, "***REDACTED***").into_owned();
    }
    out
}
