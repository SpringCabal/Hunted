pub const LOG_SECTION: &str = "hunted-wasm-ui";

#[cfg(feature = "diagnostics")]
const LOG_PERIOD_FRAMES: i32 = 300;
#[cfg(feature = "diagnostics")]
use alloc::format;
#[cfg(feature = "diagnostics")]
use spring::log_level;

#[cfg(feature = "diagnostics")]
pub fn visibility(frame: i32, rabbits: i32, revealed: i32, failures: i32, flare_active: bool) {
    if frame.rem_euclid(LOG_PERIOD_FRAMES) != 0 {
        return;
    }
    if LAST_VISIBILITY_LOG_FRAME.with(|last| last.replace(frame)) == frame {
        return;
    }
    let _ = spring::log(
        "hunted-wasm-ui",
        log_level::DEBUG,
        &format!(
            "rabbit visibility frame={frame} total={rabbits} revealed={revealed} hidden={} flare_active={flare_active}",
            rabbits - revealed - failures,
        ),
    );
    if failures > 0 {
        let _ = spring::log(
            "hunted-wasm-ui",
            log_level::WARNING,
            &format!("rabbit visibility no-draw failures={failures}"),
        );
    }
}

#[cfg(not(feature = "diagnostics"))]
#[inline(always)]
pub fn visibility(_frame: i32, _rabbits: i32, _revealed: i32, _failures: i32, _flare_active: bool) {
}

#[cfg(feature = "diagnostics")]
thread_local! {
    static LAST_VISIBILITY_LOG_FRAME: core::cell::Cell<i32> = const { core::cell::Cell::new(-1) };
    static MOUSE_TARGET_TICKS: core::cell::Cell<i32> = const { core::cell::Cell::new(0) };
}

#[cfg(feature = "diagnostics")]
pub fn mouse_target(x: i32, y: i32, target: crate::state::Point3) {
    let due = MOUSE_TARGET_TICKS.with(|ticks| {
        let next = ticks.get() - 1;
        if next > 0 {
            ticks.set(next);
            false
        } else {
            ticks.set(60);
            true
        }
    });
    if !due {
        return;
    }
    let _ = spring::log(
        "hunted-wasm-ui",
        log_level::DEBUG,
        &format!(
            "mouse target screen=({x},{y}) world=({:.1},{:.1},{:.1})",
            target.x, target.y, target.z
        ),
    );
}

#[cfg(not(feature = "diagnostics"))]
#[inline(always)]
pub fn mouse_target(_x: i32, _y: i32, _target: crate::state::Point3) {}

#[cfg(feature = "diagnostics")]
pub fn mouse_press(x: i32, y: i32, button: i32) {
    let _ = spring::log(
        "hunted-wasm-ui",
        log_level::DEBUG,
        &format!("mouse press screen=({x},{y}) button={button}"),
    );
}

#[cfg(not(feature = "diagnostics"))]
#[inline(always)]
pub fn mouse_press(_x: i32, _y: i32, _button: i32) {}

#[cfg(feature = "diagnostics")]
pub fn status_received(frame: i32, carrots: i32, ammo: i32, game_over: bool) {
    if frame.rem_euclid(LOG_PERIOD_FRAMES) == 0 {
        let _ = spring::log(
            "hunted-wasm-ui",
            log_level::DEBUG,
            &format!(
                "status received frame={frame} carrots={carrots} ammo={ammo} game_over={game_over}"
            ),
        );
    }
}

#[cfg(not(feature = "diagnostics"))]
#[inline(always)]
pub fn status_received(_frame: i32, _carrots: i32, _ammo: i32, _game_over: bool) {}

#[cfg(feature = "diagnostics")]
pub fn info(message: &str) {
    let _ = spring::log("hunted-wasm-ui", log_level::INFO, message);
}

#[cfg(not(feature = "diagnostics"))]
#[inline(always)]
pub fn info(_message: &str) {}

#[cfg(feature = "diagnostics")]
pub fn debug(message: &str) {
    let _ = spring::log("hunted-wasm-ui", log_level::DEBUG, message);
}

#[cfg(not(feature = "diagnostics"))]
#[inline(always)]
pub fn debug(_message: &str) {}
