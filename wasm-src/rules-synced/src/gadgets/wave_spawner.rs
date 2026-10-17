use spring::commands::Pending;

use spring_addons::rules::Gadget;

use crate::defs::UnitKind;
use crate::diagnostics;
use crate::map::{self, BURROWS, LIGHTHOUSES};
use crate::state::{self, Global};
use crate::units;

const STATIC_SPAWN_BATCH: i32 = 32;

fn spawn_static(spring: &spring::Spring<'_, spring::RulesSynced>, global: &Global) {
    let mut waves = global.waves.access_mut();
    let starting_index = waves.spawn_index;
    let batch_end = (starting_index + STATIC_SPAWN_BATCH).min(state::NON_RABBIT_COUNT);
    while waves.spawn_index < batch_end {
        spawn_item(spring, waves.spawn_index);
        waves.spawn_index += 1;
    }
    if starting_index < state::NON_RABBIT_COUNT && waves.spawn_index >= state::NON_RABBIT_COUNT {
        diagnostics::info(spring, "static carrots, burrows, and lighthouses spawned");
    }
}

fn spawn_rabbit(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    burrow: usize,
) -> Pending<spring::RulesSynced, Option<i32>> {
    let (x, z) = BURROWS[burrow];
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
        * core::f32::consts::TAU;
    let x = x + radius * angle.cos();
    let z = z + radius * angle.sin();
    let y = spring.terrain().get_ground_height(x, z).unwrap_or(0.0);
    let facing = spring
        .synced_random()
        .next_int(0, 3)
        .expect("synced random int");
    units::create_at(
        spring,
        UnitKind::Rabbit,
        spring::Float3 { x, y, z },
        0,
        facing,
    )
}

fn update_wave(spring: &spring::Spring<'_, spring::RulesSynced>, global: &Global, frame: i32) {
    let mut waves = global.waves.access_mut();
    if waves.wave_target == 0 && frame >= waves.next_wave_frame {
        let rabbit_count = global.roster.access().count(UnitKind::Rabbit) as i32;
        let burrows = waves.start_wave(frame, rabbit_count, || {
            spring
                .synced_random()
                .next_int_up_to(BURROWS.len() as i32)
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

    for burrow in 0..BURROWS.len() {
        let Some(batch) = waves.next_burrow_batch(burrow, frame, || {
            spring
                .synced_random()
                .next_float()
                .expect("synced random float")
        }) else {
            continue;
        };
        for _ in 0..batch {
            let _ = spawn_rabbit(spring, burrow);
        }
    }
    crate::with_addon_context(|ctx| {
        ctx.commands()
            .run(|_spring| crate::with_global(|global| global.waves.access_mut().finish_wave()))
    });
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
        let _ = units::create(spring, UnitKind::Lighthouse, x, z, 0);
    }
}

fn spawn_turned(spring: &spring::Spring<'_, spring::RulesSynced>, kind: UnitKind, x: f32, z: f32) {
    units::create(spring, kind, x, z, 0).then(|spring, unit| {
        if let Some(unit) = unit {
            units::set_yaw(
                spring,
                unit,
                spring
                    .synced_random()
                    .next_float()
                    .expect("synced random float")
                    * core::f32::consts::TAU,
            );
        }
    });
}

pub struct WaveSpawnerGadget;

impl Gadget<Global> for WaveSpawnerGadget {
    fn name(&self) -> &'static str {
        "WaveSpawner"
    }

    fn game_frame(&self, ctx: &spring_addons::AddonContext<'_, Global>, frame: i32) {
        let spring = ctx.spring();
        spawn_static(spring, ctx.global());
        update_wave(spring, ctx.global(), frame);
    }
}
