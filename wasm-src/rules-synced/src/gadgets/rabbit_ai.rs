mod eating;
pub(crate) mod jobs;
mod memory;
mod movement;
mod senses;

use alloc::rc::Rc;
use alloc::vec::Vec;

use hunted_common::rabbit::*;
use hunted_common::scare::{self, Scary, Source};

use spring_addons::AddonContext;
use spring_addons::rules::{Gadget, UnitDestroyedEvent};

use crate::defs::{self, UnitKind};
use crate::diagnostics;
use crate::state::{self, Global};
use crate::unit_scripts::Call;
use crate::units;

pub(crate) use eating::drop_carrot;
use eating::*;
use movement::*;
use senses::*;

pub use memory::{RabbitCtx, RabbitMemory, Rabbits, with_rabbit_ctx};

const EAT_TIME: i32 = 100;
const DROPPED_EAT_TIME: i32 = 10;

const GOAL_RADIUS: f32 = 60.0;

struct Surroundings {
    static_data: Rc<StaticSurroundings>,
    lights: Vec<Source>,
}

struct StaticSurroundings {
    carrots: Vec<(i32, Position, i32)>,
    desirables: hunted_common::desired::Desirables,
    mines: Vec<(i32, Position)>,
    burrows: Vec<(i32, Position)>,
    food_grid: hunted_common::spatial::Grid,
}

#[derive(Default)]
pub struct SenseCache(Option<(u64, Rc<StaticSurroundings>)>);

impl SenseCache {
    fn snapshot(&mut self, roster: &crate::roster::Roster) -> Rc<StaticSurroundings> {
        if let Some((revision, data)) = &self.0
            && *revision == roster.static_revision()
        {
            return Rc::clone(data);
        }
        let mut carrots = food_units_with_positions(roster, UnitKind::Carrot, EAT_TIME);
        carrots.extend(food_units_with_positions(
            roster,
            UnitKind::CarrotDropped,
            DROPPED_EAT_TIME,
        ));
        carrots.sort_unstable_by_key(|(unit, _, _)| {
            roster
                .desirables
                .rank(hunted_common::desired::Key::Unit(*unit))
        });
        let food_grid = hunted_common::spatial::Grid::new(
            FOOD_RADIUS,
            carrots.iter().map(|(_, p, _)| (p.x, p.z)),
        );
        let data = Rc::new(StaticSurroundings {
            carrots,
            desirables: roster.desirables.clone(),
            food_grid,
            mines: ordered_positions(roster, UnitKind::Mine),
            burrows: ordered_positions(roster, UnitKind::Burrow),
        });
        self.0 = Some((roster.static_revision(), Rc::clone(&data)));
        data
    }
}

struct RabbitTick {
    rabbit: i32,
    position: Position,
    update_gap: i32,
    original_reverse_speed: f32,
    scary: Option<Scary>,
    speed_multiplier: f32,
    goal: (Option<CarrotChoice>, Option<senses::Target>),
}

struct Pickup {
    tick: RabbitTick,
    carrot_id: i32,
    eating_progress: i32,
}

enum AfterRabbit {
    Pickup(Pickup),
    Score(i32),
}

pub fn update(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    global: &state::Global,
    frame: i32,
) {
    let roster = global.roster.access();
    let surroundings = Rc::new(Surroundings {
        static_data: global.senses.access_mut().snapshot(&roster),
        lights: flashlight(global)
            .into_iter()
            .chain(lighthouse_beams(global))
            .collect(),
    });
    let rabbits = roster.units(UnitKind::Rabbit);
    drop(roster);
    diagnostics::rabbit_motion(spring, frame, &rabbits);
    let mut consumed_carrots = Vec::new();
    let blasts = core::mem::take(&mut global.scares.access_mut().0);
    // Finish a rabbit's synchronous engine effects before the next rabbit.
    // In particular, pickup destroys the original food and changes tie ranks.
    let mut surroundings = surroundings;
    for blast in blasts {
        for &rabbit in &rabbits {
            let Some(position) = units::unit_position(spring, rabbit) else {
                continue;
            };
            let Some(scary) = blast.felt_at(position.x, position.z) else {
                continue;
            };
            let pickup = with_rabbit_ctx(global, |game| {
                update_rabbit(
                    spring,
                    game,
                    frame,
                    rabbit,
                    &surroundings,
                    &mut consumed_carrots,
                    None,
                    Some(scary),
                )
            });
            if let Some(pickup) = pickup {
                finish_action(spring, global, pickup, &mut consumed_carrots);
                refresh_surroundings(global, &mut surroundings);
            }
        }
    }
    let prepared = jobs::prepare(spring, global, &rabbits, &surroundings, frame);
    for (index, rabbit) in rabbits.into_iter().enumerate() {
        let pickup = with_rabbit_ctx(global, |game| {
            update_rabbit(
                spring,
                game,
                frame,
                rabbit,
                &surroundings,
                &mut consumed_carrots,
                prepared.as_ref().and_then(|all| all[index].as_ref()),
                None,
            )
        });
        if let Some(pickup) = pickup {
            finish_action(spring, global, pickup, &mut consumed_carrots);
            refresh_surroundings(global, &mut surroundings);
        }
    }
}

fn refresh_surroundings(global: &Global, surroundings: &mut Rc<Surroundings>) {
    let roster = global.roster.access();
    Rc::get_mut(surroundings)
        .expect("AI owns its snapshot")
        .static_data = global.senses.access_mut().snapshot(&roster);
}

fn finish_action(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    global: &Global,
    action: AfterRabbit,
    consumed: &mut Vec<i32>,
) {
    match action {
        AfterRabbit::Pickup(pickup) => pick_up(spring, global, pickup, consumed),
        AfterRabbit::Score(carried) => {
            use spring::ResultExt;
            spring
                .rules_params()
                .set_unit_rules_param_f32(
                    carried,
                    "carrotScored",
                    1.0,
                    spring::rules_params::los::PUBLIC,
                )
                .log_err_once(spring, "marking a returned carrot");
            units::destroy_now(spring, carried, false, -1);
        }
    }
}

fn pick_up(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    global: &Global,
    pickup: Pickup,
    consumed_carrots: &mut Vec<i32>,
) {
    let (rabbit, carrot) = (pickup.tick.rabbit, pickup.carrot_id);
    let in_play = {
        let roster = global.roster.access();
        roster.kind(rabbit) == Some(UnitKind::Rabbit) && roster.kind(carrot).is_some()
    };
    if !in_play {
        return;
    }
    // unit_position already requests the model midpoint, as Lua does.
    let position = pickup.tick.position;
    // No application resource borrow crosses CreateUnit/DestroyUnit: both can
    // synchronously re-enter the gadget handlers.
    let carried_id = units::create_now(
        spring,
        UnitKind::CarrotCarried,
        position,
        0,
        units::FACING_SOUTH,
    );
    if let Some(carried) = carried_id {
        Call::rabbit_attach_carrot(rabbit, carried).run(spring);
        Call::carrot_stop_being_stolen(carrot, pickup.eating_progress).run(spring);
        with_rabbit_ctx(global, |game| {
            if let Some(index) = game.rabbits.find(rabbit) {
                game.rabbits.release_reservation(index);
                game.rabbits.memories[index].stealing_notified = false;
            }
        });
        units::destroy_now(spring, carrot, false, -1);
    }
    with_rabbit_ctx(global, |game| {
        finish_pickup(spring, game, pickup, carried_id, consumed_carrots)
    });
}

pub fn destroyed(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    game: &mut RabbitCtx<'_>,
    rabbit: i32,
) {
    let Some(memory_index) = game.rabbits.find(rabbit) else {
        return;
    };
    let memory = game.rabbits.memories[memory_index];
    if memory.stealing_notified && memory.reserved_carrot >= 0 {
        remember_carrot_progress(game.rabbits, &memory);
        Call::carrot_stop_being_stolen(memory.reserved_carrot, memory.eating_progress).queue();
    }
    if memory.carried_carrot >= 0 {
        drop_carrot(spring, rabbit, memory.carried_carrot);
    }
    game.rabbits.retire(rabbit);
}

pub fn food_destroyed(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    game: &mut RabbitCtx<'_>,
    food: i32,
) {
    if let Some(memory_index) = game.rabbits.reserving(food) {
        cancel_eating(spring, game, memory_index);
    }
    game.rabbits.clear_carrot_progress(food);
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
    spring: &spring::Spring<'_, spring::RulesSynced>,
    game: &mut RabbitCtx<'_>,
    frame: i32,
    rabbit: i32,
    surroundings: &Surroundings,
    consumed_carrots: &mut Vec<i32>,
    prepared: Option<&jobs::Prepared>,
    override_scare: Option<Scary>,
) -> Option<AfterRabbit> {
    let memory_index = game.rabbits.memory_index(rabbit, frame);
    let due = frame >= game.rabbits.memories[memory_index].next_update;
    if !due && override_scare.is_none() {
        return None;
    }
    let position = prepared
        .map(|value| value.position)
        .or_else(|| units::unit_position(spring, rabbit))?;
    if prepared.is_some_and(|value| value.checking) {
        let live = units::unit_position(spring, rabbit)
            .expect("prepared rabbit position remains available");
        assert_eq!(
            (
                position.x.to_bits(),
                position.y.to_bits(),
                position.z.to_bits()
            ),
            (live.x.to_bits(), live.y.to_bits(), live.z.to_bits()),
            "rabbit job snapshot changed before use"
        );
    }
    let original_reverse_speed =
        original_reverse_speed(spring, &mut game.rabbits.memories[memory_index]);
    let update_gap = {
        let memory = &mut game.rabbits.memories[memory_index];
        let gap = (frame - memory.last_update).max(0);
        memory.last_update = frame;
        memory.next_update = next_update_frame(
            frame,
            spring
                .synced_random()
                .next_float()
                .expect("synced random float"),
        );
        gap
    };

    let (panic_resistance, wave_speed) = (game.panic_resistance, game.speed_multiplier);
    let light_factor = carrier_light_factor(&game.rabbits.memories[memory_index], frame);
    let light = override_scare.or_else(|| {
        prepared.map_or_else(
            || scare::scariest(&surroundings.lights, position.x, position.z, light_factor),
            |value| value.light,
        )
    });
    if override_scare.is_none() && prepared.is_some_and(|value| value.checking) {
        assert_eq!(
            light,
            scare::scariest(&surroundings.lights, position.x, position.z, light_factor),
            "rabbit light state changed before use"
        );
    }
    let magnitude = light.map_or(0.0, |light| light.magnitude);
    let scary_fear = if override_scare.is_some() {
        magnitude
    } else {
        magnitude * update_gap as f32
    };
    game.rabbits.memories[memory_index].data.update_fear(
        Some(light.unwrap_or(Scary {
            x: position.x,
            z: position.z,
            magnitude: 0.0,
        })),
        scary_fear,
        update_gap,
        panic_resistance,
    );
    if game.rabbits.memories[memory_index]
        .data
        .drops_carrot(panic_resistance)
        && game.rabbits.memories[memory_index].carried_carrot >= 0
    {
        drop_carried_carrot(spring, game, memory_index);
    }
    let memory = game.rabbits.memories[memory_index];
    let expected_food = memory.carried_carrot >= 0 || memory.reserved_carrot >= 0;
    let goal = choose_target(
        game,
        rabbit,
        &position,
        expected_food,
        surroundings,
        consumed_carrots,
    );
    let data = &mut game.rabbits.memories[memory_index].data;
    data.update_boldness(goal.1.map_or(0.0, |(_, _, _, m)| m.max(0.0)), update_gap);
    let speed_multiplier = data.update_motion(magnitude, update_gap, wave_speed);

    let scary = light;

    if let Some(carried) =
        update_carried_rabbit(spring, game, memory_index, original_reverse_speed, goal.1)
    {
        return Some(AfterRabbit::Score(carried));
    }

    let tick = RabbitTick {
        rabbit,
        position,
        update_gap,
        original_reverse_speed,
        scary,
        speed_multiplier,
        goal,
    };
    match update_eating_rabbit(
        spring,
        game,
        memory_index,
        &tick,
        &surroundings.static_data.carrots,
    ) {
        Eating::No => {}
        Eating::Busy => return None,
        Eating::Finished {
            carrot_id,
            eating_progress,
        } => {
            consumed_carrots.push(carrot_id);
            return Some(AfterRabbit::Pickup(Pickup {
                tick,
                carrot_id,
                eating_progress,
            }));
        }
    }
    update_movement(spring, game, memory_index, &tick);
    None
}

fn original_reverse_speed(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    memory: &mut RabbitMemory,
) -> f32 {
    if memory.original_reverse_speed < 0.0 {
        memory.original_reverse_speed = spring
            .move_ctrl()
            .get_unit_move_type_data(memory.unit_id)
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
    let carry_seconds = (frame - memory.carrying_since) as f32 / 30.0;
    ((20.0 - carry_seconds) / 10.0).clamp(0.0, 1.0)
}

fn finish_pickup(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    game: &mut RabbitCtx<'_>,
    pickup: Pickup,
    carried_id: Option<i32>,
    consumed_carrots: &mut Vec<i32>,
) {
    let Pickup {
        tick,
        carrot_id,
        eating_progress: _,
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

    game.rabbits.clear_carrot_progress(carrot_id);
    set_stealing(spring, tick.rabbit, false);
    game.rabbits.release_reservation(memory_index);
    let memory = &mut game.rabbits.memories[memory_index];
    memory.carried_carrot = carried_id;
    memory.carrying_since = memory.last_update;
    memory.reserved_eat_time = EAT_TIME;
    memory.eating_progress = 0;
    memory.stealing_notified = false;
    memory.data.boldness += STEAL_BOLDNESS_BONUS;
    let _ = spring.effects_control().spawn_ceg(
        &spring::effects_control::DefRef {
            name: "dirtfling".into(),
            id: -1,
        },
        tick.position,
        spring::Float3::default(),
        0.0,
        0.0,
        0.0,
    );
    crate::diagnostics::info_once!(spring, "rabbit finished eating and is carrying a carrot");
    update_movement(spring, game, memory_index, &tick);
}

type Position = spring::Float3;

pub(super) fn set_stealing(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    rabbit: i32,
    active: bool,
) {
    use spring::ResultExt;
    spring
        .rules_params()
        .set_unit_rules_param_f32(
            rabbit,
            "stealing",
            if active { 1.0 } else { 0.0 },
            spring::rules_params::los::PUBLIC,
        )
        .log_err_once(spring, "publishing a rabbit's stealing state");
}

pub struct RabbitAiGadget;

impl Gadget<Global> for RabbitAiGadget {
    fn name(&self) -> &'static str {
        "RabbitAI"
    }

    fn game_frame(&self, _ctx: &AddonContext<'_, Global>, frame: i32) {
        let _spring = _ctx.spring();
        crate::with_addon_context(|ctx| {
            ctx.commands()
                .run(move |spring| crate::with_global(|global| update(spring, global, frame)))
        });
    }

    fn unit_created(
        &self,
        ctx: &AddonContext<'_, Global>,
        id: i32,
        def: i32,
        _team: i32,
        _builder: i32,
    ) {
        if defs::kind_of(ctx.spring(), def) == Some(UnitKind::Rabbit) {
            let frame = ctx.spring().game().current_frame().unwrap_or(0);
            ctx.global().rabbits.access_mut().memory_index(id, frame);
            let reverse = ctx
                .spring()
                .move_ctrl()
                .get_unit_move_type_data(id)
                .map_or(0.0, |d| d.max_reverse_speed);
            units::set_rabbit_movement_attributes(ctx.spring(), id, 1.0, 1.0, 1.0, reverse);
        } else if defs::kind_of(ctx.spring(), def) == Some(UnitKind::CarrotCarried) {
            use spring::ResultExt;
            // Lua's Attributes gadget applies its world speed scale to every
            // mobile definition, including the attached carrot.
            ctx.spring()
                .move_ctrl()
                .set_ground_move_type_max_speed(
                    id,
                    defs::defs(ctx.spring()).carried_carrot_speed * 0.05,
                )
                .log_err_once(ctx.spring(), "setting a carried carrot's max speed");
        }
    }

    fn unit_destroyed(&self, ctx: &AddonContext<'_, Global>, event: &UnitDestroyedEvent) {
        let spring = ctx.spring();
        if super::restarting(ctx.global()) {
            return;
        }
        let unit = event.unit_id;
        match defs::kind_of(spring, event.unit_def_id) {
            Some(UnitKind::Rabbit) => {
                with_rabbit_ctx(ctx.global(), |game| destroyed(spring, game, unit));
            }
            Some(kind) if kind.is_carrot() => {
                with_rabbit_ctx(ctx.global(), |game| food_destroyed(spring, game, unit));
            }
            _ => {}
        }
    }
}

#[derive(Default)]
pub struct LighthouseBeams(Vec<(i32, Source)>);

impl LighthouseBeams {
    pub fn set(&mut self, unit: i32, x: f32, z: f32) {
        let source = Source {
            x,
            z,
            area: scare::Area::LIGHTHOUSE,
        };
        if let Some((_, current)) = self.0.iter_mut().find(|(id, _)| *id == unit) {
            *current = source;
        } else {
            self.0.push((unit, source));
        }
    }
    pub fn forget(&mut self, unit: i32) {
        if let Some(index) = self.0.iter().position(|(id, _)| *id == unit) {
            self.0.swap_remove(index);
        }
    }
    pub fn sources(&self) -> Vec<Source> {
        self.0.iter().map(|(_, source)| *source).collect()
    }
}
