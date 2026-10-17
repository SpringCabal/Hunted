use hunted_common::movement::{self, normalize};

use super::*;

pub(super) fn update_movement(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    game: &mut RabbitCtx<'_>,
    memory_index: usize,
    tick: &RabbitTick,
    surroundings: &Surroundings,
    consumed_carrots: &[i32],
) {
    let RabbitTick {
        rabbit,
        position,
        update_gap,
        original_reverse_speed,
        scary,
        speed_multiplier,
    } = *tick;
    let carried = game.rabbits.memories[memory_index].carried_carrot >= 0;
    let (carrot, target) = choose_target(
        game,
        rabbit,
        &position,
        carried,
        surroundings,
        consumed_carrots,
    );

    let goal_magnitude = target.map_or(0.0, |(_, _, _, magnitude)| magnitude.max(0.0));
    game.rabbits.memories[memory_index]
        .data
        .update_boldness(goal_magnitude, update_gap);

    if let Some((carrot_id, carrot_position, distance_sq, eating_time)) = carrot
        && target.is_some_and(|(x, z, _, _)| x == carrot_position.x && z == carrot_position.z)
        && distance_sq.sqrt() < GOAL_RADIUS
        && game.rabbits.memories[memory_index].data.panic.is_none()
    {
        let rabbit_id = game.rabbits.memories[memory_index].unit_id;
        let eating_progress = game.rabbits.carrot_progress(carrot_id);
        game.rabbits.reserve(memory_index, carrot_id);
        let memory = &mut game.rabbits.memories[memory_index];
        memory.reserved_eat_time = eating_time;
        memory.eating_progress = eating_progress + update_gap;
        memory.stealing_notified = true;
        set_stealing(spring, rabbit_id, true);
        Call::carrot_start_being_stolen(carrot_id, eating_progress).queue();
        issue_goal(
            spring,
            rabbit_id,
            carrot_position.x,
            carrot_position.z,
            0.05,
            2.0,
            0.2,
            original_reverse_speed,
        );
        return;
    }

    let (target_x, target_z) = target
        .map(|(x, z, _, _)| (x, z))
        .unwrap_or((position.x, position.z));
    let move_goal = movement_goal(
        spring,
        MovementInput {
            memory: game.rabbits.memories[memory_index],
            rabbit,
            position,
            target_x,
            target_z,
            scary,
            speed_mult: speed_multiplier,
            panic_resistance: game.panic_resistance,
        },
    );
    let jitter = game.rabbits.memories[memory_index].data.jitter();
    let (acceleration_multiplier, turn_multiplier) =
        if scary.is_some_and(|value| value.magnitude > PANIC_SCARE_MAGNITUDE) {
            (5.0 / jitter, 1.5)
        } else {
            (1.0 / jitter, move_goal.2.max(f32::MIN_POSITIVE).powf(-0.1))
        };
    issue_goal(
        spring,
        rabbit,
        move_goal.0,
        move_goal.1,
        move_goal.2,
        acceleration_multiplier,
        turn_multiplier,
        original_reverse_speed,
    );
}

pub(super) struct MovementInput {
    pub(super) memory: RabbitMemory,
    pub(super) rabbit: i32,
    pub(super) position: Position,
    pub(super) target_x: f32,
    pub(super) target_z: f32,
    pub(super) scary: Option<Scary>,
    pub(super) speed_mult: f32,
    pub(super) panic_resistance: f32,
}

pub(super) fn movement_goal(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    input: MovementInput,
) -> (f32, f32, f32) {
    let data = input.memory.data;
    let scary_point = data.panic.or(input.scary).map(|scary| (scary.x, scary.z));
    let move_vector = movement::move_vector(
        &data,
        (input.position.x, input.position.z),
        scary_point,
        (input.target_x, input.target_z),
        input.memory.carried_carrot >= 0,
        input.panic_resistance,
    );
    let speed_multiplier = input.speed_mult * move_vector.speed_factor;
    let movement_magnitude = move_vector.x.hypot(move_vector.z);
    let heading = spring::Float3::new(move_vector.x, 0.0, move_vector.z).heading();
    let movement_direction =
        -f32::from(heading) / 32768.0 * core::f32::consts::PI + core::f32::consts::PI / 2.0;
    let direction_randomness =
        (core::f32::consts::PI * 0.4).min(20.0 * (10.0 + data.boldness).powf(-0.9));
    let movement_direction = movement_direction
        + spring
            .synced_random()
            .next_float()
            .expect("synced random float")
            * 2.0
            * direction_randomness
        - direction_randomness;
    let movement = (
        movement_magnitude * movement_direction.cos(),
        movement_magnitude * movement_direction.sin(),
    );
    let jitter = data.jitter();
    let velocity = spring
        .units_info()
        .get_unit_velocity(input.rabbit)
        .map(|velocity| {
            normalize(
                (10.0
                    + spring
                        .synced_random()
                        .next_float()
                        .expect("synced random float")
                        * 5.0)
                    * jitter,
                velocity.x,
                velocity.z,
            )
        })
        .unwrap_or((0.0, 0.0));
    let random_angle = spring
        .synced_random()
        .next_float()
        .expect("synced random float")
        * core::f32::consts::TAU;
    let random_magnitude = (5.0
        + spring
            .synced_random()
            .next_float()
            .expect("synced random float")
            * 10.0)
        * jitter;
    let random_vector = (
        random_angle.cos() * random_magnitude,
        random_angle.sin() * random_magnitude,
    );
    let movement = (
        movement.0 + random_vector.0 + velocity.0,
        movement.1 + random_vector.1 + velocity.1,
    );
    let movement = normalize(200.0 * speed_multiplier, movement.0, movement.1);
    let target = (
        crate::map::clamp_to_map(input.position.x + movement.0),
        crate::map::clamp_to_map(input.position.z + movement.1),
    );
    (target.0, target.1, speed_multiplier)
}

pub(super) fn issue_goal(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    unit_id: i32,
    target_x: f32,
    target_z: f32,
    speed_multiplier: f32,
    acceleration_multiplier: f32,
    turn_multiplier: f32,
    original_reverse_speed: f32,
) {
    units::set_rabbit_movement_attributes(
        spring,
        unit_id,
        speed_multiplier,
        acceleration_multiplier,
        turn_multiplier,
        original_reverse_speed,
    );
    units::set_goal(spring, unit_id, target_x, target_z, 0.0);
}
