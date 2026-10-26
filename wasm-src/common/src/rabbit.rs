use crate::scare::Scary;

pub const FEAR_DECAY: f32 = 0.995;
pub const STAMINA_DECAY: f32 = 0.998;
pub const FEAR_ADDED: f32 = 0.3;
pub const BOLDNESS_ADDED: f32 = 0.3;

pub const DROP_CARROT_FEAR: f32 = 500.0;
pub const PANIC_START_FEAR: f32 = 250.0;
pub const PANIC_END_FEAR: f32 = 150.0;
pub const IGNORE_GOAL_FEAR: f32 = 180.0;
pub const CARRIER_FEAR_ALLOWANCE: f32 = 150.0;

pub const PANIC_SCARE_MAGNITUDE: f32 = 80.0;

pub const STEAL_BOLDNESS_BONUS: f32 = 150.0;

const BOLDNESS_SPEED_THRESHOLD: f32 = 320.0;
const FEAR_SPEED_DIVISOR: f32 = 45.0;
const FEAR_SPEED_EXPONENT: f32 = 0.8;
const BOLDNESS_SPEED_DIVISOR: f32 = 200.0;
const BOLDNESS_SPEED_EXPONENT: f32 = 0.45;
const BASE_SPEED_DIVISOR: f32 = 2.3;
const STAMINA_SPEED_DIVISOR: f32 = 150.0;
const SCARE_SPEED_THRESHOLD: f32 = 0.7;
const SPEED_MULTIPLIER: f32 = 1.8;
const STAMINA_COST_EXPONENT: f32 = 0.2;
const STAMINA_RECOVERY: f32 = 1.3;
const BOLDNESS_FEAR_DENOMINATOR: f32 = 10_000.0;
const BOLDNESS_FEAR_SCALE: f32 = 20.0;
const MIN_BOLDNESS_FEAR_DENOMINATOR: f32 = 2_000.0;
const BOLDNESS_DECAY: f32 = 0.99;
const JITTER_BOLDNESS_THRESHOLD: f32 = 300.0;
const JITTER_BOLDNESS_DIVISOR: f32 = 100.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RabbitData {
    pub fear: f32,
    pub stamina: f32,
    pub boldness: f32,
    pub panic: Option<Scary>,
}

const START_FEAR: f32 = 50.0;
const START_STAMINA: f32 = 100.0;
const START_BOLDNESS: f32 = 150.0;
const BOLDNESS_SPEED_BASE: f32 = 2.0;
const SCARE_SPEED_MULTIPLIER: f32 = 2.0;
const MAX_SCARE_SPEED: f32 = 2.0;

impl RabbitData {
    pub const NEW: RabbitData = RabbitData {
        fear: START_FEAR,
        stamina: START_STAMINA,
        boldness: START_BOLDNESS,
        panic: None,
    };

    pub fn update_fear(
        &mut self,
        scary: Option<Scary>,
        scary_fear: f32,
        gap: i32,
        panic_resistance: f32,
    ) {
        if self.panic.is_some()
            && let Some(scary) = scary
            && (scary.magnitude + PANIC_SCARE_MAGNITUDE * panic_resistance > self.fear
                || scary.magnitude > PANIC_SCARE_MAGNITUDE * panic_resistance)
        {
            self.panic = Some(scary);
        }
        self.fear = (self.fear + scary_fear + FEAR_ADDED * gap as f32) * FEAR_DECAY.powi(gap);
        if self.panic.is_some() {
            if self.fear < PANIC_END_FEAR {
                self.panic = None;
            }
        } else if self.fear > PANIC_START_FEAR * panic_resistance
            && let Some(scary) = scary
        {
            self.panic = Some(scary);
        }
    }

    pub fn update_motion(&mut self, scary_magnitude: f32, gap: i32, speed_multiplier: f32) -> f32 {
        let base_speed = ((self.fear + (self.boldness - BOLDNESS_SPEED_THRESHOLD).max(0.0))
            / FEAR_SPEED_DIVISOR)
            .max(0.0)
            .powf(FEAR_SPEED_EXPONENT)
            * (BOLDNESS_SPEED_BASE
                + (self.boldness / BOLDNESS_SPEED_DIVISOR)
                    .max(0.0)
                    .powf(BOLDNESS_SPEED_EXPONENT))
            / BASE_SPEED_DIVISOR
            * self.stamina
            / STAMINA_SPEED_DIVISOR;
        let sudden_speed = ((scary_magnitude - SCARE_SPEED_THRESHOLD) * SCARE_SPEED_MULTIPLIER)
            .clamp(1.0, MAX_SCARE_SPEED);
        let speed = base_speed * sudden_speed;
        self.stamina = (self.stamina - speed.max(0.0).powf(STAMINA_COST_EXPONENT) * gap as f32
            + STAMINA_RECOVERY * gap as f32)
            * STAMINA_DECAY.powi(gap);
        speed * SPEED_MULTIPLIER * speed_multiplier
    }

    pub fn update_boldness(&mut self, goal_magnitude: f32, gap: i32) {
        let denominator = (BOLDNESS_FEAR_DENOMINATOR - self.boldness * BOLDNESS_FEAR_SCALE)
            .max(MIN_BOLDNESS_FEAR_DENOMINATOR);
        let decay = 1.0 / (BOLDNESS_DECAY + self.fear / denominator);
        self.boldness = (self.boldness + goal_magnitude * gap as f32 + BOLDNESS_ADDED * gap as f32)
            * decay.powi(gap);
    }

    pub fn drops_carrot(&self, panic_resistance: f32) -> bool {
        self.fear > DROP_CARROT_FEAR * panic_resistance
    }

    pub fn moves_to_goal(&self, panic_resistance: f32, carrying: bool) -> bool {
        let allowance = if carrying {
            CARRIER_FEAR_ALLOWANCE
        } else {
            0.0
        };
        self.fear < IGNORE_GOAL_FEAR * panic_resistance + allowance
    }

    pub fn jitter(&self) -> f32 {
        1.0 / (1.0 + (self.boldness - JITTER_BOLDNESS_THRESHOLD).max(0.0) / JITTER_BOLDNESS_DIVISOR)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SHOT: Scary = Scary {
        x: 10.0,
        z: 20.0,
        magnitude: 150.0,
    };

    #[test]
    fn fear_settles_without_scares() {
        let mut data = RabbitData::NEW;
        for _ in 0..200 {
            data.update_fear(None, 0.0, 10, 1.0);
        }
        let decay = FEAR_DECAY.powi(10);
        let settled = FEAR_ADDED * 10.0 * decay / (1.0 - decay);
        assert!((data.fear - settled).abs() < 0.1, "fear {}", data.fear);
        assert_eq!(data.panic, None);
    }

    #[test]
    fn two_blasts_start_a_panic_away_from_the_last_one() {
        let mut data = RabbitData::NEW;
        data.update_fear(Some(SHOT), SHOT.magnitude, 10, 1.0);
        assert_eq!(data.panic, None, "one shot leaves fear under 250");
        let second = Scary { x: 50.0, ..SHOT };
        data.update_fear(Some(second), second.magnitude, 0, 1.0);
        assert!(data.fear > PANIC_START_FEAR);
        assert_eq!(data.panic, Some(second));
    }

    #[test]
    fn panic_resistance_raises_the_threshold() {
        let mut data = RabbitData {
            fear: 200.0,
            ..RabbitData::NEW
        };
        data.update_fear(Some(SHOT), 100.0, 0, 2.0);
        assert_eq!(data.panic, None);
        data.update_fear(Some(SHOT), 100.0, 0, 1.0);
        assert!(data.panic.is_some());
    }

    #[test]
    fn panic_ends_once_fear_drops_below_150() {
        let mut data = RabbitData {
            fear: 300.0,
            panic: Some(SHOT),
            ..RabbitData::NEW
        };
        while data.panic.is_some() {
            data.update_fear(None, 0.0, 30, 1.0);
        }
        assert!(data.fear < PANIC_END_FEAR);
    }

    #[test]
    fn a_strong_scare_renews_the_panic_origin() {
        let mut data = RabbitData {
            fear: 400.0,
            panic: Some(SHOT),
            ..RabbitData::NEW
        };
        let weak = Scary {
            x: 99.0,
            z: 99.0,
            magnitude: 1.0,
        };
        data.update_fear(Some(weak), 0.0, 0, 1.0);
        assert_eq!(data.panic, Some(SHOT), "a light doesn't move the panic");
        let blast = Scary {
            x: 99.0,
            z: 99.0,
            magnitude: 100.0,
        };
        data.update_fear(Some(blast), 100.0, 0, 1.0);
        assert_eq!(data.panic, Some(blast));
    }

    #[test]
    fn scared_rabbits_run_faster_and_tire() {
        let calm = { RabbitData::NEW }.update_motion(0.0, 10, 1.0);
        let mut scared_mood = RabbitData {
            fear: 400.0,
            ..RabbitData::NEW
        };
        let scared = scared_mood.update_motion(0.0, 10, 1.0);
        assert!(scared > calm);
        let mut blasted_mood = RabbitData {
            fear: 400.0,
            ..RabbitData::NEW
        };
        let blasted = blasted_mood.update_motion(150.0, 10, 1.0);
        assert!(
            (blasted - 2.0 * scared).abs() < 1e-3,
            "a blast doubles the speed"
        );
    }

    #[test]
    fn exhausted_rabbits_slow_down_without_a_floor() {
        let mut tired = RabbitData {
            stamina: 10.0,
            ..RabbitData::NEW
        };
        let speed = tired.update_motion(0.0, 10, 1.0);
        assert!(speed > 0.0 && speed < 0.25, "speed {speed}");
    }

    #[test]
    fn goals_embolden_and_fear_discourages() {
        let mut keen = RabbitData::NEW;
        keen.update_boldness(2.0, 10);
        let mut afraid = RabbitData {
            fear: 400.0,
            ..RabbitData::NEW
        };
        afraid.update_boldness(2.0, 10);
        assert!(keen.boldness > RabbitData::NEW.boldness);
        assert!(afraid.boldness < keen.boldness);
    }

    #[test]
    fn carriers_tolerate_more_fear_before_ignoring_the_goal() {
        let data = RabbitData {
            fear: 200.0,
            ..RabbitData::NEW
        };
        assert!(!data.moves_to_goal(1.0, false));
        assert!(data.moves_to_goal(1.0, true));
        assert!(!data.drops_carrot(1.0));
        assert!(
            RabbitData {
                fear: 501.0,
                ..RabbitData::NEW
            }
            .drops_carrot(1.0)
        );
    }

    #[test]
    fn boldness_steadies_movement() {
        assert_eq!(RabbitData::NEW.jitter(), 1.0);
        let bold = RabbitData {
            boldness: 400.0,
            ..RabbitData::NEW
        };
        assert_eq!(bold.jitter(), 0.5);
    }
}
