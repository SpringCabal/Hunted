use alloc::format;

use hunted_common::protocol;
use hunted_common::protocol::status as key;
use springcabal::message::Encoder;

use crate::defs::UnitKind;
use crate::diagnostics;
use crate::state::{self, Global};

pub fn send(global: &Global, frame: i32) {
    let rabbit_count = global.roster.access().count(UnitKind::Rabbit);
    let score = *global.score.access();
    let flare = *global.flare.access();
    let weapons_ammo = {
        let weapons = global.weapons.access();
        (weapons.shotgun_ammo, weapons.mine_ammo)
    };
    let (game_over, round_start_frame) = {
        let session = global.session.access();
        (session.game_over, session.round_start_frame)
    };
    let elapsed_time = (frame - round_start_frame).max(0) / spring::consts::GAME_SPEED;
    let flash = if flare.active {
        ((flare.active_until - frame).max(0) * 100 / state::FLARE_DURATION_FRAMES).min(100)
    } else {
        ((score.rabbits_killed - flare.previous_kills).max(0) * 100 / flare.required_kills).min(100)
    };
    let message = Encoder::new(protocol::STATUS)
        .kv(key::FRAME, frame)
        .kv(key::RABBITS, rabbit_count)
        .kv(key::KILLS, score.rabbits_killed)
        .kv(key::CARROTS, score.carrots)
        .kv(key::SCORE, score.score)
        .kv(key::TIME, elapsed_time)
        .kv(key::AMMO, weapons_ammo.0.floor() as i32)
        .kv(key::MINES, weapons_ammo.1.floor() as i32)
        .kv(key::FLASH, flash)
        .kv_flag(key::FLARE_ACTIVE, flare.active)
        .kv_flag(key::GAME_OVER, game_over)
        .finish();
    spring::commands::run(move || {
        springcabal::message::send_to_ui(&message);
    });
    global.session.access_mut().status_dirty = false;
    if frame.rem_euclid(diagnostics::LOG_PERIOD_FRAMES) == 0 {
        diagnostics::info(&format!(
            "status sent frame={frame} rabbits={rabbit_count} kills={} carrots={} stolen={} shots={} flare_active={}",
            score.rabbits_killed,
            score.carrots,
            score.carrots_stolen,
            score.shots_fired,
            flare.active,
        ));
    }
}

pub const RESOURCE_PERIOD_FRAMES: i32 = spring::consts::GAME_SPEED;
