//! Which release files fit this system.

use super::ComponentError;

/// Release file names for one OS and architecture.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Assets {
    /// yt-dlp release file.
    pub ytdlp: &'static str,
    /// Rust target triple in Deno's release names (`deno-<triple>.zip`); empty when Deno is not downloaded.
    pub deno_triple: &'static str,
    pub exe_ext: &'static str,
}

/// Every OS and architecture releases are built for.
pub const SUPPORTED: &[(&str, &str)] = &[
    ("windows", "x86_64"),
    ("windows", "aarch64"),
    ("macos", "x86_64"),
    ("macos", "aarch64"),
    ("linux", "x86_64"),
    ("linux", "aarch64"),
];

pub fn current() -> Result<Assets, ComponentError> {
    assets_for(std::env::consts::OS, std::env::consts::ARCH)
}

/// Android: the platform-independent yt-dlp zipapp, run by the Python inside the APK.
pub fn zipapp() -> Assets {
    Assets { ytdlp: "yt-dlp", deno_triple: "", exe_ext: "" }
}

pub fn assets_for(os: &str, arch: &str) -> Result<Assets, ComponentError> {
    let (ytdlp, deno_triple, exe_ext) = match (os, arch) {
        ("windows", "x86_64") => ("yt-dlp.exe", "x86_64-pc-windows-msvc", ".exe"),
        ("windows", "aarch64") => ("yt-dlp_arm64.exe", "aarch64-pc-windows-msvc", ".exe"),
        // One universal build for both Mac architectures.
        ("macos", "aarch64") => ("yt-dlp_macos", "aarch64-apple-darwin", ""),
        ("macos", "x86_64") => ("yt-dlp_macos", "x86_64-apple-darwin", ""),
        ("linux", "x86_64") => ("yt-dlp_linux", "x86_64-unknown-linux-gnu", ""),
        ("linux", "aarch64") => ("yt-dlp_linux_aarch64", "aarch64-unknown-linux-gnu", ""),
        _ => return Err(ComponentError::Unsupported(format!("{os}/{arch}"))),
    };
    Ok(Assets { ytdlp, deno_triple, exe_ext })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_supported_system_has_its_files() {
        assert_eq!(assets_for("windows", "aarch64").unwrap().ytdlp, "yt-dlp_arm64.exe");
        assert_eq!(assets_for("windows", "x86_64").unwrap().exe_ext, ".exe");
        assert_eq!(assets_for("macos", "x86_64").unwrap().ytdlp, "yt-dlp_macos");
        assert_eq!(assets_for("linux", "aarch64").unwrap().deno_triple, "aarch64-unknown-linux-gnu");
        for (os, arch) in SUPPORTED {
            assert!(assets_for(os, arch).is_ok(), "{os}/{arch}");
        }
        assert!(current().is_ok());
    }

    #[test]
    fn other_systems_are_not_supported() {
        assert!(matches!(assets_for("freebsd", "x86_64"), Err(ComponentError::Unsupported(_))));
        assert!(matches!(assets_for("linux", "riscv64"), Err(ComponentError::Unsupported(_))));
    }
}
