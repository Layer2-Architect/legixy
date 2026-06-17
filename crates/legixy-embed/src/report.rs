// Document ID: SRC-LGX-010
// run_report / ReportOutput / ReportSummary / SkipWarning / SkipReasonSummary / ReportFormat /
// EdgeKind::to_json_str（DD-LGX-010 §2.1・§2.2・§3、v1.2）。
//
// TC[RED] scaffold。report は計測専用（閾値判定なし・severity 概念なし）。skip 検出・理由別集計・
// stderr 集約 Warning は bulk エンジン（DD-007 §6）所有。report 層は skipped: usize（試行 − 算出）
// のみ保持する（DD-010 v1.2）。
//
// SHARED-NEED: `EdgeKind::to_json_str`（DD-LGX-010 §2.2）は EdgeKind の inherent メソッドとして
//   凍結されているが、EdgeKind は legixy-graph 所有（編集禁止）。Rust は外部型に inherent impl を
//   付けられないため、本 crate に拡張トレイト `EdgeKindJson`（free 関数 `to_json_str`）を置く。
//   統合時に legixy-graph 側へ inherent メソッドを移設するか、本トレイトを正式 API とするか裁定要。

use legixy_core::Config;
use legixy_graph::EdgeKind;

use crate::error::ReportError;
use crate::similarity::{CandidateScore, EdgeScore};
use crate::store::EmbeddingStore;

/// report の出力形式（DD-LGX-010 §2.2）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReportFormat {
    Text,
    Json,
}

/// links の統計サマリ（DD-LGX-010 §2.1）。
#[derive(Debug, Clone, PartialEq)]
pub struct ReportSummary {
    pub total_links: usize,
    pub total_candidates: usize,
    pub min_link_score: Option<f32>,
    pub max_link_score: Option<f32>,
    pub mean_link_score: Option<f32>,
}

/// report コマンドの算出結果集約（DD-LGX-010 §2.1、v1.2: skipped: usize）。
#[derive(Debug, Clone, PartialEq)]
pub struct ReportOutput {
    pub links: Vec<EdgeScore>,
    pub candidates: Vec<CandidateScore>,
    pub summary: ReportSummary,
    /// スキップ件数 = 試行エッジ数 − links.len()（report 層で算出可能）。理由別内訳・stderr 集約は
    /// bulk エンジン（DD-007 §6）所有（DD-010 v1.2、ADR-LGX-021 §2.3）。
    pub skipped: usize,
}

/// スキップ集約 Warning エントリ（DD-LGX-010 §2.1。bulk エンジン DD-007 §6 所有の集約型・参照）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SkipWarning {
    pub skip_count: usize,
    pub reasons: SkipReasonSummary,
}

/// スキップ理由別内訳（DD-LGX-010 §2.1。bulk エンジン所有・stderr のみ）。
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SkipReasonSummary {
    pub missing_embedding: usize,
    pub dim_mismatch: usize,
    pub non_finite_score: usize,
}

/// EdgeKind の --json 文字列表現（DD-LGX-010 §2.2、"chain"/"custom"/"parent_child"）。
/// SHARED-NEED: 本来 EdgeKind の inherent メソッド（legixy-graph 所有のため拡張トレイトで提供）。
pub trait EdgeKindJson {
    fn to_json_str(&self) -> &'static str;
}

impl EdgeKindJson for EdgeKind {
    fn to_json_str(&self) -> &'static str {
        match self {
            EdgeKind::Chain => "chain",
            EdgeKind::Custom => "custom",
            EdgeKind::ParentChild => "parent_child",
        }
    }
}

/// JSON 用の有限値フォーマット。非有限は `null`（REQ.09 非有限非出力）。
fn opt_f32_json(v: Option<f32>) -> String {
    match v {
        Some(x) if x.is_finite() => format_f32(x),
        _ => "null".to_string(),
    }
}

/// f32 を JSON 数値リテラルへ（決定的・科学記法回避の素直な表現）。
fn format_f32(x: f32) -> String {
    // Rust の f32 Display は最短往復表現を返し決定的。NaN/Inf は呼出側で除外済。
    let s = format!("{}", x);
    if s.contains('.') || s.contains('e') || s.contains('E') {
        s
    } else {
        // 整数値も JSON 数値として妥当だが、小数点なしの素表現で十分。
        s
    }
}

fn json_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            _ => out.push(c),
        }
    }
    out
}

impl ReportOutput {
    /// v3 実測の text フォーマット（DD-LGX-010 §3、SUPP-010 R-4）。診断 Warning は含まない。
    pub fn to_text(&self) -> String {
        use crate::report::EdgeKindJson;
        let mut out = String::new();
        out.push_str("=== Traceability Report ===\n\n");
        out.push_str(&format!("links: {}\n", self.summary.total_links));
        for e in &self.links {
            out.push_str(&format!(
                "  {} -> {} (score={:.4}, kind={})\n",
                e.from,
                e.to,
                e.score,
                e.edge_kind.to_json_str()
            ));
        }
        out.push_str(&format!("candidates: {}\n", self.summary.total_candidates));
        for c in &self.candidates {
            out.push_str(&format!("  {} <-> {} (score={:.4})\n", c.from, c.to, c.score));
        }
        out.push('\n');
        match (
            self.summary.min_link_score,
            self.summary.max_link_score,
            self.summary.mean_link_score,
        ) {
            (Some(mn), Some(mx), Some(me)) => {
                out.push_str(&format!("min={:.4} max={:.4} mean={:.4}\n", mn, mx, me));
            }
            _ => {
                out.push_str("min=- max=- mean=-\n");
            }
        }
        out
    }

    /// `{"links":[...], "candidates":[...], "summary":{...}}` の構造化 JSON（DD-LGX-010 §3）。
    /// 非有限値非出力（REQ.09）。warnings フィールドなし（C-7）。skipped は stdout に出さない。
    pub fn to_json(&self) -> String {
        let mut s = String::new();
        s.push_str("{\"links\":[");
        let mut first = true;
        for e in &self.links {
            // 非有限スコアのリンクは出力しない（REQ.09）。
            if !e.score.is_finite() {
                continue;
            }
            if !first {
                s.push(',');
            }
            first = false;
            s.push_str(&format!(
                "{{\"from\":\"{}\",\"to\":\"{}\",\"score\":{},\"kind\":\"{}\"}}",
                json_escape(&e.from),
                json_escape(&e.to),
                format_f32(e.score),
                e.edge_kind.to_json_str()
            ));
        }
        s.push_str("],\"candidates\":[");
        let mut first = true;
        for c in &self.candidates {
            if !c.score.is_finite() {
                continue;
            }
            if !first {
                s.push(',');
            }
            first = false;
            s.push_str(&format!(
                "{{\"from\":\"{}\",\"to\":\"{}\",\"score\":{}}}",
                json_escape(&c.from),
                json_escape(&c.to),
                format_f32(c.score)
            ));
        }
        s.push_str("],\"summary\":{");
        s.push_str(&format!("\"total_links\":{},", self.summary.total_links));
        s.push_str(&format!(
            "\"total_candidates\":{},",
            self.summary.total_candidates
        ));
        s.push_str(&format!(
            "\"min_link_score\":{},",
            opt_f32_json(self.summary.min_link_score)
        ));
        s.push_str(&format!(
            "\"max_link_score\":{},",
            opt_f32_json(self.summary.max_link_score)
        ));
        s.push_str(&format!(
            "\"mean_link_score\":{}",
            opt_f32_json(self.summary.mean_link_score)
        ));
        s.push_str("}}");
        s
    }
}

/// DD-007 の compute_edge_scores / compute_link_candidates を呼び ReportOutput へ集約
/// （DD-LGX-010 §3）。skipped = 試行エッジ数 − links.len()。空ストア時は空 ReportOutput（exit 0）。
pub fn run_report(
    graph: &legixy_graph::TraceGraph,
    store: &EmbeddingStore,
    _config: &Config,
) -> Result<ReportOutput, ReportError> {
    let links = crate::similarity::compute_edge_scores(graph, store)
        .map_err(|e| ReportError::GraphLoad(legixy_graph::GraphError::Parse(e.to_string())))?;
    // link 候補の閾値（Config に意味層閾値が無い scaffold では既定 0.7 を用いる）。
    const DEFAULT_LINK_THRESHOLD: f32 = 0.7;
    let candidates = crate::similarity::compute_link_candidates(graph, store, DEFAULT_LINK_THRESHOLD)
        .map_err(|e| ReportError::GraphLoad(legixy_graph::GraphError::Parse(e.to_string())))?;

    // skipped = 試行エッジ数 − links.len()（report 層算出）。
    let tried_edges = graph.edges().len();
    let skipped = tried_edges.saturating_sub(links.len());

    // summary 統計（非有限は除外して算出。空なら None）。
    let finite_scores: Vec<f32> = links
        .iter()
        .map(|e| e.score)
        .filter(|s| s.is_finite())
        .collect();
    let (min, max, mean) = if finite_scores.is_empty() {
        (None, None, None)
    } else {
        let sum: f32 = finite_scores.iter().sum();
        (
            Some(finite_scores.iter().cloned().fold(f32::INFINITY, f32::min)),
            Some(
                finite_scores
                    .iter()
                    .cloned()
                    .fold(f32::NEG_INFINITY, f32::max),
            ),
            Some(sum / finite_scores.len() as f32),
        )
    };

    let total_links = links.len();
    let total_candidates = candidates.len();
    Ok(ReportOutput {
        links,
        candidates,
        summary: ReportSummary {
            total_links,
            total_candidates,
            min_link_score: min,
            max_link_score: max,
            mean_link_score: mean,
        },
        skipped,
    })
}
