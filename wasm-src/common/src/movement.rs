use crate::rabbit::RabbitData;

pub const NEAR_GOAL: f32 = 200.0;
const NEAR_GOAL_SPEED: f32 = 1.2;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MoveVector {
    pub x: f32,
    pub z: f32,
    pub speed_factor: f32,
}

pub fn move_vector(
    data: &RabbitData,
    position: (f32, f32),
    scary_point: Option<(f32, f32)>,
    goal: (f32, f32),
    carrying: bool,
    panic_resistance: f32,
) -> MoveVector {
    let flee = scary_point
        .map(|(x, z)| normalize(-data.fear / 8.0, x - position.0, z - position.1))
        .unwrap_or((0.0, 0.0));
    let goal_dx = goal.0 - position.0;
    let goal_dz = goal.1 - position.1;
    if !data.moves_to_goal(panic_resistance, carrying) {
        return MoveVector {
            x: flee.0,
            z: flee.1,
            speed_factor: 1.0,
        };
    }
    let (goal_weight, speed_factor) = if (goal_dx * goal_dx + goal_dz * goal_dz).sqrt() < NEAR_GOAL
    {
        (data.boldness / 10.0, NEAR_GOAL_SPEED)
    } else {
        (data.boldness / 30.0, 1.0)
    };
    let seek = normalize(goal_weight, goal_dx, goal_dz);
    MoveVector {
        x: flee.0 + seek.0,
        z: flee.1 + seek.1,
        speed_factor,
    }
}

pub fn normalize(magnitude: f32, x: f32, z: f32) -> (f32, f32) {
    let length = (x * x + z * z).sqrt();
    if length > 0.0 {
        (magnitude * x / length, magnitude * z / length)
    } else {
        (x, z)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_calm_rabbit_heads_for_its_goal() {
        let move_vector = move_vector(
            &RabbitData::NEW,
            (0.0, 0.0),
            None,
            (1000.0, 0.0),
            false,
            1.0,
        );
        assert!(move_vector.x > 0.0);
        assert!(move_vector.z.abs() < 1e-4);
        assert_eq!(move_vector.speed_factor, 1.0);
    }

    #[test]
    fn near_the_goal_the_pull_is_stronger() {
        let far = move_vector(
            &RabbitData::NEW,
            (0.0, 0.0),
            None,
            (1000.0, 0.0),
            false,
            1.0,
        );
        let near = move_vector(&RabbitData::NEW, (0.0, 0.0), None, (100.0, 0.0), false, 1.0);
        assert!((near.x - 3.0 * far.x).abs() < 1e-3);
        assert_eq!(near.speed_factor, NEAR_GOAL_SPEED);
    }

    #[test]
    fn a_terrified_rabbit_only_flees() {
        let data = RabbitData {
            fear: 400.0,
            ..RabbitData::NEW
        };
        let move_vector = move_vector(
            &data,
            (0.0, 0.0),
            Some((0.0, 10.0)),
            (0.0, 1000.0),
            false,
            1.0,
        );
        assert!(move_vector.x.abs() < 1e-4);
        assert!(
            (move_vector.z + 50.0).abs() < 1e-3,
            "fear / 8 away from the scare"
        );
        let near_goal = pull_toward(&data, (0.0, 100.0));
        assert_eq!(
            near_goal.speed_factor, 1.0,
            "no speed-up from an ignored goal"
        );
    }

    fn pull_toward(data: &RabbitData, goal: (f32, f32)) -> MoveVector {
        move_vector(data, (0.0, 0.0), Some((0.0, 10.0)), goal, false, 1.0)
    }

    #[test]
    fn a_carrier_keeps_heading_home_through_more_fear() {
        let data = RabbitData {
            fear: 200.0,
            ..RabbitData::NEW
        };
        let carrier = move_vector(
            &data,
            (0.0, 0.0),
            Some((0.0, -10.0)),
            (0.0, 1000.0),
            true,
            1.0,
        );
        let empty = move_vector(
            &data,
            (0.0, 0.0),
            Some((0.0, -10.0)),
            (0.0, 1000.0),
            false,
            1.0,
        );
        assert!(carrier.z > empty.z);
    }

    #[test]
    fn normalize_keeps_direction_and_ignores_zero() {
        assert_eq!(normalize(5.0, 3.0, 4.0), (3.0, 4.0));
        assert_eq!(normalize(5.0, 0.0, 0.0), (0.0, 0.0));
    }
}
