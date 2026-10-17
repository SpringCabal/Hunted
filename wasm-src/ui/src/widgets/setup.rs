use spring::default_interface::{ALL, CONSOLE, MAP_BORDER};
use spring_addons::register_widgets;
use spring_addons::ui::{HideDefaultInterface, WidgetHandler};

use super::*;
use crate::audio;
use crate::state::Global;

pub const HIDDEN_INTERFACE: u32 = ALL & !MAP_BORDER;

#[cfg(feature = "e2e")]
const E2E_SHOT_FRAMES: [u32; 5] = [18, 47, 85, 130, 180];
#[cfg(feature = "e2e")]
const E2E_QUIT_DELAY_FRAMES: u32 = 30;

pub fn setup(handler: &mut WidgetHandler<Global>, spring: &spring::Spring<'_, spring::Ui>) {
    register_widgets!(
        handler,
        [springcabal::controls::GameControls::new().with_pause_key("space")]
    );
    if springcabal::verification::world_only(spring) {
        gallery::setup(spring, handler);
        return;
    }
    register_widgets!(
        handler,
        [
            HideDefaultInterface::new(HIDDEN_INTERFACE & !CONSOLE),
            CameraWidget::new(),
            GuiRmlWidget::new(),
            audio::music(),
            GuiInputWidget::new(),
            GfxRabbitVisibilityWidget::new(),
            GfxFlashlightWidget::new(),
        ]
    );
    #[cfg(feature = "e2e")]
    register_widgets!(
        handler,
        [springcabal::shots::Screenshots::new(
            springcabal::shots::Clock::GameFrame,
            &E2E_SHOT_FRAMES
        )
        .done_marker("HUNTED VISUAL SHOTS TAKEN")
        .quit_after(E2E_QUIT_DELAY_FRAMES)]
    );
}
