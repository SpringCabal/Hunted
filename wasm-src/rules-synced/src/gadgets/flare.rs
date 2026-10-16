use spring_addons::rules::Gadget;

use crate::state::{self, Global};

const SHOTGUN_AMMO_BONUS: f32 = 20.0;
const MINE_AMMO_BONUS: f32 = 5.0;
const REQUIRED_KILLS_STEP: i32 = 10;

pub struct FlareGadget;

impl Gadget<Global> for FlareGadget {
    fn name(&self) -> &'static str {
        "Flare"
    }

    fn game_frame(&self, ctx: &spring_addons::AddonContext<'_, Global>, frame: i32) {
        let global = ctx.global();
        let rabbits_killed = global.score.access().rabbits_killed;
        let mut flare = global.flare.access_mut();
        if flare.active {
            if frame >= flare.active_until {
                flare.active = false;
                flare.previous_kills = rabbits_killed;
                flare.required_kills += REQUIRED_KILLS_STEP;
                drop(flare);
                global.status_changed();
            }
            return;
        }
        if rabbits_killed - flare.previous_kills >= flare.required_kills {
            flare.active = true;
            flare.active_until = frame + state::FLARE_DURATION_FRAMES;
            let mut weapons = global.weapons.access_mut();
            weapons.shotgun_ammo = (weapons.shotgun_ammo + SHOTGUN_AMMO_BONUS).min(state::MAX_AMMO);
            weapons.mine_ammo = (weapons.mine_ammo + MINE_AMMO_BONUS).min(state::MAX_AMMO);
            drop((flare, weapons));
            global.status_changed();
        }
    }
}
