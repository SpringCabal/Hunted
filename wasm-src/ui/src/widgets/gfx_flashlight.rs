use core::cell::RefCell;

use spring_addons::ui::Widget;

use crate::state::Global;
use springcabal::volumes::Volumes;

const FLASHLIGHT_COLOR: [f32; 4] = [0.69, 0.61, 0.85, 0.30];
const FLASHLIGHT_RADIUS: f32 = 150.0;

#[derive(Default)]
pub struct GfxFlashlightWidget {
    volumes: RefCell<Volumes>,
}

impl GfxFlashlightWidget {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Widget<Global> for GfxFlashlightWidget {
    fn name(&self) -> &'static str {
        "GfxFlashlight"
    }

    fn draw_world(&self, ctx: &spring_addons::AddonContext<'_, Global>) {
        let lights = ctx.global().lights.access();
        let mut volumes = self.volumes.borrow_mut();
        if let Some(target) = lights.flashlight {
            volumes.draw_ground_circle(target.x, target.z, FLASHLIGHT_RADIUS, FLASHLIGHT_COLOR);
        }
        for beam in &lights.beams {
            volumes.draw_ground_circle(beam.x, beam.z, beam.size, FLASHLIGHT_COLOR);
        }
    }
}
