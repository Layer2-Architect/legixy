// (module of SRC-LGX-007; anchor: orchestrator.rs)
// bulk similarity エンジン: compute_edge_scores / compute_link_candidates /
// compute_all_pair_scores / cosine_similarity / EdgeScore / CandidateScore / AggregatedWarnings
// （DD-LGX-007 §2.1・§3、本 crate 所有・report=DD-010 が consumer、ADR-LGX-021 §2.3）。
//
// 【上流裁定の反映 / cross-DD 署名衝突】
//   本ファイルの `compute_all_pair_scores` は **DD-007 正典（タプル列・EmbedError）** を採用する。
//   DD-011（calibrate 所有）の `AllPairScores` 返却版は calibrate.rs の
//   `compute_all_pair_scores_calibrate` に置く。

use std::collections::{HashMap, HashSet};

use legixy_graph::{EdgeKind, TraceGraph};

use crate::error::EmbedError;
use crate::store::{EmbeddingRow, EmbeddingStore};

/// bulk API: エッジ類似度スコア 1 件（DD-LGX-007 §2.1、REQ.11。legixy-embed 所有共有型）。
#[derive(Debug, Clone, PartialEq)]
pub struct EdgeScore {
    pub from: String,
    pub to: String,
    pub score: f32,
    pub edge_kind: EdgeKind,
}

/// bulk API: リンク候補スコア 1 件（DD-LGX-007 §2.1、REQ.11。legixy-embed 所有共有型）。
#[derive(Debug, Clone, PartialEq)]
pub struct CandidateScore {
    pub from: String,
    pub to: String,
    pub score: f32,
}

/// 集約 Warning の収集バッファ（DD-LGX-007 §2.1。ノード毎でなく 1 件まとめ出力のため）。
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AggregatedWarnings {
    pub dim_mismatch_count: usize,
    pub zero_norm_count: usize,
    pub truncated_count: usize,
    pub empty_skip_count: usize,
}

/// 次元不一致・未登録ペアは skip。出力順は graph.edges() 挿入順（DD-LGX-007 §3、REQ.11）。
pub fn compute_edge_scores(
    graph: &TraceGraph,
    store: &EmbeddingStore,
) -> Result<Vec<EdgeScore>, EmbedError> {
    let rows = store.load_all()?;
    let by_id: HashMap<&str, &EmbeddingRow> =
        rows.iter().map(|r| (r.node_id.as_str(), r)).collect();

    let mut scores = Vec::new();
    for edge in graph.edges() {
        let a = match by_id.get(edge.from.as_str()) {
            Some(r) => r,
            None => continue,
        };
        let b = match by_id.get(edge.to.as_str()) {
            Some(r) => r,
            None => continue,
        };
        if a.dim != b.dim || a.dim == 0 {
            continue;
        }
        let score = cosine_similarity(&a.embedding, &b.embedding);
        scores.push(EdgeScore {
            from: edge.from.clone(),
            to: edge.to.clone(),
            score,
            edge_kind: edge.kind,
        });
    }
    Ok(scores)
}

/// 次元不一致は skip。O(N²)。出力順は (from, to) 昇順（DD-LGX-007 §3、REQ.11）。
/// `score >= threshold` 境界（= 含む / < 除外。v3 similarity.rs L131）。
pub fn compute_link_candidates(
    graph: &TraceGraph,
    store: &EmbeddingStore,
    threshold: f32,
) -> Result<Vec<CandidateScore>, EmbedError> {
    let rows = store.load_all()?;

    // 既存エッジを無向 HashSet 化（from↔to 両方向）。
    let mut existing: HashSet<(String, String)> = HashSet::new();
    for e in graph.edges() {
        existing.insert((e.from.clone(), e.to.clone()));
        existing.insert((e.to.clone(), e.from.clone()));
    }

    let mut candidates = Vec::new();
    for i in 0..rows.len() {
        for j in (i + 1)..rows.len() {
            let a = &rows[i];
            let b = &rows[j];
            if a.dim != b.dim || a.dim == 0 {
                continue;
            }
            if existing.contains(&(a.node_id.clone(), b.node_id.clone())) {
                continue;
            }
            let score = cosine_similarity(&a.embedding, &b.embedding);
            if score >= threshold {
                candidates.push(CandidateScore {
                    from: a.node_id.clone(),
                    to: b.node_id.clone(),
                    score,
                });
            }
        }
    }
    Ok(candidates)
}

/// 次元不一致は skip。i < j の昇順（DD-LGX-007 §3 正典、REQ.11）。
pub fn compute_all_pair_scores(
    store: &EmbeddingStore,
) -> Result<Vec<(String, String, f32)>, EmbedError> {
    let rows = store.load_all()?;
    let mut scores = Vec::new();
    for i in 0..rows.len() {
        for j in (i + 1)..rows.len() {
            let a = &rows[i];
            let b = &rows[j];
            if a.dim != b.dim || a.dim == 0 {
                continue;
            }
            let score = cosine_similarity(&a.embedding, &b.embedding);
            scores.push((a.node_id.clone(), b.node_id.clone(), score));
        }
    }
    Ok(scores)
}

/// 値域 [-1.0, 1.0] に clamp（DD-LGX-007 §3、REQ.04 GAP-LGX-105）。
/// ゼロノルム時は 0.0（v3 継承挙動。呼出側で skip 集計対象）。
pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() {
        return 0.0;
    }
    let mut dot = 0.0_f32;
    let mut norm_a = 0.0_f32;
    let mut norm_b = 0.0_f32;
    for (x, y) in a.iter().zip(b.iter()) {
        dot += x * y;
        norm_a += x * x;
        norm_b += y * y;
    }
    let na = norm_a.sqrt();
    let nb = norm_b.sqrt();
    if na == 0.0 || nb == 0.0 {
        0.0
    } else {
        (dot / (na * nb)).clamp(-1.0, 1.0)
    }
}
