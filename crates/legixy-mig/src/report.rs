// (module of SRC-LGX-009; anchor: lib.rs)
// legixy-mig: InitReport / MigrationReport / MigrationSummaryJson / MigOutputFormat
// および付随データ型（DD-LGX-009 §2.1 / §3 MigrateOpts）。
// データ型は実体（pub フィールド付き、テストが構築可能）。to_json のロジックは SRC[GREEN]。

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::error::MigError;

/// init 結果サマリ（DD-LGX-009 §2.1 / SEQD-LGX-009 §1 / REQ.07）。
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct InitReport {
    /// 新規生成したファイル一覧（stdout 変更サマリ用、REQ.08）
    pub created_files: Vec<PathBuf>,
    /// 既に存在しスキップしたファイル（force=false 時は AlreadyExists で早期中断のため通常空）
    pub skipped_files: Vec<PathBuf>,
    /// 生成した engine.db の絶対パス（.legixy/engine.db、REQ.07）
    pub engine_db_path: PathBuf,
}

/// migrate 実行結果サマリ（DD-LGX-009 §2.1 / SEQD-LGX-009 §2 / REQ.08）。
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MigrationReport {
    /// 生成・更新したファイル一覧（REQ.08 成功時変更サマリ: stdout）
    pub files_written: Vec<PathBuf>,
    /// 書き換えた ID の件数（REQ.08, REQ.11）
    pub ids_rewritten_count: usize,
    /// id-map ファイルの配置パス（REQ.11: .legixy/migration-id-map.toml）
    pub id_map_path: PathBuf,
    /// 退避ファイルのパス一覧（REQ.02a: <元名>.bak.{epoch}）
    pub backup_paths: Vec<PathBuf>,
    /// 非致命警告（vectors.bin スキップ等、REQ.08: stderr 経由で別途出力）
    pub warnings: Vec<String>,
    /// コピーしたテーブル名（DB 移行レポート用）
    pub tables_copied: Vec<String>,
    /// コピーしたレコード総数
    pub rows_copied: usize,
}

/// --format の出力形式（LGX-COMPAT-001 §4 #2 凍結、DD-LGX-009 §2.1）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MigrationSummaryJson {
    pub files_written: Vec<String>,
    pub ids_rewritten_count: usize,
    pub id_map_path: String,
    pub backups: Vec<String>,
    pub warnings: Vec<String>,
}

impl MigrationReport {
    /// `MigrationSummaryJson` スキーマの JSON 出力（DD-LGX-009 §3、REQ.08 / `--format json`）。
    /// stdout に出力。診断・進捗は stderr（NFR OBS.02）。SRC[GREEN] で実装する。
    pub fn to_json(&self) -> String {
        let summary = MigrationSummaryJson {
            files_written: self
                .files_written
                .iter()
                .map(|p| p.to_string_lossy().to_string())
                .collect(),
            ids_rewritten_count: self.ids_rewritten_count,
            id_map_path: self.id_map_path.to_string_lossy().to_string(),
            backups: self
                .backup_paths
                .iter()
                .map(|p| p.to_string_lossy().to_string())
                .collect(),
            warnings: self.warnings.clone(),
        };
        serde_json::to_string(&summary).unwrap_or_default()
    }
}

/// migrate のオプション（DD-LGX-009 §3 MigrateOpts）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MigrateOpts {
    pub dry_run: bool,
    pub format: MigOutputFormat,
    pub unmapped_policy: UnmappedIdPolicy,
}

/// 出力形式（DD-LGX-009 §3、LGX-COMPAT-001 §4 #2 凍結）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MigOutputFormat {
    Markdown,
    Json,
}

/// マッピング不可 ID の継続フラグ（DD-LGX-009 §2.2 / REQ.11 / SUPP-008 §2.15）。
/// `--skip-unmapped` は新規追加引数（HR7 加算的追加）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnmappedIdPolicy {
    Abort,    // 既定（REQ.11: 非破壊性優先）
    SkipEdge, // --skip-unmapped 指定時: 当該エッジを除外して継続（dangling 防止）
}

/// init --force 退避ファイル命名で使用する epoch + 連番（DD-LGX-009 §2.1）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackupName {
    /// `<元名>.bak.{epoch}` または `<元名>.bak.{epoch}.{seq}`（REQ.02a / SUPP-008 §2.12）
    pub path: PathBuf,
}

// 参照されない場合の警告抑止（report 内では MigError を `?` 経由で間接利用する想定）。
#[allow(dead_code)]
fn _assert_migerror_in_scope(e: MigError) -> MigError {
    e
}
