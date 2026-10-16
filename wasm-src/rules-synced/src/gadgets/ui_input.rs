use alloc::vec::Vec;

use hunted_common::protocol;
use spring::commands;
use spring_addons::AddonContext;
use spring_addons::rules::Gadget;
use springcabal::message;

use crate::defs::UnitKind;
use crate::state::{Global, PendingAction};
use crate::{state, status, units};

pub struct UiInputGadget;

impl Gadget<Global> for UiInputGadget {
    fn name(&self) -> &'static str {
        "UiInput"
    }

    fn handle_lua_msg(
        &self,
        ctx: &AddonContext<'_, Global>,
        _player_id: i32,
        _script: i32,
        _mode: i32,
        data: &[u8],
    ) {
        receive(ctx, data);
    }
}

pub fn receive(ctx: &AddonContext<'_, Global>, data: &[u8]) -> bool {
    let Ok(text) = core::str::from_utf8(data) else {
        return false;
    };
    if text == protocol::RESTART {
        let accepted = request_restart(ctx.global());
        if !accepted {
            crate::diagnostics::info(
                "restart ignored: the hunt isn't over, or is already restarting",
            );
        }
        return accepted;
    }
    let Some((command, mut fields)) = message::command(text) else {
        return false;
    };
    let action = match command {
        protocol::MOVE_GUN => None,
        protocol::FIRE => Some(PendingAction::Shotgun),
        protocol::PLACE_MINE => Some(PendingAction::Mine),
        _ => return false,
    };
    let Some((x, y, z)) = fields.xyz() else {
        return false;
    };
    if !fields.is_done() {
        return false;
    }
    let mut weapons = ctx.global().weapons.access_mut();
    weapons.target = state::Target {
        x: crate::map::clamp_to_map(x),
        y,
        z: crate::map::clamp_to_map(z),
        valid: true,
    };
    if let Some(action) = action {
        weapons.pending_action = Some(action);
    }
    true
}

fn request_restart(global: &state::Global) -> bool {
    {
        let mut session = global.session.access_mut();
        if !session.game_over || session.restart_in_progress {
            return false;
        }
        session.restart_in_progress = true;
    }
    commands::run(|| {
        crate::with_global(|global| {
            let units: Vec<i32> = {
                let roster = global.roster.access();
                UnitKind::ALL
                    .into_iter()
                    .filter(|kind| *kind != UnitKind::Shotgun)
                    .flat_map(|kind| roster.units(kind))
                    .collect()
            };
            for unit_id in units {
                units::destroy_now(unit_id, false, -1);
            }
            reset_round(global, spring::game::frame().unwrap_or(0) as i32);
        });
    });
    true
}

fn reset_round(global: &state::Global, frame: i32) {
    let (gun_id, target) = {
        let weapons = global.weapons.access();
        (weapons.gun_id, weapons.target)
    };
    let session = state::Session {
        round_start_frame: frame,
        ..state::Session::default()
    };
    *global.session.access_mut() = session;
    *global.score.access_mut() = state::Score::default();
    *global.weapons.access_mut() = state::Weapons {
        gun_id,
        target,
        ..state::Weapons::default()
    };
    *global.scares.access_mut() = state::Scares::default();
    *global.flare.access_mut() = state::Flare::default();
    *global.rabbits.access_mut() = crate::gadgets::rabbit_ai::Rabbits::default();
    {
        let mut waves = global.waves.access_mut();
        *waves = state::Waves::default();
        waves.next_wave_frame = frame + hunted_common::waves::FIRST_WAVE_FRAME;
    }
    status::send(global, frame);
    crate::diagnostics::info("hunt restarted");
}
