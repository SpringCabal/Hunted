use alloc::rc::Rc;
use core::cell::RefCell;
use spring::ResultExt;

use spring::cus::{Axis, Piece, SignalMask, TaskDefinition, UnitCtx};

use super::common::*;

const SIG_STEAL: SignalMask = SignalMask::new(1);

pub struct CarrotState {
    pub(super) random: hunted_common::cosmetic::Random,
    pub(super) base: Piece,
    pub(super) x: f32,
    pub(super) y: f32,
    pub(super) z: f32,
}

pub struct CarriedState {
    pub(super) base: Piece,
}

pub(super) fn start_being_stolen(ctx: &UnitCtx, state: Rc<RefCell<CarrotState>>, progress: f32) {
    ctx.signal(SIG_STEAL);
    let start = -10.0 + 6.5 * progress / 30.0;
    let base = state.borrow().base;
    ctx.spawn(TaskDefinition::with_state(
        "hunted-carrot-steal",
        state,
        stealing,
    ));
    move_piece(ctx, base, Axis::Y, start, 0.0);
    move_piece(ctx, base, Axis::Y, 20.0, 6.5);
}

pub(super) fn stop_being_stolen(ctx: &UnitCtx, base: Piece, progress: f32) {
    ctx.signal(SIG_STEAL);
    let target = -10.0 + 6.5 * progress / 30.0;
    move_piece(ctx, base, Axis::Y, target, 0.0);
    move_piece(ctx, base, Axis::Y, target, 0.0001);
}

async fn stealing(state: Rc<RefCell<CarrotState>>, ctx: UnitCtx) {
    ctx.set_signal_mask(SIG_STEAL);
    ctx.spawn(TaskDefinition::with_state(
        "hunted-carrot-sound",
        Rc::clone(&state),
        sound,
    ));
    loop {
        let position = {
            let state = state.borrow();
            spring::Float3::new(state.x, state.y, state.z)
        };
        let _ = ctx.with_spring(|spring| {
            spring.effects_control().spawn_ceg(
                &spring::effects_control::DefRef {
                    name: ("dirtfling").to_owned(),
                    id: -1,
                },
                position,
                spring::Float3::new(0.0, 1.0, 0.0),
                0.0,
                0.0,
                0.0,
            )
        });
        ctx.sleep(spring::cus::Duration::from_millis(500)).await;
    }
}

async fn sound(state: Rc<RefCell<CarrotState>>, ctx: UnitCtx) {
    ctx.set_signal_mask(SIG_STEAL);
    loop {
        let play_now = state.borrow_mut().random.float() < 0.7;
        if play_now {
            let position = {
                let state = state.borrow();
                spring::Float3::new(state.x, state.y, state.z)
            };
            play_sound_at(&ctx, "sounds/digitout.wav", 4.0, position);
        }
        ctx.sleep(spring::cus::Duration::from_millis(600)).await;
    }
}

pub(super) async fn consume(ctx: UnitCtx) {
    ctx.sleep(spring::cus::Duration::from_millis(1)).await;
    destroy(&ctx);
}

pub(super) fn be_dropped(ctx: &UnitCtx, state: Rc<RefCell<CarriedState>>) {
    let base = state.borrow().base;
    move_piece(ctx, base, Axis::Y, -7.0, 29.0);
    turn(
        ctx,
        base,
        Axis::X,
        core::f32::consts::FRAC_PI_2,
        core::f32::consts::PI * 0.2,
    );
    ctx.spawn(TaskDefinition::with_state(
        "hunted-carried-drop",
        state,
        dropped,
    ));
}

async fn dropped(state: Rc<RefCell<CarriedState>>, ctx: UnitCtx) {
    ctx.set_signal_mask(SIG_STEAL);
    ctx.sleep(spring::cus::Duration::from_millis(500)).await;
    let base = state.borrow().base;
    move_piece(&ctx, base, Axis::Y, -100.0, 0.0);
    let position = unit_position(&ctx);
    let team = ctx
        .with_spring(|spring| spring.units_info().get_unit_team(ctx.unit().0))
        .unwrap_or(0);
    let carried = ctx.unit().0;
    ctx.with_spring(|spring| {
        Ok(crate::units::create(
            spring,
            crate::defs::UnitKind::CarrotDropped,
            position.x,
            position.z,
            team,
        ))
    })
    .expect("active unit-script scope")
    .then(move |spring, carrot| {
        if let Some(carrot) = carrot
            && let Ok(direction) = spring.units_info().get_unit_direction(carried)
        {
            let right = spring::Float3::new(-direction.z, 0.0, direction.x);
            let _ = spring
                .unit_control()
                .set_unit_direction(carrot, direction, right);
        }
        spring
            .rules_params()
            .set_unit_rules_param_f32(
                carried,
                "internalDestroy",
                1.0,
                spring::rules_params::los::PUBLIC,
            )
            .log_err_once(spring, "marking a dropped carried carrot");
    });
    destroy(&ctx);
}

pub(super) async fn score(ctx: UnitCtx) {
    ctx.sleep(spring::cus::Duration::from_millis(1)).await;
    destroy(&ctx);
}
