// (module of SRC-LGX-008; anchor: manager.rs)（observer 部）
// DD-LGX-008 §2.1 / §2.2 / §3.1: ObservationStatus / Observation / NewObservation /
//   ObserveCategoryInput / FeedbackCategory / AutoObserver / drift_from_embed_error。

use legixy_check::{CheckCategory, CheckReport, CheckResult, Severity};

use crate::embed::{mask_api_key, EmbedError};

/// Observation の永続化状態（SPEC-LGX-007 REQ.08 v0.6.0 の 4 値モデル / DD §2.2）。
///
/// 遷移グラフ:
///   observe/feedback → Pending
///   analyze 取込    → Analyzing（Pessimistic Claim）
///   Proposal 生成成功 → Resolved（対応 proposal approve 後）
///   analyze 失敗（Claim Release）→ Pending
///   変換規則なし（orphan_file / semantic_similarity）→ Skipped（終端・永久再 claim 解消）
///
/// Resolved / Skipped は終端・不可逆（REQ.08。【v3 差分】skipped 終端追加 ADR-LGX-019）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObservationStatus {
    Pending,
    Analyzing,
    Resolved,
    Skipped, // 終端（変換規則なし category）
}

impl ObservationStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Analyzing => "analyzing",
            Self::Resolved => "resolved",
            Self::Skipped => "skipped",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "pending" => Some(Self::Pending),
            "analyzing" => Some(Self::Analyzing),
            "resolved" => Some(Self::Resolved),
            "skipped" => Some(Self::Skipped),
            _ => None,
        }
    }
}

/// CLI / MCP 入口の category 3 値（REQ.01 凍結 / DD §2.2）。
/// CLI 層は clap ValueEnum 相当で強制。不正値は exit 2（【v3 差分】v3 は String 無検証）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObserveCategoryInput {
    CompileMiss,      // "compile_miss"
    ReviewCorrection, // "review_correction"
    ManualNote,       // "manual_note"
}

impl ObserveCategoryInput {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::CompileMiss => "compile_miss",
            Self::ReviewCorrection => "review_correction",
            Self::ManualNote => "manual_note",
        }
    }

    /// 凍結 3 値以外は None（CLI 層はこれを exit 2 に写像する。ケース 6）。
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "compile_miss" => Some(Self::CompileMiss),
            "review_correction" => Some(Self::ReviewCorrection),
            "manual_note" => Some(Self::ManualNote),
            _ => None,
        }
    }
}

/// feedback コマンドが AutoObserver を通じて生成する category 値（DD §2.2）。
/// REQ.01 の凍結 3 値（ObserveCategoryInput）とは別の集合（混同禁止、SUPP-LGX-007 §2-5）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FeedbackCategory {
    ChainIntegrity,     // "chain_integrity"
    LinkCandidate,      // "link_candidate"
    Drift,              // "drift"
    OrphanFile,         // "orphan_file"
    SemanticSimilarity, // "semantic_similarity"
}

impl FeedbackCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::ChainIntegrity => "chain_integrity",
            Self::LinkCandidate => "link_candidate",
            Self::Drift => "drift",
            Self::OrphanFile => "orphan_file",
            Self::SemanticSimilarity => "semantic_similarity",
        }
    }
}

/// 新規記録前の Observation 入力値（DD §2.1）。
/// source / severity は FB 内部で決定するため呼出し側（CLI / AutoObserver）が設定する。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewObservation {
    pub source: String,       // "manual" | "auto:{category}" | "drift:contextual_retrieval"
    pub category: String,     // 凍結 3 値（CLI/MCP 入口）or feedback 生成カテゴリ
    pub severity: String,     // "error" | "warning" | "info"
    pub message: String,      // trim 後 1 文字以上（REQ.01 GAP-LGX-121）
    pub related_ids: Vec<String>, // 正準化は record() 内部（distinct→昇順 sort→JSON）
    pub context_json: Option<String>, // --target-file / --missing-doc / --source-glob 由来
}

/// engine.db に永続化済みの Observation（DD §2.1）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Observation {
    pub id: i64,
    pub source: String,
    pub category: String,
    pub severity: String,
    pub message: String,
    pub related_ids: Vec<String>,
    pub context_json: Option<String>,
    pub status: ObservationStatus,
    pub created_at: String,
}

/// CheckReport → NewObservation 列の変換器（DD §3.1）。read-only。
pub struct AutoObserver;

impl AutoObserver {
    /// フィルタ規則（severity=Ok 除外 / FileExistence×Error 除外 / DocumentId×Warning 除外 /
    /// 既知 5 カテゴリのみ）に従い NewObservation 列を生成。message には mask_api_key を適用。
    /// read-only（report 不変）。DD §3.1 凍結シグネチャ。
    pub fn from_check_results(report: &CheckReport) -> Vec<NewObservation> {
        report
            .findings
            .iter()
            .filter_map(Self::to_new_observation)
            .collect()
    }

    /// 単一 finding → NewObservation。除外規則に該当すれば None（DD §3.1 / v3 lx-feedback 底本）。
    fn to_new_observation(r: &CheckResult) -> Option<NewObservation> {
        if matches!(r.severity, Severity::Ok) {
            return None;
        }
        if matches!(r.category, CheckCategory::FileExistence) && matches!(r.severity, Severity::Error)
        {
            return None;
        }
        if matches!(r.category, CheckCategory::DocumentId) && matches!(r.severity, Severity::Warning)
        {
            return None;
        }

        let category = match r.category {
            CheckCategory::ChainIntegrity => "chain_integrity",
            CheckCategory::OrphanFile => "orphan_file",
            // check 意味層（SPEC-LGX-004.REQ.02）は 3 種の所見を一律 CheckCategory::SemanticSimilarity で
            // 発行する。FeedbackCategory（DD-LGX-008 §2.2）の link_candidate / drift / semantic_similarity へは
            // 所見の構造（severity + related_ids 数）で逆多重化する（R-4/R-5）。message 文字列ではなく
            // check_semantic の発行形に基づく決定論的判定:
            //   - リンク候補:    Info + 2 ids   → link_candidate（変換可: add_link）
            //   - content drift:  Warning + 1 id → drift（変換可: update_doc）
            //   - 低類似度エッジ:  Warning + 2 ids → semantic_similarity（変換規則なし → skipped）
            //   - embeddings未生成 notice: Info + 0 id → semantic_similarity（誤った add_link を防ぐ、skipped）
            CheckCategory::SemanticSimilarity => match (r.severity, r.related_ids.len()) {
                (Severity::Info, 2) => "link_candidate",
                (Severity::Warning, 1) => "drift",
                _ => "semantic_similarity",
            },
            // それ以外（Freshness / Subnode* / UnresolvedEdge / GraphDag / Id* / FileExistence(非Error)
            // / DocumentId(非Warning)）は Proposal 化ルールがないため Observation 化しない（DD §3.1）。
            _ => return None,
        };

        let severity = match r.severity {
            Severity::Error => "error",
            Severity::Warning => "warning",
            Severity::Info => "info",
            Severity::Ok => return None,
        };

        let mut related: Vec<String> = r.related_ids.iter().map(|id| id.as_str().to_string()).collect();
        related.sort();

        // message には mask_api_key を適用（NFR SEC.05）。finding.message が空の場合は record の
        // EmptyObservationMessage 不変（ケース 1）に抵触しないよう category 由来の非空フォールバックを
        // 合成する（auto 生成 observation は manual observe の trim 空拒否対象ではない）。
        let masked = mask_api_key(&r.message);
        let message = if masked.trim().is_empty() {
            format!("{} finding", category)
        } else {
            masked
        };

        Some(NewObservation {
            source: format!("auto:{}", category),
            category: category.to_string(),
            severity: severity.to_string(),
            message,
            related_ids: related,
            context_json: None,
        })
    }
}

/// EmbedError から drift カテゴリ Observation を生成（DD §3.1）。
/// `ContextualRetrievalFailed` の場合のみ `Some`。message は必ず mask_api_key を通す（SEC.05）。
pub fn drift_from_embed_error(err: &EmbedError, node_id: &str) -> Option<NewObservation> {
    match err {
        EmbedError::ContextualRetrievalFailed { detail, .. } => Some(NewObservation {
            source: "drift:contextual_retrieval".to_string(),
            category: "drift".to_string(),
            severity: "warning".to_string(),
            message: mask_api_key(detail),
            related_ids: vec![node_id.to_string()],
            context_json: None,
        }),
        _ => None,
    }
}
