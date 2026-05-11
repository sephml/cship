//! `cship update` — downloads and installs the latest release without touching any config files.

use std::io::Read as _;
use std::time::Duration;

const REPO: &str = "stephenleo/cship";
const API_URL: &str = "https://api.github.com/repos/stephenleo/cship/releases/latest";
const DOWNLOAD_BASE: &str = "https://github.com/stephenleo/cship/releases/latest/download";

pub fn run() {
    let current_version = env!("CARGO_PKG_VERSION");

    let asset = match asset_name() {
        Ok(a) => a,
        Err(e) => {
            println!("Update not supported on this platform: {e}");
            return;
        }
    };

    let exe = match std::env::current_exe() {
        Ok(p) => p,
        Err(e) => {
            println!("Cannot determine current binary path: {e}");
            return;
        }
    };

    println!("Checking for updates to cship v{current_version}...");

    let agent = ureq::Agent::new_with_config(
        ureq::config::Config::builder()
            .timeout_global(Some(Duration::from_secs(30)))
            .build(),
    );

    let latest_tag = match fetch_latest_tag(&agent) {
        Ok(t) => t,
        Err(e) => {
            println!("Failed to check for updates: {e}");
            return;
        }
    };

    // GitHub tags are typically "v1.2.3"; strip leading 'v' before comparing.
    let latest_version = latest_tag.trim_start_matches('v');

    match compare_versions(latest_version, current_version) {
        VersionOrder::Equal => {
            println!("Already up to date (v{current_version}).");
            return;
        }
        VersionOrder::OlderThanCurrent => {
            println!(
                "You are running a pre-release version (v{current_version}); latest stable is v{latest_version}."
            );
            return;
        }
        VersionOrder::NewerThanCurrent => {}
        VersionOrder::Unparseable => {
            println!(
                "Could not compare versions (current: v{current_version}, latest: v{latest_version}) — aborting."
            );
            return;
        }
    }

    println!("New version available: v{latest_version} (current: v{current_version})");

    if std::env::var("CSHIP_UPDATE_DRY_RUN").is_ok() {
        println!("[dry run] download skipped.");
        return;
    }

    println!("Downloading {asset}...");

    let url = format!("{DOWNLOAD_BASE}/{asset}");
    let bytes = match download_bytes(&agent, &url) {
        Ok(b) => b,
        Err(e) => {
            println!("Download failed: {e}");
            return;
        }
    };

    if bytes.is_empty() {
        println!("Downloaded binary is empty — aborting.");
        return;
    }

    match replace_binary(&exe, &bytes) {
        Ok(()) => println!("cship updated to v{latest_version} successfully."),
        Err(e) => println!("Failed to replace binary: {e}"),
    }
}

fn asset_name() -> Result<String, String> {
    let os = std::env::consts::OS;
    let arch = std::env::consts::ARCH;

    let target = match (os, arch) {
        ("macos", "aarch64") => "aarch64-apple-darwin",
        ("macos", "x86_64") => "x86_64-apple-darwin",
        ("linux", "x86_64") => "x86_64-unknown-linux-musl",
        ("linux", "aarch64") => "aarch64-unknown-linux-musl",
        ("windows", "x86_64") => "x86_64-pc-windows-msvc",
        ("windows", "aarch64") => "aarch64-pc-windows-msvc",
        _ => return Err(format!("{os}/{arch}")),
    };

    let name = if os == "windows" {
        format!("cship-{target}.exe")
    } else {
        format!("cship-{target}")
    };

    Ok(name)
}

fn fetch_latest_tag(agent: &ureq::Agent) -> Result<String, String> {
    let mut resp = agent
        .get(API_URL)
        .header(
            "User-Agent",
            &format!("cship/{} ({})", env!("CARGO_PKG_VERSION"), REPO),
        )
        .call()
        .map_err(|e| format!("network error: {e}"))?;

    if resp.status() != 200 {
        return Err(format!("GitHub API returned {}", resp.status()));
    }

    let body = resp
        .body_mut()
        .read_to_string()
        .map_err(|e| format!("failed to read response: {e}"))?;

    let json: serde_json::Value =
        serde_json::from_str(&body).map_err(|e| format!("failed to parse response: {e}"))?;

    json["tag_name"]
        .as_str()
        .map(|s| s.to_owned())
        .ok_or_else(|| "missing tag_name in release response".to_owned())
}

fn download_bytes(agent: &ureq::Agent, url: &str) -> Result<Vec<u8>, String> {
    let mut resp = agent
        .get(url)
        .header(
            "User-Agent",
            &format!("cship/{} ({})", env!("CARGO_PKG_VERSION"), REPO),
        )
        .call()
        .map_err(|e| format!("network error: {e}"))?;

    if resp.status() != 200 {
        return Err(format!("server returned {}", resp.status()));
    }

    let mut buf = Vec::new();
    resp.body_mut()
        .as_reader()
        .read_to_end(&mut buf)
        .map_err(|e| format!("failed to read download: {e}"))?;

    Ok(buf)
}

enum VersionOrder {
    NewerThanCurrent,
    Equal,
    OlderThanCurrent,
    Unparseable,
}

fn parse_semver(v: &str) -> Option<(u64, u64, u64)> {
    let mut parts = v.splitn(3, '.');
    let major = parts.next()?.parse().ok()?;
    let minor = parts.next()?.parse().ok()?;
    // Ignore any pre-release suffix after the patch number.
    let patch_str = parts.next().unwrap_or("0");
    let patch = patch_str
        .split(|c: char| !c.is_ascii_digit())
        .next()?
        .parse()
        .ok()?;
    Some((major, minor, patch))
}

fn compare_versions(latest: &str, current: &str) -> VersionOrder {
    match (parse_semver(latest), parse_semver(current)) {
        (Some(l), Some(c)) if l > c => VersionOrder::NewerThanCurrent,
        (Some(l), Some(c)) if l == c => VersionOrder::Equal,
        (Some(_), Some(_)) => VersionOrder::OlderThanCurrent,
        _ => VersionOrder::Unparseable,
    }
}

fn replace_binary(exe: &std::path::Path, bytes: &[u8]) -> Result<(), String> {
    #[cfg(not(target_os = "windows"))]
    {
        use std::os::unix::fs::PermissionsExt as _;

        let tmp = exe.with_extension("tmp");
        std::fs::write(&tmp, bytes).map_err(|e| format!("write temp file: {e}"))?;

        // Preserve executable permission bits from the current binary.
        let perms = std::fs::metadata(exe)
            .map(|m| m.permissions())
            .unwrap_or_else(|_| std::fs::Permissions::from_mode(0o755));
        if let Err(e) = std::fs::set_permissions(&tmp, perms) {
            let _ = std::fs::remove_file(&tmp);
            return Err(format!("set temp file permissions: {e}"));
        }

        std::fs::rename(&tmp, exe).map_err(|e| format!("atomic rename: {e}"))?;
    }

    #[cfg(target_os = "windows")]
    {
        // On Windows the running exe cannot be written to, but CAN be renamed.
        // Rename it first, write the new binary in its place, then clean up.
        let old = exe.with_extension("exe.old");
        std::fs::rename(exe, &old).map_err(|e| format!("rename current binary: {e}"))?;
        if let Err(e) = std::fs::write(exe, bytes) {
            // Attempt to restore the original on failure.
            let _ = std::fs::rename(&old, exe);
            return Err(format!("write new binary: {e}"));
        }
        // The old exe may still be locked by the OS until the process exits; ignore the error.
        let _ = std::fs::remove_file(&old);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn current_target_is_supported() -> bool {
        matches!(
            (std::env::consts::OS, std::env::consts::ARCH),
            ("linux", "x86_64")
                | ("linux", "aarch64")
                | ("macos", "x86_64")
                | ("macos", "aarch64")
                | ("windows", "x86_64")
                | ("windows", "aarch64")
        )
    }

    // ── asset_name ────────────────────────────────────────────────────────────

    #[test]
    fn asset_name_returns_known_target() {
        let result = asset_name();
        if current_target_is_supported() {
            assert!(result.is_ok(), "asset_name() failed: {result:?}");
            assert!(
                result.unwrap().starts_with("cship-"),
                "unexpected asset name prefix"
            );
        } else {
            assert!(
                result.is_err(),
                "asset_name() should fail on unsupported target"
            );
        }
    }

    #[test]
    fn asset_name_exe_suffix_on_windows_only() {
        let result = asset_name();
        if current_target_is_supported() {
            let name = result.unwrap();
            if cfg!(target_os = "windows") {
                assert!(
                    name.ends_with(".exe"),
                    "Windows asset must end in .exe: {name}"
                );
            } else {
                assert!(
                    !name.ends_with(".exe"),
                    "Non-Windows asset must not end in .exe: {name}"
                );
            }
        } else {
            assert!(
                result.is_err(),
                "asset_name() should fail on unsupported target"
            );
        }
    }

    #[test]
    fn asset_name_contains_arch() {
        let result = asset_name();
        if current_target_is_supported() {
            let name = result.unwrap();
            let arch = std::env::consts::ARCH;
            assert!(
                name.contains(arch),
                "asset name '{name}' should contain arch '{arch}'"
            );
        } else {
            assert!(
                result.is_err(),
                "asset_name() should fail on unsupported target"
            );
        }
    }

    // ── compare_versions ─────────────────────────────────────────────────────

    #[test]
    fn compare_versions_newer_detected() {
        assert!(matches!(
            compare_versions("1.8.0", "1.7.0"),
            VersionOrder::NewerThanCurrent
        ));
        assert!(matches!(
            compare_versions("2.0.0", "1.99.99"),
            VersionOrder::NewerThanCurrent
        ));
        assert!(matches!(
            compare_versions("1.7.1", "1.7.0"),
            VersionOrder::NewerThanCurrent
        ));
    }

    #[test]
    fn compare_versions_equal_detected() {
        assert!(matches!(
            compare_versions("1.7.0", "1.7.0"),
            VersionOrder::Equal
        ));
    }

    #[test]
    fn compare_versions_older_detected() {
        assert!(matches!(
            compare_versions("1.6.0", "1.7.0"),
            VersionOrder::OlderThanCurrent
        ));
        assert!(matches!(
            compare_versions("1.7.0", "2.0.0"),
            VersionOrder::OlderThanCurrent
        ));
    }

    #[test]
    fn compare_versions_pre_release_suffix_ignored() {
        assert!(matches!(
            compare_versions("1.8.0-beta", "1.7.0"),
            VersionOrder::NewerThanCurrent
        ));
    }

    #[test]
    fn compare_versions_unparseable_returns_variant() {
        assert!(matches!(
            compare_versions("not-a-version", "1.7.0"),
            VersionOrder::Unparseable
        ));
        assert!(matches!(
            compare_versions("1.7.0", "not-a-version"),
            VersionOrder::Unparseable
        ));
    }

    // ── replace_binary ───────────────────────────────────────────────────────

    #[test]
    fn replace_binary_round_trips_bytes() {
        let dir = tempfile::tempdir().unwrap();
        let bin = dir.path().join(if cfg!(target_os = "windows") {
            "cship.exe"
        } else {
            "cship"
        });
        std::fs::write(&bin, b"old content").unwrap();

        #[cfg(not(target_os = "windows"))]
        {
            use std::os::unix::fs::PermissionsExt as _;
            std::fs::set_permissions(&bin, std::fs::Permissions::from_mode(0o755)).unwrap();
        }

        replace_binary(&bin, b"new content").expect("replace_binary should succeed");

        assert_eq!(std::fs::read(&bin).unwrap(), b"new content");

        #[cfg(not(target_os = "windows"))]
        {
            use std::os::unix::fs::PermissionsExt as _;
            let mode = std::fs::metadata(&bin).unwrap().permissions().mode();
            assert!(
                mode & 0o111 != 0,
                "executable bit should be set after replace"
            );
        }
    }

    #[test]
    fn replace_binary_fails_when_exe_does_not_exist() {
        // Verifies that replace_binary returns Err (rather than panicking) when
        // the target path's parent directory does not exist — the rename/write
        // step fails before any restoration logic is reached.
        let dir = tempfile::tempdir().unwrap();
        let bad_path = dir.path().join("no_such_dir").join("cship.exe");
        let result = replace_binary(&bad_path, b"new");
        assert!(result.is_err(), "should fail when target path is invalid");
    }
}
