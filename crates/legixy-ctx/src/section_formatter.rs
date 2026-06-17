// (module of SRC-LGX-002; anchor: compiler.rs)
// legixy-ctx::section_formatter — SectionFormatter（DD-LGX-002 §3 / DD-LGX-004 §2.2/§3）
//
// TC[RED] scaffold。
//  - render / enforce_size_limit はロジックを持つため todo!()。
//  - upstream_sort_rule / RENDER_SORT_STRATEGY は純データで実体（DD-LGX-004 §2.2、A-1 裁定）。
//
// REQ.10-14 / CACHE-INV-1/2/3。6 セクション整列・キャッシュブレーク点マーカ・サイズ上限。

use crate::compiler::Granularity;
use crate::error::ContextError;
use crate::result::{
    ContextResult, CustomDocument, LayerDocument, TargetNodeMetadata, UpstreamArtifact,
};
use crate::{CACHE_BREAKPOINT_MARKER, RESULT_SIZE_LIMIT_CHARS};

/// 6 セクション整列・マーカ・サイズ上限を担保するフォーマッタ（DD-LGX-002 §3）。
pub struct SectionFormatter;

impl SectionFormatter {
    /// DD-LGX-002 §3 / DD-LGX-004 §3 凍結。6 セクション・バイト決定論・LF 固定。
    /// 500,000 文字超過で `ResultTooLarge`（CACHE-INV-3）。
    /// 整列は index 配列（Vec<usize>）で行い Vec<T>::clone を発生させない（v3 §10.3 底本）。
    pub fn render(result: &ContextResult) -> Result<String, ContextError> {
        let mut out = String::new();
        let mut current_chars: usize = 0;

        // ① Layer Guidelines（file_path バイト昇順）
        let layer_order = sorted_indices(&result.layer_guidelines, |a, b| {
            a.file_path
                .to_string_lossy()
                .as_bytes()
                .cmp(b.file_path.to_string_lossy().as_bytes())
        });
        render_indexed_section_block(
            &mut out,
            &mut current_chars,
            "Layer Guidelines",
            &result.layer_guidelines,
            &layer_order,
            render_layer_entry,
        )?;

        // ② Additional Guidelines（file_path バイト昇順）
        let additional_order = sorted_indices(&result.additional_guidelines, |a, b| {
            a.file_path
                .to_string_lossy()
                .as_bytes()
                .cmp(b.file_path.to_string_lossy().as_bytes())
        });
        render_indexed_section_block(
            &mut out,
            &mut current_chars,
            "Additional Guidelines",
            &result.additional_guidelines,
            &additional_order,
            render_layer_entry,
        )?;

        // キャッシュブレーク点マーカ（REQ.12 / CACHE-INV-2、安定部 ↔ 上流部の境界）
        append_and_count(&mut out, &mut current_chars, CACHE_BREAKPOINT_MARKER);
        append_and_count(&mut out, &mut current_chars, "\n\n");
        check_early_cut(current_chars)?;

        // ③ Upstream Artifacts
        //    Document: artifact_id バイト昇順。
        //    Subnode (A-1 裁定 2026-06-13): parent_id(artifact_id) バイト昇順 + アンカー出現順。
        //    出現順 = 入力 Vec の挿入順（subnodes_of の物理位置順）。sort_by は安定ソートのため
        //    同一 parent_id 内では入力順（= 出現順）が保存される。v3 の anchor バイト辞書順ではない。
        let upstream_order = match result.granularity {
            Granularity::Document => sorted_indices(&result.upstream, |a, b| {
                a.artifact_id.as_bytes().cmp(b.artifact_id.as_bytes())
            }),
            Granularity::Subnode => sorted_indices(&result.upstream, |a, b| {
                a.artifact_id.as_bytes().cmp(b.artifact_id.as_bytes())
            }),
        };
        render_indexed_section_block(
            &mut out,
            &mut current_chars,
            "Upstream Artifacts",
            &result.upstream,
            &upstream_order,
            render_upstream_entry,
        )?;

        // ④ Target Node Metadata（artifact_id バイト昇順）
        let metadata_order = sorted_indices(&result.target_metadata, |a, b| {
            a.artifact_id.as_bytes().cmp(b.artifact_id.as_bytes())
        });
        render_indexed_section_block(
            &mut out,
            &mut current_chars,
            "Target Node Metadata",
            &result.target_metadata,
            &metadata_order,
            render_metadata_entry,
        )?;

        // ⑤ Custom Documents（from_id → to_id バイト昇順）
        let custom_order = sorted_indices(&result.custom_documents, |a, b| {
            a.from_id
                .as_bytes()
                .cmp(b.from_id.as_bytes())
                .then_with(|| a.to_id.as_bytes().cmp(b.to_id.as_bytes()))
        });
        render_indexed_section_block(
            &mut out,
            &mut current_chars,
            "Custom Documents",
            &result.custom_documents,
            &custom_order,
            render_custom_entry,
        )?;

        // safety net（defence-in-depth、CACHE-INV-3）
        Self::enforce_size_limit(&out)?;
        Ok(out)
    }

    /// DD-LGX-002 §3 / DD-LGX-004 §3 凍結。`rendered.chars().count() > 500_000` で
    /// `ResultTooLarge`（defence-in-depth、CACHE-INV-3）。判定単位 = Unicode コードポイント。
    pub fn enforce_size_limit(rendered: &str) -> Result<(), ContextError> {
        let n = rendered.chars().count();
        if n > RESULT_SIZE_LIMIT_CHARS {
            return Err(ContextError::ResultTooLarge {
                current: n,
                limit: RESULT_SIZE_LIMIT_CHARS,
            });
        }
        Ok(())
    }

    /// --granularity に応じた Upstream Artifacts 整列規則名（DD-LGX-004 §2.2、テスト用）。
    /// A-1 裁定（2026-06-13）: Subnode は **アンカー出現順**（v3 バイト辞書順ではない）。
    pub fn upstream_sort_rule(granularity: Granularity) -> &'static str {
        match granularity {
            Granularity::Document => "artifact_id-asc",
            Granularity::Subnode => "parent_id-asc,anchor-appearance-order",
        }
    }

    /// render の整列戦略名（DD-LGX-004 §2.2）。
    pub const RENDER_SORT_STRATEGY: &'static str = "index-array";
}

/// slice を比較関数で安定ソートした index 配列を返す（Vec<T>::clone 非発生）。
fn sorted_indices<T, F>(slice: &[T], mut cmp: F) -> Vec<usize>
where
    F: FnMut(&T, &T) -> std::cmp::Ordering,
{
    let mut idx: Vec<usize> = (0..slice.len()).collect();
    idx.sort_by(|&a, &b| cmp(&slice[a], &slice[b]));
    idx
}

fn render_indexed_section_block<T>(
    out: &mut String,
    current_chars: &mut usize,
    title: &str,
    entries: &[T],
    order: &[usize],
    render_entry: fn(&T) -> String,
) -> Result<(), ContextError> {
    append_and_count(out, current_chars, "# ");
    append_and_count(out, current_chars, title);
    append_and_count(out, current_chars, "\n\n");
    for (i, &idx) in order.iter().enumerate() {
        if i > 0 {
            append_and_count(out, current_chars, "\n---\n");
        }
        let s = render_entry(&entries[idx]);
        append_and_count(out, current_chars, &s);
    }
    if !order.is_empty() {
        append_and_count(out, current_chars, "\n");
    }
    append_and_count(out, current_chars, "\n");
    check_early_cut(*current_chars)
}

fn append_and_count(out: &mut String, current_chars: &mut usize, s: &str) {
    out.push_str(s);
    *current_chars += s.chars().count();
}

fn check_early_cut(current_chars: usize) -> Result<(), ContextError> {
    if current_chars > RESULT_SIZE_LIMIT_CHARS {
        return Err(ContextError::ResultTooLarge {
            current: current_chars,
            limit: RESULT_SIZE_LIMIT_CHARS,
        });
    }
    Ok(())
}

fn render_layer_entry(doc: &LayerDocument) -> String {
    let mut s = String::new();
    s.push_str(&format!("layer: {}\n", doc.layer_name));
    s.push_str(&format!("node_id: {}\n", doc.node_id));
    s.push_str(&format!(
        "file_path: {}\n",
        doc.file_path.to_string_lossy().replace('\\', "/")
    ));
    s.push_str(&format!("specificity: {}\n", doc.specificity));
    s.push_str(&format!("priority: {}\n", doc.priority));
    s.push('\n');
    s.push_str(&doc.body);
    s
}

fn render_upstream_entry(entry: &UpstreamArtifact) -> String {
    let mut s = String::new();
    s.push_str(&format!("artifact_id: {}\n", entry.artifact_id));
    s.push_str(&format!("type: {}\n", entry.type_code));
    s.push_str(&format!(
        "file_path: {}\n",
        entry.file_path.to_string_lossy().replace('\\', "/")
    ));
    s.push_str(&format!("chain_distance: {}\n", entry.chain_distance));
    if let Some(sid) = &entry.subnode_id {
        s.push_str(&format!("subnode_id: {}\n", sid));
    }
    if let Some(anchor) = &entry.anchor {
        s.push_str(&format!("anchor: {}\n", anchor));
    }
    if let Some(ds) = entry.drift_score {
        s.push_str(&format!("drift_score: {}\n", ds));
    }
    s.push('\n');
    s.push_str(&entry.body);
    s
}

fn render_metadata_entry(entry: &TargetNodeMetadata) -> String {
    let mut s = String::new();
    s.push_str(&format!("artifact_id: {}\n", entry.artifact_id));
    s.push_str(&format!("outgoing_edges: {}\n", entry.outgoing_edges.len()));
    s.push_str(&format!("incoming_edges: {}\n", entry.incoming_edges.len()));
    s.push_str(&format!("subnode_count: {}", entry.subnode_count));
    // REQ.20 / S2-24 / S2-26: 未解決起点を末尾に決定論記録（PathBuf 辞書順昇順）。
    // 空の場合はフィールド自体を省略しバイト決定論（REQ.14）を保全する。
    if !entry.unresolved_targets.is_empty() {
        let mut sorted: Vec<String> = entry
            .unresolved_targets
            .iter()
            .map(|p| p.to_string_lossy().replace('\\', "/"))
            .collect();
        sorted.sort();
        s.push_str(&format!("\nunresolved_targets: {}", sorted.join(",")));
    }
    s
}

fn render_custom_entry(entry: &CustomDocument) -> String {
    let mut s = String::new();
    s.push_str(&format!("from_id: {}\n", entry.from_id));
    s.push_str(&format!("to_id: {}\n", entry.to_id));
    s.push_str(&format!(
        "file_path: {}\n",
        entry.file_path.to_string_lossy().replace('\\', "/")
    ));
    s.push_str(&format!("reason: {}\n", entry.reason.as_deref().unwrap_or("")));
    s.push('\n');
    s.push_str(&entry.body);
    s
}
