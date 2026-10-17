use spring_addons::AddonContext;
use spring_addons::rules::{Gadget, UnitDestroyedEvent};

use crate::cus;
use crate::defs;
use crate::state::Global;

pub struct RosterGadget;

impl Gadget<Global> for RosterGadget {
    fn name(&self) -> &'static str {
        "Roster"
    }

    fn init(&self, ctx: &AddonContext<'_, Global>) {
        let spring = ctx.spring();
        if let Ok(units) = spring.units().get_all_units() {
            for unit in units {
                if let Ok(def) = spring.units_info().get_unit_def_id(unit) {
                    track(spring, ctx.global(), unit, def);
                }
            }
        }
        cus::attach_existing(spring);
    }

    fn game_frame(&self, _ctx: &AddonContext<'_, Global>, frame: i32) {
        let spring = _ctx.spring();
        cus::game_frame(spring, frame);
    }

    fn unit_created(
        &self,
        ctx: &AddonContext<'_, Global>,
        unit: i32,
        def: i32,
        _team: i32,
        _builder: i32,
    ) {
        let spring = ctx.spring();
        track(spring, ctx.global(), unit, def);
        cus::attach(spring, unit, def);
    }

    fn unit_destroyed(&self, ctx: &AddonContext<'_, Global>, event: &UnitDestroyedEvent) {
        ctx.global().roster.access_mut().untrack(event.unit_id);
    }
}

fn track(spring: &spring::Spring<'_, spring::RulesSynced>, global: &Global, unit: i32, def: i32) {
    let Some(kind) = defs::kind_of(spring, def) else {
        return;
    };
    let position = spring
        .units_info()
        .get_unit_position(unit, Default::default())
        .unwrap_or_default();
    global.roster.access_mut().track(unit, kind, position);
}
