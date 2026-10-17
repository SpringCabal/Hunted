use spring::cus::{Angle, AngularSpeed, Axis, InitCtx, Piece, UnitCtx};

pub const PUBLIC: i32 = spring::rules_params::los::PUBLIC;
pub const TAU: f32 = core::f32::consts::TAU;

pub mod random {
    use spring::cus::UnitCtx;
    pub fn float(ctx: &UnitCtx) -> f32 {
        ctx.with_spring(|spring| spring.synced_random().next_float())
            .expect("synced random float")
    }
    pub fn int(ctx: &UnitCtx, lower: i32, upper: i32) -> i32 {
        ctx.with_spring(|spring| spring.synced_random().next_int(lower, upper))
            .expect("synced random int")
    }
}

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

pub fn unit_position(ctx: &UnitCtx) -> spring::Float3 {
    ctx.with_spring(|spring| {
        spring
            .units_info()
            .get_unit_position(ctx.unit().0, Default::default())
    })
    .unwrap_or_default()
}

pub fn destroy(ctx: &UnitCtx) {
    ctx.with_spring(|spring| {
        crate::units::destroy(spring, ctx.unit().0, false, -1);
        Ok(())
    })
    .expect("active unit-script scope");
}

pub fn play_sound_at(ctx: &UnitCtx, name: &str, volume: f32, position: spring::Float3) {
    let _ = ctx.with_spring(|spring| {
        spring
            .sound()
            .play_sound_file(name, volume, position, spring::Float3::default(), 0)
    });
}
