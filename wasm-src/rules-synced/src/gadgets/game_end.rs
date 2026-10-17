use spring_addons::rules::Gadget;

use crate::state::{self, Global};

pub struct GameEndGadget;

impl Gadget<Global> for GameEndGadget {
    fn name(&self) -> &'static str {
        "GameEnd"
    }

    fn game_frame(&self, _ctx: &spring_addons::AddonContext<'_, Global>, _frame: i32) {
        crate::with_addon_context(|ctx| {
            ctx.commands()
                .run(|_spring| crate::with_global(count_carrots))
        });
    }
}

fn count_carrots(global: &Global) {
    let carrots = global.roster.access().carrot_count();
    global.score.access_mut().carrots = carrots;
    let lost = global.waves.access().spawn_index >= state::NON_RABBIT_COUNT && carrots == 0;
    if lost && !global.session.access().game_over {
        global.session.access_mut().game_over = true;
        global.status_changed();
    }
}
