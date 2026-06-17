// (module of SRC-LGX-007; anchor: orchestrator.rs)
// ContextualConfig / CrOptions / LlmClient / synthesize_with_fallback（DD-LGX-007 §2.1・§3・§6）。
//
// Phase 1（CACHE-CR-002 / R-6）: context 生成を実装する。LlmClient trait（DD §3 contextual.rs）を
// 介して context を合成し、embedding 対象テキストへ前置する経路（embedder.rs）を有効化する。
// 既定クライアントは LLM/network 不要の決定論実装（DeterministicContextClient）。実 Anthropic API
// 呼出し（reqwest::blocking、DD §6）は feature-gate の後続課題として分離する（LGX-EXT-001 §5.8 で
// エラー処理・フォールバックの作り込みは Phase 2 へ先送り）。

use crate::error::EmbedError;

/// CrOptions（フォールバック制御パラメータ、DD-LGX-007 §2.1、v3 contextual.rs:15-23）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CrOptions {
    pub timeout_sec: u64,
    pub max_retries: u32,
    pub base_backoff_ms: u64,
}

impl Default for CrOptions {
    fn default() -> Self {
        Self {
            timeout_sec: 30,
            max_retries: 3,
            base_backoff_ms: 1000,
        }
    }
}

/// Contextual Retrieval の詳細設定（DD-LGX-007 §2.1、REQ.06/06.1）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextualConfig {
    pub opts: CrOptions,
}

/// context 合成クライアント（DD-LGX-007 §3 contextual.rs「LlmClient trait」）。
/// `Ok(None)` = context 生成スキップ（CR 無効扱いで通常 embedding 継続）。
/// 実装は決定論既定（[`DeterministicContextClient`]）と、後続の feature-gate な実 LLM backend。
pub trait LlmClient {
    /// `text`（embedding 対象チャンク）を `node_id` の文脈に位置づける短い context を返す。
    fn synthesize(&self, text: &str, node_id: &str) -> Result<Option<String>, EmbedError>;
}

/// 決定論的既定クライアント（LLM/network 不要、Phase 1）。
/// チャンク先頭の ATX 見出し（あれば）と node_id から「この内容の位置づけ」を合成する。
/// 同一入力 → 同一 context（CTX/embedding 決定性を保つ）。
pub struct DeterministicContextClient;

impl LlmClient for DeterministicContextClient {
    fn synthesize(&self, text: &str, node_id: &str) -> Result<Option<String>, EmbedError> {
        let ctx = match first_heading(text) {
            Some(h) => format!("出典: {node_id} — 「{h}」"),
            None => format!("出典: {node_id}"),
        };
        Ok(Some(ctx))
    }
}

/// 先頭の ATX 見出し（`# 〜`〜`###### 〜`）のテキストを返す（# マーカー・closing # 除去・trim 済）。
/// 見出しが無ければ None。
fn first_heading(text: &str) -> Option<String> {
    for line in text.lines() {
        let trimmed = line.trim_start();
        let level = trimmed.chars().take_while(|c| *c == '#').count();
        if level == 0 || level > 6 {
            continue;
        }
        let after = &trimmed[level..];
        if !after.starts_with(' ') {
            continue;
        }
        let title = after.trim_start().trim_end_matches('#').trim();
        if !title.is_empty() {
            return Some(title.to_string());
        }
    }
    None
}

/// CR フォールバック付き context 合成（DD-LGX-007 §6、REQ.06.1）。
/// Phase 1 は決定論既定クライアントを使う。クライアント失敗時は CR 無効扱い（`Ok(None)`）+ stderr
/// Warning（Err には昇格しない＝通常 embedding 継続）。
pub fn synthesize_with_fallback(
    _config: &ContextualConfig,
    text: &str,
    node_id: &str,
) -> Result<Option<String>, EmbedError> {
    let client = DeterministicContextClient;
    match client.synthesize(text, node_id) {
        Ok(ctx) => Ok(ctx),
        Err(_e) => {
            // REQ.06.1: 永続失敗時のフォールバック（CR 無効扱いで継続）。
            eprintln!(
                "warning: contextual retrieval failed for {node_id}; CR を無効扱いで継続（REQ.06.1）"
            );
            Ok(None)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deterministic_client_uses_heading_then_node_id() {
        let c = DeterministicContextClient;
        let with_h = c.synthesize("# タイトル\n\n本文", "UC-LGX-001").unwrap().unwrap();
        assert!(with_h.contains("UC-LGX-001") && with_h.contains("タイトル"));
        let no_h = c.synthesize("見出しなし本文", "DD-LGX-002").unwrap().unwrap();
        assert!(no_h.contains("DD-LGX-002"));
        // 同一入力 → 同一 context（決定性）。
        assert_eq!(
            c.synthesize("# A\n", "X-LGX-001").unwrap(),
            c.synthesize("# A\n", "X-LGX-001").unwrap()
        );
    }
}
