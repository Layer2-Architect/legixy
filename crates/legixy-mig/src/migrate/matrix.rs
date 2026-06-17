// (module of SRC-LGX-009; anchor: lib.rs)
// legixy-mig::migrate::matrix — parse_matrix()（DD-LGX-009 §3 / SUPP-008 §2.5 抽出規則）。
// 空入力（抽出 0 件）= 正常、[id.chain] 欠落 = ChainConfigMissing（破損 Error）。

use crate::error::MigError;
use crate::migrate::config_parse::MigrationConfig;

/// 成果物 ID 集合（マトリクスパース結果、DD-LGX-009 §2.1 / SEQD-LGX-009 §2）。
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ArtifactIdSet {
    /// ノード（typecode, id_str）のコレクション（重複排除済）
    pub items: Vec<ArtifactItem>,
}

/// マトリクス内の単一成果物項目（DD-LGX-009 §2.1）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArtifactItem {
    pub typecode: String,
    pub id_str: String,
    pub path: Option<String>,
}

/// マトリクスパース（DD-LGX-009 §3、凍結 API surface）。
/// SUPP-008 §2.5 抽出規則。0 件は空正常（REQ.03 空入力）。[id.chain] 欠落は ChainConfigMissing。
/// [id.chains] 変種は MigrationConfig.is_multi_area=true で受理（ADR-LGX-018#15）。
pub fn parse_matrix(content: &str, config: &MigrationConfig) -> Result<ArtifactIdSet, MigError> {
    // chain 定義欠落 = 破損（構造情報の黙殺禁止、REQ.03）。
    if config.chain_order.is_empty() {
        return Err(MigError::ChainConfigMissing);
    }

    // `|` 始まりの table 行のみ対象（SUPP-008 §2.5）。
    let table_lines: Vec<&str> = content
        .lines()
        .map(|l| l.trim())
        .filter(|l| l.starts_with('|'))
        .collect();

    // 先頭行（ヘッダ）はノード化しない。区切り行（--- のみ）はスキップ。
    let mut items: Vec<ArtifactItem> = Vec::new();
    let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();

    for line in table_lines.iter().skip(1) {
        if is_separator_row(line) {
            continue;
        }
        let cells = split_row(line);
        // ID パターン一致セルを抽出（`-` / 空は不在）。
        for cell in &cells {
            if cell == "-" || cell.is_empty() {
                continue;
            }
            if let Some(typecode) = id_typecode(cell) {
                if seen.insert(cell.clone()) {
                    let path = cells
                        .iter()
                        .find(|c| is_path_cell(c))
                        .map(|c| c.to_string());
                    items.push(ArtifactItem {
                        typecode,
                        id_str: cell.clone(),
                        path,
                    });
                }
            }
        }
    }

    Ok(ArtifactIdSet { items })
}

/// `| ID | path |` 行をセル列へ分割（前後の `|` 除去 + トリム）。
fn split_row(line: &str) -> Vec<String> {
    let inner = line.trim().trim_start_matches('|').trim_end_matches('|');
    inner.split('|').map(|c| c.trim().to_string()).collect()
}

/// 区切り行（`---|---|...` 形式、`-` と空白のみ）か。
fn is_separator_row(line: &str) -> bool {
    let inner = line.trim().trim_start_matches('|').trim_end_matches('|');
    let trimmed = inner.replace(['|', '-', ' ', ':'], "");
    trimmed.is_empty() && inner.contains('-')
}

/// セルが `{TYPE}-{AREA}-{seq}` 形式の ID か判定し、type コードを返す。
/// area / seq 桁は固定せず、`-` 区切り 3 トークン・末尾が数字・先頭が英大文字で近似。
fn id_typecode(cell: &str) -> Option<String> {
    let parts: Vec<&str> = cell.split('-').collect();
    if parts.len() != 3 {
        return None;
    }
    let (typecode, _area, seq) = (parts[0], parts[1], parts[2]);
    if typecode.is_empty() || !typecode.chars().all(|c| c.is_ascii_uppercase()) {
        return None;
    }
    if seq.is_empty() || !seq.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    Some(typecode.to_string())
}

/// セルがパスらしい（`/` を含む、または `.md` 等の拡張子）か。
fn is_path_cell(cell: &str) -> bool {
    cell.contains('/') || cell.ends_with(".md") || cell.ends_with(".rs")
}
