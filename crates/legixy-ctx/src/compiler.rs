// Document ID: SRC-LGX-002
// legixy-ctx::compiler — CompileInput / Granularity / ContextCompiler / build_outline
//
// TC[RED] scaffold。
//  - データ型（CompileInput / Granularity）は実体。
//  - ContextCompiler::new はライフタイム借用を保持する実体（テストが構築可能）。
//  - compile / render はロジックを持つため todo!()（TC[RED] を panic で失敗させる）。
//  - build_outline（DD-LGX-004 §3、pub(crate)）も todo!()。
//
// DD-LGX-002 §3 / DD-LGX-004 §3 の凍結シグネチャに厳密整合（HR7）。
//
// SHARED-NEED: `TraceConfig`（DD-LGX-002 §2.1 / §3 の `config: &'a TraceConfig`）は本来
//   legixy-core 所有。共有 stub には `Config` のみ存在するため、本 scaffold では
//   `legixy_core::Config` を `TraceConfig` として参照する（型エイリアス）。統合時に
//   legixy-core が `TraceConfig` を公開した時点で差し替える。

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use legixy_graph::{EdgeKind, NodeId, TraceGraph};

use crate::audit_logger::AuditLogger;
use crate::db::DbConn;
use crate::error::ContextError;
use crate::result::{ContextResult, ResolvedTarget, TargetNodeMetadata, UpstreamArtifact};
use crate::section_formatter::SectionFormatter;
use crate::upstream_walker::UpstreamWalker;

/// SHARED-NEED: 本来 legixy-core 所有の `TraceConfig`。共有 stub の `Config` で代替。
pub use legixy_core::Config as TraceConfig;

/// 粒度種別（DD-LGX-002 §2.2 / DD-LGX-004 §2.1。2 値のみ、auto は存在しない）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub enum Granularity {
    /// v0.1.0 互換。ドキュメント全文を返却（REQ.01、既定値）。
    #[default]
    Document,
    /// サブノード単位で返却（SPEC-LGX-003.REQ.03 / UC-LGX-004）。
    Subnode,
}

impl Granularity {
    /// context_log の granularity カラム値 / CLI 引数文字列と一致（DD-LGX-004 §2.1）。
    pub fn as_str(&self) -> &'static str {
        match self {
            Granularity::Document => "document",
            Granularity::Subnode => "subnode",
        }
    }
}

/// compile_context の入力（DD-LGX-002 §2.1 / DD-LGX-004 §2.1）。
#[derive(Debug, Clone, PartialEq)]
pub struct CompileInput {
    /// 必須: 編集対象ファイルのパス（1 件以上）。
    pub target_files: Vec<PathBuf>,
    /// 粒度制御（既定 Document）。
    pub granularity: Granularity,
    /// context_log payload.command のみ（返却内容に影響しない、REQ.01 S2-06）。
    pub command: Option<String>,
    /// REQ.15: true 時、upstream body を ATX 見出し階層リストに置換。
    pub outline_only: bool,
    /// REQ.16: Some(vec) 時、subnode 粒度で指定 ID のみ通す。None = フィルタなし。
    pub sections: Option<Vec<String>>,
    /// REQ.17: Some(N) で上流 N 階層に制限、None で無制限。
    pub depth_limit: Option<usize>,
}

impl Default for CompileInput {
    fn default() -> Self {
        Self {
            target_files: Vec::new(),
            granularity: Granularity::Document,
            command: None,
            outline_only: false,
            sections: None,
            depth_limit: None,
        }
    }
}

/// コンテキスト解決メインクラス（DD-LGX-002 §2.1）。
/// graph/config/db/project_root を **借用**（read-only、所有権を取らない、§5）。
pub struct ContextCompiler<'a> {
    graph: &'a TraceGraph,
    #[allow(dead_code)]
    config: &'a TraceConfig,
    db: Option<&'a DbConn>,
    project_root: &'a Path,
}

impl<'a> ContextCompiler<'a> {
    /// DD-LGX-002 §3 凍結シグネチャ。借用のみ保持（read-only）。
    pub fn new(
        graph: &'a TraceGraph,
        config: &'a TraceConfig,
        db: Option<&'a DbConn>,
        project_root: &'a Path,
    ) -> Self {
        ContextCompiler {
            graph,
            config,
            db,
            project_root,
        }
    }

    /// DD-LGX-002 §3 凍結。同一入力 → 同一 ContextResult（REQ.04/14、CACHE-INV-1）。
    /// 監査ログ失敗は Ok 維持（REQ.19）。起点未解決は partial success・exit 0（REQ.20）。
    /// ResultTooLarge は render 経路で Err（v3 互換、B-1）。
    pub fn compile(&self, input: &CompileInput) -> Result<ContextResult, ContextError> {
        // ── target_files → NodeId 逆引き（REQ.20、FileResolver 相当）──
        let targets = self.resolve_targets(&input.target_files);

        // 未解決起点を決定論記録（PathBuf 辞書順昇順、REQ.20 / S2-24）。
        let mut unresolved_targets: Vec<PathBuf> = targets
            .iter()
            .filter(|t| t.artifact_id.is_none())
            .map(|t| t.file_path.clone())
            .collect();
        unresolved_targets.sort();

        let target_ids: Vec<NodeId> = targets
            .iter()
            .filter_map(|t| t.artifact_id.clone())
            .collect();

        // ── 上流連鎖収集（REQ.02/03/08/15/16/17）──
        let upstream = self.collect_upstream(
            &target_ids,
            input.granularity,
            input.outline_only,
            input.sections.as_deref(),
            input.depth_limit,
        )?;

        let target_metadata = self.collect_metadata(&target_ids, &unresolved_targets);

        let result = ContextResult {
            targets,
            // Layer / Additional / Custom は本 crate 範囲では空（REQ.10 セクション枠は render が出す）。
            layer_guidelines: Vec::new(),
            additional_guidelines: Vec::new(),
            upstream,
            custom_documents: Vec::new(),
            target_metadata,
            granularity: input.granularity,
            unresolved_targets,
        };

        // 監査ログはベストエフォート（REQ.19、書込失敗でも Ok 維持）。
        let audit = AuditLogger::new(self.db);
        audit.log(input, &result)?;

        Ok(result)
    }

    /// DD-LGX-002 §3 凍結。6 セクション・決定論的整列（REQ.10/11/12/14）。LF 固定・バイト決定論。
    pub fn render(&self, result: &ContextResult) -> Result<String, ContextError> {
        SectionFormatter::render(result)
    }

    /// target_files の各パスを graph 上の node.path と突合して NodeId を逆引き（REQ.20）。
    /// 未解決は artifact_id=None で保持（部分成功、exit 0）。
    fn resolve_targets(&self, target_files: &[PathBuf]) -> Vec<ResolvedTarget> {
        target_files
            .iter()
            .map(|fp| {
                let key = fp.to_string_lossy();
                let matched = self.graph.nodes().find(|n| n.path == key);
                match matched {
                    Some(node) => ResolvedTarget {
                        file_path: fp.clone(),
                        artifact_id: Some(node.id.clone()),
                        type_code: Some(node.type_code.clone()),
                    },
                    None => ResolvedTarget {
                        file_path: fp.clone(),
                        artifact_id: None,
                        type_code: None,
                    },
                }
            })
            .collect()
    }

    /// 上流連鎖を収集し、粒度・フィルタ・outline を適用する（v3 compiler.rs:163-271 底本）。
    fn collect_upstream(
        &self,
        target_ids: &[NodeId],
        granularity: Granularity,
        outline_only: bool,
        sections: Option<&[String]>,
        depth_limit: Option<usize>,
    ) -> Result<Vec<UpstreamArtifact>, ContextError> {
        let walker = UpstreamWalker::new(self.graph);

        // エッジ誘導の絞り込み（R-1/R-2）: target が直接結線するサブノード集合。
        // 上流 doc に「target が指す特定サブノード」があれば、それのみを返す（関連サブノードに絞る）。
        let linked_subnodes = self.target_linked_subnodes(target_ids);

        // sections フィルタ（subnode 粒度のみ有効、REQ.16/18）。
        let sections_filter: Option<std::collections::HashSet<&str>> =
            sections.map(|v| v.iter().map(|s| s.as_str()).collect());

        let mut seen: BTreeSet<NodeId> = BTreeSet::new();
        let mut out: Vec<UpstreamArtifact> = Vec::new();

        for target_id in target_ids {
            let walked = walker.walk_chain_parent_only_with_depth(target_id, depth_limit)?;
            for art in walked {
                match granularity {
                    Granularity::Document => {
                        if !seen.insert(art.artifact_id.clone()) {
                            continue;
                        }
                        let mut enriched = self.enrich_document(art);
                        if outline_only {
                            enriched.body = build_outline(&enriched.body);
                        }
                        out.push(enriched);
                    }
                    Granularity::Subnode => {
                        let parent_id = art.artifact_id.clone();
                        let parent_path = art.file_path.clone();
                        let chain_distance = art.chain_distance;
                        let all_subnodes = self.subnodes_of(&parent_id);
                        // エッジ誘導の絞り込み（R-1、LGX-EXT-001 目的1）: target が当該 doc 配下の特定
                        // サブノードへ chain/custom で結線していれば、それのみへ絞る（関連サブノードに限定）。
                        let linked_here: Vec<(NodeId, String, Option<String>)> = all_subnodes
                            .iter()
                            .filter(|(id, _, _)| linked_subnodes.contains(id))
                            .cloned()
                            .collect();
                        // 絞り込みが有効（クエリ全体でいずれかの target が subnode 結線を持つ）かつ当該 doc に
                        // 結線サブノードが無い場合は、ドキュメント粒度で返す（chain 文脈を保ちつつ圧縮＝削減）。
                        // 絞り込み無効時はサブノード不在のみ document fallback（従来動作）。
                        let narrowing_active = !linked_subnodes.is_empty();
                        let emit_document = all_subnodes.is_empty()
                            || (narrowing_active && linked_here.is_empty());
                        if emit_document {
                            // sections 指定時はサブノードのみ要求 → document へ落とさず除外（TC-002/004）。
                            if sections_filter.is_some() {
                                continue;
                            }
                            if !seen.insert(parent_id.clone()) {
                                continue;
                            }
                            let mut enriched = self.enrich_document(art);
                            if outline_only {
                                enriched.body = build_outline(&enriched.body);
                            }
                            out.push(enriched);
                            continue;
                        }
                        let subnodes = if linked_here.is_empty() {
                            all_subnodes
                        } else {
                            linked_here
                        };

                        // サブノードを物理位置順（graph 挿入順 = 出現順、A-1）で展開。
                        let parent_content = self.load_body(&parent_path);
                        let extracted = legixy_graph::subnode::extract_subnodes_with_levels(
                            &parent_id,
                            &parent_content,
                            &[2, 3],
                        );
                        // auto サブノードは id 一致で区画解決（embed と同一抽出経路）。
                        let range_by_id: std::collections::HashMap<&str, (usize, usize)> = extracted
                            .iter()
                            .map(|s| (s.id.as_str(), s.content_range))
                            .collect();
                        // 明示 `#s:` サブノードは anchor 見出しで区画解決（R-3。auto は本マップ未使用）。
                        let range_by_anchor: std::collections::HashMap<String, (usize, usize)> =
                            extracted
                                .iter()
                                .map(|s| {
                                    (
                                        legixy_graph::subnode::normalize_heading(&s.anchor),
                                        s.content_range,
                                    )
                                })
                                .collect();

                        // sections フィルタ・seen 重複除去を適用しつつ各サブノードの区画を解決。
                        let mut resolved: Vec<(NodeId, String, Option<String>, Option<(usize, usize)>)> =
                            Vec::new();
                        for (sub_id, sub_type, anchor) in subnodes {
                            if let Some(filter) = &sections_filter {
                                if !filter.contains(sub_id.as_str()) {
                                    continue;
                                }
                            }
                            if !seen.insert(sub_id.clone()) {
                                continue;
                            }
                            let range = range_by_id.get(sub_id.as_str()).copied().or_else(|| {
                                // explicit: anchor 見出し（例 `## 状態遷移`）→ 抽出区画へ照合。
                                anchor
                                    .as_deref()
                                    .map(strip_heading_markers)
                                    .map(|a| legixy_graph::subnode::normalize_heading(&a))
                                    .and_then(|na| range_by_anchor.get(&na).copied())
                            });
                            resolved.push((sub_id, sub_type, anchor, range));
                        }

                        // 非重複化（R-1: h2 が h3 を内包する重複を排除）。各区画 end を「次に始まる区画の
                        // start」でクリップし、出現順で重複なく分割する（トークン削減・全文化防止）。
                        let mut starts: Vec<usize> =
                            resolved.iter().filter_map(|r| r.3.map(|(s, _)| s)).collect();
                        starts.sort_unstable();
                        for (sub_id, sub_type, anchor, range) in resolved {
                            let body = if outline_only {
                                anchor.clone().unwrap_or_default()
                            } else {
                                match range {
                                    Some((start, end)) => {
                                        let clip = starts
                                            .iter()
                                            .copied()
                                            .find(|&s| s > start)
                                            .map_or(end, |next| next.min(end));
                                        parent_content.get(start..clip).unwrap_or("").to_string()
                                    }
                                    None => String::new(),
                                }
                            };
                            out.push(UpstreamArtifact {
                                artifact_id: parent_id.clone(),
                                type_code: sub_type,
                                file_path: parent_path.clone(),
                                chain_distance,
                                body,
                                subnode_id: Some(sub_id),
                                anchor,
                                drift_score: None,
                            });
                        }
                    }
                }
            }
        }
        Ok(out)
    }

    /// document 粒度の本文補填（ファイル不在は空 body で継続、REQ.20-2）。
    fn enrich_document(&self, mut art: UpstreamArtifact) -> UpstreamArtifact {
        art.subnode_id = None;
        art.anchor = None;
        art.body = self.load_body(&art.file_path);
        art
    }

    /// project_root 配下のファイルを読み込む。不在・読込失敗は空文字列（部分成功、REQ.20-2）。
    fn load_body(&self, rel_path: &Path) -> String {
        let abs = self.project_root.join(rel_path);
        std::fs::read_to_string(&abs).unwrap_or_default()
    }

    /// 親ノード ID の ParentChild 子（サブノード）を graph 挿入順（出現順）で収集する。
    /// 返却 = (subnode_id, type_code, anchor)。anchor は graph 由来（明示=graph.toml の見出しアンカー /
    /// 自動=見出しテキスト）。graph に anchor が無い場合のみ `#` 以降の fragment へ fallback。
    fn subnodes_of(&self, parent_id: &str) -> Vec<(NodeId, String, Option<String>)> {
        self.graph
            .edges()
            .iter()
            .filter(|e| matches!(e.kind, EdgeKind::ParentChild) && e.from == parent_id)
            .filter_map(|e| {
                self.graph.node(&e.to).map(|n| {
                    let anchor = n
                        .anchor
                        .clone()
                        .or_else(|| n.id.split_once('#').map(|(_, frag)| frag.to_string()));
                    (n.id.clone(), n.type_code.clone(), anchor)
                })
            })
            .collect()
    }

    /// target_ids が **chain/custom エッジ**で結線するサブノード（id に `#` を含む to 端点）の集合
    /// （エッジ誘導の絞り込み、R-1/R-2）。fixture 例: `SRC-SN-001 → DD-SN-001#s:state-machine`。
    /// ParentChild（materialize エッジ）は対象外＝「成果物間の細粒度トレース」のみを関連と見なす
    /// （ドキュメント自身を起点にした場合に全サブノードへ絞り込まれてしまうのを防ぐ）。
    fn target_linked_subnodes(&self, target_ids: &[NodeId]) -> std::collections::HashSet<NodeId> {
        let targets: std::collections::HashSet<&str> =
            target_ids.iter().map(|s| s.as_str()).collect();
        self.graph
            .edges()
            .iter()
            .filter(|e| {
                matches!(e.kind, EdgeKind::Chain | EdgeKind::Custom)
                    && targets.contains(e.from.as_str())
                    && e.to.contains('#')
            })
            .map(|e| e.to.clone())
            .collect()
    }

    /// target ノードの隣接エッジ・サブノード数を収集する（v3 compiler.rs:332-362 底本）。
    /// unresolved_targets は最初の metadata エントリにのみ記録（render が末尾へ決定論出力）。
    fn collect_metadata(
        &self,
        target_ids: &[NodeId],
        unresolved_targets: &[PathBuf],
    ) -> Vec<TargetNodeMetadata> {
        let mut out = Vec::with_capacity(target_ids.len());
        for (i, id) in target_ids.iter().enumerate() {
            let outgoing: Vec<(NodeId, EdgeKind)> = self
                .graph
                .edges()
                .iter()
                .filter(|e| &e.from == id)
                .map(|e| (e.to.clone(), e.kind))
                .collect();
            let incoming: Vec<(NodeId, EdgeKind)> = self
                .graph
                .edges()
                .iter()
                .filter(|e| &e.to == id)
                .map(|e| (e.from.clone(), e.kind))
                .collect();
            let subnode_count = self.subnodes_of(id).len();
            out.push(TargetNodeMetadata {
                artifact_id: id.clone(),
                outgoing_edges: outgoing,
                incoming_edges: incoming,
                subnode_count,
                // unresolved は先頭エントリにのみ集約（決定論・重複出力回避）。
                unresolved_targets: if i == 0 {
                    unresolved_targets.to_vec()
                } else {
                    Vec::new()
                },
            });
        }
        out
    }
}

/// 明示サブノード anchor（例 `## 状態遷移` / `状態遷移`）から見出しマーカー（先頭 `#`・前後空白）を
/// 除去して見出しテキストのみ返す（R-3 の区画照合キー生成用）。
fn strip_heading_markers(s: &str) -> String {
    s.trim()
        .trim_start_matches('#')
        .trim_start()
        .trim_end_matches('#')
        .trim()
        .to_string()
}

/// REQ.15: ATX 見出し（h1〜h3）抽出・階層インデント（DD-LGX-004 §3）。
/// h1〜h3 のみ、スペース必須（`# title` 形式）、h4+・空タイトル除外。
/// インデント `"  " × (level - 1)`。見出し皆無で空文字列（S2-25）。
///
/// DD-LGX-004 §3 は `pub(crate)` だが、TC-LGX-004（tests/ 統合テストクレート）から
/// build_outline 単体を束縛するため `pub` へ可視性拡張（HR7 additive。crate 外公開は
/// 整列・サイズ上限と同列の純関数であり、契約縮小・改名・型変更は行わない）。
pub fn build_outline(content: &str) -> String {
    let mut out = String::new();
    for line in content.lines() {
        let trimmed = line.trim_start();
        let level = trimmed.chars().take_while(|c| *c == '#').count();
        if level == 0 || level > 3 {
            continue;
        }
        // "#### 〜" のような h4+ は対象外、"# " の後にスペースが必要。
        let after = &trimmed[level..];
        if !after.starts_with(' ') {
            continue;
        }
        let title = after.trim_start().trim_end_matches('#').trim_end();
        if title.is_empty() {
            continue;
        }
        let indent = "  ".repeat(level.saturating_sub(1));
        out.push_str(&format!("{}- {}\n", indent, title));
    }
    out
}
