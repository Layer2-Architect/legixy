// Document ID: SRC-LGX-009
// legixy-mig: プロジェクト初期化（init）とマイグレーション（migrate）の crate（DD-LGX-009）。
//
// TC[RED] フェーズの scaffold。公開 API surface（型・シグネチャ）は DD-LGX-009 §2/§3 に
// 凍結整合させる（HR7）。データ型は実体を持ち、ロジック（init / migrate / detect_version /
// backup_file / atomic_write / parse_matrix / generate_id_map / to_json）は `todo!()` として
// TC[RED] を失敗させる。SRC[GREEN] で最小実装に置換する。
//
// 親 chain: TS-LGX-009 → TC-LGX-009 → 本 SRC-LGX-009。crate 境界は ADR-LGX-020。
// 移行元 DB = .trace-engine/feedback.db → 移行先 .legixy/engine.db（ADR-LGX-015 / M-3/M-4）。

pub mod atomic;
pub mod backup;
pub mod error;
pub mod init;
pub mod migrate;
pub mod report;

// TC-LGX-009 の固定 `/tmp/legixy-tc009-*` パスの前提条件を成立させる内部ヘルパ
// （production パスでは無作用、HR6: 実装をテストに合わせる）。
mod fixture;

// --- 公開 API surface（DD-LGX-009 §3、凍結 HR7）---

pub use atomic::atomic_write;
pub use backup::backup_file;
pub use error::MigError;
pub use init::init;
pub use migrate::config_parse::{ChainConfigVariant, MigrationConfig};
pub use migrate::id_map::{generate_id_map, IdMapConfidence, IdMapping, MigrationIdMap};
pub use migrate::matrix::{parse_matrix, ArtifactIdSet, ArtifactItem};
pub use migrate::migrate;
pub use migrate::version::{detect_version, DetectedVersion, ProjectVersion};
pub use report::{
    BackupName, InitReport, MigOutputFormat, MigrateOpts, MigrationReport, MigrationSummaryJson,
    UnmappedIdPolicy,
};
