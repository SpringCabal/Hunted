mod eating;
mod memory;
mod movement;
mod senses;

use alloc::rc::Rc;
use alloc::vec::Vec;
use core::cell::RefCell;

use hunted_common::rabbit::*;
use hunted_common::scare::{self, Scary, Source};
use spring::commands;
use spring::random::synced as random;
use spring_addons::AddonContext;
use spring_addons::rules::{Gadget, UnitDestroyedEvent};

use crate::defs::{self, UnitKind};
use crate::diagnostics;
use crate::state::{self, Global};
use crate::unit_scripts::Call;
use crate::units;

use eating::*;
use movement::*;
use senses::*;

pub use memory::{RabbitCtx, RabbitMemory, Rabbits, with_rabbit_ctx};

const EAT_TIME: i32 = 100;
const DROPPED_EAT_TIME: i32 = 10;

const GOAL_RADIUS: f32 = 60.0;

struct Surroundings {
    carrots: Vec<(i32, Position, i32)>,
    field_carrots: Vec<Vec<i32>>,
    mines: Vec<(i32, Position)>,
    burrows: Vec<(i32, Position)>,
    lights: Vec<Source>,
    blasts: Vec<Source>,
}

struct RabbitTick {
    rabbit: i32,
    position: Position,
    update_gap: i32,
    original_reverse_speed: f32,
    scary: Option<Scary>,
    speed_multiplier: f32,
}

struct Pickup {
    tick: RabbitTick,
    carrot_id: i32,
    eating_progress: i32,
}

pub fn update(global: &state::Global, frame: i32) {
    let roster = global.roster.access();
    let mut carrots = food_units_with_positions(&roster, UnitKind::Carrot, EAT_TIME);
    let field_carrots = carrots_by_field(&carrots);
    carrots.extend(food_units_with_positions(
        &roster,
        UnitKind::CarrotDropped,
        DROPPED_EAT_TIME,
    ));
    let surroundings = Rc::new(Surroundings {
        carrots,
        field_carrots,
        mines: roster.positions(UnitKind::Mine),
        burrows: roster.positions(UnitKind::Burrow),
        lights: lighthouse_beams(&roster)
            .into_iter()
            .chain(flashlight(global))
            .collect(),
        blasts: core::mem::take(&mut global.scares.access_mut().0),
    });
    let rabbits = roster.units(UnitKind::Rabbit);
    drop(roster);
    diagnostics::rabbit_motion(frame, &rabbits);
    let consumed_carrots = Rc::new(RefCell::new(Vec::new()));

    with_rabbit_ctx(global, |game| {
        for rabbit in rabbits {
            let pickup = update_rabbit(
                game,
                frame,
                rabbit,
                &surroundings,
                &mut consumed_carrots.borrow_mut(),
            );
            if let Some(pickup) = pickup {
                pick_up(
                    pickup,
                    Rc::clone(&surroundings),
                    Rc::clone(&consumed_carrots),
                );
            }
        }
    });
}

fn pick_up(
    pickup: Pickup,
    surroundings: Rc<Surroundings>,
    consumed_carrots: Rc<RefCell<Vec<i32>>>,
) {
    let (rabbit, carrot) = (pickup.tick.rabbit, pickup.carrot_id);
    let position = spring::Float3 {
        y: pickup.tick.position.y + 8.0,
        ..pickup.tick.position
    };
    commands::queue(move || {
        let in_play = crate::with_global(|global| {
            let roster = global.roster.access();
            roster.kind(rabbit) == Some(UnitKind::Rabbit) && roster.kind(carrot).is_some()
        });
        if !in_play {
            return None;
        }
        units::create_now(UnitKind::CarrotCarried, position, 0, units::FACING_SOUTH)
    })
    .then(move |carried_id| {
        crate::with_global(|global| {
            with_rabbit_ctx(global, |game| {
                finish_pickup(
                    game,
                    pickup,
                    carried_id,
                    &surroundings,
                    &mut consumed_carrots.borrow_mut(),
                );
            });
        });
    });
}

pub fn destroyed(game: &mut RabbitCtx<'_>, rabbit: i32) {
    let Some(memory_index) = game.rabbits.find(rabbit) else {
        return;
    };
    let memory = game.rabbits.memories[memory_index];
    if memory.stealing_notified && memory.reserved_carrot >= 0 {
        remember_carrot_progress(game.rabbits, &memory);
        Call::carrot_stop_being_stolen(memory.reserved_carrot, memory.eating_progress).queue();
    }
    if memory.carried_carrot >= 0 {
        drop_carrot(rabbit, memory.carried_carrot);
    }
    game.rabbits.retire(rabbit);
}

pub fn food_destroyed(game: &mut RabbitCtx<'_>, food: i32) {
    if let Some(memory_index) = game.rabbits.reserving(food) {
        cancel_eating(game, memory_index);
        game.rabbits.clear_carrot_progress(food);
    }
    if let Some(memory) = game
        .rabbits
        .memories
        .iter_mut()
        .find(|memory| memory.carried_carrot == food)
    {
        memory.carried_carrot = -1;
        memory.carrying_since = -1;
    }
}

fn update_rabbit(
    game: &mut RabbitCtx<'_>,
    frame: i32,
    rabbit: i32,
    surroundings: &Surroundings,
    consumed_carrots: &mut Vec<i32>,
) -> Option<Pickup> {
    let memory_index = game.rabbits.memory_index(rabbit, frame);
    let due = frame >= game.rabbits.memories[memory_index].next_update;
    if !due && surroundings.blasts.is_empty() {
        return None;
    }
    let position = units::unit_position(rabbit)?;
    let blasts: Vec<Scary> = surroundings
        .blasts
        .iter()
        .filter_map(|blast| blast.felt_at(position.x, position.z))
        .collect();
    if !due && blasts.is_empty() {
        return None;
    }
    let original_reverse_speed = original_reverse_speed(&mut game.rabbits.memories[memory_index]);
    let update_gap = {
        let memory = &mut game.rabbits.memories[memory_index];
        let gap = (frame - memory.last_update).max(0);
        memory.last_update = frame;
        memory.next_update = frame + 5 + (random::float() * 10.0).ceil() as i32;
        gap
    };

    let (panic_resistance, wave_speed) = (game.panic_resistance, game.speed_multiplier);
    let light_factor = carrier_light_factor(&game.rabbits.memories[memory_index], frame);
    let data = &mut game.rabbits.memories[memory_index].data;
    let (scary, speed_multiplier) = if blasts.is_empty() {
        let light = scare::scariest(&surroundings.lights, position.x, position.z, light_factor);
        let magnitude = light.map_or(0.0, |light| light.magnitude);
        data.update_fear(
            light,
            magnitude * update_gap as f32,
            update_gap,
            panic_resistance,
        );
        (light, data.update_motion(magnitude, update_gap, wave_speed))
    } else {
        let mut gap = update_gap;
        let mut speed = 0.0;
        for blast in &blasts {
            data.update_fear(Some(*blast), blast.magnitude, gap, panic_resistance);
            speed = data.update_motion(blast.magnitude, gap, wave_speed);
            gap = 0;
        }
        (blasts.last().copied(), speed)
    };

    if data.drops_carrot(panic_resistance)
        && game.rabbits.memories[memory_index].carried_carrot >= 0
    {
        drop_carried_carrot(game, memory_index);
    }

    if update_carried_rabbit(
        game,
        memory_index,
        &position,
        original_reverse_speed,
        &surroundings.burrows,
    ) {
        return None;
    }

    let tick = RabbitTick {
        rabbit,
        position,
        update_gap,
        original_reverse_speed,
        scary,
        speed_multiplier,
    };
    match update_eating_rabbit(game, memory_index, &tick, &surroundings.carrots) {
        Eating::No => {}
        Eating::Busy => return None,
        Eating::Finished {
            carrot_id,
            eating_progress,
        } => {
            consumed_carrots.push(carrot_id);
            return Some(Pickup {
                tick,
                carrot_id,
                eating_progress,
            });
        }
    }
    update_movement(game, memory_index, &tick, surroundings, consumed_carrots);
    None
}

fn original_reverse_speed(memory: &mut RabbitMemory) -> f32 {
    if memory.original_reverse_speed < 0.0 {
        memory.original_reverse_speed = spring::move_ctrl::get_unit_move_type_data(memory.unit_id)
            .ok()
            .filter(|data| data.name == "ground")
            .map_or(0.0, |data| data.max_reverse_speed);
    }
    memory.original_reverse_speed
}

fn carrier_light_factor(memory: &RabbitMemory, frame: i32) -> f32 {
    if memory.carried_carrot < 0 {
        return 1.0;
    }
    memory
        .carrying_since
        .checked_add(CARRIER_LIGHT_FADE_FRAMES)
        .map_or(1.0, |ignore_lights_after| {
            if frame <= ignore_lights_after {
                1.0
            } else {
                (1.0 - (frame - ignore_lights_after) as f32 / CARRIER_LIGHT_FADE_FRAMES as f32)
                    .clamp(0.0, 1.0)
            }
        })
}

fn finish_pickup(
    game: &mut RabbitCtx<'_>,
    pickup: Pickup,
    carried_id: Option<i32>,
    surroundings: &Surroundings,
    consumed_carrots: &mut Vec<i32>,
) {
    let Pickup {
        tick,
        carrot_id,
        eating_progress,
    } = pickup;
    let Some(memory_index) = game.rabbits.find(tick.rabbit) else {
        return;
    };
    let Some(carried_id) = carried_id else {
        consumed_carrots.retain(|id| *id != carrot_id);
        let memory = &mut game.rabbits.memories[memory_index];
        memory.eating_progress = memory.reserved_eat_time - tick.update_gap;
        return;
    };
    Call::carrot_stop_being_stolen(carrot_id, eating_progress).queue();
    Call::carrot_consume(carrot_id).queue();
    game.rabbits.clear_carrot_progress(carrot_id);
    set_stealing(tick.rabbit, false);
    game.rabbits.release_reservation(memory_index);
    let memory = &mut game.rabbits.memories[memory_index];
    memory.carried_carrot = carried_id;
    memory.carrying_since = memory.last_update;
    memory.reserved_eat_time = EAT_TIME;
    memory.eating_progress = 0;
    memory.stealing_notified = false;
    memory.data.boldness += STEAL_BOLDNESS_BONUS;
    Call::rabbit_attach_carrot(tick.rabbit, carried_id).queue();
    let _ = spring::spawn_ceg(
        "dirtfling",
        tick.position,
        spring::Float3::default(),
        0.0,
        0.0,
        0.0,
    );
    crate::diagnostics::info_once!("rabbit finished eating and is carrying a carrot");
    update_movement(game, memory_index, &tick, surroundings, consumed_carrots);
}

type Position = spring::Float3;

pub(super) fn set_stealing(rabbit: i32, active: bool) {
    use spring::ResultExt;
    spring::set_unit_rules_param_f32(
        rabbit,
        "stealing",
        if active { 1.0 } else { 0.0 },
        spring::rules_params::los::PUBLIC,
    )
    .log_err_once("publishing a rabbit's stealing state");
}

pub struct RabbitAiGadget;

impl Gadget<Global> for RabbitAiGadget {
    fn name(&self) -> &'static str {
        "RabbitAI"
    }

    fn game_frame(&self, _ctx: &AddonContext<'_, Global>, frame: i32) {
        spring::commands::run(move || crate::with_global(|global| update(global, frame)));
    }

    fn unit_destroyed(&self, ctx: &AddonContext<'_, Global>, event: &UnitDestroyedEvent) {
        if super::restarting(ctx.global()) {
            return;
        }
        let unit = event.unit_id;
        match defs::kind_of(event.unit_def_id) {
            Some(UnitKind::Rabbit) => {
                with_rabbit_ctx(ctx.global(), |game| destroyed(game, unit));
            }
            Some(kind) if kind.is_carrot() => {
                with_rabbit_ctx(ctx.global(), |game| food_destroyed(game, unit));
            }
            _ => {}
        }
    }
}
