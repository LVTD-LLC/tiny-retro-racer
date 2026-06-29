# Tiny Retro Racer Design Notes

## Design Problem

Create a game where a 4-year-old can drive a retro arcade car around a simple circuit and feel fast, successful, and unblocked under very simple controls.

## Target Experience

The game should feel like a very old arcade driving game: pixelated, readable, bright, and viewed from above/behind the car. It should reward holding accelerate and turning, not punish imperfect driving.

## MVP Scope

- One car.
- One circuit.
- No AI cars.
- Start screen with a start button or simple key input.
- Arrow-key controls only at first.
- Forgiving road and boundary behavior.
- No crash/fail state in the first playable.

## Non-Goals For The First PR

- Final art.
- AI opponents.
- Menus, settings, profiles, or save files.
- Steam release setup beyond documented planning.
- Controller input beyond keeping Steam Deck in mind.

## Top Risks

| Rank | Risk | Severity | Why It Matters |
| --- | --- | --- | --- |
| 1 | Driving feel is too hard for a 4-year-old | Fatal | The core player cannot enjoy the game if steering and speed require precision. |
| 2 | The car can get stuck or point backward forever | Fatal | Rasul explicitly wants forward progress to remain possible. |
| 3 | Camera angle is confusing | Major | Behind/above views can make turning hard to parse if the camera shifts too much. |
| 4 | Bevy version churn slows development | Major | The project should start on current Bevy, but keep code small and boring. |
| 5 | Distribution needs are ignored until too late | Major | macOS, Windows, Steam, and Steam Deck expectations affect build and input decisions. |

## Prototype Plan

The first implementation slice should answer two questions:

- Can the project launch and render a stable placeholder driving scene?
- Can the car-control math be tested and tuned without needing a full Bevy runtime?

Success for the initial skeleton is a compiling Bevy app plus unit-tested acceleration, braking, steering, and speed clamps. The next slice should turn the placeholder scene into a real start screen and gameplay state.

The first playable slice now answers the next risk with an oval circuit:

- Can a 4-year-old start the game with one clear action?
- Can the car stay recoverable on a closed circuit without a crash/fail state?
- Can the camera show the car from behind/above without flipping or hiding the road ahead?

## Kid-Friendly Driving Model

Player model: the target player may hold `Up Arrow`, oversteer, reverse by accident, and ignore precise racing lines. The desired promise is that simple inputs make the car feel fast, readable, and successful without demanding recovery skill.

Mechanic brief: the core loop is hold accelerate, steer around the oval, see the road and car react immediately, and get gently corrected when crossing a boundary. There is no crash, fail, or blocked state in the MVP.

Central tuning knobs:

- `DrivingTuning` owns acceleration, braking, drag, maximum forward speed, reverse limit, turn rate, and boundary recovery speed behavior.
- `TrackSpec` owns the oval radii and half-width. The default half-width is intentionally wide for low-precision steering.
- `TrackSpec::recover_car_state_with_margin` owns boundary recovery for position, tangent heading, and speed together, using the car footprint margin supplied by the Bevy layer.

Current feel targets:

- Acceleration should respond quickly without reaching an unreadable top speed.
- Braking should stop the car fast and allow only a tiny, gentle reverse.
- Boundary contact should preserve forward movement for fast hits, stop accidental reverse hits, and nudge the car forward when the player is holding accelerate into an edge.

## Build Notes

- Keep car movement tuning centralized.
- Prefer gentle correction and recovery over failure states.
- Separate pure gameplay math from Bevy systems where practical. The initial driving model uses a tiny local `Vec2` to avoid pulling Bevy/glam into library tests; switch to `glam` if the math surface grows beyond simple position storage.
- Represent the first circuit as a simple elliptical road band. The pure track model owns containment/recovery; the Bevy mesh only visualizes it.
- Run driving simulation in a fixed timestep. Keep the camera in frame updates so it can smooth toward the car every rendered frame.
- Keep the MVP camera in 2D with a behind-car offset rather than full 3D. This preserves the old-arcade above/behind feel with much lower tuning and asset cost; revisit a true 3D camera only after the driving loop proves fun.
- Keep visual placeholders intentionally simple until the driving loop feels right.
