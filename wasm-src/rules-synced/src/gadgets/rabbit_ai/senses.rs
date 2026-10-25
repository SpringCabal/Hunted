use alloc::vec::Vec;

use super::*;
use crate::roster::Roster;

pub(super) const FIELD_RADIUS: f32 = 9000.0;

pub(super) const FIELD_EDGE_MAGNITUDE: f32 = 0.0;

pub(super) const FIELD_PROXIMITY_MAGNITUDE: f32 = 0.1;

pub(super) const FOOD_RADIUS: f32 = 1500.0;

pub(super) const CARRIER_LIGHT_FADE_FRAMES: i32 = 300;

pub(super) type Target = (f32, f32, f32, f32);

pub(super) type CarrotChoice = (i32, Position, f32, i32);

pub(super) fn choose_target(
    game: &RabbitCtx<'_>,
    rabbit: i32,
    position: &Position,
    carried: bool,
    surroundings: &Surroundings,
    consumed_carrots: &[i32],
) -> (Option<CarrotChoice>, Option<Target>) {
    let position = *position;
    let carrot = if carried {
        None
    } else {
        nearest_unreserved_carrot(
            game,
            rabbit,
            &position,
            &surroundings.static_data,
            consumed_carrots,
        )
    };
    let mine = if carried {
        None
    } else {
        nearest_mine(&position, &surroundings.static_data.mines)
    };
    let burrow = units::nearest(position, &surroundings.static_data.burrows);
    let target = if carried {
        burrow.map(|(_, target, distance)| {
            (
                target.x,
                target.z,
                distance,
                2.0 * (1.0 - distance.sqrt() / FIELD_RADIUS) + 1.0,
            )
        })
    } else {
        let carrot_target = carrot.as_ref().map(|(_, target, distance, eating_time)| {
            (
                target.x,
                target.z,
                *distance,
                food_magnitude(*distance, *eating_time),
            )
        });
        let mine_target = mine.as_ref().map(|(_, target, distance)| {
            (
                target.x,
                target.z,
                *distance,
                food_magnitude(*distance, EAT_TIME),
            )
        });
        let field_target = nearest_field(
            &position,
            &surroundings.static_data.field_carrots,
            consumed_carrots,
        );
        [carrot_target, mine_target, field_target]
            .into_iter()
            .flatten()
            .max_by(|left, right| {
                left.3
                    .partial_cmp(&right.3)
                    .unwrap_or(core::cmp::Ordering::Equal)
            })
    };

    (carrot, target)
}

pub(super) fn nearest_unreserved_carrot(
    game: &RabbitCtx<'_>,
    rabbit: i32,
    position: &Position,
    surroundings: &StaticSurroundings,
    consumed_carrots: &[i32],
) -> Option<(i32, Position, f32, i32)> {
    surroundings
        .food_grid
        .nearby(position.x, position.z)
        .map(|index| (index, &surroundings.carrots[index]))
        .filter(|(_, (_, carrot, _))| {
            horizontal_distance_squared(position, carrot) < FOOD_RADIUS * FOOD_RADIUS
        })
        .filter(|(_, (id, _, _))| !consumed_carrots.contains(id))
        .filter(|(_, (id, _, _))| !carrot_reserved_by_other(game, rabbit, *id))
        .map(|(index, (id, carrot, eating_time))| {
            let distance = horizontal_distance_squared(position, carrot);
            (
                *id,
                *carrot,
                distance,
                *eating_time,
                food_magnitude(distance, *eating_time),
                index,
            )
        })
        .max_by(|left, right| {
            left.4
                .partial_cmp(&right.4)
                .unwrap_or(core::cmp::Ordering::Equal)
                .then_with(|| left.5.cmp(&right.5))
        })
        .map(|(id, carrot, distance, eating_time, _, _)| (id, carrot, distance, eating_time))
}

pub(super) fn nearest_mine(
    position: &Position,
    mines: &[(i32, Position)],
) -> Option<(i32, Position, f32)> {
    mines
        .iter()
        .map(|(id, mine)| (*id, *mine, horizontal_distance_squared(position, mine)))
        .filter(|(_, _, distance_sq)| *distance_sq < FOOD_RADIUS * FOOD_RADIUS)
        .min_by(|left, right| {
            left.2
                .partial_cmp(&right.2)
                .unwrap_or(core::cmp::Ordering::Equal)
        })
}

pub(super) fn carrot_reserved_by_other(game: &RabbitCtx<'_>, rabbit: i32, carrot: i32) -> bool {
    game.rabbits.reserved_by_other(rabbit, carrot)
}

pub(super) fn food_units_with_positions(
    roster: &Roster,
    kind: UnitKind,
    eating_time: i32,
) -> Vec<(i32, Position, i32)> {
    roster
        .positions(kind)
        .into_iter()
        .map(|(unit_id, position)| (unit_id, position, eating_time))
        .collect()
}

pub(super) fn lighthouse_beams(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    roster: &Roster,
) -> Vec<Source> {
    roster
        .positions(UnitKind::Lighthouse)
        .into_iter()
        .map(|(unit_id, position)| {
            let (x, z) = crate::unit_scripts::lighthouse_beam(spring, unit_id)
                .unwrap_or((position.x, position.z));
            Source {
                x,
                z,
                area: scare::Area::LIGHTHOUSE,
            }
        })
        .collect()
}

pub(super) fn flashlight(global: &crate::state::Global) -> Option<Source> {
    let target = global.weapons.access().target;
    let (x, z) = if target.valid {
        (target.x, target.z)
    } else {
        (0.0, 0.0)
    };
    Some(Source {
        x,
        z,
        area: scare::Area::flashlight(x, z),
    })
}

pub(super) fn horizontal_distance_squared(left: &Position, right: &Position) -> f32 {
    let dx = right.x - left.x;
    let dz = right.z - left.z;
    dx * dx + dz * dz
}

pub(super) fn food_magnitude(distance_sq: f32, eating_time: i32) -> f32 {
    let proximity_magnitude = if eating_time == DROPPED_EAT_TIME {
        2.15
    } else {
        2.0
    };
    proximity_magnitude * (1.0 - distance_sq.sqrt() / FOOD_RADIUS) + 0.1
}

pub(super) fn nearest_field(
    position: &Position,
    field_carrots: &[Vec<i32>],
    consumed_carrots: &[i32],
) -> Option<(f32, f32, f32, f32)> {
    crate::map::FIELD_SPECS
        .iter()
        .zip(field_carrots)
        .filter(|(_, carrots)| carrots.iter().any(|id| !consumed_carrots.contains(id)))
        .map(|((top_x, top_z, bottom_x, bottom_z, _, _), _)| {
            let x = (top_x + bottom_x) / 2.0;
            let z = (top_z + bottom_z) / 2.0;
            let distance_sq = (x - position.x).powi(2) + (z - position.z).powi(2);
            let magnitude = FIELD_PROXIMITY_MAGNITUDE * (1.0 - distance_sq.sqrt() / FIELD_RADIUS)
                + FIELD_EDGE_MAGNITUDE;
            (x, z, distance_sq, magnitude)
        })
        .filter(|(_, _, distance_sq, _)| *distance_sq < FIELD_RADIUS * FIELD_RADIUS)
        .max_by(|left, right| {
            left.3
                .partial_cmp(&right.3)
                .unwrap_or(core::cmp::Ordering::Equal)
        })
}

pub(super) fn carrots_by_field(static_carrots: &[(i32, Position, i32)]) -> Vec<Vec<i32>> {
    crate::map::FIELD_SPECS
        .iter()
        .map(|field| {
            static_carrots
                .iter()
                .filter(|(_, carrot, _)| {
                    carrot.x >= field.0
                        && carrot.x <= field.2
                        && carrot.z >= field.1
                        && carrot.z <= field.3
                })
                .map(|(id, _, _)| *id)
                .collect()
        })
        .collect()
}
