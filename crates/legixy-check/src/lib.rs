// Document ID: SRC-LGX-001
// legixy-check: 検証カテゴリ・severity・finding 生成・CheckReport 集約（DD-LGX-001）
//
// TC[RED] フェーズの scaffold。公開 API surface（型・シグネチャ）は DD-LGX-001 §2/§3 に
// 凍結整合させる。データ型は実体を持ち、検証ロジック（run / exit_code / to_json）は
// `todo!()` として TC[RED] を失敗させる。SRC[GREEN] で最小実装に置換する。
//
// 親 chain: TS-LGX-001 → TC-LGX-001 → 本 SRC-LGX-001。crate 境界は ADR-LGX-020。

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

pub use legixy_core::Severity; // 共有型（再エクスポート、ADR-LGX-020）
use legixy_core::{Config, ConfigError, Id};
use legixy_db::DbError;
// 意味層は legixy-embed の bulk similarity API を呼ぶ（SPEC-LGX-004.REQ.02、BUG-005）。
// EmbeddingStore も legixy-embed のものを使う（CLI の open_embed_store と同一型）。
use legixy_embed::{
    compute_edge_scores, compute_link_candidates, content_hash_for, read_current_content_for_node,
    EmbedError, EmbeddingStore,
};
use legixy_graph::{EdgeKind, GraphError, TraceGraph};

/// 検証カテゴリ（DD-LGX-001 §2.2。SPEC-LGX-004.REQ.15 割当表に 1:1）。
/// `GraphDag`（グラフ全体 CTX-INV-4）と `SubnodeDag`（SUBNODE-INV-4）は別カテゴリ。
/// 宣言順 = REQ.06 第2ソートキーの category 順序。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum CheckCategory {
    FileExistence,
    DocumentId,
    ChainIntegrity,
    OrphanFile,
    GraphDag,
    Freshness,
    SubnodeIdFormat,
    SubnodeIdUniqueness,
    SubnodeParentIntegrity,
    SubnodePathConsistency,
    SubnodeDag,
    SubnodeIdCollision,
    UnresolvedEdge,
    IdRedefined,
    IdSemanticMismatch,
    IdSemanticDrift,
    SemanticSimilarity, // 全層 check のみ（--formal では発行しない）
}

/// finding の発生位置（ファイルパス + 行）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Location {
    pub path: PathBuf,
    pub line: Option<usize>,
}

/// 個別 finding（DD-LGX-001 §2.1）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CheckResult {
    pub severity: Severity,
    pub category: CheckCategory,
    pub message: String,
    pub related_ids: Vec<Id>,
    pub location: Option<Location>,
}

/// severity 別件数（DD-LGX-001 §2.1）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct SeverityCounts {
    pub error: usize,
    pub warning: usize,
    pub info: usize,
    pub ok: usize,
}

/// 検証報告（DD-LGX-001 §2.1）。findings は安定ソート済（REQ.06）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CheckReport {
    pub findings: Vec<CheckResult>,
    pub counts: SeverityCounts,
}

impl CheckReport {
    /// JSON Lines シリアライズ（DD-LGX-001 §3、REQ.08）。finding 1 件 1 行。
    pub fn to_json(&self) -> String {
        self.findings
            .iter()
            .map(|f| serde_json::to_string(f).unwrap_or_default())
            .collect::<Vec<_>>()
            .join("\n")
    }
}

/// 検証モード（DD-LGX-001 §2.2）。Formal=形式層のみ、Full=意味層追加。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CheckMode {
    Formal,
    Full,
}

/// 実行時失敗（exit 1。検証 finding とは別概念。DD-LGX-001 §2.3）。
#[derive(Debug, thiserror::Error)]
pub enum CheckError {
    #[error("graph load failed: {0}")]
    GraphLoad(#[from] GraphError),
    #[error("config load failed: {0}")]
    ConfigLoad(#[from] ConfigError),
    #[error("db error: {0}")]
    Db(#[from] DbError),
    #[error("embed error: {0}")]
    Embed(#[from] EmbedError),
}

/// 検証本体（DD-LGX-001 §3、凍結 API surface）。
/// 同一入力 → 同一 CheckReport（findings 順序含む、REQ.06）。read-only。
/// `project_root` を持たないため FileExistence / OrphanFile（ファイルシステム検査）は**省略**する。
/// 完全な形式層（SPEC-LGX-004.REQ.01 の 5 カテゴリ）には [`run_with_root`] を使う。
pub fn run(
    graph: &TraceGraph,
    config: &Config,
    mode: CheckMode,
    store: Option<&EmbeddingStore>,
) -> Result<CheckReport, CheckError> {
    run_inner(graph, config, mode, store, None)
}

/// project_root を伴う完全な形式層検証（FileExistence / OrphanFile を含む、SPEC-LGX-004.REQ.01）。
/// CLI / MCP はこちらを使う（BUG-006）。
pub fn run_with_root(
    graph: &TraceGraph,
    config: &Config,
    mode: CheckMode,
    store: Option<&EmbeddingStore>,
    project_root: &Path,
) -> Result<CheckReport, CheckError> {
    run_inner(graph, config, mode, store, Some(project_root))
}

fn run_inner(
    graph: &TraceGraph,
    config: &Config,
    mode: CheckMode,
    store: Option<&EmbeddingStore>,
    project_root: Option<&Path>,
) -> Result<CheckReport, CheckError> {
    let mut findings = Vec::new();

    // ── 形式層（SPEC-LGX-004.REQ.01）──
    findings.extend(check_chain_integrity(graph, config)); // ID 形式 + chain 到達性
    findings.extend(check_subnode_id_format(graph)); // サブノード ID 形式（`#` 接尾辞）
    findings.extend(check_graph_dag(graph)); // 全エッジ DAG
    if let Some(root) = project_root {
        findings.extend(check_file_existence(graph, root)); // path 実在（Error）
        findings.extend(check_orphan_file(graph, config, root)); // 未登録ファイル（Info）
    }

    // ── 意味層（Full のみ。SPEC-LGX-004.REQ.02、BUG-005）──
    if matches!(mode, CheckMode::Full) {
        match store {
            // embeddings 未生成は非致命 Info 1 件（exit code に影響しない、FB-INV-4）。
            None => findings.push(CheckResult {
                severity: Severity::Info,
                category: CheckCategory::SemanticSimilarity,
                message:
                    "embeddings 未生成: `legixy embed --all` を実行してください（意味層は省略）"
                        .to_string(),
                related_ids: Vec::new(),
                location: None,
            }),
            Some(s) => findings.extend(check_semantic(graph, config, s, project_root)?),
        }
    }

    // 安定ソート（REQ.06: severity 降順 → category → related_ids）
    findings.sort_by(|a, b| {
        b.severity
            .cmp(&a.severity)
            .then(a.category.cmp(&b.category))
            .then(a.related_ids.cmp(&b.related_ids))
    });

    let counts = count_severities(&findings);
    Ok(CheckReport { findings, counts })
}

/// 終了コード判定（DD-LGX-001 §3、REQ.04）。`counts.error > 0 ⇒ 1`、else 0。
pub fn exit_code(report: &CheckReport) -> i32 {
    if report.counts.error > 0 {
        1
    } else {
        0
    }
}

// ── 形式層検査器（内部） ──

/// ChainIntegrity: chain typecode のノードに親 chain エッジが存在するか（REQ.15）。
/// chain の起点 typecode（order[0]）は親不要。サブノード（id に `#`）は対象外。
/// multi-area（config.chains 非空）では node id の area で chain order を解決する（BUG-003）。
fn check_chain_integrity(graph: &TraceGraph, config: &Config) -> Vec<CheckResult> {
    let mut out = Vec::new();
    for node in graph.nodes() {
        if node.id.contains('#') {
            continue; // サブノードは ParentChild であり chain 対象外（別途 SubnodeIdFormat）
        }
        // ID 形式検証（`{type}-{area}-{seq}`。v3: ChainIntegrity Error。BUG-006 / EXT-CHK-003）。
        if !is_valid_document_id(&node.id) {
            out.push(CheckResult {
                severity: Severity::Error,
                category: CheckCategory::ChainIntegrity,
                message: format!(
                    "ID 形式が不正: {} （`{{type}}-{{area}}-{{seq}}` に一致しません）",
                    node.id
                ),
                related_ids: vec![Id::new(node.id.clone())],
                location: None,
            });
            continue; // 形式不正なら chain 親判定はスキップ
        }
        // 適用する chain order を area で解決（単一 area は chain_order を返す）。
        let order = match config.chain_order_for(&node.id) {
            Some(o) => o,
            None => continue, // 未知 area / 独立 area は chain 対象外
        };
        if !order.contains(&node.type_code) {
            continue; // chain 外 typecode（SPEC/ADR 等の independent）
        }
        if order.first() == Some(&node.type_code) {
            continue; // chain 起点は親不要
        }
        let has_chain_parent = graph
            .edges()
            .iter()
            .any(|e| e.to == node.id && matches!(e.kind, EdgeKind::Chain));
        if !has_chain_parent {
            out.push(CheckResult {
                severity: Severity::Error,
                category: CheckCategory::ChainIntegrity,
                message: format!("chain 親エッジが欠落: {}", node.id),
                related_ids: vec![Id::new(node.id.clone())],
                location: None,
            });
        }
    }
    out
}

/// document id（`{type}-{area}-{seq}`）の形式妥当性。type=英字 / area=英数 / seq=数字、3 パート。
fn is_valid_document_id(id: &str) -> bool {
    let parts: Vec<&str> = id.split('-').collect();
    parts.len() == 3
        && !parts[0].is_empty()
        && parts[0].chars().all(|c| c.is_ascii_alphabetic())
        && !parts[1].is_empty()
        && parts[1].chars().all(|c| c.is_ascii_alphanumeric())
        && !parts[2].is_empty()
        && parts[2].chars().all(|c| c.is_ascii_digit())
}

/// SubnodeIdFormat: サブノード ID（`{parent}#{suffix}`）の形式（REQ.15、BUG-006 / EXT-SUB-EXPL-003）。
/// parent は妥当な document id、suffix は hex（自動生成）または `s:<slug>`（明示）。
fn check_subnode_id_format(graph: &TraceGraph) -> Vec<CheckResult> {
    let mut out = Vec::new();
    for node in graph.nodes() {
        let id = node.id.as_str();
        let hash_pos = match id.find('#') {
            Some(p) => p,
            None => continue, // document ノードは対象外
        };
        let parent = &id[..hash_pos];
        let suffix = &id[hash_pos + 1..];
        let valid_parent = is_valid_document_id(parent);
        let valid_suffix = if let Some(slug) = suffix.strip_prefix("s:") {
            // 明示 `s:<slug>`: LGX-EXT-001 §4.5.2 の文字制約を強制（英数/ハイフン/アンダースコア・
            // 両端英数・1〜63 文字）。既存の validate_explicit_name を配線（外部検証 R-7 軽微）。
            legixy_graph::subnode::validate_explicit_name(slug).is_ok()
        } else {
            !suffix.is_empty() && suffix.chars().all(|c| c.is_ascii_hexdigit()) // 自動: hex
        };
        if !valid_parent || !valid_suffix {
            out.push(CheckResult {
                severity: Severity::Error,
                category: CheckCategory::SubnodeIdFormat,
                message: format!(
                    "サブノード ID 形式が不正: {id} （`{{parent}}#{{hex}}` または `{{parent}}#s:<slug>`）"
                ),
                related_ids: vec![Id::new(id.to_string())],
                location: None,
            });
        }
    }
    out
}

/// FileExistence: 各 document ノードの path が project_root 配下に実在するか（REQ.01、BUG-006）。
fn check_file_existence(graph: &TraceGraph, project_root: &Path) -> Vec<CheckResult> {
    let mut out = Vec::new();
    for node in graph.document_nodes() {
        let abs = project_root.join(&node.path);
        if !abs.exists() {
            out.push(CheckResult {
                severity: Severity::Error,
                category: CheckCategory::FileExistence,
                message: format!("ノード '{}' の path '{}' が存在しません", node.id, node.path),
                related_ids: vec![Id::new(node.id.clone())],
                location: Some(Location {
                    path: PathBuf::from(&node.path),
                    line: None,
                }),
            });
        }
    }
    out
}

/// OrphanFile: `[id.types]` のディレクトリに在るが graph に未登録のファイル（REQ.01、Info、BUG-006）。
/// config.types が空なら走査しない。
fn check_orphan_file(graph: &TraceGraph, config: &Config, project_root: &Path) -> Vec<CheckResult> {
    let mut out = Vec::new();
    if config.types.is_empty() {
        return out;
    }
    let registered: HashSet<String> = graph
        .document_nodes()
        .map(|n| normalize_rel(&n.path))
        .collect();

    for t in &config.types {
        let abs_dir = project_root.join(&t.dir);
        if !abs_dir.is_dir() {
            continue;
        }
        let expected_ext = t.ext.trim_start_matches('.');
        let mut files = Vec::new();
        collect_files(&abs_dir, &mut files);
        files.sort(); // 決定論
        for f in files {
            if f.extension().and_then(|e| e.to_str()) != Some(expected_ext) {
                continue;
            }
            let rel = match f.strip_prefix(project_root) {
                Ok(p) => p,
                Err(_) => continue,
            };
            let rel_str = normalize_rel(&rel.to_string_lossy());
            if registered.contains(&rel_str) {
                continue;
            }
            out.push(CheckResult {
                severity: Severity::Info,
                category: CheckCategory::OrphanFile,
                message: format!("未登録ファイル '{}' が {} に存在します", rel_str, t.dir),
                related_ids: Vec::new(),
                location: Some(Location {
                    path: PathBuf::from(&rel_str),
                    line: None,
                }),
            });
        }
    }
    out
}

/// ディレクトリ配下のファイルを再帰収集（OrphanFile 用。シンボリックリンクは辿らない std::fs 既定）。
fn collect_files(dir: &Path, out: &mut Vec<PathBuf>) {
    if let Ok(rd) = std::fs::read_dir(dir) {
        for entry in rd.flatten() {
            let p = entry.path();
            if p.is_dir() {
                collect_files(&p, out);
            } else if p.is_file() {
                out.push(p);
            }
        }
    }
}

fn normalize_rel(p: &str) -> String {
    p.replace('\\', "/")
}

/// 意味層（SemanticChecker、SPEC-LGX-004.REQ.02、BUG-005）。
/// - SemanticSimilarity: graph エッジ（chain/custom/parent_child）で類似度 < similarity_threshold → Warning
/// - LinkCandidate: 非エッジペアで類似度 ≥ link_candidate_threshold → Info（リンク漏れ候補）
/// - Drift: content_hash と保存済 hash の不一致 → Warning（project_root がある場合のみ。ファイル不在も Warning）
/// embeddings が空なら何も返さない（呼出側が None 経路で Info 済み）。
fn check_semantic(
    graph: &TraceGraph,
    config: &Config,
    store: &EmbeddingStore,
    project_root: Option<&Path>,
) -> Result<Vec<CheckResult>, CheckError> {
    let mut out = Vec::new();
    let sim_thr = config.semantic.similarity_threshold;
    let link_thr = config.semantic.link_candidate_threshold;

    // 1. SemanticSimilarity（エッジ類似度 < 閾値 → Warning）。
    let mut edges = compute_edge_scores(graph, store)?;
    edges.sort_by(|a, b| a.from.cmp(&b.from).then(a.to.cmp(&b.to))); // 決定論
    for e in &edges {
        if e.score < sim_thr {
            out.push(CheckResult {
                severity: Severity::Warning,
                category: CheckCategory::SemanticSimilarity,
                message: format!(
                    "低類似度エッジ: {} → {} = {:.4}（< 閾値 {:.2}）",
                    e.from, e.to, e.score, sim_thr
                ),
                related_ids: vec![Id::new(e.from.clone()), Id::new(e.to.clone())],
                location: None,
            });
        }
    }

    // 2. LinkCandidate（非エッジペアで類似度 ≥ 閾値 → Info）。
    let mut candidates = compute_link_candidates(graph, store, link_thr)?;
    candidates.sort_by(|a, b| a.from.cmp(&b.from).then(a.to.cmp(&b.to)));
    for c in &candidates {
        out.push(CheckResult {
            severity: Severity::Info,
            category: CheckCategory::SemanticSimilarity,
            message: format!(
                "リンク候補（未接続）: {} ↔ {} = {:.4}（≥ 閾値 {:.2}）",
                c.from, c.to, c.score, link_thr
            ),
            related_ids: vec![Id::new(c.from.clone()), Id::new(c.to.clone())],
            location: None,
        });
    }

    // 3. Drift（content_hash 不一致 → Warning）。project_root がある場合のみ。
    if let Some(root) = project_root {
        let rows = store.load_all()?;
        let stored: std::collections::HashMap<&str, &str> = rows
            .iter()
            .map(|r| (r.node_id.as_str(), r.content_hash.as_str()))
            .collect();
        for node in graph.document_nodes() {
            let stored_hash = match stored.get(node.id.as_str()) {
                Some(h) => *h,
                None => continue, // 未 embed のノードは drift 対象外
            };
            match read_current_content_for_node(node, graph, root) {
                Ok(content) => {
                    let current = content_hash_for(&content);
                    if current != stored_hash {
                        out.push(CheckResult {
                            severity: Severity::Warning,
                            category: CheckCategory::SemanticSimilarity,
                            message: format!(
                                "content drift: {} の現内容が保存済 embedding と不一致（再 embed 推奨）",
                                node.id
                            ),
                            related_ids: vec![Id::new(node.id.clone())],
                            location: None,
                        });
                    }
                }
                Err(_) => {
                    // ファイル不在/読取不可も Warning（REQ.02）。
                    out.push(CheckResult {
                        severity: Severity::Warning,
                        category: CheckCategory::SemanticSimilarity,
                        message: format!(
                            "content drift: {} のファイルを読めません（embedding 済だが現物不在）",
                            node.id
                        ),
                        related_ids: vec![Id::new(node.id.clone())],
                        location: None,
                    });
                }
            }
        }
    }

    Ok(out)
}

/// GraphDag: グラフ全体（全エッジ種別）に有向サイクルが無いか（CTX-INV-4、REQ.15）。
fn check_graph_dag(graph: &TraceGraph) -> Vec<CheckResult> {
    if let Some(cycle_node) = first_cycle_node(graph) {
        vec![CheckResult {
            severity: Severity::Error,
            category: CheckCategory::GraphDag,
            message: format!("グラフにサイクルを検出（例: {} を含む）", cycle_node),
            related_ids: vec![Id::new(cycle_node)],
            location: None,
        }]
    } else {
        Vec::new()
    }
}

/// DFS による有向サイクル検出。サイクル上のノード id を 1 つ返す。
fn first_cycle_node(graph: &TraceGraph) -> Option<String> {
    use std::collections::HashMap;
    // 隣接リスト
    let mut adj: HashMap<&str, Vec<&str>> = HashMap::new();
    for e in graph.edges() {
        adj.entry(e.from.as_str()).or_default().push(e.to.as_str());
    }
    // 0=未訪問, 1=訪問中, 2=完了
    let mut color: HashMap<&str, u8> = HashMap::new();
    fn dfs<'a>(
        n: &'a str,
        adj: &HashMap<&'a str, Vec<&'a str>>,
        color: &mut HashMap<&'a str, u8>,
    ) -> Option<String> {
        color.insert(n, 1);
        if let Some(neis) = adj.get(n) {
            for &m in neis {
                match color.get(m).copied().unwrap_or(0) {
                    1 => return Some(m.to_string()), // back-edge = cycle
                    0 => {
                        if let Some(c) = dfs(m, adj, color) {
                            return Some(c);
                        }
                    }
                    _ => {}
                }
            }
        }
        color.insert(n, 2);
        None
    }
    for node in graph.nodes() {
        if color.get(node.id.as_str()).copied().unwrap_or(0) == 0 {
            if let Some(c) = dfs(node.id.as_str(), &adj, &mut color) {
                return Some(c);
            }
        }
    }
    None
}

/// findings から severity 別件数を集計。
fn count_severities(findings: &[CheckResult]) -> SeverityCounts {
    let mut c = SeverityCounts::default();
    for f in findings {
        match f.severity {
            Severity::Error => c.error += 1,
            Severity::Warning => c.warning += 1,
            Severity::Info => c.info += 1,
            Severity::Ok => c.ok += 1,
        }
    }
    c
}
