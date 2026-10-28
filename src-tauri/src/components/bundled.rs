//! Tools inside the Android APK, built by youtubedl-android: Python runs the yt-dlp zipapp and
//! QuickJS is its JavaScript engine. Both are executables packed as native libraries, because
//! Android lets apps run files only from their native library folder. Python's own libraries
//! come as an archive, unpacked once per app version.

use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use super::ComponentError;

/// The archive of Python's shared libraries, and the folder it is unpacked into.
const PYTHON_PACKAGE: (&str, &str) = ("libpython.zip.so", "python");
/// Records which archive was unpacked, to redo it after an app update.
const SOURCE_FILE: &str = ".source";

#[derive(Debug, Clone)]
pub struct Bundled {
    /// `applicationInfo.nativeLibraryDir`.
    lib_dir: PathBuf,
    /// Where the Python archive is unpacked.
    packages_dir: PathBuf,
    /// Held while unpacking, so callers wait for a background unpack to finish.
    prepared: Arc<Mutex<bool>>,
}

impl Bundled {
    pub fn new(lib_dir: PathBuf, packages_dir: PathBuf) -> Self {
        Self { lib_dir, packages_dir, prepared: Arc::default() }
    }

    /// The folders MainActivity exports; `None` outside the Android app.
    pub fn from_env() -> Option<Self> {
        Some(Self::new(std::env::var_os("Q2T_NATIVE_LIB_DIR")?.into(), std::env::var_os("Q2T_PACKAGES_DIR")?.into()))
    }

    pub fn python(&self) -> PathBuf {
        self.lib_dir.join("libpython.so")
    }

    pub fn quickjs(&self) -> PathBuf {
        self.lib_dir.join("libqjs.so")
    }

    fn python_home(&self) -> PathBuf {
        self.packages_dir.join(PYTHON_PACKAGE.1).join("usr")
    }

    /// Unpacks Python's libraries unless this app version already did. Must finish before
    /// Python runs.
    pub fn prepare(&self) -> Result<(), ComponentError> {
        let mut prepared = self.prepared.lock().unwrap_or_else(|e| e.into_inner());
        if !*prepared {
            unpack(&self.lib_dir.join(PYTHON_PACKAGE.0), &self.packages_dir.join(PYTHON_PACKAGE.1))?;
            *prepared = true;
        }
        Ok(())
    }

    /// The environment the bundled Python needs to find its libraries and certificates.
    pub fn env(&self) -> Vec<(OsString, OsString)> {
        let home = self.python_home();
        vec![
            ("LD_LIBRARY_PATH".into(), home.join("lib").into()),
            ("SSL_CERT_FILE".into(), home.join("etc/tls/cert.pem").into()),
            ("PYTHONHOME".into(), home.clone().into()),
            ("HOME".into(), home.into()),
        ]
    }
}

/// Unpacks into a temporary folder renamed into place, so an interrupted unpack is redone.
fn unpack(archive: &Path, dest: &Path) -> Result<(), ComponentError> {
    let meta = std::fs::metadata(archive)?;
    let source = format!("{} {:?}", meta.len(), meta.modified().ok());
    if std::fs::read_to_string(dest.join(SOURCE_FILE)).ok() == Some(source.clone()) {
        return Ok(());
    }
    tracing::info!(archive = %archive.display(), "unpacking bundled libraries");
    let tmp = dest.with_extension("tmp");
    let _ = std::fs::remove_dir_all(&tmp);
    std::fs::create_dir_all(&tmp)?;
    extract(archive, &tmp)?;
    std::fs::write(tmp.join(SOURCE_FILE), source)?;
    let _ = std::fs::remove_dir_all(dest);
    std::fs::rename(&tmp, dest)?;
    Ok(())
}

#[cfg(unix)]
fn extract(archive: &Path, dest: &Path) -> Result<(), ComponentError> {
    use std::os::unix::fs::PermissionsExt;

    let bad = |e: zip::result::ZipError| ComponentError::Other(format!("bad archive: {e}"));
    let mut zip = zip::ZipArchive::new(std::fs::File::open(archive)?).map_err(bad)?;
    for i in 0..zip.len() {
        let mut entry = zip.by_index(i).map_err(bad)?;
        let Some(path) = entry.enclosed_name().map(|p| dest.join(p)) else {
            return Err(ComponentError::Other(format!("unsafe path {:?}", entry.name())));
        };
        if entry.is_dir() {
            std::fs::create_dir_all(&path)?;
            continue;
        }
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        if entry.is_symlink() {
            let mut target = String::new();
            std::io::Read::read_to_string(&mut entry, &mut target)?;
            std::os::unix::fs::symlink(target, &path)?;
            continue;
        }
        std::io::copy(&mut entry, &mut std::fs::File::create(&path)?)?;
        if let Some(mode) = entry.unix_mode() {
            std::fs::set_permissions(&path, std::fs::Permissions::from_mode(mode & 0o777))?;
        }
    }
    Ok(())
}

#[cfg(not(unix))]
fn extract(_archive: &Path, _dest: &Path) -> Result<(), ComponentError> {
    Err(ComponentError::Unsupported("bundled tools".into()))
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use std::io::Write;

    /// An archive like libpython.zip.so: files plus symlinks (libz.so -> libz.so.1).
    fn write_archive(path: &Path, version: &str) {
        let mut zip = zip::ZipWriter::new(std::fs::File::create(path).unwrap());
        let opts = zip::write::SimpleFileOptions::default();
        zip.add_directory("usr/lib/", opts).unwrap();
        zip.start_file("usr/lib/libz.so.1", opts).unwrap();
        zip.write_all(version.as_bytes()).unwrap();
        zip.add_symlink("usr/lib/libz.so", "libz.so.1", opts).unwrap();
        zip.finish().unwrap();
    }

    #[test]
    fn python_is_unpacked_once_per_app_version() {
        let dir = tempfile::tempdir().unwrap();
        let lib_dir = dir.path().join("lib");
        std::fs::create_dir_all(&lib_dir).unwrap();
        write_archive(&lib_dir.join("libpython.zip.so"), "python 1");
        let packages = dir.path().join("packages");

        Bundled::new(lib_dir.clone(), packages.clone()).prepare().unwrap();
        let python_lib = packages.join("python/usr/lib");
        // The dynamic linker follows the symlink to the real library.
        assert_eq!(std::fs::read_to_string(python_lib.join("libz.so")).unwrap(), "python 1");
        assert!(python_lib.join("libz.so").is_symlink());

        // An unchanged archive is not unpacked again.
        std::fs::write(python_lib.join("marker"), "").unwrap();
        Bundled::new(lib_dir.clone(), packages.clone()).prepare().unwrap();
        assert!(python_lib.join("marker").exists());

        // An app update brings a new archive: it replaces the old files as a whole.
        write_archive(&lib_dir.join("libpython.zip.so"), "python 2, longer");
        Bundled::new(lib_dir, packages.clone()).prepare().unwrap();
        assert!(!python_lib.join("marker").exists());
        assert_eq!(std::fs::read_to_string(python_lib.join("libz.so")).unwrap(), "python 2, longer");
    }

    /// An unpack killed halfway (the app closed on its first start) is not trusted.
    #[test]
    fn an_interrupted_unpack_is_redone() {
        let dir = tempfile::tempdir().unwrap();
        let lib_dir = dir.path().join("lib");
        std::fs::create_dir_all(&lib_dir).unwrap();
        write_archive(&lib_dir.join("libpython.zip.so"), "python");
        let packages = dir.path().join("packages");
        std::fs::create_dir_all(packages.join("python/usr/lib")).unwrap();

        Bundled::new(lib_dir, packages.clone()).prepare().unwrap();
        assert!(packages.join("python/usr/lib/libz.so.1").is_file());
    }

    #[test]
    fn python_finds_its_libraries_and_certificates() {
        let bundled = Bundled::new("/lib".into(), "/packages".into());
        let env: std::collections::HashMap<_, _> = bundled.env().into_iter().collect();
        assert_eq!(env[&OsString::from("PYTHONHOME")], OsString::from("/packages/python/usr"));
        assert_eq!(env[&OsString::from("LD_LIBRARY_PATH")], OsString::from("/packages/python/usr/lib"));
        assert_eq!(env[&OsString::from("SSL_CERT_FILE")], OsString::from("/packages/python/usr/etc/tls/cert.pem"));
        assert_eq!(bundled.python(), PathBuf::from("/lib/libpython.so"));
        assert_eq!(bundled.quickjs(), PathBuf::from("/lib/libqjs.so"));
    }
}
