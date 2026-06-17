// (module of SRC-LGX-009; anchor: lib.rs)
// legixy-mig::atomic — atomic_write()（DD-LGX-009 §3 / REQ.02）。
// 一時ファイル `{path}.tmp.{epoch}` に全量書き出し → File::sync_all() → std::fs::rename()。
// rename 失敗は AtomicWriteFailed。中断後再実行で同一最終状態に冪等収束。

use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::error::MigError;

/// atomic 書込（DD-LGX-009 §3、凍結 API surface）。
/// 一時ファイル `{path}.tmp.{epoch}` に全量書き出し → `File::sync_all()` → `std::fs::rename()`（REQ.02）。
/// rename 失敗は `AtomicWriteFailed`。中断後の再実行で同じ最終状態に収束（冪等）。
pub fn atomic_write(path: &Path, content: &[u8]) -> Result<(), MigError> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let tmp = tmp_path_for(path);

    {
        let mut f = File::create(&tmp)?;
        f.write_all(content)?;
        // 全量書き出し後に fsync（半端な書込を rename で可視化しない、REQ.02）。
        f.sync_all()?;
    }

    // atomic rename。失敗時は tmp を掃除して AtomicWriteFailed。
    std::fs::rename(&tmp, path).map_err(|source| {
        let _ = std::fs::remove_file(&tmp);
        MigError::AtomicWriteFailed {
            path: path.to_path_buf(),
            source,
        }
    })?;

    Ok(())
}

/// 一時ファイルパス `{path}.tmp.{epoch}`（DD-LGX-009 §3）。
fn tmp_path_for(path: &Path) -> PathBuf {
    let epoch = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let mut s = path.as_os_str().to_os_string();
    s.push(format!(".tmp.{}", epoch));
    PathBuf::from(s)
}
