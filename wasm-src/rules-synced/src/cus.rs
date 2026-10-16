mod carrot;
mod common;
mod lighthouse;
mod mine;
mod rabbit;
mod script;
mod shotgun;

use spring::UnitId;
use spring::cus::host::CusHost;

use crate::unit_scripts::Call;
pub use script::HuntedUnitScript;

pub type HuntedCusModule = CusHost<HuntedUnitScript>;

fn with_host<R>(what: &str, f: impl FnOnce(&mut HuntedCusModule) -> R) -> Option<R> {
    let result = crate::with_cus_module(f);
    if result.is_none() {
        spring::log_once!(
            spring::log_level::WARNING,
            crate::diagnostics::LOG_SECTION,
            "{what}: the unit scripts are busy or not initialized"
        );
    }
    result
}

pub fn attach(unit: i32, def: i32) {
    with_host("attaching a unit script", |host| {
        host.unit_created(UnitId(unit), def)
    });
}

pub fn attach_existing() {
    with_host("attaching the existing units' scripts", |host| {
        host.attach_existing()
    });
}

pub fn game_frame(frame: i32) {
    with_host("running the unit scripts", |host| host.game_frame(frame));
}

pub fn call(call: Call) {
    with_host(call.function, |host| {
        if host
            .call(UnitId(call.unit), call.function, call.arguments(), &mut [])
            .is_none()
        {
            spring::log_warning!(
                crate::diagnostics::LOG_SECTION,
                "{} for unit {}: the unit has no script with that function",
                call.function,
                call.unit
            );
        }
    });
}
