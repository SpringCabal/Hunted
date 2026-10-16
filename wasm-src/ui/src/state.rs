use alloc::vec::Vec;

use spring_addons::Resource;

use crate::widgets::gfx_rabbit_visibility::Beam;

#[derive(Clone, Copy)]
pub struct Point3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[derive(Clone, Copy, Default)]
pub struct Round {
    pub game_over: bool,
    pub restart_pending: bool,
    pub flare_active: bool,
    pub frame: i32,
}

#[derive(Default)]
pub struct Lights {
    pub flashlight: Option<Point3>,
    pub beams: Vec<Beam>,
}

#[derive(Clone, Copy)]
pub struct View {
    pub width: i32,
    pub height: i32,
}

impl Default for View {
    fn default() -> Self {
        Self {
            width: 1280,
            height: 720,
        }
    }
}

#[derive(Default)]
pub struct Global {
    pub round: Resource<Round>,
    pub lights: Resource<Lights>,
    pub view: Resource<View>,
}

pub fn request_restart(global: &Global) -> bool {
    if !global.round.access().game_over {
        return false;
    }
    if !springcabal::message::send_to_rules(hunted_common::protocol::RESTART) {
        return false;
    }
    let mut round = global.round.access_mut();
    round.game_over = false;
    round.restart_pending = true;
    true
}

pub fn request_quit() {
    springcabal::menu::quit_to_menu();
}
