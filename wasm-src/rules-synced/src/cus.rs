mod carrot;
mod common;
mod lighthouse;
mod mine;
mod rabbit;
mod script;
mod shotgun;

use spring::cus::host::CusHost;
use spring::{Result, RulesSynced, Spring, UnitId};

use crate::unit_scripts::Call;
pub use script::HuntedUnitScript;

pub type HuntedCusModule = CusHost<HuntedUnitScript>;

fn with_host<R>(
    spring: &Spring<'_, RulesSynced>,
    what: &str,
    f: impl FnOnce(&HuntedCusModule, &Spring<'_, RulesSynced>) -> Result<R>,
) -> Option<R> {
    match crate::with_cus_module(|host, spring| f(host, spring)) {
        Ok(result) => Some(result),
        Err(error) => {
            spring::log_warning!(spring, crate::diagnostics::LOG_SECTION, "{what}: {error:?}");
            None
        }
    }
}

pub fn attach(spring: &Spring<'_, RulesSynced>, unit: i32, def: i32) {
    with_host(spring, "attaching a unit script", |host, spring| {
        host.unit_created(spring, UnitId(unit), def)
    });
}

pub fn attach_existing(spring: &Spring<'_, RulesSynced>) {
    with_host(spring, "attaching existing unit scripts", |host, spring| {
        host.attach_existing(spring)
    });
}

pub fn game_frame(spring: &Spring<'_, RulesSynced>, frame: i32) {
    with_host(spring, "running unit scripts", |host, _| {
        host.game_frame(frame);
        Ok(())
    });
}

pub fn call(spring: &Spring<'_, RulesSynced>, call: Call) {
    with_host(spring, call.function, |host, spring| {
        if host
            .call(
                spring,
                UnitId(call.unit),
                call.function,
                call.arguments(),
                &mut [],
            )?
            .is_none()
        {
            spring::log_warning!(
                spring,
                crate::diagnostics::LOG_SECTION,
                "{} for unit {}: the unit has no script with that function",
                call.function,
                call.unit
            );
        }
        Ok(())
    });
}
