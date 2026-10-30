use crate::defs::{UnitKind, defs};
use spring::ResultExt;
use spring::commands::Pending;

const RABBIT_SPEED_FACTOR: f32 = 0.05;
const RABBIT_MAX_ACCELERATION: f32 = 0.2;
const RABBIT_MAX_DECELERATION: f32 = 0.1;
const RABBIT_TURN_RATE: f32 = 3000.0;
const RABBIT_TURN_ACCELERATION: f32 = 500.0;
const SHOTGUN_SPEED_FACTOR: f32 = 0.05;

pub fn unit_position(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    unit_id: i32,
) -> Option<spring::Float3> {
    spring
        .units_info()
        .get_unit_position(
            unit_id,
            spring::units_info::GetUnitPositionOptions {
                mid_pos: true,
                aim_pos: false,
            },
        )
        .ok()
}

pub const FACING_SOUTH: i32 = 0;

pub fn create(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    kind: UnitKind,
    x: f32,
    z: f32,
    team: i32,
) -> Pending<spring::RulesSynced, Option<i32>> {
    let y = spring.terrain().get_ground_height(x, z).unwrap_or(0.0);
    create_at(spring, kind, spring::Float3 { x, y, z }, team, FACING_SOUTH)
}

pub fn create_at(
    _spring: &spring::Spring<'_, spring::RulesSynced>,
    kind: UnitKind,
    position: spring::Float3,
    team: i32,
    facing: i32,
) -> Pending<spring::RulesSynced, Option<i32>> {
    crate::with_addon_context(|ctx| {
        ctx.commands()
            .queue(move |spring| create_now(spring, kind, position, team, facing))
    })
}

pub fn create_now(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    kind: UnitKind,
    position: spring::Float3,
    team: i32,
    facing: i32,
) -> Option<i32> {
    let def_id = defs(spring).unit_def(kind);
    if def_id < 0 {
        return None;
    }
    let unit = spring.unit_control().create_unit(
        &spring::unit_control::DefRef {
            name: kind.def_name().into(),
            id: def_id,
        },
        position,
        facing,
        team,
        spring::unit_control::CreateUnitOptions {
            build: false,
            flatten_ground: false,
            unit_id: -1,
            builder_id: -1,
        },
    );
    match unit {
        Ok(unit) if unit >= 0 => Some(unit),
        _ => {
            spring::log_warning!(
                spring,
                crate::diagnostics::LOG_SECTION,
                "creating a {} failed: {unit:?}",
                kind.def_name()
            );
            None
        }
    }
}

pub fn set_yaw(spring: &spring::Spring<'_, spring::RulesSynced>, unit_id: i32, yaw: f32) {
    let _ = spring.unit_control().set_unit_rotation(
        unit_id,
        spring::Float3 {
            x: 0.0,
            y: yaw,
            z: 0.0,
        },
    );
}

pub fn destroy(
    _spring: &spring::Spring<'_, spring::RulesSynced>,
    unit_id: i32,
    selfd: bool,
    attacker: i32,
) {
    crate::with_addon_context(|ctx| {
        ctx.commands()
            .destroy_unit(unit_id, destroy_options(selfd, attacker))
    })
    .then(move |spring, result| {
        log_destroy_error(spring, unit_id, result);
    });
}

pub fn destroy_now(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    unit_id: i32,
    selfd: bool,
    attacker: i32,
) {
    log_destroy_error(
        spring,
        unit_id,
        spring
            .unit_control()
            .destroy_unit(unit_id, destroy_options(selfd, attacker)),
    );
}

fn destroy_options(selfd: bool, attacker: i32) -> spring::unit_control::DestroyUnitOptions {
    spring::unit_control::DestroyUnitOptions {
        selfd,
        reclaimed: false,
        attacker_id: attacker,
        recycle_id: false,
    }
}

fn log_destroy_error(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    unit_id: i32,
    result: spring::Result<bool>,
) {
    if let Err(error) = result {
        spring::log_warning!(
            spring,
            crate::diagnostics::LOG_SECTION,
            "destroying unit {unit_id} failed: {error:?}"
        );
    }
}

pub fn set_goal(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    unit_id: i32,
    x: f32,
    z: f32,
    speed: f32,
) {
    let result = spring.unit_control().set_unit_move_goal(
        unit_id,
        spring::Float3 { x, y: 0.0, z },
        16.0,
        speed,
        true,
    );
    if let Err(error) = result {
        spring::log_once!(
            spring,
            spring::log_level::WARNING,
            crate::diagnostics::LOG_SECTION,
            "setting the move goal of rabbit {unit_id} failed: {error:?}"
        );
    }
}

pub fn destroy_internal_now(spring: &spring::Spring<'_, spring::RulesSynced>, unit: i32) {
    spring
        .rules_params()
        .set_unit_rules_param_f32(
            unit,
            "internalDestroy",
            1.0,
            spring::rules_params::los::PUBLIC,
        )
        .log_err_once(spring, "marking internal unit removal");
    destroy_now(spring, unit, false, -1);
}

pub fn set_rabbit_movement_attributes(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    unit_id: i32,
    speed_multiplier: f32,
    acceleration_multiplier: f32,
    turn_multiplier: f32,
    original_reverse_speed: f32,
) -> bool {
    let unit_speed = defs(spring).rabbit_speed;
    let speed_multiplier = speed_multiplier.max(0.0);
    let acceleration_multiplier = acceleration_multiplier.max(0.0);
    let turn_multiplier = turn_multiplier.max(0.0);
    let deceleration_multiplier = if speed_multiplier == 0.0 {
        if let Some(position) = unit_position(spring, unit_id)
            && spring
                .terrain()
                .get_ground_height(position.x, position.z)
                .is_ok_and(|ground| ground >= position.y)
        {
            let _ = spring
                .unit_control()
                .set_unit_velocity(unit_id, spring::Float3::ZERO);
        }
        100_000.0
    } else if speed_multiplier < 1.0 {
        1000.0
    } else {
        speed_multiplier
    };
    let mut acceleration_rate =
        RABBIT_MAX_ACCELERATION * speed_multiplier * (speed_multiplier * acceleration_multiplier);
    if speed_multiplier < 1.0 {
        acceleration_rate = acceleration_rate.min(speed_multiplier);
    }

    let mut applied = true;
    applied &= spring
        .move_ctrl()
        .set_ground_move_type_max_speed(
            unit_id,
            unit_speed * RABBIT_SPEED_FACTOR * speed_multiplier,
        )
        .log_err_once(spring, "setting a rabbit's max speed")
        == Some(true);
    applied &= spring
        .move_ctrl()
        .set_move_type_numeric(
            unit_id,
            spring::move_ctrl::MoveTypeNumericField::MoveTypeGroundAccRate,
            acceleration_rate,
        )
        .log_err_once(spring, "setting a rabbit's MoveTypeGroundAccRate")
        == Some(true);
    applied &= spring
        .move_ctrl()
        .set_move_type_numeric(
            unit_id,
            spring::move_ctrl::MoveTypeNumericField::MoveTypeGroundDecRate,
            RABBIT_MAX_DECELERATION * deceleration_multiplier,
        )
        .log_err_once(spring, "setting a rabbit's MoveTypeGroundDecRate")
        == Some(true);
    applied &= spring
        .move_ctrl()
        .set_move_type_numeric(
            unit_id,
            spring::move_ctrl::MoveTypeNumericField::MoveTypeGroundMaxReverseSpeed,
            if speed_multiplier < 1.0 {
                0.0
            } else {
                original_reverse_speed
            },
        )
        .log_err_once(spring, "setting a rabbit's MoveTypeGroundMaxReverseSpeed")
        == Some(true);
    applied &= spring
        .move_ctrl()
        .set_move_type_numeric(
            unit_id,
            spring::move_ctrl::MoveTypeNumericField::MoveTypeGroundTurnRate,
            RABBIT_TURN_RATE * turn_multiplier,
        )
        .log_err_once(spring, "setting a rabbit's MoveTypeGroundTurnRate")
        == Some(true);
    applied &= spring
        .move_ctrl()
        .set_move_type_numeric(
            unit_id,
            spring::move_ctrl::MoveTypeNumericField::MoveTypeGroundTurnAccel,
            RABBIT_TURN_ACCELERATION * turn_multiplier,
        )
        .log_err_once(spring, "setting a rabbit's MoveTypeGroundTurnAccel")
        == Some(true);
    applied
}

pub fn set_gunship_movement_attributes(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    unit_id: i32,
) {
    spring
        .move_ctrl()
        .set_move_type_numeric(
            unit_id,
            spring::move_ctrl::MoveTypeNumericField::MoveTypeMaxSpeed,
            defs(spring).shotgun_speed * SHOTGUN_SPEED_FACTOR,
        )
        .log_err_once(spring, "setting the gun's MoveTypeMaxSpeed");
}

pub fn order(
    _spring: &spring::Spring<'_, spring::RulesSynced>,
    unit_id: i32,
    command: i32,
    params: &[f32],
) {
    crate::with_addon_context(|ctx| {
        ctx.commands()
            .give_order_to_unit(unit_id, command, params, 0, -1)
    })
    .then(move |spring, result| {
        if !matches!(result, Ok(true)) {
            spring::log_once!(
                spring,
                spring::log_level::WARNING,
                crate::diagnostics::LOG_SECTION,
                "order {command} to unit {unit_id} was refused: {result:?}"
            );
        }
    });
}

pub fn set_heading(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    unit_id: i32,
    dx: f32,
    dz: f32,
) {
    let angle = dx.atan2(dz);
    let heading = (angle / (2.0 * core::f32::consts::PI) * 65536.0) as i32;
    let _ = spring.unit_control().set_unit_heading_and_up_dir(
        unit_id,
        heading,
        spring::Float3 {
            x: 0.0,
            y: 1.0,
            z: 0.0,
        },
    );
}

#[cfg(feature = "headless-integration")]
pub fn nearest(
    position: spring::Float3,
    units: &[(i32, spring::Float3)],
) -> Option<(i32, spring::Float3, f32)> {
    units
        .iter()
        .map(|(id, pos)| {
            let dx = pos.x - position.x;
            let dz = pos.z - position.z;
            (*id, *pos, dx * dx + dz * dz)
        })
        .min_by(|a, b| a.2.partial_cmp(&b.2).unwrap_or(core::cmp::Ordering::Equal))
}
