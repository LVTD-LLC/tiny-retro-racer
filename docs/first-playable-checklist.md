# First Playable Verification Checklist

Use this before calling the first playable ready in a PR summary. Keep notes short and include the commit SHA being checked.

## Automated Checks

- [ ] `cargo fmt --check`
- [ ] `cargo test --lib --no-default-features`
- [ ] `cargo clippy --lib --no-default-features -- -D warnings`
- [ ] `cargo check --bin tiny-retro-racer`
- [ ] `cargo clippy --bin tiny-retro-racer -- -D warnings`

## Launch And Start Flow

- [ ] `cargo run --bin tiny-retro-racer` opens a window without panics.
- [ ] The title screen is visible before gameplay starts.
- [ ] Clicking `Play` starts gameplay.
- [ ] `Enter` starts gameplay from the title screen.
- [ ] `Space` starts gameplay from the title screen.
- [ ] `Esc` returns from gameplay to the title screen.
- [ ] Starting, returning, and starting again does not leave duplicate cars, tracks, or title UI.

## Controls

- [ ] `Up Arrow` accelerates.
- [ ] `Down Arrow` brakes and only reverses gently.
- [ ] `Left Arrow` and `Right Arrow` steer predictably.
- [ ] Pressing both steering keys cancels steering rather than snapping.
- [ ] `R` resets the car to the start position during gameplay.

## Recovery And Camera

- [ ] Driving beyond the outer road edge pushes the car back onto the road.
- [ ] Driving toward the inner grass pushes the car back onto the road.
- [ ] The car remains recoverable after repeated boundary hits.
- [ ] The camera follows smoothly and keeps more road visible ahead of the car.
- [ ] The camera does not flip or disorient the steering direction.

## Platform Smoke

- [ ] macOS local release build completes or the failure is noted.
- [ ] Windows CI/manual build artifact completes or the failure is noted.
- [ ] Any observed audio, window, input, or GPU issues are listed in the PR summary.
