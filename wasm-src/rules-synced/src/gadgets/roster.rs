use core::cell::Cell;

use spring::ResultExt;
use spring_addons::AddonContext;
use spring_addons::rules::{Gadget, UnitDestroyedEvent};

use crate::cus;
use crate::defs;
use crate::state::Global;

#[derive(Default)]
pub struct RosterGadget {
    global_los_enabled: Cell<bool>,
}

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

    fn game_frame(&self, ctx: &AddonContext<'_, Global>, frame: i32) {
        let spring = ctx.spring();
        // Lua's start_force widget enables global LOS for every ally team,
        // including Gaia. Rabbit visibility is controlled by the flashlight
        // widget, so native LOS updates only add work. Wait until game startup
        // is complete before using the synced control API.
        if !self.global_los_enabled.get() {
            self.global_los_enabled.set(enable_global_los(spring));
        }
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

fn enable_global_los(spring: &spring::Spring<'_, spring::RulesSynced>) -> bool {
    let Some(teams) = spring
        .teams()
        .get_ally_team_list()
        .log_err(spring, "list ally teams for global LOS")
    else {
        return false;
    };
    let mut enabled = !teams.is_empty();
    for team in teams {
        let result = spring
            .team_control()
            .set_global_los(team, true)
            .log_err(spring, "enable global LOS");
        if result == Some(false) {
            spring::log_warning!(
                spring,
                "hunted-rules",
                "global LOS rejected for ally team {team}"
            );
        }
        enabled &= result == Some(true);
    }
    enabled
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
