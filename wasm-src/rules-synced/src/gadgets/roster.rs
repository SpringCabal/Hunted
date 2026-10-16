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
        if let Ok(units) = spring::get_all_units() {
            for unit in units {
                if let Ok(def) = spring::get_unit_def_id(unit) {
                    track(ctx.global(), unit, def.0);
                }
            }
        }
        cus::attach_existing();
    }

    fn game_frame(&self, _ctx: &AddonContext<'_, Global>, frame: i32) {
        cus::game_frame(frame);
    }

    fn unit_created(
        &self,
        ctx: &AddonContext<'_, Global>,
        unit: i32,
        def: i32,
        _team: i32,
        _builder: i32,
    ) {
        track(ctx.global(), unit, def);
        cus::attach(unit, def);
    }

    fn unit_destroyed(&self, ctx: &AddonContext<'_, Global>, event: &UnitDestroyedEvent) {
        ctx.global().roster.access_mut().untrack(event.unit_id);
    }
}

fn track(global: &Global, unit: i32, def: i32) {
    let Some(kind) = defs::kind_of(def) else {
        return;
    };
    let position = spring::get_unit_position(unit, false, false).unwrap_or_default();
    global.roster.access_mut().track(unit, kind, position);
}
