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
        if matches!(
            defs::kind_of(def),
            Some(UnitKind::Burrow | UnitKind::CarrotCarried)
        ) {
            make_non_blocking(unit);
        }
    }
}

fn make_non_blocking(unit: i32) {
    let Some(state) = spring::units_info::get_unit_blocking(unit).log_err_once("unit blocking")
    else {
        return;
    };
    spring::set_unit_blocking(
        unit,
        spring::generated::owned::unit_control::SetUnitBlockingOptions {
            blocking: false,
            solid_objects: false,
            projectiles: state.is_projectile_collidable,
            quad_map_rays: state.is_ray_segment_collidable,
            crushable: state.crushable,
            block_enemy_pushing: state.block_enemy_pushing,
            block_height_changes: state.block_height_changes,
        },
    )
    .log_err_once("making a burrow or carried carrot non-blocking");
}
