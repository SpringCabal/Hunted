use spring::cus::{SignalMask, UnitCtx};

use super::common::*;

const SIG_FUSE: SignalMask = SignalMask::new(1);

pub(super) async fn fuse(ctx: UnitCtx) {
    ctx.set_signal_mask(SIG_FUSE);
    let position = unit_position(ctx.unit());
    let ground = spring::get_ground_height(position.x, position.z).unwrap_or(position.y);
    let mut t = 90;
    loop {
        let fraction = t as f32 / 90.0;
        let _ = spring::spawn_ceg(
            "fuse",
            spring::Float3::new(
                position.x + 11.0 * fraction * fraction,
                ground + 50.0 + t as f32 * 0.33,
                position.z - 11.0 * fraction * fraction,
            ),
            spring::Float3::new(0.0, 1.0, 0.0),
            0.0,
            0.0,
            0.0,
        );
        t -= 1;
        ctx.sleep(spring::cus::Duration::from_millis(33)).await;
    }
}
