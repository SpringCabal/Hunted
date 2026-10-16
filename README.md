# Hunted (Zeus vs Bunnies), WASM

The Rust/WASM port of Hunted. Game assets and definitions retain their original formats.

For the engine, launcher, builds and verification, use the [SpringCabal development repository](https://github.com/gajop/springcabal-dev).

## Modules

| Module | Source | Purpose |
| --- | --- | --- |
| `hunted-rules` | `wasm-src/rules-synced` | Game rules; unit scripts run in Lua. |
| `hunted-ui` | `wasm-src/ui` | HUD, controls and rendering. |

## Differences from the Lua version

The HUD uses RmlUi instead of Chili. Restart resets the round in place, and the end screen also accepts R to restart and Q to quit.
