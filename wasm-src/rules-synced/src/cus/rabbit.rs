use alloc::rc::Rc;
use core::cell::RefCell;

use spring::cus::{
    Axis, InitCtx, Piece, SfxFlags, SignalMask, TaskDefinition, UnitCtx, WreckLevel,
};

use super::common::*;

const SIG_MOVE: SignalMask = SignalMask::new(2);

#[derive(Clone, Copy)]
pub struct RabbitPieces {
    pub(super) center: Piece,
    pub(super) head: Piece,
    pub(super) wiggle_tail: Piece,
    pub(super) body: Piece,
    pub(super) bleg1: Piece,
    pub(super) bleg2: Piece,
    pub(super) fleg1: Piece,
    pub(super) fleg2: Piece,
}

impl RabbitPieces {
    pub fn from_init(ctx: &InitCtx<'_>) -> Self {
        Self {
            center: piece(ctx, "center"),
            head: piece(ctx, "Head"),
            wiggle_tail: piece(ctx, "WiggleTail"),
            body: piece(ctx, "Body"),
            bleg1: piece(ctx, "BLeg1"),
            bleg2: piece(ctx, "BLeg2"),
            fleg1: piece(ctx, "FLeg1"),
            fleg2: piece(ctx, "FLeg2"),
        }
    }
}

pub struct RabbitState {
    pub(super) random: hunted_common::cosmetic::Random,
    pub(super) pieces: RabbitPieces,
    pub(super) moving: Option<bool>,
}

pub(super) async fn controller(state: Rc<RefCell<RabbitState>>, ctx: UnitCtx) {
    loop {
        let pieces = state.borrow().pieces;
        if state.borrow().moving == Some(true) {
            move_animation(&ctx, pieces).await;
        }
        if state.borrow().moving == Some(false) {
            reset_pose(&ctx, pieces, 17.0);
            let idle_now = state.borrow_mut().random.int(0, 1) == 1;
            if idle_now {
                idle(&state, &ctx, pieces).await;
            }
        }
        ctx.sleep(spring::cus::Duration::from_millis(100)).await;
    }
}

async fn move_animation(ctx: &UnitCtx, p: RabbitPieces) {
    move_piece(ctx, p.center, Axis::Y, 5.0, 15.0);
    turn(ctx, p.body, Axis::X, -14.0f32.to_radians(), 7.0);
    turn(ctx, p.bleg1, Axis::X, 47.0f32.to_radians(), 17.0);
    turn(ctx, p.bleg2, Axis::X, 51.0f32.to_radians(), 17.0);
    turn(ctx, p.fleg1, Axis::X, 44.0f32.to_radians(), 7.0);
    turn(ctx, p.fleg2, Axis::X, 44.0f32.to_radians(), 17.0);
    turn(ctx, p.head, Axis::X, -27.0f32.to_radians(), 7.0);
    ctx.sleep(spring::cus::Duration::from_millis(700)).await;
    ctx.wait_for_move(p.center, Axis::Y).await;

    move_piece(ctx, p.center, Axis::Y, 0.0, 15.0);
    turn(ctx, p.body, Axis::X, -14.0f32.to_radians(), 7.0);
    turn(ctx, p.bleg1, Axis::X, 9.0f32.to_radians(), 7.0);
    turn(ctx, p.bleg2, Axis::X, 10.0f32.to_radians(), 7.0);
    turn(ctx, p.fleg1, Axis::X, 10.0f32.to_radians(), 7.0);
    turn(ctx, p.fleg2, Axis::X, 10.0f32.to_radians(), 7.0);
    turn(ctx, p.head, Axis::X, 17.0f32.to_radians(), 7.0);
    ctx.sleep(spring::cus::Duration::from_millis(300)).await;
    ctx.wait_for_move(p.center, Axis::Y).await;
}

async fn idle(state: &Rc<RefCell<RabbitState>>, ctx: &UnitCtx, p: RabbitPieces) {
    let count = state.borrow_mut().random.int(1, 6);
    for _ in 0..count {
        let degrees = state.borrow_mut().random.int(10, 35) as f32;
        let tail_degrees = state.borrow_mut().random.int(-10, 10) as f32;
        turn(ctx, p.head, Axis::Y, degrees.to_radians(), 9.0);
        turn(ctx, p.wiggle_tail, Axis::X, tail_degrees.to_radians(), 8.0);
        ctx.wait_for_turn(p.head, Axis::Y).await;

        let tail_degrees = state.borrow_mut().random.int(-10, 10) as f32;
        turn(ctx, p.head, Axis::Y, -degrees.to_radians(), 9.0);
        turn(ctx, p.wiggle_tail, Axis::X, tail_degrees.to_radians(), 8.0);
        ctx.wait_for_turn(p.head, Axis::Y).await;
    }
    reset_pose(ctx, p, 0.0);
}

fn reset_pose(ctx: &UnitCtx, p: RabbitPieces, speed: f32) {
    for piece in [
        p.center,
        p.head,
        p.wiggle_tail,
        p.body,
        p.bleg1,
        p.bleg2,
        p.fleg1,
        p.fleg2,
    ] {
        for axis in [Axis::X, Axis::Y, Axis::Z] {
            turn(ctx, piece, axis, 0.0, speed);
        }
        for axis in [Axis::X, Axis::Y, Axis::Z] {
            move_piece(ctx, piece, axis, 0.0, speed);
        }
    }
}

pub(super) fn start_moving(ctx: &UnitCtx, state: &Rc<RefCell<RabbitState>>) {
    state.borrow_mut().moving = Some(true);
    ctx.signal(SIG_MOVE);
}

pub(super) fn stop_moving(ctx: &UnitCtx, state: &Rc<RefCell<RabbitState>>) {
    ctx.signal(SIG_MOVE);
    ctx.spawn(TaskDefinition::with_state(
        "hunted-rabbit-stop",
        Rc::clone(state),
        stop_moving_task,
    ));
}

pub(super) fn killed(ctx: &UnitCtx, state: &Rc<RefCell<RabbitState>>) -> WreckLevel {
    let p = state.borrow().pieces;
    let script = ctx.clone();
    crate::with_addon_context(|addon| {
        addon.commands().run(move |_spring| {
            for piece in [p.body, p.bleg1, p.bleg2, p.fleg1, p.fleg2] {
                if piece != Piece::INVALID {
                    script.explode(piece, SfxFlags::EXPLODE);
                }
            }
        })
    });
    let position = ctx
        .with_spring(|spring| {
            spring
                .units_info()
                .get_unit_position(ctx.unit().0, Default::default())
        })
        .unwrap_or_default();
    play_sound_at(ctx, "sounds/splat1.wav", 100.0, position);
    WreckLevel::One
}

async fn stop_moving_task(state: Rc<RefCell<RabbitState>>, ctx: UnitCtx) {
    ctx.set_signal_mask(SIG_MOVE);
    ctx.sleep(spring::cus::Duration::from_millis(1500)).await;
    state.borrow_mut().moving = Some(false);
}
