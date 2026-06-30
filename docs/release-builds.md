# Desktop Build Notes

Desktop builds are for internal testing until Rasul explicitly approves a public release.

## Local macOS Build

Requires Xcode Command Line Tools for `iconutil`:

```bash
xcode-select --install
```

```bash
cargo build --release --bin tiny-retro-racer
scripts/package-macos-app.sh
```

Expected local artifact path:

```text
dist/tiny-retro-racer-macos/Tiny Retro Racer.app
```

This produces an unsigned and unnotarized `.app` bundle with a generated
`AppIcon.icns` logo. The raw executable lives inside the app bundle at
`Tiny Retro Racer.app/Contents/MacOS/tiny-retro-racer`.

For CI artifacts, use explicit architecture names:

- `tiny-retro-racer-macos-arm64` from `aarch64-apple-darwin`
- `tiny-retro-racer-macos-x64` from `x86_64-apple-darwin`

## Windows Build

Build on a Windows host or use the manual `Desktop Builds` GitHub Actions workflow:

```bash
cargo build --release --bin tiny-retro-racer
mkdir -p dist/tiny-retro-racer-windows
cp target/release/tiny-retro-racer.exe dist/tiny-retro-racer-windows/
cp README.md CHANGELOG.md dist/tiny-retro-racer-windows/
```

Expected artifact path:

```text
dist/tiny-retro-racer-windows/tiny-retro-racer.exe
```

## CI Artifact Workflow

The manual `.github/workflows/desktop-builds.yml` workflow builds:

- `tiny-retro-racer-macos-arm64` containing `Tiny Retro Racer.app`
- `tiny-retro-racer-macos-x64` containing `Tiny Retro Racer.app`
- `tiny-retro-racer-windows-x64`

Run it from GitHub Actions with `workflow_dispatch` when a desktop smoke build is needed. It uploads macOS `.app` bundles and the Windows raw executable with the README and changelog. It does not sign, notarize, create installers, or publish a release.

## Release Notes Convention

For internal test builds, include:

- Version or commit SHA.
- Platform artifact name.
- Controls and known limitations.
- Whether the first-playable checklist passed.
- Any manual playtest notes about recovery, camera, or input.

Public store release notes, Steam setup, signing, notarization, and installer work are out of scope until explicitly approved.
