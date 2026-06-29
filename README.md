# Tiny Retro Racer

Tiny Retro Racer is a kid-friendly retro arcade racing game for Rust + Bevy. The first target is a simple single-car prototype for a 4-year-old: easy controls, a forgiving track, and no way to get stuck.

The project intentionally tracks current stable Rust and current Bevy APIs. It starts on Rust 2024 edition and Bevy 0.19.0.

## Controls

- `Up Arrow`: accelerate
- `Left Arrow`: steer left
- `Right Arrow`: steer right
- `Down Arrow`: brake

## Local Development

Install Rust with rustup, then run:

```bash
cargo run
```

Quality checks:

```bash
cargo fmt --check
cargo test --lib --no-default-features
cargo clippy --lib --no-default-features -- -D warnings
cargo check --bin tiny-retro-racer
```

## First Playable Goal

The first playable version should open on a start screen, launch one controllable car on one circuit, and keep the player recoverable at all times. macOS and Windows builds are the first desktop priorities. Steam and Steam Deck support should stay visible in release planning, but no public store action should happen without Rasul's approval.

## Project Links

- Rowset project: `5da907b9-a8b6-468c-8e71-00393ea3c44f`
- Rowset task dataset: `6618cfd3-fcc2-450a-a3d4-7e3ac2f59452`
- Todoist execution task: `6h26GVX9Jwwxg5gC`
