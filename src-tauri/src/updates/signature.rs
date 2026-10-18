//! Checks an update's minisign signature the way the Tauri updater does: base64 of the public
//! key and signature files, and the version the signature was made for.

use base64::Engine;
use minisign_verify::{PublicKey, Signature};

fn decoded(base64_text: &str, what: &str) -> Result<String, String> {
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(base64_text.trim())
        .map_err(|e| format!("The {what} is not valid base64: {e}"))?;
    String::from_utf8(bytes).map_err(|_| format!("The {what} is not text"))
}

/// Verifies that `data` was signed with the key `public_key` for `version`. A signature
/// without a version is rejected, so an older signed file cannot pass for a newer release.
pub fn verify(data: &[u8], signature: &str, public_key: &str, version: &str) -> Result<(), String> {
    if public_key.trim().is_empty() {
        return Err("This build has no public key for updates".into());
    }
    let key = PublicKey::decode(&decoded(public_key, "public key")?).map_err(|e| format!("Invalid public key: {e}"))?;
    let signature = Signature::decode(&decoded(signature, "signature")?).map_err(|e| format!("Invalid signature: {e}"))?;
    key.verify(data, &signature, true).map_err(|e| format!("The signature does not match: {e}"))?;
    // Read only now: the trusted comment is covered by the signature checked above.
    let signed = signature
        .trusted_comment()
        .split('\t')
        .find_map(|field| field.strip_prefix("version:"))
        .ok_or("The signature does not say which version it was made for")?;
    let same = match (semver::Version::parse(signed.trim_start_matches('v')), semver::Version::parse(version.trim_start_matches('v'))) {
        (Ok(a), Ok(b)) => a == b,
        _ => signed == version,
    };
    if same {
        Ok(())
    } else {
        Err(format!("The signature is for version {signed}, not {version}"))
    }
}

#[cfg(test)]
mod tests {
    use super::verify;

    // Made with `tauri signer` and a throwaway key; the file is FILE.
    const FILE: &[u8] = b"Query2Table update test file\n";
    const KEY: &str = "dW50cnVzdGVkIGNvbW1lbnQ6IG1pbmlzaWduIHB1YmxpYyBrZXk6IDY2QjQwMzAwNjRGNkMwQkIKUldTN3dQWmtBQU8wWnJHT0xKOXlHRkhGRmsyRzlUOFdRdUJib1JvUVJzUzU2SmxwSGcyTEtybnEK";
    const OTHER_KEY: &str = "dW50cnVzdGVkIGNvbW1lbnQ6IG1pbmlzaWduIHB1YmxpYyBrZXk6IEM1MEZENUREN0FGRkU2MTMKUldRVDV2OTYzZFVQeGExUE5pWUpQTXVWbGRNWU8rckJFWUxuWFU2QXh4YlNLdUNjOGJpaFFFNksK";
    /// Signed with `--app-version 1.2.3`.
    const SIGNED_123: &str = "dW50cnVzdGVkIGNvbW1lbnQ6IHNpZ25hdHVyZSBmcm9tIHRhdXJpIHNlY3JldCBrZXkKUlVTN3dQWmtBQU8wWmxLVXg3KzZ2OTVPYlN3TTBuaU5rSlBzWEVVSGxaUWdkanNFREFDY1FnK2E2MmJZTVIzdmpTdzFrNTRwZXp3MHFNdjVUUDRodk9WQmRubGhteWFqdFF3PQp0cnVzdGVkIGNvbW1lbnQ6IHRpbWVzdGFtcDoxNzkyMzMxMzE4CWZpbGU6YXBwLmFwawl2ZXJzaW9uOjEuMi4zClRzSVE4VktROTNOVE9GWXY1YkdwR2NZbHJjeDlMd2p5UW9pd1RHSHZyNUxrU014K2d3SWRleUhZYmtPbXh5RXdZdmlzVmFPQXlsZ28ybzdkTWNMQkJ3PT0K";
    /// Signed without a version.
    const UNVERSIONED: &str = "dW50cnVzdGVkIGNvbW1lbnQ6IHNpZ25hdHVyZSBmcm9tIHRhdXJpIHNlY3JldCBrZXkKUlVTN3dQWmtBQU8wWnI3VWZvMDQyYUMxMExlUU1rclBYaVdHSnhCU2xCQ1A2RTJpMmtvdHVTRHV3bzdCVmZRVmFlb0htN1RrbUg4YjdnQTUvUUNmZjB4dU1MeERxMkZjZWdRPQp0cnVzdGVkIGNvbW1lbnQ6IHRpbWVzdGFtcDoxNzkyMzMxMzE5CWZpbGU6YXBwLmFwawp3RGYzK2wyQnp2M1E3SmUvenFnOHdVRU9ENWpvVmYvb2hIcDVMWnFSTGF6VDc5b2dpN3QzWUFETmVhUTYrRWlSMWdpMnFVZUhIU1ZvR0dOaDB6by9EUT09Cg==";

    #[test]
    fn the_signed_file_passes() {
        assert_eq!(verify(FILE, SIGNED_123, KEY, "1.2.3"), Ok(()));
        assert_eq!(verify(FILE, SIGNED_123, KEY, "v1.2.3"), Ok(()));
    }

    #[test]
    fn a_changed_file_fails() {
        let error = verify(b"Query2Table update test file!\n", SIGNED_123, KEY, "1.2.3").unwrap_err();
        assert!(error.starts_with("The signature does not match"), "{error}");
    }

    #[test]
    fn another_key_fails() {
        assert!(verify(FILE, SIGNED_123, OTHER_KEY, "1.2.3").unwrap_err().starts_with("The signature does not match"));
    }

    #[test]
    fn the_version_must_be_the_signed_one() {
        assert_eq!(verify(FILE, SIGNED_123, KEY, "1.2.4").unwrap_err(), "The signature is for version 1.2.3, not 1.2.4");
        assert_eq!(verify(FILE, UNVERSIONED, KEY, "1.2.3").unwrap_err(), "The signature does not say which version it was made for");
    }

    #[test]
    fn broken_inputs_fail() {
        assert_eq!(verify(FILE, SIGNED_123, "", "1.2.3").unwrap_err(), "This build has no public key for updates");
        assert!(verify(FILE, "not base64!", KEY, "1.2.3").is_err());
        assert!(verify(FILE, SIGNED_123, "bm90IGEga2V5", "1.2.3").is_err());
    }
}
