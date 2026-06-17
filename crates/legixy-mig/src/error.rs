// (module of SRC-LGX-009; anchor: lib.rs)
// legixy-mig: MigError（DD-LGX-009 §2.3）。
// 実行時失敗 = exit 1。使用法誤り（clap）= exit 2。成功 = exit 0（LGX-COMPAT-001 §3 / SUPP-008 §2.14）。
// 14 variant を DD §2.3 に忠実に再現する。エラーバリアントのフィールド名・型は凍結シグネチャ整合。

use std::path::PathBuf;

/// legixy-mig の実行時失敗（DD-LGX-009 §2.3、14 variant）。
#[derive(Debug, thiserror::Error)]
pub enum MigError {
    // --- init 固有 ---
    #[error("project already initialized at {path:?} (use --force to overwrite)")]
    AlreadyExists { path: PathBuf },

    // --- migrate 固有 ---
    #[error("v0.1.0 project not found at {path:?}")]
    V01NotFound { path: PathBuf },

    #[error("version mismatch: config={config_version}, db={db_version}")]
    VersionMismatch {
        config_version: String,
        db_version: String,
    },

    // 設定破損・必須セクション欠落（REQ.03a / SEQD-LGX-009 例外 E2）
    #[error("source config corrupt: {detail}")]
    ConfigCorrupt { detail: String },

    // [id.chain] / [id.chains] 双方なし（REQ.03: 破損として Error）
    #[error("chain config missing or invalid: neither [id.chain] nor [id.chains] found")]
    ChainConfigMissing,

    // REQ.03a: feedback.db 必須テーブル欠落
    #[error("schema incompatible: table {table}, detail: {detail}")]
    SchemaIncompatible { table: String, detail: String },

    // REQ.11 全単射違反（GAP-LGX-159 / SUPP-008 §2.7）
    #[error("id bijection violation: {detail}")]
    IdBijectionViolation { detail: String },

    // REQ.11 マッピング不可 ID（既定 abort、--skip-unmapped で SkipEdge）
    #[error("unmapped id(s) detected: {ids:?}")]
    UnmappedIds { ids: Vec<String> },

    // REQ.03a: 出力 graph.toml の妥当性検証失敗（SEQD-LGX-009 例外 E1）
    #[error("output graph validation failed: {detail}")]
    OutputGraphInvalid { detail: String },

    // 退避ファイル生成失敗（REQ.02a）
    #[error("backup failed for {path:?}: {source}")]
    BackupFailed {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    // atomic rename 失敗（REQ.02: temp+fsync+rename 方式）
    #[error("atomic write failed for {path:?}: {source}")]
    AtomicWriteFailed {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    // --- 共有エラー ---
    #[error("db error: {0}")]
    Db(#[from] legixy_db::DbError),

    #[error("sqlite error: {0}")]
    Sqlite(#[from] rusqlite::Error),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("toml parse error: {0}")]
    TomlParse(#[from] toml::de::Error),

    #[error("toml serialize error: {0}")]
    TomlSerialize(#[from] toml::ser::Error),

    #[error("config load error: {0}")]
    ConfigLoad(#[from] legixy_core::ConfigError),
}
