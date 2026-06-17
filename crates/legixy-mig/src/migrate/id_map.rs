// (module of SRC-LGX-009; anchor: lib.rs)
// legixy-mig::migrate::id_map — generate_id_map()（DD-LGX-009 §3 / REQ.11 / SHA-256 方式）。
// 全単射違反は IdBijectionViolation（v3 の桁伸長回避は不採用）。--dry-run でも検証実施。

use std::collections::HashSet;

use sha2::{Digest, Sha256};

use crate::error::MigError;
use crate::migrate::config_parse::MigrationConfig;
use crate::migrate::matrix::ArtifactIdSet;
use crate::report::UnmappedIdPolicy;

/// ID マッピング表（DD-LGX-009 §2.1 / REQ.11 / SEQD-LGX-009 §2）。
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MigrationIdMap {
    pub mappings: Vec<IdMapping>,
}

/// 単一の旧→新 ID 対応（DD-LGX-009 §2.1）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IdMapping {
    pub old_id: String,
    pub new_id: String,
    pub confidence: IdMapConfidence,
}

/// ID マッピング確信度（DD-LGX-009 §2.2 / REQ.11 / SUPP-008 §2.7）。
/// 衝突は MigError::IdBijectionViolation で Error にするため High のみが通常ケース。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IdMapConfidence {
    High, // 衝突なし（唯一の SHA-256 先頭 seq_digits 桁で確定）
}

/// ID マッピング生成（DD-LGX-009 §3、凍結 API surface）。
/// SHA-256 ハッシュ入力 `"{path}\n{typecode}"`（SUPP-008 §2.7）。
/// 旧 ID 重複 / 新 ID 衝突 / graph 全体一意性違反は IdBijectionViolation（REQ.11 GAP-LGX-159）。
/// マッピング不可 ID は policy に従い Abort（UnmappedIds）または SkipEdge。--dry-run でも検証実施。
pub fn generate_id_map(
    artifact_set: &ArtifactIdSet,
    existing_refs: &[String],
    config: &MigrationConfig,
    policy: UnmappedIdPolicy,
) -> Result<MigrationIdMap, MigError> {
    let mut mappings: Vec<IdMapping> = Vec::new();
    let mut seen_old: HashSet<String> = HashSet::new();
    let mut seen_new: HashSet<String> = HashSet::new();

    for item in &artifact_set.items {
        // 旧 ID 重複（曖昧性）= 全単射違反。
        if !seen_old.insert(item.id_str.clone()) {
            return Err(MigError::IdBijectionViolation {
                detail: format!("duplicate old id: {}", item.id_str),
            });
        }

        let new_id = generate_new_id(item, config);

        // 新 ID 衝突（多対一）= 全単射違反。
        if !seen_new.insert(new_id.clone()) {
            return Err(MigError::IdBijectionViolation {
                detail: format!(
                    "new id collision: old={} -> new={}",
                    item.id_str, new_id
                ),
            });
        }

        mappings.push(IdMapping {
            old_id: item.id_str.clone(),
            new_id,
            confidence: IdMapConfidence::High,
        });
    }

    // 解決できない参照（dangling 候補）の検出。
    let unmapped: Vec<String> = existing_refs
        .iter()
        .filter(|r| !seen_old.contains(r.as_str()))
        .cloned()
        .collect();

    if !unmapped.is_empty() {
        match policy {
            // 既定: 非破壊性優先で中断。
            UnmappedIdPolicy::Abort => {
                return Err(MigError::UnmappedIds { ids: unmapped });
            }
            // --skip-unmapped: 当該エッジ除外で継続（dangling を残置しない）。
            UnmappedIdPolicy::SkipEdge => {
                // mappings に dangling 参照は含めない（既に解決済のみ）。
            }
        }
    }

    Ok(MigrationIdMap { mappings })
}

/// SHA-256 入力 `"{path}\n{typecode}"` の先頭 seq_digits 桁から新 ID を生成。
fn generate_new_id(item: &crate::migrate::matrix::ArtifactItem, config: &MigrationConfig) -> String {
    let path = item.path.clone().unwrap_or_default();
    let hash_hex = sha256_hex(&format!("{}\n{}", path, item.typecode));
    let digits = config.seq_digits.min(hash_hex.len());
    format!(
        "{}-{}-{}",
        item.typecode,
        config.id_area,
        &hash_hex[..digits]
    )
}

fn sha256_hex(s: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(s.as_bytes());
    hex::encode(hasher.finalize())
}
