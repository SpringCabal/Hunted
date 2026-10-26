use super::*;

pub(super) fn update_carried_rabbit(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    game: &mut RabbitCtx<'_>,
    memory_index: usize,
    position: &Position,
    original_reverse_speed: f32,
    burrows: &[(i32, Position)],
) -> bool {
    let memory = game.rabbits.memories[memory_index];
    if memory.data.panic.is_some() || !memory.data.moves_to_goal(game.panic_resistance, true) {
        return false;
    }
    let carried_id = memory.carried_carrot;
    if carried_id < 0 {
        return false;
    }

    let Some((_, burrow, distance_sq)) = units::nearest(*position, burrows) else {
        return false;
    };
    if distance_sq.sqrt() < GOAL_RADIUS {
        Call::carrot_score(carried_id).queue();
        let rabbit_id = game.rabbits.memories[memory_index].unit_id;
        game.rabbits.memories[memory_index].carried_carrot = -1;
        game.rabbits.memories[memory_index].carrying_since = -1;
        *game.carrots_stolen += 1;
        crate::diagnostics::info_once!(spring, "rabbit returned a carrot to a burrow");
        issue_goal(
            spring,
            rabbit_id,
            burrow.x,
            burrow.z,
            0.05,
            2.0,
            1.0,
            original_reverse_speed,
        );
        true
    } else {
        false
    }
}

pub(super) fn drop_carried_carrot(game: &mut RabbitCtx<'_>, memory_index: usize) {
    let carried_id = game.rabbits.memories[memory_index].carried_carrot;
    if carried_id < 0 {
        return;
    }
    let rabbit_id = game.rabbits.memories[memory_index].unit_id;
    drop_carrot(rabbit_id, carried_id);
    game.rabbits.memories[memory_index].carried_carrot = -1;
    game.rabbits.memories[memory_index].carrying_since = -1;
}

pub(super) fn drop_carrot(rabbit_id: i32, carried_id: i32) {
    Call::rabbit_drop_carrot(rabbit_id, carried_id).queue();
    Call::carrot_be_dropped(carried_id).queue();
}

pub(super) enum Eating {
    No,
    Busy,
    Finished {
        carrot_id: i32,
        eating_progress: i32,
    },
}

pub(super) fn update_eating_rabbit(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    game: &mut RabbitCtx<'_>,
    memory_index: usize,
    tick: &RabbitTick,
    carrots: &[(i32, Position, i32)],
) -> Eating {
    let carrot_id = game.rabbits.memories[memory_index].reserved_carrot;
    if carrot_id < 0 {
        return Eating::No;
    }
    if game.rabbits.memories[memory_index].data.panic.is_some() {
        cancel_eating(spring, game, memory_index);
        return Eating::No;
    }
    let Some((_, carrot_position, _)) = carrots.iter().find(|(id, _, _)| *id == carrot_id) else {
        cancel_eating(spring, game, memory_index);
        return Eating::No;
    };

    let (eating_progress, reserved_eat_time, stealing_notified) = {
        let memory = &mut game.rabbits.memories[memory_index];
        memory.eating_progress += tick.update_gap;
        (
            memory.eating_progress,
            memory.reserved_eat_time,
            memory.stealing_notified,
        )
    };
    if eating_progress > reserved_eat_time {
        return Eating::Finished {
            carrot_id,
            eating_progress,
        };
    }
    if !stealing_notified {
        game.rabbits.memories[memory_index].stealing_notified = true;
        set_stealing(spring, tick.rabbit, true);
        Call::carrot_start_being_stolen(carrot_id, eating_progress).queue();
    }
    issue_goal(
        spring,
        tick.rabbit,
        carrot_position.x,
        carrot_position.z,
        0.05,
        2.0,
        0.2,
        tick.original_reverse_speed,
    );
    Eating::Busy
}

pub(super) fn cancel_eating(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    game: &mut RabbitCtx<'_>,
    memory_index: usize,
) {
    let memory = game.rabbits.memories[memory_index];
    if memory.stealing_notified {
        let carrot_id = memory.reserved_carrot;
        if carrot_id >= 0 {
            remember_carrot_progress(game.rabbits, &memory);
            Call::carrot_stop_being_stolen(carrot_id, memory.eating_progress).queue();
        }
    }
    game.rabbits.release_reservation(memory_index);
    let memory = &mut game.rabbits.memories[memory_index];
    memory.eating_progress = 0;
    memory.stealing_notified = false;
    set_stealing(spring, memory.unit_id, false);
}

pub(super) fn remember_carrot_progress(rabbits: &mut Rabbits, memory: &RabbitMemory) {
    if memory.reserved_eat_time == EAT_TIME && memory.reserved_carrot >= 0 {
        rabbits.set_carrot_progress(memory.reserved_carrot, memory.eating_progress);
    }
}
