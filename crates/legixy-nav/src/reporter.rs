// (module of SRC-LGX-005; anchor: investigate.rs)
// legixy-nav Reporter: 出力整形（Text / JsonLines）と打ち切り可観測性。
// DD-LGX-005 §2.2 / §3（render_pruned / render_outcome / render_multi / ReportFormat）正典、
// DD-LGX-006 §3（render_multi / emit_truncation_info / detect_truncation）追加。
//
// 書式定義は DD-005 §3「出力書式の凍結」/ DD-006 §3 render_multi 行に厳密整合（v3 reporter.rs 整合）。

use std::collections::HashSet;

use serde_json::json;

use legixy_graph::TraceGraph;

use crate::result::{
    InvestigateOutcome, MultiTraversalResult, PrunedTraversalResult, SuspiciousNode, TruncationInfo,
    VisitedNode,
};

/// Reporter の出力形式（DD-LGX-005 §2.2 正典 / DD-LGX-006 §2.2 と 1:1）。
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ReportFormat {
    /// 既定。人間可読テキスト（v3 互換書式）。
    #[default]
    Text,
    /// --json 指定時。JSON Lines（SPEC-LGX-005.REQ.09 機能化、v3 差分）。
    JsonLines,
}

/// drift 枝刈り済み結果の整形（DD-LGX-005 §3）。打ち切り**非発生**時用。
/// Text: v3 互換書式（SUPP-005 §2.3）。JsonLines: v3 reporter.rs JSON Lines 機能化（REQ.09）。
pub fn render_pruned(result: &PrunedTraversalResult, format: ReportFormat) -> String {
    match format {
        ReportFormat::Text => {
            let mut out = String::new();
            for v in &result.traversal.visited {
                push_visited_text(&mut out, v);
            }
            out.push_str(&format!(
                "Suspicious nodes (drift_threshold={}):\n",
                result.drift_threshold
            ));
            for sn in &result.suspicious_nodes {
                out.push_str(&format!(
                    "{} (drift={}, type={}, path={})\n",
                    sn.id, sn.drift_score, sn.type_code, sn.path
                ));
            }
            out.push_str(&format!(
                "Summary: visited={}, suspicious={}\n",
                result.traversal.visited.len(),
                result.suspicious_nodes.len()
            ));
            out
        }
        ReportFormat::JsonLines => {
            let mut out = String::new();
            for v in &result.traversal.visited {
                push_visited_json(&mut out, v);
            }
            for sn in &result.suspicious_nodes {
                push_suspicious_json(&mut out, sn);
            }
            let summary = json!({
                "summary": {
                    "visited": result.traversal.visited.len(),
                    "suspicious": result.suspicious_nodes.len(),
                    "drift_threshold": f32_to_clean_f64(result.drift_threshold),
                }
            });
            out.push_str(&summary.to_string());
            out.push('\n');
            out
        }
    }
}

/// 打ち切り可観測性付き整形（DD-LGX-005 §3、v1.1 新設）。
/// `render_pruned(&outcome.result, format)` を基に、`outcome.truncated == true` のとき
/// summary 行へ `"truncated":true,"excluded":K`（JsonLines）/ 相当の Text 注記を加える。
/// `truncated == false` のとき出力は `render_pruned` と同一。
pub fn render_outcome(outcome: &InvestigateOutcome, format: ReportFormat) -> String {
    if !outcome.truncated {
        return render_pruned(&outcome.result, format);
    }

    match format {
        ReportFormat::Text => {
            // truncated=false 時と同じ本体に Text 注記を 1 行追加。
            let mut out = render_pruned(&outcome.result, format);
            out.push_str(&format!(
                "Truncated: max-depth reached; {} reachable node(s) excluded\n",
                outcome.excluded_count
            ));
            out
        }
        ReportFormat::JsonLines => {
            // summary 行へ truncated/excluded を加える（render_pruned の summary を再構築）。
            let result = &outcome.result;
            let mut out = String::new();
            for v in &result.traversal.visited {
                push_visited_json(&mut out, v);
            }
            for sn in &result.suspicious_nodes {
                push_suspicious_json(&mut out, sn);
            }
            let summary = json!({
                "summary": {
                    "visited": result.traversal.visited.len(),
                    "suspicious": result.suspicious_nodes.len(),
                    "drift_threshold": f32_to_clean_f64(result.drift_threshold),
                    "truncated": true,
                    "excluded": outcome.excluded_count,
                }
            });
            out.push_str(&summary.to_string());
            out.push('\n');
            out
        }
    }
}

/// 多起点走査結果の整形（DD-LGX-006 §3、impact コマンド用。DD-005 §3 から参照可）。
/// Text: `{id} (type={t}, depth={d}, path={p})` 各行 + `Summary: visited={n}` 末尾。
/// JsonLines: visited 各行 `{"id","type","depth","path"}` + `{"summary":{"visited":n}}` 末尾。
pub fn render_multi(result: &MultiTraversalResult, format: ReportFormat) -> String {
    match format {
        ReportFormat::Text => {
            let mut out = String::new();
            for v in &result.visited {
                push_visited_text(&mut out, v);
            }
            out.push_str(&format!("Summary: visited={}\n", result.visited.len()));
            out
        }
        ReportFormat::JsonLines => {
            let mut out = String::new();
            for v in &result.visited {
                push_visited_json(&mut out, v);
            }
            let summary = json!({
                "summary": { "visited": result.visited.len() }
            });
            out.push_str(&summary.to_string());
            out.push('\n');
            out
        }
    }
}

/// 打ち切り Info の stderr 出力（DD-LGX-006 §3、REQ.04 打ち切り可観測性、v3 差分）。
/// `info.truncated` のとき stderr へ
/// `[nav] info: max-depth {N} truncated traversal; {k} reachable node(s) excluded` を出力。
/// stdout・終了コード不変。副作用: stderr。
pub fn emit_truncation_info(info: &TruncationInfo) {
    if info.truncated {
        eprintln!(
            "[nav] info: max-depth {} truncated traversal; {} reachable node(s) excluded",
            info.max_depth, info.excluded_count
        );
    }
}

/// 打ち切り検出（DD-LGX-006 §3 / §6 算定法）。
/// `result` の境界深度ノード（`depth_map[id] == max_depth`）の未訪問隣接ノード数を集計して
/// `excluded_count` とする。stdout 不変・走査コスト小・純関数。
pub fn detect_truncation(
    graph: &TraceGraph,
    result: &MultiTraversalResult,
    max_depth: usize,
) -> TruncationInfo {
    let visited: HashSet<&str> = result.visited.iter().map(|v| v.id.as_str()).collect();
    let mut excluded_count = 0usize;
    for v in &result.visited {
        if v.depth != max_depth {
            continue;
        }
        // 順方向の境界ノード v から出る未訪問隣接（from==v.id の edge.to）を集計。
        for edge in graph.edges() {
            if edge.from == v.id && !visited.contains(edge.to.as_str()) {
                excluded_count += 1;
            }
        }
    }
    TruncationInfo {
        truncated: excluded_count > 0,
        excluded_count,
        max_depth,
    }
}

fn push_visited_text(out: &mut String, v: &VisitedNode) {
    out.push_str(&format!(
        "{} (type={}, depth={}, path={})\n",
        v.id, v.type_code, v.depth, v.path
    ));
}

fn push_visited_json(out: &mut String, v: &VisitedNode) {
    let value = json!({
        "id": v.id,
        "type": v.type_code,
        "depth": v.depth,
        "path": v.path,
    });
    out.push_str(&value.to_string());
    out.push('\n');
}

fn push_suspicious_json(out: &mut String, sn: &SuspiciousNode) {
    let value = json!({
        "suspicious": {
            "id": sn.id,
            "drift": f32_to_clean_f64(sn.drift_score),
            "type": sn.type_code,
            "path": sn.path,
        }
    });
    out.push_str(&value.to_string());
    out.push('\n');
}

/// f32 の Display 表現を経由して f64 に変換する。
///
/// 目的: serde_json は数値を f64 で保持するため、f32 をそのまま `as f64` で
/// 広げると 0.3 → 0.30000001192092896 のような末尾桁が露出する。Display 経由で
/// 丸められた短い 10 進表現を一度得てから f64 に再 parse することで、
/// JSON 出力が `0.3` のような短い表現になる。
fn f32_to_clean_f64(v: f32) -> f64 {
    format!("{}", v).parse::<f64>().unwrap_or(v as f64)
}
