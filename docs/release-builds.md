# Desktop Build Notes

Desktop builds are for internal testing until Rasul explicitly approves a public release.

## Local macOS Build

```bash
cargo build --release --bin tiny-retro-racer
mkdir -p dist/tiny-retro-racer-macos
cp target/release/tiny-retro-racer dist/tiny-retro-racer-macos/
cp README.md CHANGELOG.md dist/tiny-retro-racer-macos/
```

Expected local artifact path:

```text
dist/tiny-retro-racer-macos/tiny-retro-racer
```

This produces a raw executable, not a signed or notarized `.app` bundle.

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

- `tiny-retro-racer-macos`
- `tiny-retro-racer-windows`

Run it from GitHub Actions with `workflow_dispatch` when a desktop smoke build is needed. It uploads raw executable artifacts with the README and changelog. It does not sign, notarize, create installers, or publish a release.

## Release Notes Convention

For internal test builds, include:

- Version or commit SHA.
- Platform artifact name.
- Controls and known limitations.
- Whether the first-playable checklist passed.
- Any manual playtest notes about recovery, camera, or input.

Public store release notes, Steam setup, signing, notarization, and installer work are out of scope until explicitly approved.
