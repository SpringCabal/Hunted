use alloc::vec::Vec;
use core::cell::RefCell;

use spring_addons::rules::Gadget;

use crate::defs::UnitKind;
use crate::diagnostics;
use crate::map::{self, BURROWS, LIGHTHOUSES};
use crate::state::{self, Global};
use crate::units;

fn spawn_static(spring: &spring::Spring<'_, spring::RulesSynced>, global: &Global) {
    let mut waves = global.waves.access_mut();
    let starting_index = waves.spawn_index;
    let batch_end = state::NON_RABBIT_COUNT;
    while waves.spawn_index < batch_end {
        spawn_item(spring, waves.spawn_index);
        waves.spawn_index += 1;
    }
    if starting_index < state::NON_RABBIT_COUNT && waves.spawn_index >= state::NON_RABBIT_COUNT {
        diagnostics::info(spring, "static carrots, burrows, and lighthouses spawned");
    }
}

fn spawn_rabbit(spring: &spring::Spring<'_, spring::RulesSynced>, origin: spring::Float3) {
    let radius = (12.0
        * spring
            .synced_random()
            .next_float()
            .expect("synced random float"))
    .powi(2);
    let angle = spring
        .synced_random()
        .next_float()
        .expect("synced random float")
        * 2.0
        * core::f32::consts::PI;
    let facing = spring
        .synced_random()
        .next_int(0, 3)
        .expect("synced random int");
    units::create_now(
        spring,
        UnitKind::Rabbit,
        spring::Float3::new(
            origin.x + radius * angle.cos(),
            origin.y,
            origin.z + radius * angle.sin(),
        ),
        0,
        facing,
    );
}

fn update_wave(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    global: &Global,
    origins: &RefCell<Vec<(i32, spring::Float3)>>,
    frame: i32,
) {
    let mut waves = global.waves.access_mut();
    if waves.wave_target == 0 && frame >= waves.next_wave_frame {
        let mut origins = origins.borrow_mut();
        *origins = global
            .roster
            .access()
            .units(UnitKind::Burrow)
            .into_iter()
            .filter_map(|id| {
                spring
                    .units_info()
                    .get_unit_position(id, Default::default())
                    .ok()
                    .map(|position| (id, position))
            })
            .collect();
        if origins.is_empty() {
            return;
        }
        waves
            .burrow_spawns
            .resize(origins.len(), Default::default());
        let rabbit_count = global.roster.access().count(UnitKind::Rabbit) as i32;
        let burrows = waves.start_wave(frame, rabbit_count, || {
            spring
                .synced_random()
                .next_int(1, origins.len() as i32)
                .expect("synced random up_to") as usize
                - 1
        });
        diagnostics::info(
            spring,
            &alloc::format!(
                "rabbit wave started: {} rabbits from {} burrows",
                waves.wave_target,
                burrows.min(BURROWS.len() as i32),
            ),
        );
        return;
    }

    for (burrow, &(unit, position)) in origins.borrow().iter().enumerate() {
        if spring.units_info().get_unit_is_dead(unit).unwrap_or(true) {
            waves.burrow_spawns[burrow].active = false;
            continue;
        }
        waves.emit_burrow_batch(
            burrow,
            frame,
            || {
                spring
                    .synced_random()
                    .next_float()
                    .expect("synced random float")
            },
            |count| {
                for _ in 0..count {
                    spawn_rabbit(spring, position);
                }
            },
        );
    }
    waves.finish_wave();
}

fn spawn_item(spring: &spring::Spring<'_, spring::RulesSynced>, index: i32) {
    if index < map::CARROT_COUNT {
        if let Some((x, z)) = map::carrot_position(index) {
            spawn_turned(spring, UnitKind::Carrot, x, z);
        }
        return;
    }
    let index = (index - map::CARROT_COUNT) as usize;
    if let Some(&(x, z)) = BURROWS.get(index) {
        spawn_turned(spring, UnitKind::Burrow, x, z);
    } else if let Some(&(x, z)) = LIGHTHOUSES.get(index - BURROWS.len()) {
        let y = spring.terrain().get_ground_height(x, z).unwrap_or(0.0);
        units::create_now(
            spring,
            UnitKind::Lighthouse,
            spring::Float3::new(x, y, z),
            0,
            0,
        );
    }
}

fn spawn_turned(spring: &spring::Spring<'_, spring::RulesSynced>, kind: UnitKind, x: f32, z: f32) {
    let y = spring.terrain().get_ground_height(x, z).unwrap_or(0.0);
    if let Some(unit) = units::create_now(spring, kind, spring::Float3::new(x, y, z), 0, 0) {
        units::set_yaw(
            spring,
            unit,
            spring
                .synced_random()
                .next_float()
                .expect("synced random float")
                * 2.0
                * core::f32::consts::PI,
        );
    }
}

#[derive(Default)]
pub struct WaveSpawnerGadget {
    origins: alloc::rc::Rc<RefCell<Vec<(i32, spring::Float3)>>>,
}

impl Gadget<Global> for WaveSpawnerGadget {
    fn name(&self) -> &'static str {
        "WaveSpawner"
    }

    fn game_frame(&self, ctx: &spring_addons::AddonContext<'_, Global>, frame: i32) {
        let origins = alloc::rc::Rc::clone(&self.origins);
        ctx.commands().run(move |spring| {
            crate::with_global(|global| {
                spawn_static(spring, global);
                update_wave(spring, global, &origins, frame);
            })
        });
    }
}
