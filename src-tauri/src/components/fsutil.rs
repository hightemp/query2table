//! Putting a downloaded tool in place.

use std::path::Path;

use super::ComponentError;

pub const TMP_PREFIX: &str = ".tmp-";
pub const OLD_SUFFIX: &str = ".old";

/// Makes a freshly downloaded tool runnable, before it is tried.
pub async fn prepare(tmp: &Path) -> Result<(), ComponentError> {
    make_executable(tmp)?;
    #[cfg(target_os = "macos")]
    macos_fixup(tmp).await;
    Ok(())
}

#[cfg(unix)]
fn make_executable(path: &Path) -> Result<(), ComponentError> {
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o755))?;
    Ok(())
}

#[cfg(not(unix))]
fn make_executable(_path: &Path) -> Result<(), ComponentError> {
    Ok(())
}

/// Moves `tmp` over `dest`, both in the same folder so the rename is atomic. Running processes
/// keep the old file.
#[cfg(unix)]
pub fn replace(tmp: &Path, dest: &Path) -> Result<(), ComponentError> {
    std::fs::rename(tmp, dest)?;
    Ok(())
}

/// Windows cannot overwrite a running program but can rename it: the current one is moved
/// aside (removed at the next start) under a unique name, since an earlier one may still run.
#[cfg(not(unix))]
pub fn replace(tmp: &Path, dest: &Path) -> Result<(), ComponentError> {
    if dest.exists() {
        let stamp = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_nanos()).unwrap_or_default();
        let mut old = dest.as_os_str().to_owned();
        old.push(format!(".{stamp}{OLD_SUFFIX}"));
        std::fs::rename(dest, &old)?;
    }
    std::fs::rename(tmp, dest)?;
    Ok(())
}

/// Removes a tool; on Windows a running one is moved aside instead.
pub fn remove(path: &Path) -> Result<(), ComponentError> {
    match std::fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        #[cfg(not(unix))]
        Err(_) => {
            let mut old = path.as_os_str().to_owned();
            old.push(OLD_SUFFIX);
            std::fs::rename(path, old)?;
            Ok(())
        }
        #[cfg(unix)]
        Err(e) => Err(e.into()),
    }
}

/// Files we download ourselves get no quarantine attribute, but it is cleared just in case.
/// Apple Silicon runs only signed code, so an unsigned tool is signed ad hoc.
#[cfg(target_os = "macos")]
async fn macos_fixup(path: &Path) {
    use tokio::process::Command;
    let _ = Command::new("xattr").args(["-d", "com.apple.quarantine"]).arg(path).output().await;
    let signed = Command::new("codesign").arg("-v").arg(path).output().await.is_ok_and(|o| o.status.success());
    if !signed {
        match Command::new("codesign").args(["--force", "-s", "-"]).arg(path).output().await {
            Ok(o) if o.status.success() => tracing::info!(?path, "ad-hoc signed tool"),
            Ok(o) => tracing::warn!(?path, stderr = %String::from_utf8_lossy(&o.stderr), "ad-hoc signing failed"),
            Err(e) => tracing::warn!(?path, error = %e, "codesign is not available"),
        }
    }
}

/// Removes what interrupted installs and Windows replacements left behind.
pub fn cleanup(dir: &Path) {
    let Ok(entries) = std::fs::read_dir(dir) else { return };
    for entry in entries.flatten() {
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if (name.starts_with(TMP_PREFIX) || name.ends_with(OLD_SUFFIX)) && std::fs::remove_file(entry.path()).is_err() {
            tracing::debug!(file = %name, "could not remove a stale file");
        }
    }
}
