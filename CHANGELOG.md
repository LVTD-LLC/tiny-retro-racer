# Changelog

## Unreleased

- Added `Cargo.toml`, `Cargo.lock`, and Rust/Bevy source layout for the initial Tiny Retro Racer app shell.
- Added `src/driving.rs` with test-covered acceleration, braking, steering, speed limits, tuning sanitization, and frame-delta clamping.
- Added `src/main.rs` with a placeholder Bevy scene, camera, road sprites, player car, keyboard input mapping, and road-boundary clamping.
- Added `.github/workflows/ci.yml` for format, test, clippy, and cargo check on pull requests.
- Added `.github/workflows/reviewgate.yml` with pinned ReviewGate review enforcement.
- Added `README.md`, `docs/game-design.md`, and `docs/research-notes.md` for MVP scope, controls, risks, and distribution notes.
