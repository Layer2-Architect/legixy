// (module of SRC-LGX-009; anchor: lib.rs)
// legixy-mig::migrate::config_parse — MigrationConfig 抽出（DD-LGX-009 §2.1 / REQ.03/03a）。
// [id.chain]（単数形）/ [id.chains]+[id.areas]（複数形 multi-area 変種）両受理（ADR-LGX-018#15）。

use std::path::{Path, PathBuf};

use crate::error::MigError;

/// v0.1.0 設定から抽出したマイグレーション設定（DD-LGX-009 §2.1 / SEQD-LGX-009 §2 / REQ.03）。
#[derive(Debug, Clone, PartialEq)]
pub struct MigrationConfig {
    /// chain order（[id.chain].order または [id.chains] 変種から抽出）
    pub chain_order: Vec<String>,
    /// independent typecodes（[id.chain].independent から抽出）
    pub independent: Vec<String>,
    /// matrix.md 所在（[matrix].file または既定値）
    pub matrix_file: PathBuf,
    /// [matrix].section（パース対象節名）
    pub matrix_section: Option<String>,
    /// ID パターン文字列（[id].pattern）
    pub id_pattern: String,
    /// area コード（[id].area）
    pub id_area: String,
    /// seq_digits（[id].seq_digits）
    pub seq_digits: usize,
    /// multi-area 変種（[id.chains] + [id.areas] 使用フラグ、SUPP-008 §2.4）
    pub is_multi_area: bool,
    /// 元の TOML 全文（設定変換処理 REQ.04 で additive 変更に使用）
    pub raw_toml: toml::Value,
}

/// [id.chains] 変種の受理モード（DD-LGX-009 §2.2 / SUPP-008 §2.4 / ADR-LGX-018#15）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChainConfigVariant {
    Single,    // [id.chain].order（単数形）
    MultiArea, // [id.chains]（複数形）+ [id.areas]
}

/// v0.1.0 config（`.legixy.toml` 優先 → `.trace-engine.toml` フォールバック）から
/// `MigrationConfig` を抽出する（migrate 内部利用）。
///
/// - TOML パース失敗 → `MigError::TomlParse`（REQ.03a 破損）。
/// - `[id.chain].order`（単数形）と `[id.chains]`（複数形）のどちらも欠落 → `ChainConfigMissing`。
pub(crate) fn extract_migration_config(project_root: &Path) -> Result<MigrationConfig, MigError> {
    let (config_path, _used_fallback) = resolve_config_path(project_root).ok_or_else(|| {
        MigError::ConfigCorrupt {
            detail: "neither .legixy.toml nor .trace-engine.toml found".to_string(),
        }
    })?;

    let content = std::fs::read_to_string(&config_path)?;
    let raw: toml::Value = toml::from_str(&content)?; // 失敗は TomlParse（REQ.03a）

    let id_tbl = raw.get("id").and_then(|v| v.as_table());

    // chain order 抽出: [id.chain].order（単数形） or [id.chains]（複数形 multi-area）。
    let (chain_order, independent, is_multi_area) = extract_chain(id_tbl);

    if chain_order.is_empty() {
        // [id.chain] / [id.chains] 双方欠落 = 破損（構造情報の黙殺禁止、REQ.03）。
        return Err(MigError::ChainConfigMissing);
    }

    let id_pattern = id_tbl
        .and_then(|t| t.get("pattern"))
        .and_then(|v| v.as_str())
        .unwrap_or(r"^[A-Z]+-LGX-\d{3}$")
        .to_string();
    let id_area = id_tbl
        .and_then(|t| t.get("area"))
        .and_then(|v| v.as_str())
        .unwrap_or("LGX")
        .to_string();
    let seq_digits = id_tbl
        .and_then(|t| t.get("seq_digits"))
        .and_then(|v| v.as_integer())
        .map(|n| n as usize)
        .unwrap_or(3);

    let matrix_tbl = raw.get("matrix").and_then(|v| v.as_table());
    let matrix_file = matrix_tbl
        .and_then(|t| t.get("file"))
        .and_then(|v| v.as_str())
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("docs/traceability/matrix.md"));
    let matrix_section = matrix_tbl
        .and_then(|t| t.get("section"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    Ok(MigrationConfig {
        chain_order,
        independent,
        matrix_file,
        matrix_section,
        id_pattern,
        id_area,
        seq_digits,
        is_multi_area,
        raw_toml: raw,
    })
}

/// 設定ファイル探索（`.legixy.toml` 優先 → `.trace-engine.toml`）。戻り: (path, used_fallback)。
fn resolve_config_path(project_root: &Path) -> Option<(PathBuf, bool)> {
    let legixy = project_root.join(".legixy.toml");
    if legixy.exists() {
        return Some((legixy, false));
    }
    let trace = project_root.join(".trace-engine.toml");
    if trace.exists() {
        return Some((trace, true));
    }
    None
}

/// (chain_order, independent, is_multi_area) を抽出。
fn extract_chain(
    id_tbl: Option<&toml::value::Table>,
) -> (Vec<String>, Vec<String>, bool) {
    let Some(id_tbl) = id_tbl else {
        return (Vec::new(), Vec::new(), false);
    };

    // 単数形 [id.chain].order。
    if let Some(chain) = id_tbl.get("chain").and_then(|v| v.as_table()) {
        let order = chain
            .get("order")
            .and_then(|v| v.as_array())
            .map(|a| str_vec(a))
            .unwrap_or_default();
        let independent = chain
            .get("independent")
            .and_then(|v| v.as_array())
            .map(|a| str_vec(a))
            .unwrap_or_default();
        if !order.is_empty() {
            return (order, independent, false);
        }
    }

    // 複数形 [id.chains]（multi-area 変種）。最初の chain の order を採用。
    if let Some(chains) = id_tbl.get("chains").and_then(|v| v.as_table()) {
        for (_name, chain_val) in chains.iter() {
            if let Some(chain) = chain_val.as_table() {
                let order = chain
                    .get("order")
                    .and_then(|v| v.as_array())
                    .map(|a| str_vec(a))
                    .unwrap_or_default();
                let independent = chain
                    .get("independent")
                    .and_then(|v| v.as_array())
                    .map(|a| str_vec(a))
                    .unwrap_or_default();
                if !order.is_empty() {
                    return (order, independent, true);
                }
            }
        }
    }

    (Vec::new(), Vec::new(), false)
}

fn str_vec(arr: &[toml::Value]) -> Vec<String> {
    arr.iter()
        .filter_map(|v| v.as_str().map(|s| s.to_string()))
        .collect()
}
