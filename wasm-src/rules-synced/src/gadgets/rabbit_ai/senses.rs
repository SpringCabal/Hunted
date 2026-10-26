use alloc::vec::Vec;

use super::*;
use crate::roster::Roster;
use hunted_common::desired::{Desirables, Key};

pub(super) const FIELD_RADIUS: f32 = 9000.0;

pub(super) const FIELD_EDGE_MAGNITUDE: f32 = 0.0;

pub(super) const FIELD_PROXIMITY_MAGNITUDE: f32 = 0.1;

pub(super) const FOOD_RADIUS: f32 = 1500.0;

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
    let mut carrot = if carried {
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
    let burrow = strongest_unit(
        &position,
        &surroundings.static_data.burrows,
        FIELD_RADIUS,
        2.0,
        1.0,
    );
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
            &surroundings.static_data.desirables,
            consumed_carrots,
        );
        let order = &surroundings.static_data.desirables;
        let ranked_carrot = carrot_target.zip(carrot.as_ref().map(|c| Key::Unit(c.0)));
        let ranked_mine = mine_target.zip(mine.as_ref().map(|m| Key::Unit(m.0)));
        let ranked_field = field_target.map(|(field, target)| (target, Key::Field(field)));
        [ranked_carrot, ranked_mine, ranked_field]
            .into_iter()
            .flatten()
            .max_by(|(left, lk), (right, rk)| {
                left.3
                    .partial_cmp(&right.3)
                    .unwrap_or(core::cmp::Ordering::Equal)
                    .then_with(|| order.rank(*rk).cmp(&order.rank(*lk)))
            })
            .map(|(target, key)| {
                carrot = carrot.filter(|(id, _, _, _)| key == Key::Unit(*id));
                target
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
    let mut best: Option<(CarrotChoice, f32, usize)> = None;
    let mut nearest: [Option<(f32, usize)>; 2] = [None, None];
    surroundings
        .food_grid
        .visit_nearby(position.x, position.z, |index| {
            let (id, carrot, eating_time) = surroundings.carrots[index];
            let distance = horizontal_distance_squared(position, &carrot);
            if distance >= FOOD_RADIUS * FOOD_RADIUS {
                return;
            }
            let category = usize::from(eating_time == DROPPED_EAT_TIME);
            // A farther, later entry cannot win, including rounded falloff ties.
            if nearest[category].is_some_and(|(d, i)| d <= distance && i <= index) {
                return;
            }
            if consumed_carrots.contains(&id) || carrot_reserved_by_other(game, rabbit, id) {
                return;
            }
            if nearest[category].is_none_or(|(d, i)| distance < d || (distance == d && index < i)) {
                nearest[category] = Some((distance, index));
            }
            let magnitude = food_magnitude(distance, eating_time);
            if best.is_none_or(|(_, m, i)| magnitude > m || (magnitude == m && index < i)) {
                best = Some(((id, carrot, distance, eating_time), magnitude, index));
            }
        });
    best.map(|(choice, _, _)| choice)
}

pub(super) fn nearest_mine(
    position: &Position,
    mines: &[(i32, Position)],
) -> Option<(i32, Position, f32)> {
    strongest_unit(position, mines, FOOD_RADIUS, 2.0, 0.1)
}

fn strongest_unit(
    position: &Position,
    units: &[(i32, Position)],
    radius: f32,
    proximity: f32,
    edge: f32,
) -> Option<(i32, Position, f32)> {
    hunted_common::desired::best_falloff(
        units
            .iter()
            .map(|(_, target)| horizontal_distance_squared(position, target)),
        radius,
        proximity,
        edge,
    )
    .map(|(index, square, _)| (units[index].0, units[index].1, square))
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

pub(super) fn lighthouse_beams(global: &Global) -> Vec<Source> {
    global.beams.access().sources()
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
        // Lua creates the initial torch at (0, 0) with its base attributes;
        // edge fading starts only when the first valid mouse target arrives.
        area: if target.valid {
            scare::Area::flashlight(x, z)
        } else {
            scare::Area::FLASHLIGHT
        },
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
    desirables: &Desirables,
    consumed_carrots: &[i32],
) -> Option<(usize, Target)> {
    crate::map::FIELD_SPECS
        .iter()
        .zip(desirables.fields())
        .enumerate()
        .filter(|(_, (_, carrots))| carrots.iter().any(|id| !consumed_carrots.contains(id)))
        .map(|(field, ((top_x, top_z, bottom_x, bottom_z, _, _), _))| {
            let x = (top_x + bottom_x) / 2.0;
            let z = (top_z + bottom_z) / 2.0;
            let distance_sq = (x - position.x).powi(2) + (z - position.z).powi(2);
            let magnitude = FIELD_PROXIMITY_MAGNITUDE * (1.0 - distance_sq.sqrt() / FIELD_RADIUS)
                + FIELD_EDGE_MAGNITUDE;
            (field, (x, z, distance_sq, magnitude))
        })
        .filter(|(_, (_, _, distance_sq, _))| *distance_sq < FIELD_RADIUS * FIELD_RADIUS)
        .max_by(|(lf, left), (rf, right)| {
            left.3
                .partial_cmp(&right.3)
                .unwrap_or(core::cmp::Ordering::Equal)
                .then_with(|| {
                    desirables
                        .rank(Key::Field(*rf))
                        .cmp(&desirables.rank(Key::Field(*lf)))
                })
        })
}

pub(super) fn ordered_positions(roster: &Roster, kind: UnitKind) -> Vec<(i32, Position)> {
    let mut positions = roster.positions(kind);
    positions.sort_unstable_by_key(|(unit, _)| roster.desirables.rank(Key::Unit(*unit)));
    positions
}
