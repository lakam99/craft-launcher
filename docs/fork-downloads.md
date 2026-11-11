# Craft Fork Launcher

This independent fork is based on ArtCraft Launcher by the ArtCraft team and community. It uses its own name and original download-arrow artwork.

| App | Install, update and version history source |
|---|---|
| FilmCraft | [lakam99/filmcraft releases](https://github.com/lakam99/filmcraft/releases) |
| EffectCraft | [lakam99/effectcraft releases](https://github.com/lakam99/effectcraft/releases) |
| SoundCraft | [lakam99/soundcraft releases](https://github.com/lakam99/soundcraft/releases) |
| Craft Fork Launcher | [lakam99/craft-launcher releases](https://github.com/lakam99/craft-launcher/releases) |

The other catalog apps retain their existing repositories. FilmCraft, EffectCraft and SoundCraft are labelled as community builds by `lakam99`. Their original executable names are preserved, so normal launch and version detection work.

Fork releases are read directly from their pinned GitHub repositories. The upstream aggregate feed cannot override them or mark a fork as having no releases. Download URLs must belong to the selected repository; the existing SHA-256 verification, bounded extraction, rollback and user-data preservation remain in place. The network relay ZIP is not an editor installer.

The published editor builds currently offer Windows x64 portable ZIPs, which the launcher can install into a folder you choose. No MSI or macOS/Linux build is invented or substituted from upstream. A platform without a matching asset has no fork installer. macOS signature requirements remain enforced.

Extract the launcher's Windows portable ZIP and run `artcraft-launcher.exe`. Its adjacent `portable.txt` keeps its preferences, download cache and installation records under `CraftForkLauncherData`, separate from an upstream launcher. Keep that directory when replacing the launcher files.

The fork also uses its own local startup channel, so an upstream launcher running alongside it is not asked to show, quit or hand over its window.

Windows builds are unsigned. Starting with 0.14.3, this fork's Windows portable launcher updates itself from `lakam99/craft-launcher` over verified HTTPS with a mandatory published SHA-256. The parent validates the entire ZIP before closing; its helper waits for exit, checks the archive and installed file hashes again, preserves `CraftForkLauncherData`, and rolls back a failed copy. MSI, standalone copies without `portable.txt`, and other operating systems retain the shared updater's signing and package rules.

**One-time upgrade from 0.14.2 or earlier:** close the launcher (including its tray icon), extract the new portable ZIP, and copy its program files over the old folder. Keep `CraftForkLauncherData`. The old updater cannot install the fix that changes its own signing requirement.

The source keeps the original crate, binary and release asset stems for compatibility. The published product name is Craft Fork Launcher. All original copyrights and open-source notices are retained.
