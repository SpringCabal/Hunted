# Hunted (Zeus vs Bunnies), WASM

The Rust/WASM port of Hunted. Game assets and definitions retain their original formats.

For the engine, launcher, builds and verification, use the [SpringCabal development repository](https://github.com/gajop/springcabal-dev).

## Modules

| Module | Source | Purpose |
| --- | --- | --- |
| `hunted-rules` | `wasm-src/rules-synced` | Game rules and unit scripts. |
| `hunted-ui` | `wasm-src/ui` | HUD, controls and rendering. |

## Differences from the Lua version

The HUD uses RmlUi instead of Chili. Restart resets the round in place, and the end screen also accepts R to restart and Q to quit.

## Scoped SDK port

This worktree builds the Wasm rules/UI modules against `spring-sdk`, the
shared addon handler and the shared CUS host. Helpers receive `&Spring`;
queued commands, Rml callbacks and unit-script operations obtain a fresh
reference when they run. This does not build native game modules.

Verification on the SDK test engine:

- Strict Clippy passes for all workspace targets/features and for the default
  build. All 32 common Rust unit tests pass.
- All 35 existing integration checks pass on Linux and Wine. A repeated Linux
  failure also reproduced in the old code: the nearby-shot fear test sometimes
  killed its moving rabbit before checking its fear. The fixture now chooses
  among the same nearby aim offsets using the actual muzzle position, so the
  shot goes beside the rabbit. All fear, ammunition and damage assertions are
  unchanged. The corrected suite passes at three different run speeds.
- The five-shot visual scenario and HUD binding/restart checks complete on
  GPU and software renderers. All eight manually measured images/crops meet
  the existing limits (full frames 3.66–4.83% against 10%; HUD/end screen
  0.48–0.56% against 2%). The visual suite still rejects this machine's Mesa
  Ubuntu package revision `.4` versus the recorded `.2`. Goldens, renderer
  checks and tolerances are unchanged.

Use the parent development repository's `just verify hunted` commands with
`SPRING_ENGINE_DIR` pointing to the engine to test.

## Startup visibility

The rules enable global LOS for every ally team on the first game frame,
matching Lua's `start_force` widget. The flashlight and lighthouse visibility
pass still decides which rabbits are drawn. Omitting global LOS made the engine
recompute visibility unnecessarily, including expensive terrain raycasts.

The existing integration suite now has 36 checks, including the startup LOS
assertion. That assertion fails before the fix and passes afterwards.

## AI search

Static food, mine and burrow snapshots are shared until their roster revision
changes. Food queries use spatial buckets, followed by the original strict
radius, reservation and desirability checks. Equal scores retain the original
carrot order, including across bucket boundaries. Dynamic lights and blasts
are still gathered each update; eating callbacks retain their own snapshot.
The common tests cover negative coordinates, cell boundaries and empty queries.

Release builds use whole-program LTO and one code generation unit. The
workspace profile applies to both installed modules and verification builds.

## Optional Rust jobs

The default `rust_jobs=off` keeps the serial path. Set the game modoption
`rust_jobs=on` to calculate rabbit light sensing in same-tick jobs.
`rust_jobs=check` also compares worker output with inline computation; it is
for verification, not timing. RNG, engine calls and result application keep
their original order. A dedicated worker module has an 8 MiB fixed memory.

For measurements, use the springcabal-dev performance runner with
`--rust-jobs off|on|check --worker-threads 1|4`; compare jobs at the same
engine pool width.
