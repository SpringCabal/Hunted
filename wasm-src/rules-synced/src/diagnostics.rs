pub const LOG_PERIOD_FRAMES: i32 = 300;

pub const LOG_SECTION: &str = "hunted-wasm";
#[cfg(feature = "diagnostics")]
use alloc::format;
#[cfg(feature = "diagnostics")]
use spring::log_level;

#[cfg(feature = "diagnostics")]
pub fn info(spring: &spring::Spring<'_, spring::RulesSynced>, message: &str) {
    let _ = spring.messages().log(LOG_SECTION, log_level::INFO, message);
}

#[cfg(not(feature = "diagnostics"))]
#[inline(always)]
pub fn info(_spring: &spring::Spring<'_, spring::RulesSynced>, _message: &str) {}

macro_rules! info_once {
    ($spring:expr, $($arg:tt)+) => {{
        #[cfg(feature = "diagnostics")]
        spring::log_once!($spring, spring::log_level::INFO, $crate::diagnostics::LOG_SECTION, $($arg)+);
    }};
}
pub(crate) use info_once;

macro_rules! debug_once {
    ($spring:expr, $($arg:tt)+) => {{
        #[cfg(feature = "diagnostics")]
        spring::log_once!($spring, spring::log_level::DEBUG, $crate::diagnostics::LOG_SECTION, $($arg)+);
    }};
}
pub(crate) use debug_once;

#[cfg(feature = "diagnostics")]
pub fn debug(spring: &spring::Spring<'_, spring::RulesSynced>, message: &str) {
    let _ = spring
        .messages()
        .log(LOG_SECTION, log_level::DEBUG, message);
}

#[cfg(not(feature = "diagnostics"))]
#[inline(always)]
pub fn debug(_spring: &spring::Spring<'_, spring::RulesSynced>, _message: &str) {}

#[cfg(feature = "diagnostics")]
pub fn rabbit_motion(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    frame: i32,
    rabbits: &[i32],
) {
    if frame.rem_euclid(LOG_PERIOD_FRAMES) != 0 {
        return;
    }
    let mut moving = 0;
    let mut fastest = 0.0_f32;
    for rabbit in rabbits {
        let Ok(velocity) = spring.units_info().get_unit_velocity(*rabbit) else {
            continue;
        };
        let speed = (velocity.x * velocity.x + velocity.z * velocity.z).sqrt();
        if speed > 0.5 {
            moving += 1;
        }
        fastest = fastest.max(speed);
    }
    info(
        spring,
        &format!(
            "rabbit motion frame={frame} total={} moving={moving} fastest={fastest:.2}",
            rabbits.len(),
        ),
    );
}

#[cfg(not(feature = "diagnostics"))]
#[inline(always)]
pub fn rabbit_motion(
    _spring: &spring::Spring<'_, spring::RulesSynced>,
    _frame: i32,
    _rabbits: &[i32],
) {
}
