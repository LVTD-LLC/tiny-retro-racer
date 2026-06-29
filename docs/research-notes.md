# Research Notes

## Bevy Baseline

Bevy 0.19 was announced on June 19, 2026. The project starts on `bevy = "0.19.0"` so we are not beginning on an already-stale API. Because Bevy still releases breaking changes regularly, gameplay logic should stay decoupled from engine-specific systems where it is cheap to do so.

The Rust baseline is current stable with Rust 2024 edition. The repo includes `rust-toolchain.toml` with `channel = "stable"` so local development and CI stay on the latest stable toolchain rather than an old pinned compiler.

Useful references:

- Bevy news: https://bevy.org/news/
- Bevy quick start: https://bevy.org/learn/quick-start/getting-started/
- Bevy crate: https://crates.io/crates/bevy
- Bevy examples: https://github.com/bevyengine/bevy/tree/main/examples

## Open-Source Learning Targets

Prefer small examples over full games because the first playable needs a stable
movement loop faster than it needs production art or large code patterns.

### Bevy 0.19 References To Reuse

- State flow: Bevy's `examples/state/states.rs` and `examples/ecs/state_scoped.rs`
  show `States`, `OnEnter`, `OnExit`, `run_if(in_state(...))`, and state-scoped
  cleanup. Reuse this for start screen, playing, pause, and later game-over
  phases instead of scattered boolean flags.
- Fixed-step movement: Bevy's `examples/ecs/fixed_timestep.rs` and
  `examples/movement/physics_in_fixed_timestep.rs` keep input sampling and
  simulation timing explicit. The racer should run car integration in
  `FixedUpdate`, with camera smoothing in frame updates.
- Camera follow: Bevy's `examples/camera/2d_top_down_camera.rs` uses
  `smooth_nudge` for readable player tracking. For this game, offset the camera
  behind the car heading so the player sees more road ahead.
- Simple 2D geometry: Bevy's `examples/2d/2d_shapes.rs` supports primitive
  meshes such as `Ellipse` and `Ring`. This is enough for a deliberate oval
  placeholder circuit before custom track art exists.
- Pixelated visuals: Bevy's `examples/2d/pixel_grid_snap.rs` demonstrates a
  low-resolution render target and `ImagePlugin::default_nearest()`. Keep this
  for the visual-style pass instead of adding it to the first control slice.

Source links:

- Bevy 0.19 examples tag:
  https://github.com/bevyengine/bevy/tree/v0.19.0/examples
- Bevy states example:
  https://github.com/bevyengine/bevy/blob/v0.19.0/examples/state/states.rs
- Bevy state-scoped cleanup example:
  https://github.com/bevyengine/bevy/blob/v0.19.0/examples/ecs/state_scoped.rs
- Bevy fixed timestep example:
  https://github.com/bevyengine/bevy/blob/v0.19.0/examples/ecs/fixed_timestep.rs
- Bevy fixed physics example:
  https://github.com/bevyengine/bevy/blob/v0.19.0/examples/movement/physics_in_fixed_timestep.rs
- Bevy top-down camera example:
  https://github.com/bevyengine/bevy/blob/v0.19.0/examples/camera/2d_top_down_camera.rs
- Bevy pixel-grid snap example:
  https://github.com/bevyengine/bevy/blob/v0.19.0/examples/2d/pixel_grid_snap.rs

### Lessons To Avoid

- Do not copy large racing demos until they prove Bevy 0.19 compatibility.
  Small official examples are easier to port and review.
- Do not put track collision in rendering meshes. The first circuit uses a
  test-covered elliptical band model, then renders that model with Bevy shapes.
- Do not add pixel-perfect rendering before the car and camera feel good. It is
  a style pass, not a core playability dependency.

## Distribution Notes

Steam distribution is possible for a small Rust game, but it is not a first-PR task. Track these before release work starts:

- Steam Direct requires a Steamworks partner account, paperwork, tax/payment
  setup, and the per-app Steam Direct fee before an app can be released.
- Steam's release process expects a store page, build review, and release
  checklist review. Do not submit anything publicly without Rasul's explicit
  approval.
- SteamPipe is the upload path for depots/builds. Plan separate depots or build
  outputs for Windows, macOS, and Linux when packaging work begins.
- Steam Deck support should be treated as Linux + controller + legibility work:
  build a Linux target, verify Proton/native behavior, add controller input, and
  keep text readable at handheld resolution.
- macOS and Windows remain the first desktop priorities. Linux/Deck planning
  should not block the first playable.

Steam source links:

- Steam Direct:
  https://partner.steamgames.com/doc/gettingstarted/appfee
- Steam release process:
  https://partner.steamgames.com/doc/store/releasing
- SteamPipe build uploads:
  https://partner.steamgames.com/doc/sdk/uploading
- Steam Deck compatibility review:
  https://partner.steamgames.com/doc/steamdeck/compat
