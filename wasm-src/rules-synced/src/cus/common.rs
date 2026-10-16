use spring::UnitId;
use spring::cus::{Angle, AngularSpeed, Axis, InitCtx, Piece, UnitCtx};

pub const PUBLIC: i32 = spring::rules_params::los::PUBLIC;
pub const TAU: f32 = core::f32::consts::TAU;

pub use spring::random::synced as random;

pub fn piece(ctx: &InitCtx<'_>, name: &str) -> Piece {
    let direct = ctx.piece(name);
    if direct != Piece::INVALID {
        return direct;
    }
    match name {
        "FLeg1" => ctx.piece("AFLeg1"),
        "FLeg2" => ctx.piece("AFLeg2"),
        _ => Piece::INVALID,
    }
}

pub fn turn(ctx: &UnitCtx, piece: Piece, axis: Axis, target: f32, speed: f32) {
    if piece == Piece::INVALID {
        return;
    }
    if speed == 0.0 {
        ctx.turn_now(piece, axis, Angle::radians(target));
    } else {
        ctx.turn(
            piece,
            axis,
            Angle::radians(target),
            AngularSpeed::radians_per_second(speed),
        );
    }
}

pub fn move_piece(ctx: &UnitCtx, piece: Piece, axis: Axis, target: f32, speed: f32) {
    if piece == Piece::INVALID {
        return;
    }
    if speed == 0.0 {
        ctx.move_now(piece, axis, target);
    } else {
        ctx.move_piece(piece, axis, target, speed);
    }
}

pub fn unit_position(unit: UnitId) -> spring::Float3 {
    spring::get_unit_position(unit, false, false).unwrap_or_default()
}

pub fn destroy(unit: UnitId) {
    crate::units::destroy(unit.0, false, -1);
}

pub fn play_sound_at(name: &str, volume: f32, position: spring::Float3) {
    let _ = spring::sound::play_sound_file(name, volume, position, spring::Float3::default(), 0);
}
