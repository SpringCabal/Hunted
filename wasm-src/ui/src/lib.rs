extern crate alloc;

pub mod audio;
pub mod defs;
pub mod diagnostics;
pub mod state;
pub mod widgets;

spring_addons::export_ui_widgets! {
    state: state::Global,
    setup: widgets::setup,
    scratch: 4096,
}
