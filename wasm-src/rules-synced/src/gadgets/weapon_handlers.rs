use alloc::vec::Vec;

use hunted_common::scare::{Area, Source};
use spring::ResultExt;
use spring::commands::Pending;
use spring::consts::GAME_SPEED;

use spring_addons::AddonContext;
use spring_addons::rules::{Gadget, UnitDestroyedEvent};

use crate::defs::{self, UnitKind, defs};
use crate::diagnostics;
use crate::state::{self, Global, Mine, Target, Weapons};
use crate::status;
use crate::unit_scripts::Call;
use crate::units;

const SHOTGUN_RECOIL: f32 = 20.0;
pub const MINE_TRIGGER_RADIUS: f32 = 80.0;
const MINE_FUSE_FRAMES: i32 = 3 * GAME_SPEED;
const ALL_UNITS: i32 = -1;
const GUN_OFFSET_X: f32 = 100.0;
const GUN_OFFSET_Z: f32 = -100.0;
const SHOTGUN_AMMO_PER_TICK: f32 = 6.0;
const MINE_AMMO_PER_TICK: f32 = 0.5;

pub fn regenerate_ammo(global: &Global) {
    let carrots = global.roster.access().carrot_count();
    let reload_multiplier = carrots as f32 / crate::map::CARROT_COUNT as f32;
    let mut weapons = global.weapons.access_mut();
    weapons.shotgun_ammo =
        (weapons.shotgun_ammo + SHOTGUN_AMMO_PER_TICK * reload_multiplier).min(state::MAX_AMMO);
    weapons.mine_ammo =
        (weapons.mine_ammo + MINE_AMMO_PER_TICK * reload_multiplier).min(state::MAX_AMMO);
}

pub fn update_gun(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    global: &Global,
    frame: i32,
    then: impl FnOnce(&spring::Spring<'_, spring::RulesSynced>, &Global) + 'static,
) {
    let (target, gun_id, previous_goal) = {
        let weapons = global.weapons.access();
        (weapons.target, weapons.gun_id, weapons.gun_goal)
    };
    if !target.valid {
        then(spring, global);
        return;
    }
    if gun_id >= 0 {
        track_target(spring, global, frame, target, gun_id, previous_goal);
        then(spring, global);
        return;
    }
    spawn_gun(spring, target).then(move |spring, gun_id| {
        crate::with_global(|global| {
            match gun_id {
                Some(gun_id) => {
                    let _ = spring.unit_control().set_unit_always_visible(gun_id, true);
                    diagnostics::info(spring, "shotgun spawned for the first mouse target");
                    track_target(spring, global, frame, target, gun_id, None);
                }
                None => global.weapons.access_mut().gun_goal = None,
            }
            then(spring, global);
        });
    });
}

fn track_target(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    global: &Global,
    frame: i32,
    target: Target,
    gun_id: i32,
    previous_goal: Option<(f32, f32, f32)>,
) {
    let goal = gun_goal(target);
    let goal_changed = previous_goal.is_none_or(|old| {
        (old.0 - goal.x).abs() > 1.0 || (old.1 - goal.y).abs() > 1.0 || (old.2 - goal.z).abs() > 1.0
    });
    if goal_changed {
        units::order(spring, gun_id, spring::cmd::MOVE, &[goal.x, goal.y, goal.z]);
    }
    {
        let mut weapons = global.weapons.access_mut();
        weapons.gun_id = gun_id;
        if goal_changed {
            weapons.gun_goal = Some((goal.x, goal.y, goal.z));
        }
    }
    let position = units::unit_position(spring, gun_id).unwrap_or(spring::Float3 {
        x: target.x + 50.0,
        y: target.y + 100.0,
        z: target.z + 50.0,
    });
    units::set_heading(spring, gun_id, target.x - position.x, target.z - position.z);
    let horizontal = ((target.x - position.x).powi(2) + (target.z - position.z).powi(2))
        .sqrt()
        .max(0.001);
    Call::shotgun_set_pitch(gun_id, (position.y - target.y).atan2(horizontal)).run(spring);
    if frame.rem_euclid(diagnostics::LOG_PERIOD_FRAMES) == 0 {
        diagnostics::debug(
            spring,
            &alloc::format!(
                "gun tracking frame={frame} id={gun_id} target=({:.1},{:.1},{:.1}) position=({:.1},{:.1},{:.1})",
                target.x,
                target.y,
                target.z,
                position.x,
                position.y,
                position.z,
            ),
        );
    }
}

fn spawn_gun(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    target: Target,
) -> Pending<spring::RulesSynced, Option<i32>> {
    let gaia_team = spring.game().get_gaia_team_id().unwrap_or(0);
    units::create_at(
        spring,
        UnitKind::Shotgun,
        spring::Float3 {
            x: target.x + 50.0,
            y: target.y + 100.0,
            z: target.z + 50.0,
        },
        gaia_team,
        units::FACING_SOUTH,
    )
}

pub fn fire_shotgun(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    global: &Global,
    target: Target,
) {
    let (gun_id, ammo, muzzle_piece) = {
        let weapons = global.weapons.access();
        (weapons.gun_id, weapons.shotgun_ammo, weapons.muzzle_piece)
    };
    if ammo < 1.0 {
        return;
    }

    let muzzle_piece = muzzle_piece.or_else(|| {
        let piece = muzzle_piece_number(spring, gun_id)?;
        global.weapons.access_mut().muzzle_piece = Some(piece);
        Some(piece)
    });
    let muzzle = muzzle_piece
        .and_then(|piece| {
            spring
                .units_pieces()
                .get_unit_piece_pos_dir(gun_id, piece)
                .ok()
        })
        .map(|pos_dir| pos_dir.position)
        .or_else(|| units::unit_position(spring, gun_id));
    let Some(muzzle) = muzzle else {
        diagnostics::info(spring, "shotgun fire skipped: muzzle position unavailable");
        return;
    };
    let direction = direction_to(muzzle, target);

    let Some(weapon_def_id) = defs(spring).shotgun_weapon else {
        diagnostics::info(
            spring,
            "shotgun fire skipped: weapon definition unavailable",
        );
        return;
    };
    global.weapons.access_mut().shotgun_ammo -= 1.0;
    global.score.access_mut().shots_fired += 1;
    global.status_changed();
    global.scares.access_mut().0.push(Source {
        x: target.x,
        z: target.z,
        area: Area::SHOTGUN,
    });
    Call::shotgun_fire(gun_id).run(spring);
    crate::diagnostics::debug_once!(spring, "shotgun fired");
    let _ = spring.sound().play_sound_file(
        "sounds/shotgun1.wav",
        20.0,
        muzzle,
        spring::Float3::default(),
        0,
    );
    let projectile_speed = defs(spring).shotgun_projectile_speed;
    for _ in 0..defs(spring).shotgun_projectiles {
        let pellet_direction = spread_direction(spring, direction);
        let result = spring.projectile_control().spawn_projectile(
            weapon_def_id,
            &spring::projectile_control::NativeProjectileParams {
                pos: muzzle,
                speed: multiply(pellet_direction, projectile_speed),
                owner: gun_id,
                team: -1,
                weapon_num: -1,
                up_time: -1.0,
                end_alpha: 1.0,
                ..Default::default()
            },
        );
        if result.is_err() {
            spring::log_warning!(
                spring,
                crate::diagnostics::LOG_SECTION,
                "shotgun pellet spawn failed"
            );
        }
    }
    spring
        .unit_control()
        .set_unit_velocity(gun_id, multiply(direction, -SHOTGUN_RECOIL))
        .log_err_once(spring, "shotgun recoil");

    units::order(spring, gun_id, spring::cmd::STOP, &[]);
    let goal = gun_goal(target);
    units::order(spring, gun_id, spring::cmd::MOVE, &[goal.x, goal.y, goal.z]);
    global.weapons.access_mut().gun_goal = Some((goal.x, goal.y, goal.z));
}

fn gun_goal(target: Target) -> spring::Float3 {
    spring::Float3 {
        x: crate::map::clamp_to_map(target.x + GUN_OFFSET_X),
        y: target.y,
        z: crate::map::clamp_to_map(target.z + GUN_OFFSET_Z),
    }
}

pub fn place_mine(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    global: &Global,
    x: f32,
    z: f32,
) {
    if global.weapons.access().mine_ammo < 1.0 {
        return;
    }
    units::create(spring, UnitKind::Mine, x, z, 0).then(move |_spring, unit_id| {
        let Some(unit_id) = unit_id else {
            return;
        };
        crate::with_global(|global| {
            let mut weapons = global.weapons.access_mut();
            weapons.mines.push(Mine {
                unit_id,
                x,
                z,
                fuse: None,
            });
            weapons.mine_ammo -= 1.0;
            drop(weapons);
            global.status_changed();
        });
    });
}

pub fn destroyed(state: &mut Weapons, mine_id: i32) -> Option<(f32, f32)> {
    let index = state
        .mines
        .iter()
        .position(|mine| mine.unit_id == mine_id)?;
    let mine = state.mines.swap_remove(index);
    Some((mine.x, mine.z))
}

pub fn update_mines(spring: &spring::Spring<'_, spring::RulesSynced>, global: &Global) {
    let mines = global.weapons.access().mines.clone();
    if mines.is_empty() {
        return;
    }
    let mut fuses = Vec::new();
    let mut detonations = Vec::new();
    for mine in &mines {
        match mine.fuse {
            None if rabbit_near(spring, global, mine) => fuses.push(mine.unit_id),
            Some(frames_left) if frames_left <= 0 => detonations.push(*mine),
            _ => {}
        }
    }
    let gun_id = {
        let mut weapons = global.weapons.access_mut();
        for mine in weapons.mines.iter_mut() {
            if fuses.contains(&mine.unit_id) {
                mine.fuse = Some(MINE_FUSE_FRAMES);
            } else if let Some(frames_left) = mine.fuse.as_mut() {
                *frames_left -= 1;
            }
        }
        weapons
            .mines
            .retain(|mine| !detonations.iter().any(|gone| gone.unit_id == mine.unit_id));
        weapons.gun_id
    };
    if !fuses.is_empty() {
        crate::diagnostics::info_once!(spring, "mine fuse started");
    }
    for mine in fuses {
        Call::mine_start_fuse(mine).run(spring);
    }
    for mine in detonations {
        units::destroy(spring, mine.unit_id, true, gun_id);
        scare_from_mine(global, mine.x, mine.z);
        crate::diagnostics::info_once!(spring, "mine detonated");
    }
}

pub fn scare_from_mine(global: &Global, x: f32, z: f32) {
    global.scares.access_mut().0.push(Source {
        x,
        z,
        area: Area::MINE,
    });
}

fn rabbit_near(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    global: &Global,
    mine: &Mine,
) -> bool {
    let Ok(units) =
        spring
            .units_query()
            .get_units_in_cylinder(mine.x, mine.z, MINE_TRIGGER_RADIUS, ALL_UNITS)
    else {
        return false;
    };
    let roster = global.roster.access();
    units
        .into_iter()
        .any(|unit| roster.kind(unit) == Some(UnitKind::Rabbit))
}

fn muzzle_piece_number(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    unit_id: i32,
) -> Option<i32> {
    spring
        .units_pieces()
        .get_unit_piece_map(unit_id)
        .ok()?
        .into_iter()
        .find(|piece| piece.name == "flare")
        .map(|piece| piece.piece_num)
}

fn direction_to(from: spring::Float3, target: Target) -> spring::Float3 {
    let mut direction = spring::Float3 {
        x: target.x - from.x,
        y: target.y - from.y,
        z: target.z - from.z,
    };
    let length =
        (direction.x * direction.x + direction.y * direction.y + direction.z * direction.z)
            .sqrt()
            .max(0.001);
    direction.x /= length;
    direction.y /= length;
    direction.z /= length;
    direction
}

fn spread_direction(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    direction: spring::Float3,
) -> spring::Float3 {
    let angle = defs(spring).shotgun_spray_angle;
    let mut spread = spring::Float3 {
        x: direction.x
            + (spring
                .synced_random()
                .next_float()
                .expect("synced random float")
                * 2.0
                - 1.0)
                * angle,
        y: direction.y
            + (spring
                .synced_random()
                .next_float()
                .expect("synced random float")
                * 2.0
                - 1.0)
                * angle,
        z: direction.z
            + (spring
                .synced_random()
                .next_float()
                .expect("synced random float")
                * 2.0
                - 1.0)
                * angle,
    };
    let length = (spread.x * spread.x + spread.y * spread.y + spread.z * spread.z)
        .sqrt()
        .max(0.001);
    spread.x /= length;
    spread.y /= length;
    spread.z /= length;
    spread
}

fn multiply(value: spring::Float3, scalar: f32) -> spring::Float3 {
    spring::Float3 {
        x: value.x * scalar,
        y: value.y * scalar,
        z: value.z * scalar,
    }
}

pub struct WeaponHandlersGadget;

impl Gadget<Global> for WeaponHandlersGadget {
    fn name(&self) -> &'static str {
        "WeaponHandlers"
    }

    fn game_frame(&self, ctx: &AddonContext<'_, Global>, frame: i32) {
        let spring = ctx.spring();
        let global = ctx.global();
        update_mines(spring, global);
        if frame % status::RESOURCE_PERIOD_FRAMES == 0 {
            crate::with_addon_context(|ctx| {
                ctx.commands()
                    .run(|_spring| crate::with_global(regenerate_ammo))
            });
        }
        if global.session.access().game_over {
            global.weapons.access_mut().pending_action = None;
            return;
        }
        update_gun(spring, global, frame, |spring, global| {
            let action = {
                let mut weapons = global.weapons.access_mut();
                weapons
                    .pending_action
                    .take()
                    .map(|action| (action, weapons.target))
            };
            match action {
                Some((state::PendingAction::Shotgun, target)) => {
                    fire_shotgun(spring, global, target)
                }
                Some((state::PendingAction::Mine, target)) => {
                    place_mine(spring, global, target.x, target.z)
                }
                None => {}
            }
        });
    }

    fn unit_created(
        &self,
        ctx: &AddonContext<'_, Global>,
        unit: i32,
        def: i32,
        _team: i32,
        _builder: i32,
    ) {
        let spring = ctx.spring();
        if defs::kind_of(spring, def) != Some(UnitKind::Shotgun) {
            return;
        }
        units::set_gunship_movement_attributes(spring, unit);
        units::order(spring, unit, spring::cmd::IDLEMODE, &[0.0]);
        ctx.global().weapons.access_mut().gun_id = unit;
    }

    fn unit_destroyed(&self, ctx: &AddonContext<'_, Global>, event: &UnitDestroyedEvent) {
        let spring = ctx.spring();
        if super::restarting(ctx.global()) {
            return;
        }
        let global = ctx.global();
        let unit = event.unit_id;
        match defs::kind_of(spring, event.unit_def_id) {
            Some(UnitKind::Shotgun) => {
                let mut weapons = global.weapons.access_mut();
                if weapons.gun_id == unit {
                    weapons.gun_id = -1;
                    weapons.gun_goal = None;
                }
            }
            Some(UnitKind::Mine) => {
                let position = destroyed(&mut global.weapons.access_mut(), unit);
                if let Some((x, z)) = position {
                    scare_from_mine(global, x, z);
                }
            }
            _ => {}
        }
    }
}
