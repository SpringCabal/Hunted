use alloc::vec::Vec;

use spring_addons::AddonContext;
use spring_addons::rules::Gadget;

use crate::defs::UnitKind;
use crate::state::Global;
use crate::units;

const LOSE_FRAME: i32 = 150;

pub struct E2eGadget;

impl Gadget<Global> for E2eGadget {
    fn name(&self) -> &'static str {
        "E2e"
    }

    fn game_frame(&self, _ctx: &AddonContext<'_, Global>, frame: i32) {
        if frame != LOSE_FRAME {
            return;
        }
        spring::commands::run(|| {
            crate::with_global(|global| {
                let carrots: Vec<i32> = {
                    let roster = global.roster.access();
                    [
                        UnitKind::Carrot,
                        UnitKind::CarrotCarried,
                        UnitKind::CarrotDropped,
                    ]
                    .into_iter()
                    .flat_map(|kind| roster.units(kind))
                    .collect()
                };
                for carrot in carrots {
                    units::destroy_now(carrot, false, -1);
                }
            });
        });
    }
}
