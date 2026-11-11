# Craft Fork Launcher

Independent fork of [ArtCraft Launcher](https://github.com/storytold/craft-launcher) by the ArtCraft team and community. This version uses its own name and original open-licensed artwork.

![Craft Fork Launcher icon](assets/app-icon/hicolor/128x128/apps/ai.storyteller.artcraft-launcher.png)

Install, update and open the Craft apps, with FilmCraft, EffectCraft and SoundCraft sourced from your forks:

| App | Repository used for releases and installers |
|---|---|
| FilmCraft | [lakam99/filmcraft](https://github.com/lakam99/filmcraft/releases) |
| EffectCraft | [lakam99/effectcraft](https://github.com/lakam99/effectcraft/releases) |
| SoundCraft | [lakam99/soundcraft releases](https://github.com/lakam99/soundcraft/releases) |
| Craft Fork Launcher | [lakam99/craft-launcher](https://github.com/lakam99/craft-launcher/releases) |

Other catalog apps retain their existing repositories. The three editor forks are shown as community builds by lakam99. Their release feeds are pinned to these repositories; the upstream aggregate feed cannot replace or hide their releases. Downloads retain the existing SHA-256 verification, safe extraction and user-data preservation.

## Download

Get the Windows x64 portable ZIP from [this fork's releases](https://github.com/lakam99/craft-launcher/releases/latest). Extract it and run `artcraft-launcher.exe`. The adjacent `CraftForkLauncherData` directory holds the launcher's settings, installation records and cache. Keep it when upgrading the launcher.

The current FilmCraft, EffectCraft and SoundCraft fork releases provide Windows x64 portable ZIPs. The launcher selects these automatically; it does not substitute upstream packages on a platform without a matching fork asset. macOS signing checks remain enforced. Windows binaries are unsigned. Versions 0.14.3 and newer can update their own portable installation using this fork's GitHub release checksum. Older launchers need one manual upgrade.

See [the fork download guide](docs/fork-downloads.md) for details. Machine names, crate names and release asset stems retain `artcraft-launcher` for compatibility; the published app name is Craft Fork Launcher.

## Build and test

Use the Rust toolchain specified by `rust-toolchain.toml`, if present, or the minimum version in `Cargo.toml`.

```sh
cargo build --release -p artcraft-launcher
cargo xtask ci
```

The existing browser build remains available through `cargo xtask wasm`. Native filesystem and installation code stays outside the browser build.

The opt-in Windows acceptance test downloads the actual fork releases and installs them only into an isolated temporary folder, then verifies that reinstalling preserves editor data. It never launches an app or changes an existing installation:

```sh
cargo test -p artcraft-launcher-native --test fork_install -- --ignored --nocapture
```

## Licenses

Code and original fork artwork use MIT OR Apache-2.0. All original copyrights are retained in [NOTICE](NOTICE), [LICENSE-MIT](LICENSE-MIT) and [LICENSE-APACHE](LICENSE-APACHE). Third-party assets keep their own licenses, recorded in [ATTRIBUTION.md](ATTRIBUTION.md). Restricted ArtCraft marks have been removed from this distributed fork.
