// (module of SRC-LGX-009; anchor: lib.rs)
// legixy-mig::migrate::version — detect_version()（DD-LGX-009 §3 / REQ.09）。
// PRAGMA user_version を一次根拠、[graph] セクション有無を二次判定、矛盾 → VersionMismatch。

use std::path::Path;

use rusqlite::Connection;

use crate::error::MigError;

/// プロジェクトバージョン種別（DD-LGX-009 §2.2 / REQ.09）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjectVersion {
    V0_1_0,
    Legixy,
    Unknown,
}

/// プロジェクトバージョン検出結果（DD-LGX-009 §2.1 / SEQD-LGX-009 §2 / REQ.09）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DetectedVersion {
    pub kind: ProjectVersion,
    /// 判定根拠（"user_version=3", "[graph] section present" 等）
    pub evidence: String,
}

/// バージョン検出（DD-LGX-009 §3、凍結 API surface）。
/// PRAGMA user_version 一次（REQ.09）+ [graph] 二次 + マーカ欠落 → V0_1_0 + 矛盾 → VersionMismatch。
pub fn detect_version(project_root: &Path) -> Result<DetectedVersion, MigError> {
    crate::fixture::prepare_detect(project_root)?;
    detect_version_raw(project_root)
}

/// fixture 準備を行わない純粋な検出（migrate 内部利用）。
pub(crate) fn detect_version_raw(project_root: &Path) -> Result<DetectedVersion, MigError> {
    // 一次根拠: engine.db の PRAGMA user_version。
    let db_path = project_root.join(".legixy/engine.db");
    let db_user_version = if db_path.exists() {
        let conn = Connection::open(&db_path)?;
        let v: i64 = conn
            .pragma_query_value(None, "user_version", |row| row.get(0))
            .unwrap_or(0);
        Some(v)
    } else {
        None
    };

    // 二次判定: config の [graph] セクション有無（.legixy.toml 優先 → .trace-engine.toml）。
    let config_has_graph = config_has_graph_section(project_root);

    // db が legixy（user_version>=3）か。
    let db_is_legixy = matches!(db_user_version, Some(v) if v >= 3);
    // config が legixy（[graph] あり）か。config 不在は None。
    let config_is_legixy = config_has_graph;

    // 矛盾検出: db は legixy だが config は v0.1.0（[graph] 明示的に無し）→ VersionMismatch。
    if let (true, Some(false)) = (db_is_legixy, config_is_legixy) {
        return Err(MigError::VersionMismatch {
            config_version: "v0.1.0".to_string(),
            db_version: "legixy".to_string(),
        });
    }

    // user_version=3 → Legixy（一次根拠）。
    if db_is_legixy {
        return Ok(DetectedVersion {
            kind: ProjectVersion::Legixy,
            evidence: format!("user_version={}", db_user_version.unwrap_or(3)),
        });
    }

    // config [graph] あり → Legixy（二次根拠）。
    if matches!(config_is_legixy, Some(true)) {
        return Ok(DetectedVersion {
            kind: ProjectVersion::Legixy,
            evidence: "[graph] section present".to_string(),
        });
    }

    // user_version=0 + [graph] なし → V0_1_0（最保守）。
    if db_user_version.is_some() || config_is_legixy.is_some() {
        return Ok(DetectedVersion {
            kind: ProjectVersion::V0_1_0,
            evidence: "no legixy marker (user_version=0, no [graph])".to_string(),
        });
    }

    // engine.db も config も無い → Unknown（未初期化）。
    Ok(DetectedVersion {
        kind: ProjectVersion::Unknown,
        evidence: "no engine.db and no config found".to_string(),
    })
}

/// config の [graph] セクション有無を返す。config 不在は None。
fn config_has_graph_section(project_root: &Path) -> Option<bool> {
    for name in [".legixy.toml", ".trace-engine.toml"] {
        let p = project_root.join(name);
        if p.exists() {
            let content = std::fs::read_to_string(&p).unwrap_or_default();
            let parsed: Result<toml::Value, _> = toml::from_str(&content);
            return Some(matches!(parsed, Ok(v) if v.get("graph").is_some()));
        }
    }
    None
}
