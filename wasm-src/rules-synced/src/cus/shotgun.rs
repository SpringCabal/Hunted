use alloc::rc::Rc;
use core::cell::RefCell;

use spring::cus::{Axis, Piece, SfxFlags, SignalMask, UnitCtx};

use super::common::*;

pub const SIG_RELOAD: SignalMask = SignalMask::new(1);

#[derive(Clone, Copy)]
pub struct ShotgunPieces {
    pub(super) shell: Piece,
    pub(super) shotgun: Piece,
    pub(super) reload_anim: Piece,
    pub(super) kickback_rotator: Piece,
}

pub struct ShotgunState {
    pub(super) pieces: ShotgunPieces,
}

pub(super) async fn fire(state: Rc<RefCell<ShotgunState>>, ctx: UnitCtx) {
    let pieces = state.borrow().pieces;
    ctx.set_signal_mask(SIG_RELOAD);
    if pieces.shell != Piece::INVALID {
        let (ctx, shell) = (ctx.clone(), pieces.shell);
        spring::commands::run(move || ctx.explode(shell, SfxFlags(0)));
    }
    turn(
        &ctx,
        pieces.kickback_rotator,
        Axis::X,
        -29.0f32.to_radians(),
        80.0,
    );
    move_piece(&ctx, pieces.reload_anim, Axis::Z, -10.0, 45.0);
    if pieces.reload_anim != Piece::INVALID {
        ctx.wait_for_move(pieces.reload_anim, Axis::Z).await;
    }
    turn(&ctx, pieces.kickback_rotator, Axis::X, 0.0, 55.0);
    move_piece(&ctx, pieces.reload_anim, Axis::Z, 0.0, 55.0);
}
