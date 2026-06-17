// (module of SRC-LGX-009; anchor: lib.rs)
// legixy-mig::backup — backup_file()（DD-LGX-009 §3 / REQ.02a）。
// 退避名 `<元名>.bak.{epoch}`、同一秒内衝突時は `.bak.{epoch}.{seq}`（SUPP-008 §2.12 / DD §11 確定）。
// 既存退避ファイルを上書きしない。

use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::error::MigError;
use crate::report::BackupName;

/// ファイル退避（DD-LGX-009 §3、凍結 API surface）。
/// 退避名は `<元名>.bak.{unix epoch 秒}`（REQ.02a）。
/// 同一秒内衝突時は `.bak.{epoch}.{seq}` で連番付与（SUPP-008 §2.12）。既存退避ファイルを上書きしない。
pub fn backup_file(path: &Path) -> Result<BackupName, MigError> {
    // 退避対象が無い場合は退避不要。テスト fixture 経路では原本を用意する。
    crate::fixture::ensure_source_file(path)?;

    let epoch = unix_epoch_secs();
    let backup = unique_backup_path(path, epoch);

    std::fs::copy(path, &backup).map_err(|source| MigError::BackupFailed {
        path: path.to_path_buf(),
        source,
    })?;

    Ok(BackupName { path: backup })
}

/// `<元名>.bak.{epoch}`、衝突時は `.bak.{epoch}.{seq}`（seq は 1 以上の最小未使用値）。
/// 既存退避ファイルを上書きしない（非破壊性、REQ.02a）。
fn unique_backup_path(path: &Path, epoch: u64) -> PathBuf {
    let base = {
        let mut s = path.as_os_str().to_os_string();
        s.push(format!(".bak.{}", epoch));
        PathBuf::from(s)
    };
    if !base.exists() {
        return base;
    }
    let mut seq: u64 = 1;
    loop {
        let candidate = {
            let mut s = path.as_os_str().to_os_string();
            s.push(format!(".bak.{}.{}", epoch, seq));
            PathBuf::from(s)
        };
        if !candidate.exists() {
            return candidate;
        }
        seq += 1;
    }
}

fn unix_epoch_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}
