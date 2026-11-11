//! The fork's Windows portable releases use GitHub HTTPS plus mandatory release SHA-256.
//! This policy is restricted to our pinned repository and portable layout; MSI and other OS
//! installations retain the shared updater's publisher-signature checks.
use super::{Install, JobPhase, Platform, Release};
use crate::{
    http::{Http, Stop},
    paths::Paths,
    portable, process,
};
use artcraft_launcher_core::{PackageKind, catalog::LAUNCHER, platform::pick_asset, protocol::InstallLocations, release};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    ffi::OsString,
    io::Read,
    path::{Component, Path, PathBuf},
    time::Duration,
};

const FLAG: &str = "--craft-fork-finish-update";
const EXE: &str = "artcraft-launcher.exe";
const HELPER: &str = "fork-update-helper.exe";
const PLAN: &str = "plan.json";

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Plan {
    exe: PathBuf,
    pid: u32,
    sha256: String,
    helper_sha256: String,
    tag: String,
    version: String,
}

/// Only a genuine portable installation of this fork takes the unsigned-package policy.
pub(super) fn applies(install: &Install) -> bool {
    LAUNCHER.repo == "lakam99/craft-launcher" && matches!(install, Install::Portable(_))
}

fn plain(path: &Path, directory: bool) -> Result<(), String> {
    use std::os::windows::fs::MetadataExt;
    let meta = std::fs::symlink_metadata(path).map_err(|e| e.to_string())?;
    if meta.file_attributes() & 0x400 != 0 || (directory && !meta.is_dir()) || (!directory && !meta.is_file()) {
        return Err("The portable update contains a link or an unexpected file type.".into());
    }
    Ok(())
}

fn paths_for(exe: &Path) -> Result<Paths, String> {
    plain(exe, false)?;
    if exe.file_name().and_then(|n| n.to_str()) != Some(EXE) {
        return Err("This helper only updates Craft Fork Launcher.".into());
    }
    let folder = exe.parent().ok_or("no portable installation folder")?;
    plain(folder, true)?;
    plain(&folder.join("portable.txt"), false)?;
    Ok(Paths::under(&folder.join("CraftForkLauncherData")))
}

fn check_release(release: &Release, platform: Platform) -> Result<&artcraft_launcher_core::release::Asset, String> {
    let (asset, _) =
        pick_asset(release, &[LAUNCHER.id], platform, &[PackageKind::PortableZip]).ok_or("No matching portable fork update is available.")?;
    let prefix = release::download_prefix(LAUNCHER.repo);
    if !release::download_url_under(&asset.url, &prefix)
        || release.sums_asset().is_some_and(|a| !release::download_url_under(&a.url, &prefix))
        || release.version.is_none()
    {
        return Err("The update must come from this fork's GitHub releases.".into());
    }
    crate::install::bind_artifact_version(&LAUNCHER, release, asset)?;
    Ok(asset)
}

/// Validate all files before exit and again in the helper. User data is never an update payload.
fn inspect(zip: &Path, directory: &Path, version: &str) -> Result<(PathBuf, BTreeMap<PathBuf, String>), String> {
    let (root, files) = portable::prepare_update(zip, directory, &LAUNCHER)?;
    if root != directory {
        return Err("Fork updates must use the flat portable ZIP layout.".into());
    }
    let mut hashes = BTreeMap::new();
    for file in files {
        if file.components().any(|c| match c {
            Component::Normal(n) => {
                // Win32 trailing-dot/space and DOS short-name aliases must not bypass the data exclusion.
                n.to_str().is_none_or(|n| {
                    n.ends_with(['.', ' '])
                        || n.contains('~')
                        || n.eq_ignore_ascii_case("CraftForkLauncherData")
                        || n.eq_ignore_ascii_case("ArtCraftLauncherData")
                })
            }
            _ => true,
        }) {
            return Err("An update must not contain unsafe file names or launcher user data.".into());
        }
        let source = root.join(&file);
        plain(&source, false)?;
        hashes.insert(file, crate::http::sha256_file(&source)?);
    }
    if !hashes.contains_key(Path::new(EXE)) || !hashes.contains_key(Path::new("portable.txt")) {
        return Err("The update must contain the launcher and portable.txt at its root.".into());
    }
    validate_program(&root.join(EXE), version)?;
    Ok((root, hashes))
}

fn validate_program(exe: &Path, version: &str) -> Result<(), String> {
    if std::fs::metadata(exe).map_err(|e| e.to_string())?.len() > 256 * 1024 * 1024 {
        return Err("The update executable is too large.".into());
    }
    let version = artcraft_launcher_core::version::parse(version).ok_or("invalid update version")?;
    if crate::detect::windows::executable_version(exe).as_ref() != Some(&version) {
        return Err("The update executable's version does not match its release.".into());
    }
    let mut file = std::fs::File::open(exe).map_err(|e| e.to_string())?;
    let mut header = [0u8; 64];
    file.read_exact(&mut header).map_err(|e| e.to_string())?;
    let offset = header.get(60..64).and_then(|v| <[u8; 4]>::try_from(v).ok()).map(u32::from_le_bytes).ok_or("invalid PE header")?;
    if header.get(..2) != Some(b"MZ") || offset > 16 * 1024 * 1024 {
        return Err("invalid Windows executable".into());
    }
    use std::io::{Seek, SeekFrom};
    file.seek(SeekFrom::Start(u64::from(offset))).map_err(|e| e.to_string())?;
    let mut pe = [0u8; 6];
    file.read_exact(&mut pe).map_err(|e| e.to_string())?;
    let machine = u16::from_le_bytes([pe[4], pe[5]]);
    let expected = match std::env::consts::ARCH {
        "x86_64" => 0x8664,
        "x86" => 0x14c,
        "aarch64" => 0xaa64,
        _ => return Err("unsupported update architecture".into()),
    };
    if pe.get(..4) != Some(b"PE\0\0") || machine != expected {
        return Err("The update has the wrong executable architecture.".into());
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub(super) fn update(
    install: &Install,
    release: &Release,
    platform: Platform,
    http: &Http,
    paths: &Paths,
    stop: &Stop,
    phase: &mut dyn FnMut(JobPhase),
    last: &dyn Fn() -> Result<(), String>,
) -> Result<(), String> {
    let Install::Portable(exe) = install else { return Err("portable fork installation required".into()) };
    let exe = dunce::canonicalize(exe).map_err(|e| e.to_string())?;
    if dunce::canonicalize(std::env::current_exe().map_err(|e| e.to_string())?).map_err(|e| e.to_string())? != exe {
        return Err("Only the running launcher can schedule its own update.".into());
    }
    let own = paths_for(&exe)?;
    std::fs::create_dir_all(&own.cache).map_err(|e| e.to_string())?;
    if dunce::canonicalize(&own.cache).map_err(|e| e.to_string())? != dunce::canonicalize(&paths.cache).map_err(|e| e.to_string())? {
        return Err("The update cache does not belong to this portable installation.".into());
    }
    let asset = check_release(release, platform)?;
    let locations = InstallLocations::default();
    let mut job = crate::install::Job {
        http,
        paths,
        stop,
        phase,
        install_locations: &locations,
        replace: false,
        approved_path: None,
        downgrade: None,
        batch: None,
    };
    let (download, sha256) = crate::install::download(release, asset, &mut job)?;
    let staging = tempfile::Builder::new().prefix("fork-update-").tempdir_in(&own.cache).map_err(|e| e.to_string())?;
    let zip = staging.path().join("update.zip");
    std::fs::copy(download, &zip).map_err(|e| e.to_string())?;
    verify_hash(&zip, &sha256)?;
    inspect(&zip, &staging.path().join("preview"), &release.version_label())?;
    let helper = staging.path().join(HELPER);
    std::fs::copy(&exe, &helper).map_err(|e| e.to_string())?;
    let plan = Plan {
        exe,
        pid: std::process::id(),
        sha256,
        helper_sha256: crate::http::sha256_file(&helper)?,
        tag: release.tag.clone(),
        version: release.version_label(),
    };
    let plan_file = staging.path().join(PLAN);
    crate::paths::write_atomic(&plan_file, &serde_json::to_vec(&plan).map_err(|e| e.to_string())?)?;
    stop.check()?;
    last()?;
    process::command(&helper).arg(FLAG).arg(&plan_file).spawn().map_err(|e| format!("couldn't start the update helper ({e})"))?;
    let _ = staging.keep();
    Ok(())
}

fn verify_hash(file: &Path, expected: &str) -> Result<(), String> {
    if expected.len() != 64 || !expected.bytes().all(|b| b.is_ascii_hexdigit()) || crate::http::sha256_file(file)? != expected.to_ascii_lowercase() {
        return Err("The update checksum does not match its published SHA-256.".into());
    }
    Ok(())
}

pub(super) fn finish(args: &[OsString]) -> Option<Result<(), String>> {
    if args.first().is_none_or(|a| a != FLAG) {
        return None;
    }
    Some((|| {
        if args.len() != 2 {
            return Err("invalid portable update helper arguments".into());
        }
        let path = args.get(1).map(PathBuf::from).ok_or("missing update plan")?;
        let (plan, paths, directory) = read_plan(&path)?;
        let result = complete(&plan, &directory);
        if let Err(error) = &result {
            let _ = std::fs::remove_file(paths.data.join("self-update.txt"));
            let _ = crate::paths::write_atomic(&paths.logs().join("self-update-error.txt"), error.as_bytes());
            if process::wait_for_exit(plan.pid, Duration::ZERO) && plan.exe.is_file() {
                let _ = process::command(&plan.exe).env_remove(super::PRETEND_VAR).spawn();
            }
        }
        result
    })())
}

fn read_plan(path: &Path) -> Result<(Plan, Paths, PathBuf), String> {
    plain(path, false)?;
    let mut bytes = Vec::new();
    std::fs::File::open(path).map_err(|e| e.to_string())?.take(16 * 1024 + 1).read_to_end(&mut bytes).map_err(|e| e.to_string())?;
    if bytes.len() > 16 * 1024 {
        return Err("update plan is too large".into());
    }
    let plan: Plan = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    if plan.pid == 0
        || plan.version.len() > 128
        || plan.tag.len() > 128
        || plan.tag.contains(['\r', '\n'])
        || artcraft_launcher_core::version::parse(&plan.version).is_none()
    {
        return Err("invalid update plan".into());
    }
    let paths = paths_for(&plan.exe)?;
    let directory = dunce::canonicalize(path.parent().ok_or("no helper directory")?).map_err(|e| e.to_string())?;
    plain(&directory, true)?;
    let cache = dunce::canonicalize(&paths.cache).map_err(|e| e.to_string())?;
    if directory.parent() != Some(cache.as_path())
        || !directory.file_name().and_then(|s| s.to_str()).is_some_and(|s| s.starts_with("fork-update-"))
        || path.file_name().and_then(|s| s.to_str()) != Some(PLAN)
    {
        return Err("The helper plan is outside this launcher's cache.".into());
    }
    let helper = directory.join(HELPER);
    if dunce::canonicalize(std::env::current_exe().map_err(|e| e.to_string())?).map_err(|e| e.to_string())? != helper {
        return Err("The update helper must run from this launcher's cache.".into());
    }
    verify_hash(&helper, &plan.helper_sha256)?;
    verify_hash(&plan.exe, &plan.helper_sha256)?;
    Ok((plan, paths, directory))
}

fn complete(plan: &Plan, directory: &Path) -> Result<(), String> {
    complete_with(plan, directory, &|pid| process::wait_for_exit(pid, Duration::from_secs(120)), &|exe| {
        process::command(exe)
            .current_dir(exe.parent().ok_or("no installation folder")?)
            .env_remove(super::PRETEND_VAR)
            .spawn()
            .map(|_| ())
            .map_err(|e| format!("The update installed but could not restart ({e}). Open Craft Fork Launcher again."))
    })
}

fn complete_with(plan: &Plan, directory: &Path, wait: &dyn Fn(u32) -> bool, start: &dyn Fn(&Path) -> Result<(), String>) -> Result<(), String> {
    let zip = directory.join("update.zip");
    verify_hash(&zip, &plan.sha256)?;
    let (root, hashes) = inspect(&zip, &directory.join("verified"), &plan.version)?;
    if !wait(plan.pid) {
        return Err("The launcher did not close; its files were left unchanged.".into());
    }
    let folder = plan.exe.parent().ok_or("no installation folder")?;
    let staging = directory.join("replacement");
    let check = |file: &Path| {
        let relative = file.strip_prefix(staging.join("unpacked")).or_else(|_| file.strip_prefix(folder)).map_err(|e| e.to_string())?;
        let expected = hashes.get(relative).ok_or("An unexpected file appeared in the update.")?;
        verify_hash(file, expected)
    };
    portable::unpack_over(&zip, &staging, folder, std::ffi::OsStr::new(EXE), &check)?;
    verify_hash(&plan.exe, hashes.get(Path::new(EXE)).ok_or("missing launcher hash")?)?;
    let paths = paths_for(&plan.exe)?;
    let marker = format!("{}\n{}\n{}\n", plan.tag, crate::now(), plan.version);
    crate::paths::write_atomic(&paths.data.join("self-update.txt"), marker.as_bytes())?;
    if let Err(error) = start(&plan.exe) {
        let _ = std::fs::remove_file(paths.data.join("self-update.txt"));
        return Err(error);
    }
    let _ = std::fs::remove_file(paths.logs().join("self-update-error.txt"));

    let _ = std::fs::remove_file(&zip);
    let _ = std::fs::remove_dir_all(root);
    Ok(())
}

#[cfg(test)]
mod tests;
