use spring_addons::register_gadgets;
use spring_addons::rules::GadgetHandler;

use crate::state::Global;

pub mod blocking;
#[cfg(feature = "e2e")]
pub mod e2e;
pub mod flare;
pub mod game_end;
pub mod invulnerable;
pub mod rabbit_ai;
pub mod roster;
pub mod score;
pub mod ui_input;
pub mod wave_spawner;
pub mod weapon_handlers;

pub use blocking::BlockingGadget;
pub use flare::FlareGadget;
pub use game_end::GameEndGadget;
pub use invulnerable::InvulnerableGadget;
pub use rabbit_ai::RabbitAiGadget;
pub use roster::RosterGadget;
pub use score::ScoreGadget;
pub use ui_input::UiInputGadget;
pub use wave_spawner::WaveSpawnerGadget;
pub use weapon_handlers::WeaponHandlersGadget;

pub fn setup(handler: &mut GadgetHandler<Global>) {
    register_gadgets!(
        handler,
        [
            RosterGadget,
            BlockingGadget,
            InvulnerableGadget,
            WaveSpawnerGadget,
            WeaponHandlersGadget,
        ]
    );
    #[cfg(feature = "headless-integration")]
    register_gadgets!(handler, [crate::integration::IntegrationGadget::default()]);
    #[cfg(feature = "e2e")]
    register_gadgets!(handler, [e2e::E2eGadget]);
    register_gadgets!(
        handler,
        [
            RabbitAiGadget,
            FlareGadget,
            GameEndGadget,
            ScoreGadget,
            UiInputGadget,
        ]
    );
}

fn restarting(global: &Global) -> bool {
    global.session.access().restart_in_progress
}
