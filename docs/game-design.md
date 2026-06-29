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

## Build Notes

- Keep car movement tuning centralized.
- Prefer gentle correction and recovery over failure states.
- Separate pure gameplay math from Bevy systems where practical. The initial driving model uses a tiny local `Vec2` to avoid pulling Bevy/glam into library tests; switch to `glam` if the math surface grows beyond simple position storage.
- Keep visual placeholders intentionally simple until the driving loop feels right.
