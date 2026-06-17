// refresh-subnodes 実装（ADR-LGX-023、LGX-COMPAT-001 §3 #9）。
// v3 `te-cli/commands/refresh_subnodes.rs` のアルゴリズムを legixy の永続モデル（engine.db
// embeddings の subnode 行）へ移植する。
//
// 動作:
//   1. 各ドキュメントノードのファイルを再走査し、現サブノード集合（id + anchor）を抽出（既定 h2/h3）。
//   2. engine.db に永続化済みのサブノード行（is_subnode=1、id + anchor）と差分を取る。
//   3. removed / added を anchor の Levenshtein 距離で対応付け（rename 検出）。残りは orphan。
//   4. dry-run はレポートのみ。apply は embeddings.node_id を old→new に更新（呼出側がバックアップ）。

use std::collections::HashMap;
use std::path::Path;

use legixy_embed::EmbeddingStore;
use legixy_graph::subnode::extract_subnodes_with_levels;
use legixy_graph::TraceGraph;

/// 既定の見出しレベル（Node が heading_levels を持たないため h2/h3 固定、ADR-LGX-023）。
const DEFAULT_HEADING_LEVELS: [u8; 2] = [2, 3];

#[derive(Debug, Clone)]
pub struct SubnodeRename {
    pub old_id: String,
    pub new_id: String,
    pub parent_id: String,
    pub old_anchor: Option<String>,
    pub new_anchor: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrphanKind {
    /// engine.db に存在するが現ファイル抽出に出ない（削除候補）。
    Removed,
    /// 現ファイル抽出に出るが engine.db に存在しない（追加候補）。
    Added,
}

#[derive(Debug, Clone)]
pub struct OrphanSubnode {
    pub id: String,
    pub parent_id: String,
    pub anchor: Option<String>,
    pub kind: OrphanKind,
}

#[derive(Debug, Default)]
pub struct RefreshReport {
    pub renames: Vec<SubnodeRename>,
    pub orphans: Vec<OrphanSubnode>,
    pub parents_scanned: usize,
}

/// graph（document nodes）と engine.db（永続サブノード）を比較し rename / orphan を抽出する。
/// 決定論: renames は old_id ASC、orphans は (parent_id, id) ASC。
pub fn detect_changes(
    graph: &TraceGraph,
    store: &EmbeddingStore,
    project_root: &Path,
) -> Result<RefreshReport, String> {
    let mut report = RefreshReport::default();

    // 永続サブノードを parent_id でグループ化。
    let persisted = store
        .list_subnodes()
        .map_err(|e| format!("list_subnodes failed: {e}"))?;
    let mut by_parent: HashMap<String, Vec<(String, Option<String>)>> = HashMap::new();
    for s in persisted {
        if let Some(pid) = s.parent_id {
            by_parent.entry(pid).or_default().push((s.node_id, s.anchor));
        }
    }

    for doc in graph.document_nodes() {
        if !doc.path.ends_with(".md") {
            continue;
        }
        report.parents_scanned += 1;
        let abs = project_root.join(&doc.path);
        let content = match std::fs::read_to_string(&abs) {
            Ok(c) => c,
            Err(_) => continue, // 部分失敗トレランス（ファイル不在は skip）。
        };

        // 現ファイル抽出（id → anchor）。
        let extracted: HashMap<String, String> =
            extract_subnodes_with_levels(&doc.id, &content, &DEFAULT_HEADING_LEVELS)
                .into_iter()
                .map(|s| (s.id, s.anchor))
                .collect();
        // 永続（id → anchor）。
        let persisted_map: HashMap<String, Option<String>> =
            by_parent.get(&doc.id).cloned().unwrap_or_default().into_iter().collect();

        let mut removed: Vec<(String, Option<String>)> = persisted_map
            .iter()
            .filter(|(id, _)| !extracted.contains_key(*id))
            .map(|(id, a)| (id.clone(), a.clone()))
            .collect();
        let mut added: Vec<(String, String)> = extracted
            .iter()
            .filter(|(id, _)| !persisted_map.contains_key(*id))
            .map(|(id, a)| (id.clone(), a.clone()))
            .collect();

        // anchor Levenshtein 最小ペアから rename 化（v3 アルゴリズム）。
        loop {
            if removed.is_empty() || added.is_empty() {
                break;
            }
            let mut best: Option<(usize, usize, usize)> = None;
            for (ri, (_, ra)) in removed.iter().enumerate() {
                for (ai, (_, aa)) in added.iter().enumerate() {
                    let d = levenshtein(ra.as_deref().unwrap_or(""), aa);
                    if best.map(|(_, _, b)| d < b).unwrap_or(true) {
                        best = Some((ri, ai, d));
                    }
                }
            }
            let Some((ri, ai, _)) = best else { break };
            let (old_id, old_anchor) = removed.remove(ri);
            let (new_id, new_anchor) = added.remove(ai);
            report.renames.push(SubnodeRename {
                old_id,
                new_id,
                parent_id: doc.id.clone(),
                old_anchor,
                new_anchor,
            });
        }

        removed.sort();
        added.sort();
        for (id, anchor) in removed {
            report.orphans.push(OrphanSubnode {
                id,
                parent_id: doc.id.clone(),
                anchor,
                kind: OrphanKind::Removed,
            });
        }
        for (id, anchor) in added {
            report.orphans.push(OrphanSubnode {
                id,
                parent_id: doc.id.clone(),
                anchor: Some(anchor),
                kind: OrphanKind::Added,
            });
        }
    }

    report.renames.sort_by(|a, b| a.old_id.cmp(&b.old_id));
    report
        .orphans
        .sort_by(|a, b| (&a.parent_id, &a.id).cmp(&(&b.parent_id, &b.id)));
    Ok(report)
}

/// rename を engine.db に適用（embeddings.node_id 更新）。適用件数を返す（ADR-LGX-023）。
pub fn apply_renames(store: &EmbeddingStore, report: &RefreshReport) -> Result<usize, String> {
    let mut applied = 0;
    for r in &report.renames {
        store
            .rename_subnode(&r.old_id, &r.new_id)
            .map_err(|e| format!("rename {} → {} failed: {e}", r.old_id, r.new_id))?;
        applied += 1;
    }
    Ok(applied)
}

/// レポートを人間可読テキストへ整形。
pub fn render(report: &RefreshReport, applied: bool) -> String {
    let mut s = String::from("=== refresh-subnodes ");
    s.push_str(if applied { "(apply) ===\n" } else { "(dry-run) ===\n" });
    s.push_str(&format!("parents_scanned: {}\n", report.parents_scanned));
    s.push_str(&format!("renames: {}\n", report.renames.len()));
    for r in &report.renames {
        s.push_str(&format!(
            "  {} → {}  (parent={}, anchor: {:?} → {:?})\n",
            r.old_id, r.new_id, r.parent_id, r.old_anchor, r.new_anchor
        ));
    }
    s.push_str(&format!("orphans: {}\n", report.orphans.len()));
    for o in &report.orphans {
        let k = match o.kind {
            OrphanKind::Removed => "removed",
            OrphanKind::Added => "added",
        };
        s.push_str(&format!(
            "  [{k}] {} (parent={}, anchor={:?})\n",
            o.id, o.parent_id, o.anchor
        ));
    }
    if !applied && (!report.renames.is_empty() || !report.orphans.is_empty()) {
        s.push_str("\n--apply で embeddings の subnode ID を更新します（事前に engine.db をバックアップ）。\n");
    }
    s
}

/// 単純なレーベンシュタイン距離（O(m*n) DP、anchor 文字列向け。v3 底本移植）。
fn levenshtein(a: &str, b: &str) -> usize {
    let av: Vec<char> = a.chars().collect();
    let bv: Vec<char> = b.chars().collect();
    let (m, n) = (av.len(), bv.len());
    if m == 0 {
        return n;
    }
    if n == 0 {
        return m;
    }
    let mut prev: Vec<usize> = (0..=n).collect();
    let mut curr: Vec<usize> = vec![0; n + 1];
    for i in 1..=m {
        curr[0] = i;
        for j in 1..=n {
            let cost = if av[i - 1] == bv[j - 1] { 0 } else { 1 };
            curr[j] = (curr[j - 1] + 1).min(prev[j] + 1).min(prev[j - 1] + cost);
        }
        std::mem::swap(&mut prev, &mut curr);
    }
    prev[n]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn levenshtein_basic() {
        assert_eq!(levenshtein("", ""), 0);
        assert_eq!(levenshtein("abc", "abc"), 0);
        assert_eq!(levenshtein("abc", "abd"), 1);
        assert_eq!(levenshtein("kitten", "sitting"), 3);
        assert_eq!(levenshtein("", "xyz"), 3);
    }
}
