use alloc::rc::Rc;
use core::cell::RefCell;

use spring::cus::{Axis, Piece, TaskDefinition, UnitCtx};

use spring::ResultExt;

use hunted_common::lighthouse::{BEAM_DISTANCE, BEAM_SIZE, PARAM_SIZE, PARAM_X, PARAM_Z};

use super::common::*;

#[derive(Clone, Copy)]
pub struct LighthousePieces {
    pub(super) light_head: Piece,
    pub(super) light_pitch: Piece,
}

pub struct LighthouseState {
    pub(super) pieces: LighthousePieces,
    pub(super) angle: f32,
}

pub(super) fn create(ctx: &UnitCtx, state: Rc<RefCell<LighthouseState>>) {
    let angle = state.borrow().angle;
    let p = state.borrow().pieces;
    turn(
        ctx,
        p.light_head,
        Axis::Y,
        -angle + core::f32::consts::FRAC_PI_2,
        0.0,
    );
    let position = unit_position(ctx.unit());
    let x = position.x + BEAM_DISTANCE * angle.cos();
    let z = position.z + BEAM_DISTANCE * angle.sin();
    point_beam(ctx, x, z);
    spring::set_unit_rules_param_f32(ctx.unit(), PARAM_SIZE, BEAM_SIZE, PUBLIC)
        .log_err_once("publishing the lighthouse beam size");
    ctx.spawn(TaskDefinition::with_state(
        "hunted-lighthouse-loop",
        state,
        update,
    ));
}

async fn update(state: Rc<RefCell<LighthouseState>>, ctx: UnitCtx) {
    let rotation_speed = TAU / 250.0;
    loop {
        let (angle, pieces) = {
            let mut state = state.borrow_mut();
            state.angle += rotation_speed;
            if state.angle >= TAU {
                state.angle -= TAU;
            }
            (state.angle, state.pieces)
        };
        let unit = ctx.unit();
        let position = unit_position(unit);
        let x = position.x + BEAM_DISTANCE * angle.cos();
        let z = position.z + BEAM_DISTANCE * angle.sin();
        let y = spring::get_ground_height(x, z).unwrap_or(position.y);
        turn(
            &ctx,
            pieces.light_head,
            Axis::Y,
            -angle,
            rotation_speed * 30.0,
        );
        if let Ok(piece_pos) =
            spring::units_pieces::get_unit_piece_pos_dir(unit.0, pieces.light_pitch.0 + 1)
        {
            let dx = x - position.x;
            let dz = z - position.z;
            let distance = (dx * dx + dz * dz).sqrt();
            let pitch = (piece_pos.position.y - y).atan2(distance);
            turn(
                &ctx,
                pieces.light_pitch,
                Axis::X,
                pitch + core::f32::consts::PI,
                0.0,
            );
        }
        point_beam(&ctx, x, z);
        ctx.sleep(spring::cus::Duration::from_millis(33)).await;
    }
}

fn point_beam(ctx: &UnitCtx, x: f32, z: f32) {
    spring::set_unit_rules_param_f32(ctx.unit(), PARAM_X, x, PUBLIC)
        .log_err_once("publishing the lighthouse beam");
    spring::set_unit_rules_param_f32(ctx.unit(), PARAM_Z, z, PUBLIC)
        .log_err_once("publishing the lighthouse beam");
}
