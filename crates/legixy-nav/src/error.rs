// (module of SRC-LGX-005; anchor: investigate.rs)
// legixy-nav 実行時失敗エラー型。DD-LGX-005 §2.3 / DD-LGX-006 §2.3。
// 起点ノード不在・空グラフは NavError ではなく空結果（D 裁定、REQ.05）。
// グラフロード失敗は legixy-graph::GraphError → legixy-cli で捕捉。
// DB 照会失敗は NavError に昇格させずベストエフォート継続（DD-005 §6）。

/// 実行時失敗（上位に伝播して exit 1。検証 finding とは別概念、LGX-COMPAT-001 §3）。
#[derive(Debug, thiserror::Error)]
pub enum NavError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("invalid input: {0}")]
    InvalidInput(String),
}
