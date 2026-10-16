use spring_addons::Resource;

use alloc::vec::Vec;

use hunted_common::scare::Source;
pub use hunted_common::waves::Waves;

use crate::map::{BURROWS, CARROT_COUNT, LIGHTHOUSES};

pub const NON_RABBIT_COUNT: i32 = CARROT_COUNT + BURROWS.len() as i32 + LIGHTHOUSES.len() as i32;
pub const FLARE_DURATION_FRAMES: i32 = 6 * spring::consts::GAME_SPEED;
pub use hunted_common::weapons::MAX_AMMO;

#[derive(Clone, Copy)]
pub enum PendingAction {
    Shotgun,
    Mine,
}

#[derive(Clone, Copy)]
pub struct Target {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub valid: bool,
}

#[derive(Clone, Copy, Debug)]
pub struct Mine {
    pub unit_id: i32,
    pub x: f32,
    pub z: f32,
    pub fuse: Option<i32>,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Session {
    pub restart_in_progress: bool,
    pub game_over: bool,
    pub status_dirty: bool,
    pub round_start_frame: i32,
}

#[derive(Clone, Copy, Debug)]
pub struct Score {
    pub carrots: i32,
    pub carrots_stolen: i32,
    pub rabbits_killed: i32,
    pub shots_fired: i32,
    pub score: i32,
}

impl Default for Score {
    fn default() -> Self {
        Self {
            carrots: CARROT_COUNT,
            carrots_stolen: 0,
            rabbits_killed: 0,
            shots_fired: 0,
            score: 0,
        }
    }
}

#[derive(Clone)]
pub struct Weapons {
    pub gun_id: i32,
    pub shotgun_ammo: f32,
    pub mine_ammo: f32,
    pub pending_action: Option<PendingAction>,
    pub target: Target,
    pub gun_goal: Option<(f32, f32, f32)>,
    pub muzzle_piece: Option<i32>,
    pub mines: Vec<Mine>,
}

impl Default for Weapons {
    fn default() -> Self {
        Self {
            gun_id: -1,
            shotgun_ammo: hunted_common::weapons::START_SHELLS,
            mine_ammo: hunted_common::weapons::START_MINES,
            pending_action: None,
            target: Target {
                x: crate::map::CENTER,
                y: 0.0,
                z: crate::map::CENTER,
                valid: false,
            },
            gun_goal: None,
            muzzle_piece: None,
            mines: Vec::new(),
        }
    }
}

#[derive(Default)]
pub struct Scares(pub Vec<Source>);

#[derive(Clone, Copy, Debug)]
pub struct Flare {
    pub previous_kills: i32,
    pub required_kills: i32,
    pub active_until: i32,
    pub active: bool,
}

impl Default for Flare {
    fn default() -> Self {
        Self {
            previous_kills: 0,
            required_kills: 50,
            active_until: 0,
            active: false,
        }
    }
}

#[derive(Default)]
pub struct Global {
    pub session: Resource<Session>,
    pub score: Resource<Score>,
    pub weapons: Resource<Weapons>,
    pub scares: Resource<Scares>,
    pub waves: Resource<Waves>,
    pub flare: Resource<Flare>,
    pub rabbits: Resource<crate::gadgets::rabbit_ai::Rabbits>,
    pub roster: Resource<crate::roster::Roster>,
}

impl Global {
    pub fn status_changed(&self) {
        self.session.access_mut().status_dirty = true;
    }
}
