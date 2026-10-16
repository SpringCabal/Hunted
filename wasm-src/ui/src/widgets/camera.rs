use spring::Float3;
use spring_addons::AddonContext;
use spring_addons::ui::Widget;
use springcabal_control::camera::{CameraPlacement, View};

use crate::state::Global;

const VIEW: View = View::overhead(3300.0, 0.464).pitch(-0.901_499_33, -0.433_569_3);
const POSITION: Float3 = Float3::new(3150.0, 102.341_46, 3480.0);

pub struct CameraWidget {
    camera: CameraPlacement,
}

impl Default for CameraWidget {
    fn default() -> Self {
        Self::new()
    }
}

impl CameraWidget {
    pub fn new() -> Self {
        Self {
            camera: CameraPlacement::new(VIEW, POSITION),
        }
    }
}

impl Widget<Global> for CameraWidget {
    fn name(&self) -> &'static str {
        "Camera"
    }

    fn update(&self, ctx: &AddonContext<'_, Global>, dt: f32) {
        self.camera.update(ctx, dt);
    }
}
