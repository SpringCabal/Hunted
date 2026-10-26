use super::*;

pub(super) fn update_carried_rabbit(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    game: &mut RabbitCtx<'_>,
    memory_index: usize,
    original_reverse_speed: f32,
    target: Option<Target>,
) -> Option<i32> {
    let memory = game.rabbits.memories[memory_index];
    if memory.data.panic.is_some() || !memory.data.moves_to_goal(game.panic_resistance, true) {
        return None;
    }
    let carried_id = memory.carried_carrot;
    if carried_id < 0 {
        return None;
    }

    let (x, z, distance_sq, _) = target?;
    if distance_sq.sqrt() < GOAL_RADIUS {
        let rabbit_id = game.rabbits.memories[memory_index].unit_id;
        game.rabbits.memories[memory_index].carried_carrot = -1;
        game.rabbits.memories[memory_index].carrying_since = -1;
        *game.carrots_stolen += 1;
        crate::diagnostics::info_once!(spring, "rabbit returned a carrot to a burrow");
        issue_goal(
            spring,
            rabbit_id,
            x,
            z,
            0.05,
            2.0,
            1.0,
            original_reverse_speed,
        );
        Some(carried_id)
    } else {
        None
    }
}

pub(super) fn drop_carried_carrot(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    game: &mut RabbitCtx<'_>,
    memory_index: usize,
) {
    let carried_id = game.rabbits.memories[memory_index].carried_carrot;
    if carried_id < 0 {
        return;
    }
    let rabbit_id = game.rabbits.memories[memory_index].unit_id;
    drop_carrot(spring, rabbit_id, carried_id);
    game.rabbits.memories[memory_index].carried_carrot = -1;
    game.rabbits.memories[memory_index].carrying_since = -1;
}

pub(crate) fn drop_carrot(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    rabbit_id: i32,
    carried_id: i32,
) {
    use spring::ResultExt;
    // Read before the AI changes movement attributes, including death callbacks.
    let mut velocity = spring
        .units_info()
        .get_unit_velocity(rabbit_id)
        .log_err_once(spring, "reading a carrot carrier's velocity")
        .unwrap_or_default();
    let square = velocity.x * velocity.x + velocity.y * velocity.y + velocity.z * velocity.z;
    if square > 50.0 {
        let speed = square.sqrt();
        velocity.x /= speed;
        velocity.y /= speed;
        velocity.z /= speed;
    }
    crate::with_addon_context(|ctx| {
        ctx.commands().run(move |spring| {
            Call::rabbit_drop_carrot(rabbit_id, carried_id).run(spring);
            // The script queues its engine detachment. Append the remaining work
            // after it, with no CUS or application borrow crossing engine callbacks.
            crate::with_addon_context(|ctx| {
                ctx.commands().run(move |spring| {
                    crate::gadgets::blocking::make_non_blocking(spring, rabbit_id);
                    Call::carrot_be_dropped(carried_id).run(spring);
                    spring
                        .unit_control()
                        .set_unit_velocity(carried_id, velocity)
                        .log_err_once(spring, "transferring a dropped carrot's momentum");
                })
            });
        })
    });
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
        Call::carrot_start_being_stolen(carrot_id, eating_progress).run(spring);
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
            Call::carrot_stop_being_stolen(carrot_id, memory.eating_progress).run(spring);
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
