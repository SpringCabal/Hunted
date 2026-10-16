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
        let global = ctx.global();
        if frame % status::RESOURCE_PERIOD_FRAMES == 0 {
            spring::commands::run(|| {
                crate::with_global(|global| {
                    let mut score = global.score.access_mut();
                    score.score += score.carrots;
                    drop(score);
                    global.status_changed();
                });
            });
        }
        if global.session.access().status_dirty {
            status::send(global, frame);
        }
    }

    fn unit_destroyed(&self, ctx: &AddonContext<'_, Global>, event: &UnitDestroyedEvent) {
        if super::restarting(ctx.global())
            || defs::kind_of(event.unit_def_id) != Some(UnitKind::Rabbit)
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
