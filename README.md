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

The existing verification framework passes 43 common Rust tests, 38 gameplay
assertions and eight image/crop comparisons on NVIDIA hardware. Lua/Rust
recordings compare the complete movement state, orders, health and projectiles.

Use the parent development repository's `just verify hunted` commands with
`SPRING_ENGINE_DIR` pointing to the engine to test.

## Startup visibility

The rules enable global LOS for every ally team on the first game frame,
matching Lua's `start_force` widget. The flashlight and lighthouse visibility
pass still decides which rabbits are drawn. Omitting global LOS made the engine
recompute visibility unnecessarily, including expensive terrain raycasts.

The startup LOS assertion remains part of the integration suite.

## AI search

Static food, mine and burrow snapshots are shared until their roster revision
changes. Food queries use spatial buckets, followed by the original strict
radius, reservation and desirability checks. Equal scores retain the original
carrot order, including across bucket boundaries. Dynamic lights and blasts
are still gathered each update. Food destruction refreshes the snapshot before
the next rabbit is processed.
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

## Matching Lua behavior

The shared `springcabal-math` crate uses the pinned engine's float32 math for
spawning and steering; building it requires Clang and its LLVM archiver.
AI deadlines and wave growth preserve Lua's float rounding. Reservations,
pickup, deposit and dropping complete in the same order as Lua, including
carried-carrot speed and momentum. Burrows retain their creation positions.
Equal rounded desirability uses creation/swap-removal order.

Idle animations and chewing sounds use independent per-unit random streams in
both ports. Coroutine wake order therefore cannot change gameplay randomness.
