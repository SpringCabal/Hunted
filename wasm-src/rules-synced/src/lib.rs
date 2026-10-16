extern crate alloc;

pub mod defs;
pub mod diagnostics;
pub mod gadgets;
#[cfg(feature = "headless-integration")]
pub mod integration;
pub use hunted_common::map;
pub mod roster;
pub mod state;
pub mod status;
pub mod unit_scripts;
pub mod units;

spring_addons::export_rules_gadgets! {
    state: state::Global,
    setup: gadgets::setup,
    scratch: 4096,
}
