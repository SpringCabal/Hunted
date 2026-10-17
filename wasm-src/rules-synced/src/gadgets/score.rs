use spring_addons::AddonContext;
use spring_addons::rules::{Gadget, UnitDestroyedEvent};

use crate::defs::{self, UnitKind};
use crate::state::Global;
use crate::status;

const KILL_SCORE: i32 = 50;

pub struct ScoreGadget;

impl Gadget<Global> for ScoreGadget {
    fn name(&self) -> &'static str {
        "Score"
    }

    fn game_frame(&self, ctx: &AddonContext<'_, Global>, frame: i32) {
        let spring = ctx.spring();
        let global = ctx.global();
        if frame % status::RESOURCE_PERIOD_FRAMES == 0 {
            crate::with_addon_context(|ctx| {
                ctx.commands().run(|_spring| {
                    crate::with_global(|global| {
                        let mut score = global.score.access_mut();
                        score.score += score.carrots;
                        drop(score);
                        global.status_changed();
                    });
                })
            });
        }
        if global.session.access().status_dirty {
            status::send(spring, global, frame);
        }
    }

    fn unit_destroyed(&self, ctx: &AddonContext<'_, Global>, event: &UnitDestroyedEvent) {
        let spring = ctx.spring();
        if super::restarting(ctx.global())
            || defs::kind_of(spring, event.unit_def_id) != Some(UnitKind::Rabbit)
        {
            return;
        }
        let global = ctx.global();
        {
            let mut score = global.score.access_mut();
            score.rabbits_killed += 1;
            score.score += KILL_SCORE;
        }
        global.status_changed();
    }
}
