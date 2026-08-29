use crate::types::ReleaseInfo;
use eyre::{Context, Result, eyre};
use semver::Version;
use serde::Deserialize;
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

const GITHUB_REPO_OWNER: &str = "ur-wesley";
const GITHUB_REPO_NAME: &str = "fast-md";
const USER_AGENT: &str = "fast-md-updater";

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct GitHubAsset {
    name: String,
    browser_download_url: String,
    #[serde(default)]
    size: u64,
}

#[derive(Debug, Deserialize)]
struct GitHubReleaseResponse {
    tag_name: String,
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    body: Option<String>,
    #[serde(default)]
    html_url: Option<String>,
    #[serde(default)]
    published_at: Option<String>,
    #[serde(default)]
    assets: Vec<GitHubAsset>,
    #[serde(default)]
    draft: bool,
    #[serde(default)]
    prerelease: bool,
}

/// Retrieve the expected asset platform matchers for the current operating system and architecture.
const fn get_platform_asset_keywords() -> (&'static [&'static str], &'static str) {
    #[cfg(all(target_os = "windows", target_arch = "x86_64"))]
    {
        (&["windows", "x86_64"], "fast-md.exe")
    }

    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    {
        (&["macos", "arm64"], "fast-md")
    }

    #[cfg(all(target_os = "macos", target_arch = "x86_64"))]
    {
        (&["macos", "x86_64"], "fast-md")
    }

    #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
    {
        (&["linux", "x86_64"], "fast-md")
    }

    #[cfg(not(any(
        all(target_os = "windows", target_arch = "x86_64"),
        all(target_os = "macos", target_arch = "aarch64"),
        all(target_os = "macos", target_arch = "x86_64"),
        all(target_os = "linux", target_arch = "x86_64"),
    )))]
    {
        (&["unknown"], "fast-md")
    }
}

/// Check if a newer version of Fast-MD is available on GitHub Releases.
/// Returns `Ok(Some(ReleaseInfo))` if an update is found, or `Ok(None)` if already up-to-date.
///
/// # Errors
/// Returns an error if the network request fails, parsing fails, or rate limit is reached.
pub fn check_github_release() -> Result<Option<ReleaseInfo>> {
    let api_url = format!("https://api.github.com/repos/{GITHUB_REPO_OWNER}/{GITHUB_REPO_NAME}/releases/latest");

    let response = ureq::get(&api_url)
        .set("User-Agent", USER_AGENT)
        .set("Accept", "application/vnd.github.v3+json")
        .timeout(std::time::Duration::from_secs(10))
        .call()
        .wrap_err("Failed to connect to GitHub Releases API")?;

    let release: GitHubReleaseResponse = response
        .into_json()
        .wrap_err("Failed to parse GitHub Releases JSON payload")?;

    if release.draft || release.prerelease {
        return Ok(None);
    }

    let clean_remote_tag = release.tag_name.trim().trim_start_matches('v');
    let remote_version = Version::parse(clean_remote_tag)
        .wrap_err_with(|| format!("Invalid semantic version in remote tag: {}", release.tag_name))?;

    let current_version = Version::parse(env!("CARGO_PKG_VERSION"))
        .wrap_err("Failed to parse current CARGO_PKG_VERSION")?;

    if remote_version <= current_version {
        return Ok(None);
    }

    let (keywords, _bin_name) = get_platform_asset_keywords();

    // Find the matching release asset for the current OS/architecture
    let matching_asset = release.assets.iter().find(|asset| {
        let name_lower = asset.name.to_lowercase();
        keywords.iter().all(|&kw| name_lower.contains(kw))
    });

    let Some(asset) = matching_asset else {
        return Ok(None);
    };

    Ok(Some(ReleaseInfo {
        version: format!("{remote_version}"),
        tag_name: release.tag_name,
        name: release.name.unwrap_or_else(|| format!("Fast-MD v{remote_version}")),
        release_notes: release.body.unwrap_or_default(),
        asset_name: asset.name.clone(),
        download_url: asset.browser_download_url.clone(),
        published_at: release.published_at.unwrap_or_default(),
        html_url: release.html_url.unwrap_or_else(|| {
            format!("https://github.com/{GITHUB_REPO_OWNER}/{GITHUB_REPO_NAME}/releases")
        }),
    }))
}

/// Download the release archive, extract the updated binary, and replace the currently running executable in-place.
///
/// # Errors
/// Returns an error if download, extraction, or binary replacement fails.
pub fn download_and_apply_update<F>(release: &ReleaseInfo, progress_callback: F) -> Result<()>
where
    F: Fn(u8) + Send + Sync,
{
    let temp_dir = tempfile::Builder::new()
        .prefix("fastmd-update-")
        .tempdir()
        .wrap_err("Failed to create temporary directory for update")?;

    let downloaded_archive_path = temp_dir.path().join(&release.asset_name);

    // Download archive with progress
    let response = ureq::get(&release.download_url)
        .set("User-Agent", USER_AGENT)
        .timeout(std::time::Duration::from_mins(2))
        .call()
        .wrap_err_with(|| format!("Failed to download update from {}", release.download_url))?;

    let total_size = response
        .header("Content-Length")
        .and_then(|val| val.parse::<u64>().ok());

    let mut reader = response.into_reader();
    let mut file = File::create(&downloaded_archive_path)
        .wrap_err("Failed to create temporary file for update archive")?;

    let mut buffer = [0u8; 8 * 1024];
    let mut downloaded_bytes: u64 = 0;
    let mut last_reported_pct: u8 = 0;

    loop {
        let bytes_read = reader
            .read(&mut buffer)
            .wrap_err("Failed while reading stream from GitHub")?;
        if bytes_read == 0 {
            break;
        }

        file.write_all(&buffer[..bytes_read])
            .wrap_err("Failed while writing update archive to disk")?;

        downloaded_bytes += bytes_read as u64;

        if let Some(total) = total_size {
            if let Some(div) = (downloaded_bytes * 100).checked_div(total) {
                let pct = u8::try_from(div.min(100)).unwrap_or(100);
                if pct != last_reported_pct {
                    last_reported_pct = pct;
                    progress_callback(pct);
                }
            }
        }
    }

    drop(file);
    progress_callback(100);

    let (_keywords, expected_bin_name) = get_platform_asset_keywords();
    let extracted_bin_path = extract_binary_from_archive(
        &downloaded_archive_path,
        temp_dir.path(),
        expected_bin_name,
    )?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = std::fs::Permissions::from_mode(0o755);
        let _ = std::fs::set_permissions(&extracted_bin_path, perms);
    }

    // Atomic cross-platform executable replacement
    self_replace::self_replace(&extracted_bin_path)
        .wrap_err("Failed to replace running binary with updated version")?;

    Ok(())
}

/// Extract the target executable from `.zip` or `.tar.gz` archive.
fn extract_binary_from_archive(
    archive_path: &Path,
    dest_dir: &Path,
    expected_bin_name: &str,
) -> Result<PathBuf> {
    let filename_lower = archive_path
        .file_name()
        .and_then(|f| f.to_str())
        .unwrap_or_default()
        .to_lowercase();

    let is_zip = archive_path
        .extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("zip"));

    let is_tgz = archive_path
        .extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("tgz"));

    if is_zip {
        extract_from_zip(archive_path, dest_dir, expected_bin_name)
    } else if filename_lower.ends_with(".tar.gz") || is_tgz {
        extract_from_tar_gz(archive_path, dest_dir, expected_bin_name)
    } else {
        // Assume direct binary if not recognized as archive
        Ok(archive_path.to_path_buf())
    }
}

fn extract_from_zip(zip_path: &Path, dest_dir: &Path, expected_bin_name: &str) -> Result<PathBuf> {
    let file = File::open(zip_path).wrap_err("Failed to open zip archive")?;
    let mut archive = zip::ZipArchive::new(file).wrap_err("Failed to parse zip archive")?;

    for i in 0..archive.len() {
        let mut file = archive
            .by_index(i)
            .wrap_err("Failed to read zip archive entry")?;

        let entry_name = file.name().to_string();
        let path = Path::new(&entry_name);

        let matches_bin = path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.eq_ignore_ascii_case(expected_bin_name));

        if matches_bin {
            let outpath = dest_dir.join(expected_bin_name);
            let mut outfile = File::create(&outpath).wrap_err("Failed to create extracted binary file")?;
            std::io::copy(&mut file, &mut outfile)
                .wrap_err("Failed to write extracted binary content")?;
            return Ok(outpath);
        }
    }

    Err(eyre!(
        "Could not find executable '{expected_bin_name}' inside zip archive"
    ))
}

fn extract_from_tar_gz(tar_path: &Path, dest_dir: &Path, expected_bin_name: &str) -> Result<PathBuf> {
    let file = File::open(tar_path).wrap_err("Failed to open tar.gz archive")?;
    let gz = flate2::read::GzDecoder::new(file);
    let mut archive = tar::Archive::new(gz);

    let entries = archive.entries().wrap_err("Failed to read tar archive entries")?;

    for entry in entries {
        let mut entry = entry.wrap_err("Failed to read tar entry")?;
        let entry_path = entry.path().wrap_err("Invalid path in tar archive")?.into_owned();

        let matches_bin = entry_path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.eq_ignore_ascii_case(expected_bin_name));

        if matches_bin {
            let outpath = dest_dir.join(expected_bin_name);
            let mut outfile = File::create(&outpath).wrap_err("Failed to create extracted binary file")?;
            std::io::copy(&mut entry, &mut outfile)
                .wrap_err("Failed to write extracted binary content")?;
            return Ok(outpath);
        }
    }

    Err(eyre!(
        "Could not find executable '{expected_bin_name}' inside tar.gz archive"
    ))
}

/// Restart the current application to apply the update.
///
/// # Errors
/// Returns an error if spawning the executable fails.
pub fn restart_app() -> Result<()> {
    let current_exe = std::env::current_exe().wrap_err("Failed to get current executable path")?;

    Command::new(current_exe)
        .spawn()
        .wrap_err("Failed to spawn updated application process")?;

    std::process::exit(0);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_platform_asset_keywords() {
        let (keywords, bin_name) = get_platform_asset_keywords();
        assert!(!keywords.is_empty());
        assert!(!bin_name.is_empty());
    }

    #[test]
    fn test_semver_comparison() {
        let current = Version::parse(env!("CARGO_PKG_VERSION")).unwrap_or_else(|_| Version::new(0, 1, 0));
        let older = Version::parse("0.1.0").unwrap_or_else(|_| Version::new(0, 1, 0));
        let newer = Version::parse("99.0.0").unwrap_or_else(|_| Version::new(99, 0, 0));

        assert!(newer > current);
        assert!(current >= older);
    }
}
