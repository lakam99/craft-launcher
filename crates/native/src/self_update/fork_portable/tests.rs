use super::*;
use std::io::Write;

fn archive(path: &Path, files: &[(&str, &[u8])]) {
    let mut zip = zip::ZipWriter::new(std::fs::File::create(path).unwrap());
    for (name, bytes) in files {
        zip.start_file(*name, zip::write::SimpleFileOptions::default()).unwrap();
        zip.write_all(bytes).unwrap();
    }
    zip.finish().unwrap();
}

#[test]
fn only_the_fork_portable_installation_uses_this_policy() {
    assert!(applies(&Install::Portable(PathBuf::new())));
    assert!(!applies(&Install::Msi(PathBuf::new())));
    assert!(!applies(&Install::Standalone(PathBuf::new())));
    assert!(!applies(&Install::MacApp(PathBuf::new())));
}

#[test]
fn release_urls_cannot_escape_the_fork_or_lie_about_the_version() {
    let platform = Platform { os: artcraft_launcher_core::Os::Windows, arch: artcraft_launcher_core::Arch::X64 };
    let mut release = Release {
        tag: "v0.14.3".into(),
        version: artcraft_launcher_core::version::parse("0.14.3"),
        assets: vec![artcraft_launcher_core::release::Asset {
            name: "artcraft-launcher-0.14.3-windows-x64-portable.zip".into(),
            url: "https://github.com/lakam99/craft-launcher/releases/download/v0.14.3/artcraft-launcher-0.14.3-windows-x64-portable.zip".into(),
            ..Default::default()
        }],
        ..Default::default()
    };
    assert!(check_release(&release, platform).is_ok());
    let asset = release.assets[0].url.clone();
    for bad in [asset.replace("lakam99/", "storytold/"), asset.replace("/v0.14.3/", "/v0.14.2/"), asset.replace("/v0.14.3/", "/../../evil/v0.14.3/")]
    {
        release.assets[0].url = bad;
        assert!(check_release(&release, platform).is_err());
    }
}

#[test]
fn bad_checksums_and_payloads_never_replace_an_installation() {
    let dir = tempfile::tempdir().unwrap();
    let zip = dir.path().join("update.zip");
    for (index, folder) in
        ["CraftForkLauncherData", "cRaFtFoRkLaUnChErDaTa", "CraftForkLauncherData.", "ArtCraftLauncherData ", "CRAFTF~1"].iter().enumerate()
    {
        let data = format!("{folder}/settings.json");
        archive(&zip, &[(EXE, b"not a program"), (&data, b"must not replace settings"), ("portable.txt", b"portable")]);
        assert!(verify_hash(&zip, &"0".repeat(64)).is_err());
        let error = inspect(&zip, &dir.path().join(format!("preview-{index}")), "0.14.3").unwrap_err();
        assert!(error.contains("user data") || error.contains("unsafe path"), "{folder}: {error}");
    }
    let hashes = crate::http::sha256_file(&zip).unwrap();
    verify_hash(&zip, &hashes).unwrap();
    std::fs::write(&zip, b"changed after download").unwrap();
    assert!(verify_hash(&zip, &hashes).is_err());
}

#[test]
fn malformed_and_oversized_helper_plans_are_rejected() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join(PLAN);
    std::fs::write(&path, b"{}").unwrap();
    assert!(read_plan(&path).is_err());
    std::fs::write(&path, vec![b' '; 16 * 1024 + 1]).unwrap();
    assert!(read_plan(&path).unwrap_err().contains("too large"));
}

/// Exercises the real unsigned PE and the production extract/hash/replace transaction without
/// closing or launching anybody's installed launcher. Set the built fork executable explicitly.
#[test]
#[ignore = "requires CRAFT_FORK_TEST_EXE pointing to a built Windows fork launcher"]
fn unsigned_portable_update_preserves_data_and_restarts_only_after_parent_exit() {
    let binary = PathBuf::from(std::env::var_os("CRAFT_FORK_TEST_EXE").unwrap());
    assert!(verifysign::verify_file(&binary).is_err(), "regression fixture must be unsigned");
    let version = crate::detect::windows::executable_version(&binary).unwrap().to_string();
    let bytes = std::fs::read(&binary).unwrap();
    let dir = tempfile::tempdir().unwrap();
    let install = dir.path().join("portable");
    std::fs::create_dir(&install).unwrap();
    let exe = install.join(EXE);
    std::fs::write(&exe, &bytes).unwrap();
    std::fs::write(install.join("portable.txt"), b"portable").unwrap();
    let paths = paths_for(&exe).unwrap();
    std::fs::create_dir_all(&paths.cache).unwrap();
    std::fs::create_dir_all(&paths.data).unwrap();
    let sentinel = paths.data.join("settings.json");
    std::fs::write(&sentinel, b"keep preferences and installation records").unwrap();
    let job = paths.cache.join("fork-update-test");
    std::fs::create_dir(&job).unwrap();
    let zip = job.join("update.zip");
    archive(&zip, &[(EXE, &bytes), ("portable.txt", b"portable"), ("README.md", b"new program files")]);
    let plan = Plan {
        exe: exe.clone(),
        pid: 42,
        sha256: crate::http::sha256_file(&zip).unwrap(),
        helper_sha256: String::new(),
        tag: format!("v{version}"),
        version,
    };
    let starts = std::cell::Cell::new(0);
    let start = |path: &Path| {
        assert_eq!(path, exe);
        starts.set(starts.get() + 1);
        Ok(())
    };
    assert!(complete_with(&plan, &job, &|_| false, &start).is_err());
    assert_eq!(starts.get(), 0);
    assert!(!install.join("README.md").exists());
    // A fresh helper staging directory for the next attempt.
    std::fs::remove_dir_all(job.join("verified")).unwrap();
    complete_with(&plan, &job, &|_| true, &start).unwrap();
    assert_eq!(starts.get(), 1);
    assert_eq!(std::fs::read(sentinel).unwrap(), b"keep preferences and installation records");
    assert_eq!(std::fs::read(install.join("README.md")).unwrap(), b"new program files");
    assert!(paths.data.join("self-update.txt").is_file());
}
