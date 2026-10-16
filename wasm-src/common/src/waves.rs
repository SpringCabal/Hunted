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

#[derive(Clone, Copy, Debug)]
pub struct Waves {
    pub spawn_index: i32,
    pub wave_target: i32,
    pub wave_number: i32,
    pub next_wave_frame: i32,
    pub rabbit_speed_multiplier: f32,
    pub rabbit_panic_resistance: f32,
    pub burrow_spawns: [BurrowSpawn; BURROWS.len()],
}

impl Default for Waves {
    fn default() -> Self {
        Self {
            spawn_index: 0,
            wave_target: 0,
            wave_number: 0,
            next_wave_frame: FIRST_WAVE_FRAME,
            rabbit_speed_multiplier: 1.5,
            rabbit_panic_resistance: 1.0,
            burrow_spawns: [BurrowSpawn::default(); BURROWS.len()],
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
        let difficulty = 1.0 + 0.04 * (self.wave_number - 1) as f32;
        self.rabbit_panic_resistance = difficulty;
        self.rabbit_speed_multiplier = 1.0 + difficulty / 2.0;
        let mut wave_size = FIRST_WAVE_SIZE;
        let mut burrow_count = FIRST_WAVE_BURROWS;
        for _ in 1..self.wave_number {
            wave_size = ((wave_size * 6 + 4) / 5).min(MAX_RABBITS);
            burrow_count = ((burrow_count * 6 + 4) / 5).min(MAX_RABBITS);
            if wave_size == MAX_RABBITS && burrow_count == MAX_RABBITS {
                break;
            }
        }
        self.wave_target = wave_size.min((MAX_RABBITS - rabbit_count).max(0));
        self.burrow_spawns.fill(BurrowSpawn::default());

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
        self.next_wave_frame = frame + WAVE_GAP;
        burrow_count
    }

    pub fn finish_wave(&mut self) {
        if self.burrow_spawns.iter().all(|spawn| !spawn.active) {
            self.wave_target = 0;
        }
    }

    pub fn next_burrow_batch(
        &mut self,
        burrow: usize,
        frame: i32,
        mut random: impl FnMut() -> f32,
    ) -> Option<i32> {
        let spawn = self.burrow_spawns[burrow];
        if !spawn.active || frame < spawn.next_frame {
            return None;
        }
        let batch = ((BURROW_FAMILY_MIN as f32 + BURROW_FAMILY_RANGE as f32 * random()).ceil()
            as i32)
            .min(spawn.pending);
        let pending = spawn.pending - batch;
        let next_frame = if pending > 0 {
            let delay = (BURROW_SPAWN_DELAY_MIN as f32 + BURROW_SPAWN_DELAY_RANGE as f32 * random())
                .ceil() as i32;
            frame + delay
        } else {
            0
        };
        self.burrow_spawns[burrow] = BurrowSpawn {
            pending,
            next_frame,
            active: pending > 0,
        };
        Some(batch)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pending(waves: &Waves) -> i32 {
        waves.burrow_spawns.iter().map(|spawn| spawn.pending).sum()
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
    fn waves_grow_by_a_fifth_and_get_harder() {
        let mut waves = Waves::default();
        waves.start_wave(0, 0, || 0);
        waves.start_wave(550, 0, || 0);
        assert_eq!(waves.wave_target, 30);
        assert_eq!(pending(&waves), 30, "one burrow picked six times");
        waves.start_wave(1100, 0, || 0);
        assert_eq!(waves.wave_target, 36);
        assert!((waves.rabbit_panic_resistance - 1.08).abs() < 1e-6);
        assert!((waves.rabbit_speed_multiplier - 1.54).abs() < 1e-6);
    }

    #[test]
    fn waves_stop_at_600_rabbits() {
        let mut waves = Waves::default();
        waves.start_wave(0, 590, || 0);
        assert_eq!(waves.wave_target, 10);
        waves.start_wave(0, 700, || 0);
        assert_eq!(waves.wave_target, 0);
        assert!(waves.burrow_spawns.iter().all(|spawn| !spawn.active));
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
        assert_eq!(pending(&waves), 30);
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
