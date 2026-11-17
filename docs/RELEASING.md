# Releasing FilmLogger

Releases are cut from semantic-version tags (`v0.1.0`, `v0.2.0`, …).
Pushing the tag runs `.github/workflows/release.yml`, which builds and attaches:

- Windows NSIS installer for x86_64 and for ARM64 (`aarch64`)
- Linux AppImage for x86_64 and for ARM64 (`aarch64`)
- Signed Android release APK
- Tauri updater metadata (`latest.json`) and signatures, including
  `windows-x86_64`, `windows-aarch64`, `linux-x86_64`, and `linux-aarch64`

The jobs build natively on `windows-latest` (x64), `windows-11-arm` (ARM64),
`ubuntu-latest` (x64), and `ubuntu-24.04-arm` (ARM64); the matrix is sequential
so tauri-action merges each job's updater entry into the release's existing
`latest.json`.

No tag or release is pushed by the agent.

## GitHub Actions secrets

Add these repository secrets before pushing the first tag:

| Secret | Value |
| --- | --- |
| `TMDB_API_KEY` | TMDB API key or Read Access Token |
| `ANDROID_KEYSTORE_BASE64` | Base64 of `filmlogger-release.keystore` |
| `ANDROID_KEYSTORE_PASSWORD` | Android keystore password |
| `ANDROID_KEY_ALIAS` | Android key alias (`filmlogger`) |
| `ANDROID_KEY_PASSWORD` | Android key password |
| `TAURI_SIGNING_PRIVATE_KEY` | Contents of `tauri-updater.key` |
| `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` | Password for the Tauri updater key |

`GITHUB_TOKEN` is provided by Actions and is used to create/update the release.

## Local signing material

The generated material is intentionally outside the repository:

- `C:\Users\jakel\.filmlogger\filmlogger-release.keystore`
- `C:\Users\jakel\.filmlogger\release-signing.properties`
- `C:\Users\jakel\.filmlogger\filmlogger-release.keystore.b64`
- `C:\Users\jakel\.filmlogger\tauri-updater.key`
- `C:\Users\jakel\.filmlogger\tauri-updater.key.pub`
- `C:\Users\jakel\.filmlogger\tauri-updater-password.txt`

None of these are committed. Local release builds read
`release-signing.properties` automatically from `~/.filmlogger/`.

To set the base64 secret:

```sh
cat ~/.filmlogger/filmlogger-release.keystore.b64
```

## Release updater endpoint

The desktop updater reads:

```text
https://github.com/jakelandau/filmlogger-rs/releases/latest/download/latest.json
```

The Android app reads the GitHub latest-release API and downloads the APK
asset, then opens the system APK installer.

## Cutting a release

1. Update the project version if needed.
2. Commit the version changes.
3. Tag and push:

```sh
git tag v0.1.0
git push origin v0.1.0
```

The workflow sets the desktop and Android versions from the tag; Android
also uses the Actions run number as `versionCode`.

## GitHub repository rename

The repository and local folder are intended to be named `filmlogger-rs`.
The GitHub repository should be renamed to `jakelandau/filmlogger-rs` and the
remote updated accordingly before relying on the updater URLs.

## macOS

CI builds the desktop shell in release mode on `macos-latest` (UI bundle plus
`cargo build --release -p filmlogger-rs-desktop`), so an Apple-only regression
fails the build instead of surfacing at the first macOS release.

No macOS artifact is published: Gatekeeper blocks an unsigned or un-notarized
`.dmg` for anyone who downloads it, so a macOS release waits for an Apple
Developer ID. When that exists, add `macos-latest` with `bundles: dmg` to the
`release.yml` desktop matrix and set `APPLE_CERTIFICATE`,
`APPLE_CERTIFICATE_PASSWORD`, `APPLE_SIGNING_IDENTITY`, `APPLE_ID`,
`APPLE_PASSWORD`, and `APPLE_TEAM_ID`; tauri-action then codesigns, notarizes,
and staples the bundle.
