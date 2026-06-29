# Changelog

## Unreleased

- Added `Cargo.toml`, `Cargo.lock`, and Rust/Bevy source layout for the initial Tiny Retro Racer app shell.
- Set the language and engine baseline to Rust 2024 edition, latest stable Rust via `rust-toolchain.toml`, and Bevy `0.19.0`.
- Added `src/driving.rs` with test-covered acceleration, braking, steering, speed limits, tuning sanitization, frame-delta clamping, and tiny-speed snap-to-zero behavior.
- Added `src/main.rs` with a placeholder Bevy scene, camera, road sprites, player car, keyboard input mapping, and road-boundary clamping.
- Added `.github/workflows/ci.yml` for format, test, clippy, and cargo check on pull requests.
- Added `.github/workflows/reviewgate.yml` with pinned ReviewGate review enforcement.
- Added `README.md`, `docs/game-design.md`, and `docs/research-notes.md` for MVP scope, controls, risks, and distribution notes.
- Added Bevy state flow with a start screen, Play button, keyboard start, gameplay cleanup, reset, and return-to-start controls.
- Replaced the rectangular placeholder road with a closed oval circuit, smooth follow camera, and test-covered track recovery model.
- Expanded research notes with Bevy 0.19 movement/state/camera references and a practical Steam/Steam Deck release checklist.
- Added a manual first-playable verification checklist and desktop build notes.
- Added a manual GitHub Actions workflow for macOS and Windows release artifacts.
- Added generated low-resolution pixel-art textures with nearest filtering for the first retro visual pass.
