use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

use serde_json::Value;

pub const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");
const RELEASES_LATEST_API: &str =
    "https://api.github.com/repos/williamjkc69/gity-x/releases/latest";

#[derive(Debug, Clone)]
pub struct ReleaseUpdate {
    pub version: String,
    pub html_url: String,
    pub asset_name: String,
    pub download_url: String,
    pub checksum_url: Option<String>,
    pub can_self_install: bool,
}

#[derive(Debug, Clone)]
pub enum UpdateState {
    Idle,
    Checking,
    UpToDate,
    Available(ReleaseUpdate),
    Downloading {
        version: String,
        progress: f32,
    },
    ReadyToRestart {
        version: String,
        launch_path: PathBuf,
    },
    Error(String),
}

/// Removes leftover `.old` / `.bak` binaries from a prior self-update.
pub fn cleanup_stale_backups() {
    #[cfg(target_os = "macos")]
    {
        if let Some(app_path) = current_macos_app_bundle() {
            let bak = backup_path_for(&app_path);
            if bak.exists() {
                let _ = fs::remove_dir_all(&bak);
            }
        }
    }
    #[cfg(target_os = "windows")]
    {
        if let Ok(exe) = std::env::current_exe() {
            let old = backup_path_for(&exe);
            if old.exists() {
                let _ = fs::remove_file(&old);
            }
        }
    }
}

/// True when the process is running from an installed bundle/binary that supports in-place swap.
pub fn supports_in_place_install() -> bool {
    #[cfg(target_os = "macos")]
    {
        current_macos_app_bundle().is_some()
    }
    #[cfg(target_os = "windows")]
    {
        !cfg!(debug_assertions)
    }
    #[cfg(target_os = "linux")]
    {
        std::env::var_os("APPIMAGE").is_some() || !cfg!(debug_assertions)
    }
    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
    {
        false
    }
}

/// Queries GitHub Releases latest API and returns `Some(ReleaseUpdate)` if a newer version exists.
pub fn check_latest_release() -> Result<Option<ReleaseUpdate>, String> {
    let resp = ureq::get(RELEASES_LATEST_API)
        .timeout(Duration::from_secs(12))
        .set("Accept", "application/vnd.github+json")
        .set("X-GitHub-Api-Version", "2022-11-28")
        .set("User-Agent", "gity-x")
        .call()
        .map_err(|e| format!("Update check failed: {e}"))?;

    let json: Value = resp
        .into_json()
        .map_err(|e| format!("Invalid release JSON: {e}"))?;

    let tag = json
        .get("tag_name")
        .and_then(Value::as_str)
        .unwrap_or("")
        .trim()
        .to_string();
    if tag.is_empty() || !is_newer_version(&tag, CURRENT_VERSION) {
        return Ok(None);
    }

    let version = tag.trim_start_matches('v').to_string();
    let html_url = json
        .get("html_url")
        .and_then(Value::as_str)
        .unwrap_or("https://github.com/williamjkc69/gity-x/releases/latest")
        .to_string();

    let assets = json
        .get("assets")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();

    let checksum_url = assets.iter().find_map(|a| {
        let name = a.get("name").and_then(Value::as_str)?;
        if name == "SHA256SUMS.txt" {
            a.get("browser_download_url")
                .and_then(Value::as_str)
                .map(str::to_string)
        } else {
            None
        }
    });

    let preferred_suffix = preferred_asset_suffix();
    let fallback_suffix = fallback_asset_suffix();

    let picked = find_asset_by_suffix(&assets, preferred_suffix)
        .map(|(n, u)| (n, u, true))
        .or_else(|| {
            fallback_suffix.and_then(|suf| {
                find_asset_by_suffix(&assets, suf).map(|(n, u)| (n, u, false))
            })
        });

    let (asset_name, download_url, asset_installable) = match picked {
        Some(found) => found,
        None => (String::new(), html_url.clone(), false),
    };

    Ok(Some(ReleaseUpdate {
        version,
        html_url,
        asset_name,
        download_url,
        checksum_url,
        can_self_install: asset_installable && supports_in_place_install(),
    }))
}

fn find_asset_by_suffix(assets: &[Value], suffix: &str) -> Option<(String, String)> {
    assets.iter().find_map(|a| {
        let name = a.get("name").and_then(Value::as_str)?;
        if name.ends_with(suffix) {
            let url = a.get("browser_download_url").and_then(Value::as_str)?;
            Some((name.to_string(), url.to_string()))
        } else {
            None
        }
    })
}

fn preferred_asset_suffix() -> &'static str {
    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    {
        "-macos-arm64.tar.gz"
    }
    #[cfg(all(target_os = "macos", target_arch = "x86_64"))]
    {
        "-macos-x64.tar.gz"
    }
    #[cfg(target_os = "windows")]
    {
        "-windows-x64.exe"
    }
    #[cfg(target_os = "linux")]
    {
        "-linux-x86_64.AppImage"
    }
    #[cfg(not(any(
        all(target_os = "macos", target_arch = "aarch64"),
        all(target_os = "macos", target_arch = "x86_64"),
        target_os = "windows",
        target_os = "linux"
    )))]
    {
        ""
    }
}

fn fallback_asset_suffix() -> Option<&'static str> {
    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    {
        Some("-macos-arm64.dmg")
    }
    #[cfg(all(target_os = "macos", target_arch = "x86_64"))]
    {
        Some("-macos-x64.dmg")
    }
    #[cfg(not(target_os = "macos"))]
    {
        None
    }
}

/// Downloads the release asset, verifies SHA-256 against `SHA256SUMS.txt` when available,
/// and atomically swaps the installed app bundle or executable.
pub fn download_and_install<F>(update: &ReleaseUpdate, mut on_progress: F) -> Result<PathBuf, String>
where
    F: FnMut(f32),
{
    if update.asset_name.is_empty() || update.download_url.is_empty() {
        return Err("No downloadable asset found for this platform.".to_string());
    }

    let stage_dir = std::env::temp_dir().join(format!("gity-x-update-{}", update.version));
    let _ = fs::remove_dir_all(&stage_dir);
    fs::create_dir_all(&stage_dir)
        .map_err(|e| format!("Failed to create staging directory: {e}"))?;

    let downloaded_file = stage_dir.join(&update.asset_name);
    download_file_with_progress(&update.download_url, &downloaded_file, &mut on_progress)?;

    // Verify SHA-256 integrity if SHA256SUMS.txt was published with this release
    if let Some(sum_url) = &update.checksum_url {
        let sums_text = ureq::get(sum_url)
            .timeout(Duration::from_secs(15))
            .set("User-Agent", "gity-x")
            .call()
            .map_err(|e| format!("Failed to fetch SHA256SUMS.txt: {e}"))?
            .into_string()
            .map_err(|e| format!("Failed to read SHA256SUMS.txt: {e}"))?;

        if let Some(expected_hash) = parse_sha256_for_asset(&sums_text, &update.asset_name) {
            let actual_hash = sha256_file_hex(&downloaded_file)?;
            if !actual_hash.eq_ignore_ascii_case(&expected_hash) {
                let _ = fs::remove_dir_all(&stage_dir);
                return Err(format!(
                    "SHA-256 verification failed for {}.\nExpected: {expected_hash}\nActual:   {actual_hash}",
                    update.asset_name
                ));
            }
        }
    }

    let installed_path = install_staged_artifact(&downloaded_file, &stage_dir)?;
    let _ = fs::remove_dir_all(&stage_dir);
    Ok(installed_path)
}

fn download_file_with_progress<F>(
    url: &str,
    dest: &Path,
    on_progress: &mut F,
) -> Result<(), String>
where
    F: FnMut(f32),
{
    let resp = ureq::get(url)
        .timeout(Duration::from_secs(180))
        .set("User-Agent", "gity-x")
        .call()
        .map_err(|e| format!("Download failed: {e}"))?;

    let total_bytes: Option<u64> = resp
        .header("Content-Length")
        .and_then(|h| h.parse::<u64>().ok())
        .filter(|&n| n > 0);

    let mut reader = resp.into_reader();
    let mut out = File::create(dest).map_err(|e| format!("Failed to create temp file: {e}"))?;
    let mut buf = [0u8; 64 * 1024];
    let mut downloaded = 0u64;
    let mut last_reported = -1.0_f32;

    loop {
        let n = reader
            .read(&mut buf)
            .map_err(|e| format!("Download read error: {e}"))?;
        if n == 0 {
            break;
        }
        out.write_all(&buf[..n])
            .map_err(|e| format!("Download write error: {e}"))?;
        downloaded += n as u64;

        if let Some(total) = total_bytes {
            let pct = (downloaded as f32 / total as f32).clamp(0.0, 0.99);
            if pct - last_reported >= 0.02 {
                last_reported = pct;
                on_progress(pct);
            }
        }
    }
    out.flush()
        .map_err(|e| format!("Failed to flush download: {e}"))?;
    on_progress(1.0);
    Ok(())
}

#[cfg(target_os = "macos")]
fn current_macos_app_bundle() -> Option<PathBuf> {
    let exe = std::env::current_exe().ok()?;
    for ancestor in exe.ancestors() {
        if ancestor.extension().and_then(|e| e.to_str()) == Some("app") {
            return Some(ancestor.to_path_buf());
        }
    }
    None
}

fn backup_path_for(path: &Path) -> PathBuf {
    let mut name = path.as_os_str().to_os_string();
    name.push(".old");
    PathBuf::from(name)
}

#[cfg(target_os = "macos")]
fn install_staged_artifact(archive: &Path, stage_dir: &Path) -> Result<PathBuf, String> {
    let Some(current_app) = current_macos_app_bundle() else {
        return Err(
            "Auto-install requires running from an installed 'gity x.app' bundle.".to_string(),
        );
    };

    let extract_dir = stage_dir.join("extracted");
    fs::create_dir_all(&extract_dir)
        .map_err(|e| format!("Failed to create extraction directory: {e}"))?;

    let status = Command::new("/usr/bin/tar")
        .arg("-xzf")
        .arg(archive)
        .arg("-C")
        .arg(&extract_dir)
        .status()
        .map_err(|e| format!("Failed to run tar: {e}"))?;
    if !status.success() {
        return Err("Failed to extract macOS update archive.".to_string());
    }

    let extracted_app = find_app_bundle_in(&extract_dir)
        .ok_or_else(|| "Update archive did not contain a .app bundle.".to_string())?;

    // Ensure no quarantine attribute is present on the extracted bundle
    let _ = Command::new("/usr/bin/xattr")
        .args(["-dr", "com.apple.quarantine"])
        .arg(&extracted_app)
        .status();

    let parent = current_app
        .parent()
        .ok_or_else(|| "Could not determine application parent directory.".to_string())?;
    let staged_sibling = parent.join(".gity-x-new.app");
    let backup_app = backup_path_for(&current_app);

    let _ = fs::remove_dir_all(&staged_sibling);
    let _ = fs::remove_dir_all(&backup_app);

    // Copy into same filesystem as current_app so rename() is atomic
    let cp_status = Command::new("/usr/bin/ditto")
        .arg(&extracted_app)
        .arg(&staged_sibling)
        .status()
        .map_err(|e| format!("Failed to stage new app bundle: {e}"))?;
    if !cp_status.success() {
        return Err(format!(
            "Cannot write to '{}'. Move gity x.app to a writable Applications folder or update manually.",
            parent.display()
        ));
    }

    if let Err(e) = fs::rename(&current_app, &backup_app) {
        let _ = fs::remove_dir_all(&staged_sibling);
        return Err(format!("Failed to replace existing app bundle: {e}"));
    }

    if let Err(e) = fs::rename(&staged_sibling, &current_app) {
        let _ = fs::rename(&backup_app, &current_app);
        return Err(format!("Failed to finalize new app bundle: {e}"));
    }

    let _ = fs::remove_dir_all(&backup_app);
    Ok(current_app)
}

#[cfg(target_os = "macos")]
fn find_app_bundle_in(dir: &Path) -> Option<PathBuf> {
    let entries = fs::read_dir(dir).ok()?;
    for entry in entries.flatten() {
        let p = entry.path();
        if p.is_dir() && p.extension().and_then(|e| e.to_str()) == Some("app") {
            return Some(p);
        }
    }
    None
}

#[cfg(target_os = "windows")]
fn install_staged_artifact(downloaded_exe: &Path, _stage_dir: &Path) -> Result<PathBuf, String> {
    let current_exe =
        std::env::current_exe().map_err(|e| format!("Cannot resolve current executable: {e}"))?;
    let parent = current_exe
        .parent()
        .ok_or_else(|| "Cannot resolve executable directory.".to_string())?;
    let staged_sibling = parent.join("gity-x.new.exe");
    let backup_exe = backup_path_for(&current_exe);

    let _ = fs::remove_file(&staged_sibling);
    let _ = fs::remove_file(&backup_exe);

    fs::copy(downloaded_exe, &staged_sibling)
        .map_err(|e| format!("Failed to stage new executable: {e}"))?;

    if let Err(e) = fs::rename(&current_exe, &backup_exe) {
        let _ = fs::remove_file(&staged_sibling);
        return Err(format!("Failed to rename running executable: {e}"));
    }

    if let Err(e) = fs::rename(&staged_sibling, &current_exe) {
        let _ = fs::rename(&backup_exe, &current_exe);
        return Err(format!("Failed to activate new executable: {e}"));
    }

    Ok(current_exe)
}

#[cfg(target_os = "linux")]
fn install_staged_artifact(downloaded_bin: &Path, _stage_dir: &Path) -> Result<PathBuf, String> {
    use std::os::unix::fs::PermissionsExt;

    let target = std::env::var_os("APPIMAGE")
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::current_exe().unwrap_or_default());
    if target.as_os_str().is_empty() {
        return Err("Cannot resolve current executable path.".to_string());
    }
    let parent = target
        .parent()
        .ok_or_else(|| "Cannot resolve executable directory.".to_string())?;
    let staged_sibling = parent.join(".gity-x.new");

    fs::copy(downloaded_bin, &staged_sibling)
        .map_err(|e| format!("Cannot write to '{}': {e}", parent.display()))?;
    fs::set_permissions(&staged_sibling, fs::Permissions::from_mode(0o755))
        .map_err(|e| format!("Failed to mark binary executable: {e}"))?;
    fs::rename(&staged_sibling, &target)
        .map_err(|e| format!("Failed to replace executable: {e}"))?;

    Ok(target)
}

#[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
fn install_staged_artifact(_downloaded: &Path, _stage_dir: &Path) -> Result<PathBuf, String> {
    Err("Self-update is not supported on this operating system.".to_string())
}

/// Launches the updated application and exits the current process cleanly.
pub fn relaunch_and_exit(launch_path: &Path) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        Command::new("/usr/bin/open")
            .arg("-n")
            .arg(launch_path)
            .spawn()
            .map_err(|e| format!("Failed to relaunch app: {e}"))?;
    }
    #[cfg(not(target_os = "macos"))]
    {
        Command::new(launch_path)
            .spawn()
            .map_err(|e| format!("Failed to relaunch app: {e}"))?;
    }
    std::process::exit(0);
}

pub fn is_newer_version(remote: &str, current: &str) -> bool {
    parse_semver(remote) > parse_semver(current)
}

fn parse_semver(raw: &str) -> (u64, u64, u64) {
    let trimmed = raw
        .trim()
        .trim_start_matches('v')
        .split('-')
        .next()
        .unwrap_or("");
    let mut parts = trimmed.split('.');
    let major = parts.next().and_then(|s| s.parse::<u64>().ok()).unwrap_or(0);
    let minor = parts.next().and_then(|s| s.parse::<u64>().ok()).unwrap_or(0);
    let patch = parts.next().and_then(|s| s.parse::<u64>().ok()).unwrap_or(0);
    (major, minor, patch)
}

fn parse_sha256_for_asset(sums_text: &str, asset_name: &str) -> Option<String> {
    for line in sums_text.lines() {
        let mut parts = line.split_whitespace();
        let hash = parts.next()?;
        let name = parts.next()?.trim_start_matches('*');
        if name == asset_name && hash.len() == 64 {
            return Some(hash.to_string());
        }
    }
    None
}

/// Pure-Rust FIPS 180-4 SHA-256 (zero external crate bloat).
fn sha256_file_hex(path: &Path) -> Result<String, String> {
    let mut file = File::open(path).map_err(|e| format!("Failed to open file for SHA-256: {e}"))?;
    let mut hasher = Sha256::new();
    let mut buf = [0u8; 64 * 1024];
    loop {
        let n = file
            .read(&mut buf)
            .map_err(|e| format!("SHA-256 read error: {e}"))?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    Ok(hasher.finalize_hex())
}

struct Sha256 {
    state: [u32; 8],
    buf: [u8; 64],
    buf_len: usize,
    total_bits: u64,
}

impl Sha256 {
    const K: [u32; 64] = [
        0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4,
        0xab1c5ed5, 0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe,
        0x9bdc06a7, 0xc19bf174, 0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f,
        0x4a7484aa, 0x5cb0a9dc, 0x76f988da, 0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7,
        0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967, 0x27b70a85, 0x2e1b2138, 0x4d2c6dfc,
        0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85, 0xa2bfe8a1, 0xa81a664b,
        0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070, 0x19a4c116,
        0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
        0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7,
        0xc67178f2,
    ];

    fn new() -> Self {
        Self {
            state: [
                0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c,
                0x1f83d9ab, 0x5be0cd19,
            ],
            buf: [0u8; 64],
            buf_len: 0,
            total_bits: 0,
        }
    }

    fn update(&mut self, data: &[u8]) {
        self.total_bits = self.total_bits.wrapping_add((data.len() as u64) * 8);
        let mut i = 0;
        if self.buf_len > 0 {
            let need = 64 - self.buf_len;
            let take = need.min(data.len());
            self.buf[self.buf_len..self.buf_len + take].copy_from_slice(&data[..take]);
            self.buf_len += take;
            i += take;
            if self.buf_len == 64 {
                let block = self.buf;
                self.compress(&block);
                self.buf_len = 0;
            }
        }
        while i + 64 <= data.len() {
            let mut block = [0u8; 64];
            block.copy_from_slice(&data[i..i + 64]);
            self.compress(&block);
            i += 64;
        }
        if i < data.len() {
            let rem = data.len() - i;
            self.buf[..rem].copy_from_slice(&data[i..]);
            self.buf_len = rem;
        }
    }

    fn compress(&mut self, block: &[u8; 64]) {
        let mut w = [0u32; 64];
        for i in 0..16 {
            w[i] = u32::from_be_bytes([
                block[i * 4],
                block[i * 4 + 1],
                block[i * 4 + 2],
                block[i * 4 + 3],
            ]);
        }
        for i in 16..64 {
            let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
            let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
            w[i] = w[i - 16]
                .wrapping_add(s0)
                .wrapping_add(w[i - 7])
                .wrapping_add(s1);
        }

        let [mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut h] = self.state;
        for i in 0..64 {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let ch = (e & f) ^ ((!e) & g);
            let t1 = h
                .wrapping_add(s1)
                .wrapping_add(ch)
                .wrapping_add(Self::K[i])
                .wrapping_add(w[i]);
            let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let maj = (a & b) ^ (a & c) ^ (b & c);
            let t2 = s0.wrapping_add(maj);

            h = g;
            g = f;
            f = e;
            e = d.wrapping_add(t1);
            d = c;
            c = b;
            b = a;
            a = t1.wrapping_add(t2);
        }

        self.state[0] = self.state[0].wrapping_add(a);
        self.state[1] = self.state[1].wrapping_add(b);
        self.state[2] = self.state[2].wrapping_add(c);
        self.state[3] = self.state[3].wrapping_add(d);
        self.state[4] = self.state[4].wrapping_add(e);
        self.state[5] = self.state[5].wrapping_add(f);
        self.state[6] = self.state[6].wrapping_add(g);
        self.state[7] = self.state[7].wrapping_add(h);
    }

    fn finalize_hex(mut self) -> String {
        self.buf[self.buf_len] = 0x80;
        self.buf_len += 1;
        if self.buf_len > 56 {
            for b in &mut self.buf[self.buf_len..64] {
                *b = 0;
            }
            let block = self.buf;
            self.compress(&block);
            self.buf_len = 0;
        }
        for b in &mut self.buf[self.buf_len..56] {
            *b = 0;
        }
        self.buf[56..64].copy_from_slice(&self.total_bits.to_be_bytes());
        let block = self.buf;
        self.compress(&block);

        let mut hex = String::with_capacity(64);
        for word in self.state {
            hex.push_str(&format!("{word:08x}"));
        }
        hex
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compares_semver_correctly() {
        assert!(is_newer_version("v0.2.3", "0.2.2"));
        assert!(is_newer_version("v0.3.0", "0.2.9"));
        assert!(is_newer_version("v1.0.0", "0.9.9"));
        assert!(!is_newer_version("v0.2.2", "0.2.2"));
        assert!(!is_newer_version("v0.2.1", "0.2.2"));
    }

    #[test]
    fn parses_sha256sums_correctly() {
        let sample = "\
ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad  gity-x-v0.2.3-macos-arm64.tar.gz\n\
aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa  gity-x-v0.2.3-windows-x64.exe\n";
        assert_eq!(
            parse_sha256_for_asset(sample, "gity-x-v0.2.3-macos-arm64.tar.gz").as_deref(),
            Some("ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad")
        );
        assert_eq!(parse_sha256_for_asset(sample, "missing.tar.gz"), None);
    }

    #[test]
    fn sha256_matches_nist_vectors() {
        let mut h = Sha256::new();
        h.update(b"abc");
        assert_eq!(
            h.finalize_hex(),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );

        let mut h2 = Sha256::new();
        h2.update(b"");
        assert_eq!(
            h2.finalize_hex(),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }
}
