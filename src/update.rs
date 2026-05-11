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

    if latest_version == current_version {
        println!("Already up to date (v{current_version}).");
        return;
    }

    println!("New version available: v{latest_version} (current: v{current_version})");
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
        let _ = std::fs::set_permissions(&tmp, perms);

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

    #[test]
    fn asset_name_returns_known_target() {
        // Just verify the function doesn't error on the current platform.
        let result = asset_name();
        assert!(result.is_ok(), "asset_name() failed: {result:?}");
        let name = result.unwrap();
        assert!(name.starts_with("cship-"), "unexpected asset name: {name}");
    }
}
