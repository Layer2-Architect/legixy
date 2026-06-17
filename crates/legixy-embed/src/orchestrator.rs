// Document ID: SRC-LGX-007
// embed_all / EmbedOptions / EmbedReport / EmbedErrorItem / NodeFilter / HashMatchState
// （DD-LGX-007 §2.1・§2.2・§3）。
//
// TC[RED] scaffold。部分失敗継続・ノード単位 Tx 呼出し・空テキスト skip・集約 Warning は
// SRC[GREEN] で実装する。

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use legixy_graph::{Node, TraceGraph};

use crate::content::read_current_content_for_node;
use crate::contextual::ContextualConfig;
use crate::embedder::Embedder;
use crate::error::EmbedError;
use crate::store::EmbeddingStore;

/// --all / --node <ID>+ の排他選択（DD-LGX-007 §2.2、REQ.02）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NodeFilter {
    All,
    Ids(Vec<String>),
}

/// ハッシュ照合結果の 3 状態（DD-LGX-007 §2.2、SCORE-INV-1 + SCORE-INV-2）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HashMatchState {
    /// content_hash + model_version が一致 → 再計算不要。
    Skip,
    /// content_hash 不一致（stale）または model_version 不一致。
    Regen,
    /// embeddings 行が存在しない（未生成）。
    Missing,
}

/// embed_all の振る舞い制御オプション（DD-LGX-007 §2.1）。
pub struct EmbedOptions {
    pub force: bool,
    pub include_subnodes: bool,
    pub contextual: Option<ContextualConfig>,
    pub project_root: Option<PathBuf>,
    pub node_filter: NodeFilter,
}

impl Default for EmbedOptions {
    fn default() -> Self {
        Self {
            force: false,
            include_subnodes: true,
            contextual: None,
            project_root: None,
            node_filter: NodeFilter::All,
        }
    }
}

/// errors 配列の 1 要素（DD-LGX-007 §2.1、--json スキーマ）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EmbedErrorItem {
    pub node_id: String,
    pub message: String,
}

/// embed_all の実行結果サマリ（DD-LGX-007 §2.1、--json スキーマ。v3 差分: failed + errors オブジェクト）。
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct EmbedReport {
    pub generated: usize,
    pub skipped: usize,
    pub failed: usize,
    pub errors: Vec<EmbedErrorItem>,
}

/// 全ノードの embedding を生成・格納する（DD-LGX-007 §3）。
/// ノード単位 Tx・部分失敗継続・DB 書込失敗のみ Err 昇格。EmbedReport.failed == errors.len()。
pub fn embed_all(
    graph: &TraceGraph,
    store: &EmbeddingStore,
    embedder: &Embedder,
    options: EmbedOptions,
) -> Result<EmbedReport, EmbedError> {
    // 対象ノードを決定（--all はグラフ全件、--node は ID 指定。未登録 ID は NodeNotFound で即 Err）。
    // --all では自動生成サブノード（`{parent}#{16hex}`）を除外する: parse_graph が materialize した
    // auto サブノードはドキュメントノード経由の embed_subnodes が区画スライスで所有するため、main loop で
    // も処理すると全文で上書きしてしまう（BUG-007 E2E 配線=load→parse_graph の副作用回避）。
    // 明示 `#s:` サブノードは embed_subnodes 非対象のため main loop に残す（従来挙動）。
    let targets: Vec<Node> = match &options.node_filter {
        NodeFilter::All => graph
            .nodes()
            .filter(|n| !is_auto_generated_subnode(&n.id))
            .cloned()
            .collect(),
        NodeFilter::Ids(ids) => {
            let mut v = Vec::with_capacity(ids.len());
            for id in ids {
                match graph.node(id) {
                    Some(n) => v.push(n.clone()),
                    None => return Err(EmbedError::NodeNotFound(id.clone())),
                }
            }
            v
        }
    };

    let project_root: &Path = options
        .project_root
        .as_deref()
        .unwrap_or_else(|| Path::new("."));

    let mut report = EmbedReport::default();

    for node in &targets {
        // content_range 切り出し（embed_all と共有経路）。ファイル読込不能は空テキスト扱い。
        let content =
            read_current_content_for_node(node, graph, project_root).unwrap_or_default();

        // 空テキスト（正規化後 0 文字）は --force 無しなら skip（DD §3）。
        if content.is_empty() && !options.force {
            report.skipped += 1;
            continue;
        }

        // CR フォールバック（Phase 1 パススルー骨格、Ok(None) でも通常 embedding 継続）。
        let parent_doc: Option<String> = match &options.contextual {
            Some(cfg) => crate::contextual::synthesize_with_fallback(cfg, &content, &node.id)?,
            None => None,
        };

        match embedder.embed_node(&content, parent_doc.as_deref(), &node.id) {
            Ok(result) => {
                // --force 無しかつ content_hash + model_version 双方一致なら skip（再計算不要）。
                if !options.force {
                    match store.is_up_to_date(&node.id, &result.content_hash, &result.model_version)
                    {
                        Ok(true) => {
                            // CR を新規有効化した場合、content 不変でも保存済 row に context が無ければ
                            // 1 度だけ backfill する（CACHE-CR-002）。context が既にあれば再合成しない
                            // （ADR-LGX-009「context はキャッシュ」）。CR 無効時は result.context=None で skip。
                            let needs_cr_backfill = result.context.is_some()
                                && store
                                    .load_embedding(&node.id)
                                    .ok()
                                    .flatten()
                                    .map(|r| r.context.is_none())
                                    .unwrap_or(false);
                            if !needs_cr_backfill {
                                report.skipped += 1;
                                continue;
                            }
                        }
                        Ok(false) => {}
                        Err(e) => return Err(EmbedError::Db(e)),
                    }
                }
                // ノード単位 Tx で upsert（DB 書込失敗のみ Err 昇格）。
                store
                    .upsert_with_subnode_meta(node, &result)
                    .map_err(EmbedError::Db)?;
                report.generated += 1;
            }
            Err(e) => {
                // 部分失敗継続: errors へ計上し後続を処理する。
                report.failed += 1;
                report.errors.push(EmbedErrorItem {
                    node_id: node.id.clone(),
                    message: e.to_string(),
                });
            }
        }

        // include_subnodes: ドキュメントノードのサブノードを抽出し is_subnode=1 で永続化
        //   （ADR-LGX-023。refresh-subnodes の照合元。サブノード ID は親の content_range スライスを embed）。
        if options.include_subnodes && !node.id.contains('#') {
            embed_subnodes(
                node,
                store,
                embedder,
                project_root,
                options.contextual.as_ref(),
                &mut report,
            );
        }
    }

    Ok(report)
}

/// 自動生成サブノード（`{parent}#{16hex}`）か判定する。embed_all の main loop が
/// materialize 済 auto サブノードを二重 embed（全文上書き）しないための除外条件。
fn is_auto_generated_subnode(id: &str) -> bool {
    id.split_once('#')
        .map(|(_, frag)| legixy_graph::subnode::is_auto_generated_fragment(frag))
        .unwrap_or(false)
}

/// ドキュメントノードの h2/h3 サブノードを抽出し、各 content_range スライスを embed して
/// is_subnode=1 で永続化する（ADR-LGX-023。部分失敗継続）。
fn embed_subnodes(
    node: &Node,
    store: &EmbeddingStore,
    embedder: &Embedder,
    project_root: &Path,
    contextual: Option<&ContextualConfig>,
    report: &mut EmbedReport,
) {
    let abs = project_root.join(&node.path);
    let raw = match std::fs::read_to_string(&abs) {
        Ok(c) => c,
        Err(_) => return, // ファイル不在は subnode 抽出なし（部分失敗トレランス）。
    };
    for sub in legixy_graph::subnode::extract_subnodes_with_levels(&node.id, &raw, &[2, 3]) {
        let (start, end) = sub.content_range;
        let slice = raw.get(start..end).unwrap_or("");
        let text = crate::content::normalize_content(slice);
        if text.is_empty() {
            continue;
        }
        // CR 有効時はサブノード区画に対しても context を合成（CACHE-CR-002。失敗は CR 無効扱い）。
        let ctx: Option<String> = match contextual {
            Some(cfg) => crate::contextual::synthesize_with_fallback(cfg, &text, &sub.id)
                .unwrap_or(None),
            None => None,
        };
        match embedder.embed_node(&text, ctx.as_deref(), &sub.id) {
            Ok(result) => match store.upsert_subnode(&sub.id, &node.id, &sub.anchor, &result) {
                Ok(()) => report.generated += 1,
                Err(e) => {
                    report.failed += 1;
                    report.errors.push(EmbedErrorItem {
                        node_id: sub.id.clone(),
                        message: e.to_string(),
                    });
                }
            },
            Err(e) => {
                report.failed += 1;
                report.errors.push(EmbedErrorItem {
                    node_id: sub.id.clone(),
                    message: e.to_string(),
                });
            }
        }
    }
}
