use spring::ResultExt;
use spring_addons::AddonContext;
use spring_addons::rules::Gadget;

use crate::defs::{self, UnitKind};
use crate::state::Global;

pub struct BlockingGadget;

impl Gadget<Global> for BlockingGadget {
    fn name(&self) -> &'static str {
        "Blocking"
    }

    fn unit_created(
        &self,
        _ctx: &AddonContext<'_, Global>,
        unit: i32,
        def: i32,
        _team: i32,
        _builder: i32,
    ) {
        let spring = _ctx.spring();
        if matches!(
            defs::kind_of(spring, def),
            Some(UnitKind::Burrow | UnitKind::CarrotCarried)
        ) {
            make_non_blocking(spring, unit);
        }
    }
}

fn make_non_blocking(spring: &spring::Spring<'_, spring::RulesSynced>, unit: i32) {
    let Some(state) = spring
        .units_info()
        .get_unit_blocking(unit)
        .log_err_once(spring, "unit blocking")
    else {
        return;
    };
    spring
        .unit_control()
        .set_unit_blocking(
            unit,
            spring::unit_control::SetUnitBlockingOptions {
                blocking: false,
                solid_objects: false,
                projectiles: state.is_projectile_collidable,
                quad_map_rays: state.is_ray_segment_collidable,
                crushable: state.crushable,
                block_enemy_pushing: state.block_enemy_pushing,
                block_height_changes: state.block_height_changes,
            },
        )
        .log_err_once(spring, "making a burrow or carried carrot non-blocking");
}
