use spring::commands::{self, Pending};
use spring::random::synced as random;
use spring_addons::rules::Gadget;

use crate::defs::UnitKind;
use crate::diagnostics;
use crate::map::{self, BURROWS, LIGHTHOUSES};
use crate::state::{self, Global};
use crate::units;

const STATIC_SPAWN_BATCH: i32 = 32;

fn spawn_static(global: &Global) {
    let mut waves = global.waves.access_mut();
    let starting_index = waves.spawn_index;
    let batch_end = (starting_index + STATIC_SPAWN_BATCH).min(state::NON_RABBIT_COUNT);
    while waves.spawn_index < batch_end {
        spawn_item(waves.spawn_index);
        waves.spawn_index += 1;
    }
    if starting_index < state::NON_RABBIT_COUNT && waves.spawn_index >= state::NON_RABBIT_COUNT {
        diagnostics::info("static carrots, burrows, and lighthouses spawned");
    }
}

fn spawn_rabbit(burrow: usize) -> Pending<Option<i32>> {
    let (x, z) = BURROWS[burrow];
    let radius = (12.0 * random::float()).powi(2);
    let angle = random::float() * core::f32::consts::TAU;
    let x = x + radius * angle.cos();
    let z = z + radius * angle.sin();
    let y = spring::get_ground_height(x, z).unwrap_or(0.0);
    let facing = random::int(0, 3);
    units::create_at(UnitKind::Rabbit, spring::Float3 { x, y, z }, 0, facing)
}

fn update_wave(global: &Global, frame: i32) {
    let mut waves = global.waves.access_mut();
    if waves.wave_target == 0 && frame >= waves.next_wave_frame {
        let rabbit_count = global.roster.access().count(UnitKind::Rabbit) as i32;
        let burrows = waves.start_wave(frame, rabbit_count, || {
            random::up_to(BURROWS.len() as i32) as usize - 1
        });
        diagnostics::info(&alloc::format!(
            "rabbit wave started: {} rabbits from {} burrows",
            waves.wave_target,
            burrows.min(BURROWS.len() as i32),
        ));
        return;
    }

    for burrow in 0..BURROWS.len() {
        let Some(batch) = waves.next_burrow_batch(burrow, frame, random::float) else {
            continue;
        };
        for _ in 0..batch {
            let _ = spawn_rabbit(burrow);
        }
    }
    commands::run(|| crate::with_global(|global| global.waves.access_mut().finish_wave()));
}

fn spawn_item(index: i32) {
    if index < map::CARROT_COUNT {
        if let Some((x, z)) = map::carrot_position(index) {
            spawn_turned(UnitKind::Carrot, x, z);
        }
        return;
    }
    let index = (index - map::CARROT_COUNT) as usize;
    if let Some(&(x, z)) = BURROWS.get(index) {
        spawn_turned(UnitKind::Burrow, x, z);
    } else if let Some(&(x, z)) = LIGHTHOUSES.get(index - BURROWS.len()) {
        let _ = units::create(UnitKind::Lighthouse, x, z, 0);
    }
}

fn spawn_turned(kind: UnitKind, x: f32, z: f32) {
    units::create(kind, x, z, 0).then(|unit| {
        if let Some(unit) = unit {
            units::set_yaw(unit, random::float() * core::f32::consts::TAU);
        }
    });
}

pub struct WaveSpawnerGadget;

impl Gadget<Global> for WaveSpawnerGadget {
    fn name(&self) -> &'static str {
        "WaveSpawner"
    }

    fn game_frame(&self, ctx: &spring_addons::AddonContext<'_, Global>, frame: i32) {
        spawn_static(ctx.global());
        update_wave(ctx.global(), frame);
    }
}
