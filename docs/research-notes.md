# Research Notes

## Bevy Baseline

Bevy 0.19 was announced on June 19, 2026. The project starts on `bevy = "0.19"` so we are not beginning on an already-stale API. Because Bevy still releases breaking changes regularly, gameplay logic should stay decoupled from engine-specific systems where it is cheap to do so.

Useful references:

- Bevy news: https://bevy.org/news/
- Bevy quick start: https://bevy.org/learn/quick-start/getting-started/
- Bevy crate: https://crates.io/crates/bevy
- Bevy examples: https://github.com/bevyengine/bevy/tree/main/examples

## Open-Source Learning Targets

For the next research task, prefer small examples over full games:

- Bevy official 2D examples for sprite spawning, camera setup, input, and fixed update patterns.
- Small Bevy jam projects for simple state machines and asset loading.
- Rust/Bevy driving demos only if the code is recent enough for Bevy 0.19.

## Distribution Notes

Steam distribution is possible for a small Rust game, but it is not a first-PR task. Track these before release work starts:

- Steam Direct account and app fee.
- Desktop build artifacts for Windows and macOS.
- Linux build path for Steam Deck verification.
- Controller input once keyboard driving works.
- Store assets, age-appropriate page copy, and privacy/support links.

