//! Opt-in acceptance check: real fork downloads, isolated portable installs, no app launch.
#![cfg(windows)]

use artcraft_launcher_core::catalog::CraftApp;
use artcraft_launcher_core::platform::{Arch, Os, Platform, pick_asset};
use artcraft_launcher_core::protocol::{InstallLocations, LatestRelease};
use artcraft_launcher_native::http::{Http, Stop};
use artcraft_launcher_native::install::{self, Job};
use artcraft_launcher_native::paths::Paths;
use artcraft_launcher_native::source::{GitHubSource, ReleaseSource};

#[test]
#[ignore = "downloads public FilmCraft and EffectCraft fork releases; installs only into a temporary folder"]
fn live_fork_releases_install_and_preserve_editor_data() {
    let root = tempfile::tempdir().unwrap();
    let paths = Paths::under(root.path());
    let http = Http::new().unwrap();
    let source = GitHubSource::new(http.clone(), paths.release_cache());
    let platform = Platform { os: Os::Windows, arch: Arch::X64 };
    let locations = InstallLocations::default();
    let stop = Stop::default();
    for (id, data_dir) in [("filmcraft", "FilmCraftData"), ("effectcraft", "EffectCraftData")] {
        let app = CraftApp::by_id(id).unwrap();
        let LatestRelease::Found(release) = source.latest(app.repo, false).latest else { panic!("No published fork release for {id}") };
        let stems: Vec<_> = app.stems().collect();
        let (asset, kind) = pick_asset(&release, &stems, platform, platform.default_kinds()).unwrap();
        assert!(asset.url.starts_with(&format!("https://github.com/{}/releases/download/", app.repo)));
        let mut phase = |_| {};
        let mut job = Job {
            http: &http,
            paths: &paths,
            stop: &stop,
            phase: &mut phase,
            install_locations: &locations,
            replace: false,
            approved_path: None,
            downgrade: None,
            batch: None,
        };
        let installed = install::install(app, &release, asset, kind, None, None, &mut job).unwrap().unwrap();
        assert!(installed.path.is_file());
        // The Windows installer records a canonical path with the verbatim namespace prefix.
        assert!(std::fs::canonicalize(&installed.path).unwrap().starts_with(std::fs::canonicalize(root.path()).unwrap()));
        assert_eq!(installed.version, release.version_label());
        let data = installed.path.parent().unwrap().join(data_dir);
        std::fs::create_dir_all(&data).unwrap();
        let sentinel = data.join("preserve-user-data.txt");
        std::fs::write(&sentinel, b"editor preferences and recovery must survive").unwrap();
        let updated = install::install(app, &release, asset, kind, None, Some(&installed), &mut job).unwrap().unwrap();
        assert_eq!(updated.path, installed.path);
        assert_eq!(std::fs::read(&sentinel).unwrap(), b"editor preferences and recovery must survive");
        eprintln!("{id}: {} downloaded, SHA256 verified, installed and reinstalled with user data preserved", release.version_label());
    }
}
