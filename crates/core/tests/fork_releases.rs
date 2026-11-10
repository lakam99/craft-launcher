use artcraft_launcher_core::catalog::{AppKind, CraftApp, LAUNCHER, LAUNCHER_REPO};
use artcraft_launcher_core::feed::{Feed, FeedAnswer};
use artcraft_launcher_core::platform::{Arch, Os, PackageKind, Platform, pick_asset};
use artcraft_launcher_core::release::parse_github_release;
use serde_json::json;

#[test]
fn editor_forks_and_launcher_pin_their_own_publishers_and_repositories() {
    for id in ["filmcraft", "effectcraft"] {
        let app = CraftApp::by_id(id).unwrap();
        assert_eq!(app.repo, format!("lakam99/{id}"));
        assert_eq!(app.publisher, "lakam99");
        assert_eq!(app.kind, AppKind::Community);
        assert_eq!(app.mac_team, None);
    }
    assert_eq!(LAUNCHER_REPO, "lakam99/craft-launcher");
    assert_eq!(LAUNCHER.repo, LAUNCHER_REPO);
    assert_eq!(LAUNCHER.name, "Craft Fork Launcher");
    assert_eq!(CraftApp::by_id("photocraft").unwrap().repo, "storytold/photocraft");
}

#[test]
fn upstream_feed_cannot_hide_or_redirect_fork_releases() {
    for id in ["filmcraft", "effectcraft"] {
        let app = CraftApp::by_id(id).unwrap();
        for repo in [None, Some(format!("storytold/{id}")), Some(app.repo.to_string())] {
            let feed = Feed::parse(
                &json!({"release_info":{"apps":{id:{"repo":repo,"app_last_fetched":"2026-11-10T00:00:00Z","latest_release":null}}}}).to_string(),
            )
            .unwrap();
            assert_eq!(feed.repo_for(app), app.repo);
            assert_eq!(feed.answer(app, 1_794_268_800), FeedAnswer::AskGitHub);
        }
    }
}

#[test]
fn published_editor_packages_are_selected_with_their_hashes_from_the_fork_only() {
    let platform = Platform { os: Os::Windows, arch: Arch::X64 };
    for (id, fixture, version) in [
        ("filmcraft", include_str!("fixtures/filmcraft-fork-release.json"), "0.17.2"),
        ("effectcraft", include_str!("fixtures/effectcraft-fork-release.json"), "0.23.2"),
    ] {
        let app = CraftApp::by_id(id).unwrap();
        let release = parse_github_release(fixture, app.repo).unwrap();
        let (asset, kind) = pick_asset(&release, &[id], platform, platform.default_kinds()).unwrap();
        assert_eq!(kind, PackageKind::PortableZip);
        assert_eq!(asset.name, format!("{id}-{version}-windows-x64-portable.zip"));
        assert!(asset.url.starts_with(&format!("https://github.com/lakam99/{id}/releases/download/")));
        assert_eq!(asset.sha256.as_ref().unwrap().len(), 64);
        let wrong = fixture.replace(&format!("lakam99/{id}"), &format!("storytold/{id}"));
        let rejected = parse_github_release(&wrong, app.repo).unwrap();
        assert!(pick_asset(&rejected, &[id], platform, platform.default_kinds()).is_none());
    }
}
