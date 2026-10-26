use crate::map::BURROWS;

pub const FIRST_WAVE_FRAME: i32 = 5;
pub const WAVE_GAP: i32 = 550;

const FIRST_WAVE_SIZE: i32 = 25;
const FIRST_WAVE_BURROWS: i32 = 5;
const MAX_RABBITS: i32 = 600;
const BURROW_SPAWN_DELAY_MIN: i32 = 20;
const BURROW_SPAWN_DELAY_RANGE: i32 = 30;
const BURROW_FAMILY_MIN: i32 = 2;
const BURROW_FAMILY_RANGE: i32 = 2;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct BurrowSpawn {
    pub pending: i32,
    pub next_frame: i32,
    pub active: bool,
}

#[derive(Clone, Debug)]
pub struct Waves {
    pub spawn_index: i32,
    pub wave_target: i32,
    pub wave_number: i32,
    next_difficulty: f32,
    pub next_wave_frame: i32,
    pub rabbit_speed_multiplier: f32,
    pub rabbit_panic_resistance: f32,
    pub burrow_spawns: Vec<BurrowSpawn>,
}

impl Default for Waves {
    fn default() -> Self {
        Self {
            spawn_index: 0,
            wave_target: 0,
            wave_number: 0,
            next_difficulty: 1.0,
            next_wave_frame: FIRST_WAVE_FRAME,
            rabbit_speed_multiplier: 1.5,
            rabbit_panic_resistance: 1.0,
            burrow_spawns: vec![BurrowSpawn::default(); BURROWS.len()],
        }
    }
}

impl Waves {
    pub fn start_wave(
        &mut self,
        frame: i32,
        rabbit_count: i32,
        mut pick_burrow: impl FnMut() -> usize,
    ) -> i32 {
        self.wave_number = self.wave_number.saturating_add(1);
        let difficulty = self.next_difficulty;
        self.next_difficulty *= 1.0 + 0.04 / self.next_difficulty;
        self.rabbit_panic_resistance = difficulty;
        self.rabbit_speed_multiplier = 1.0 + difficulty / 2.0;
        let mut wave_size = FIRST_WAVE_SIZE;
        let mut burrow_count = FIRST_WAVE_BURROWS;
        for _ in 1..self.wave_number {
            wave_size = ((wave_size as f32 * 1.2).ceil() as i32).min(MAX_RABBITS);
            burrow_count = ((burrow_count as f32 * 1.2).ceil() as i32).min(MAX_RABBITS);
            if wave_size == MAX_RABBITS && burrow_count == MAX_RABBITS {
                break;
            }
        }
        self.wave_target = wave_size.min((MAX_RABBITS - rabbit_count).max(0));
        self.next_wave_frame = frame + WAVE_GAP;
        // Lua leaves existing families alone and makes no picks when the
        // live population already fills the cap.
        if self.wave_target == 0 {
            return burrow_count;
        }

        let rabbits_per_burrow = if burrow_count > 0 {
            self.wave_target as f32 / burrow_count as f32
        } else {
            0.0
        };
        let mut rabbits_left = self.wave_target as f32;
        let mut last_rabbits = self.wave_target;
        for _ in 0..burrow_count {
            let burrow = pick_burrow();
            rabbits_left -= rabbits_per_burrow;
            let this_burrow = (last_rabbits as f32 - rabbits_left).ceil() as i32;
            last_rabbits = rabbits_left.floor() as i32;
            let spawn = &mut self.burrow_spawns[burrow];
            spawn.pending += this_burrow;
            spawn.next_frame = frame + 1;
            spawn.active = spawn.pending > 0;
        }
        burrow_count
    }

    pub fn finish_wave(&mut self) {
        if self.burrow_spawns.iter().all(|spawn| !spawn.active) {
            self.wave_target = 0;
        }
    }

    /// Run creation between the family-size draw and its next timer draw.
    /// Unit-script initialization consumes RNG too, so this order is gameplay.
    pub fn emit_burrow_batch(
        &mut self,
        burrow: usize,
        frame: i32,
        mut random: impl FnMut() -> f32,
        mut emit: impl FnMut(i32),
    ) -> Option<i32> {
        let spawn = self.burrow_spawns[burrow];
        if !spawn.active || frame < spawn.next_frame {
            return None;
        }
        let batch = ((BURROW_FAMILY_MIN as f32 + BURROW_FAMILY_RANGE as f32 * random()).ceil()
            as i32)
            .min(spawn.pending);
        emit(batch);
        let pending = spawn.pending - batch;
        // The original burrow script sleeps 33 ms per nominal frame at 30 Hz.
        // Draw the final sleep too, as the original coroutine does.
        let delay = (0.99
            * (BURROW_SPAWN_DELAY_MIN as f32 + BURROW_SPAWN_DELAY_RANGE as f32 * random()))
        .ceil() as i32;
        self.burrow_spawns[burrow] = BurrowSpawn {
            pending,
            next_frame: frame + delay,
            active: pending > 0,
        };
        Some(batch)
    }

    pub fn next_burrow_batch(
        &mut self,
        burrow: usize,
        frame: i32,
        random: impl FnMut() -> f32,
    ) -> Option<i32> {
        self.emit_burrow_batch(burrow, frame, random, |_| {})
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pending(waves: &Waves) -> i32 {
        waves.burrow_spawns.iter().map(|spawn| spawn.pending).sum()
    }

    #[test]
    fn a_new_wave_retains_pending_families_and_unselected_timers() {
        let mut waves = Waves::default();
        waves.start_wave(5, 0, || 0);
        waves.burrow_spawns[1] = BurrowSpawn {
            pending: 2,
            next_frame: 777,
            active: true,
        };
        waves.start_wave(555, 0, || 0);
        assert_eq!(waves.burrow_spawns[0].pending, 56);
        assert_eq!(waves.burrow_spawns[1].pending, 2);
        assert_eq!(waves.burrow_spawns[1].next_frame, 777);
    }

    #[test]
    fn wave_growth_keeps_the_original_float32_rounding() {
        let mut waves = Waves::default();
        waves.start_wave(5, 0, || 0);
        waves.start_wave(555, 25, || 0);
        assert_eq!(waves.wave_target, 31, "Lua ceil(25 * 1.2f) is 31");
    }

    #[test]
    fn creation_precedes_the_timer_draw_and_cloned_burrows_can_spawn() {
        use std::cell::RefCell;
        let mut waves = Waves::default();
        waves.burrow_spawns.resize(18, BurrowSpawn::default());
        waves.start_wave(5, 0, || 17);
        let events = RefCell::new(Vec::new());
        assert_eq!(
            waves.emit_burrow_batch(
                17,
                6,
                || {
                    events.borrow_mut().push("random");
                    0.5
                },
                |count| {
                    assert_eq!(count, 3);
                    events.borrow_mut().push("create");
                }
            ),
            Some(3)
        );
        assert_eq!(*events.borrow(), ["random", "create", "random"]);
        assert_eq!(waves.burrow_spawns[17].pending, 22);
        assert_eq!(waves.burrow_spawns[17].next_frame, 41);
    }

    #[test]
    fn the_first_wave_is_25_rabbits_from_5_burrows() {
        let mut waves = Waves::default();
        let mut next = 0;
        let picks = waves.start_wave(5, 0, || {
            next += 1;
            next - 1
        });
        assert_eq!(picks, 5);
        assert_eq!(waves.wave_number, 1);
        assert_eq!(waves.wave_target, 25);
        assert_eq!(pending(&waves), 25);
        assert_eq!(
            waves.burrow_spawns[..5]
                .iter()
                .map(|s| s.pending)
                .collect::<Vec<_>>(),
            [5; 5]
        );
        assert!(
            waves.burrow_spawns[..5]
                .iter()
                .all(|s| s.active && s.next_frame == 6)
        );
        assert_eq!(waves.next_wave_frame, 5 + WAVE_GAP);
        assert_eq!(waves.rabbit_panic_resistance, 1.0);
        assert_eq!(waves.rabbit_speed_multiplier, 1.5);
    }

    #[test]
    fn waves_follow_float_growth_and_get_harder() {
        let mut waves = Waves::default();
        waves.start_wave(0, 0, || 0);
        waves.burrow_spawns.fill(BurrowSpawn::default()); // prior wave completed
        waves.start_wave(550, 0, || 0);
        assert_eq!(waves.wave_target, 31);
        assert_eq!(pending(&waves), 31, "one burrow picked six times");
        waves.burrow_spawns.fill(BurrowSpawn::default());
        waves.start_wave(1100, 0, || 0);
        assert_eq!(waves.wave_target, 38);
        assert!((waves.rabbit_panic_resistance - 1.08).abs() < 1e-6);
        assert!((waves.rabbit_speed_multiplier - 1.54).abs() < 1e-6);
    }

    #[test]
    fn waves_stop_at_600_rabbits() {
        let mut waves = Waves::default();
        waves.start_wave(0, 590, || 0);
        assert_eq!(waves.wave_target, 10);
        waves.start_wave(550, 700, || {
            panic!("Lua does not pick burrows for an empty wave")
        });
        assert_eq!(waves.wave_target, 0);
        assert_eq!(pending(&waves), 10, "already scheduled families remain");
        assert_eq!(waves.next_wave_frame, 1100);
    }

    #[test]
    fn long_hunts_have_bounded_wave_sizes_and_burrow_picks() {
        for wave in [50, 100, 1000, i32::MAX] {
            let mut waves = Waves {
                wave_number: wave,
                ..Waves::default()
            };
            let mut picks = 0;
            waves.start_wave(0, 0, || {
                picks += 1;
                0
            });
            assert_eq!(picks, MAX_RABBITS);
            assert_eq!(waves.wave_target, MAX_RABBITS);
            assert_eq!(pending(&waves), MAX_RABBITS);
        }
    }

    #[test]
    fn failed_creations_do_not_hold_the_wave_open() {
        let mut waves = Waves::default();
        waves.start_wave(0, 0, || 0);
        waves.finish_wave();
        assert_eq!(waves.wave_target, 25, "spawn attempts remain");
        let mut frame = 1;
        while waves.next_burrow_batch(0, frame, || 0.5).is_some() {
            frame += 50;
        }
        waves.finish_wave();
        assert_eq!(waves.wave_target, 0);
        waves.start_wave(550, 0, || 0);
        assert_eq!(waves.wave_number, 2);
        assert_eq!(pending(&waves), 31);
    }

    #[test]
    fn burrows_release_families_until_empty() {
        let mut waves = Waves::default();
        waves.start_wave(0, 0, || 2);
        assert_eq!(waves.next_burrow_batch(2, 0, || 0.5), None, "not due yet");
        assert_eq!(waves.next_burrow_batch(0, 1, || 0.5), None, "not picked");
        assert_eq!(waves.next_burrow_batch(2, 1, || 0.5), Some(3));
        assert_eq!(waves.burrow_spawns[2].next_frame, 36);
        let mut released = 3;
        let mut frame = 36;
        while let Some(batch) = waves.next_burrow_batch(2, frame, || 0.99) {
            released += batch;
            frame = waves.burrow_spawns[2].next_frame;
        }
        assert_eq!(released, 25);
        assert!(!waves.burrow_spawns[2].active);
    }
}
