use super::weapons::blocks;
use super::*;

const SETTLE_FRAMES: i32 = 10;
const STILL_FROM: i32 = 40;
const START_EATING_FRAMES: i32 = 40;
const LONG_EAT_FRAMES: i32 = 250;
const POSE_START: i32 = 130;
const POSE_END: i32 = 150;
const STILL_SPEED: f32 = 0.01;
const EAT_ATTEMPTS: u32 = 6;
const CLEAR_OF_LIGHT: f32 = 400.0;
const CARROT_OFFSET: f32 = 10.0;
const SCORE_FRAMES: i32 = 3;
const BURROW_OFFSET: f32 = 30.0;

#[derive(Default)]
pub(super) struct EatTrial {
    rabbit: i32,
    attempts: u32,
    spot: spring::Float3,
    center: Option<i32>,
    hop: Option<(f32, f32)>,
    carried: i32,
    stolen_at: Option<i32>,
    drop_carried: i32,
    dropped_before: usize,
    drop_checked: bool,
}

enum Outcome {
    Waiting,
    Retry(&'static str),
    Rested(f32),
}

impl Tracker {
    pub(super) fn eat(
        &mut self,
        spring: &spring::Spring<'_, spring::RulesSynced>,
        global: &Global,
        frame: i32,
    ) {
        let target = {
            let target = global.weapons.access().target;
            spring::Float3::new(target.x, 0.0, target.z)
        };
        let rabbits: Vec<(i32, spring::Float3)> = global
            .roster
            .access()
            .units(UnitKind::Rabbit)
            .into_iter()
            .filter_map(|rabbit| Some((rabbit, crate::units::unit_position(spring, rabbit)?)))
            .collect();
        let calmest = {
            let memory = global.rabbits.access();
            rabbits
                .iter()
                .filter_map(|(rabbit, _)| {
                    let memory = memory.memories[memory.find(*rabbit)?];
                    let idle = memory.reserved_carrot < 0
                        && memory.carried_carrot < 0
                        && memory.data.panic.is_none();
                    idle.then_some((*rabbit, memory.data.fear))
                })
                .min_by(|a, b| a.1.total_cmp(&b.1))
                .map(|(rabbit, _)| rabbit)
        };
        let loneliness = |position: spring::Float3| {
            rabbits
                .iter()
                .map(|(_, rabbit)| distance_2d(*rabbit, position))
                .fold(f32::MAX, f32::min)
        };
        let carrot = free_carrots(global)
            .into_iter()
            .filter(|(carrot, position)| {
                distance_2d(*position, target) > CLEAR_OF_LIGHT
                    && global.rabbits.access().carrot_progress(*carrot) == 0
            })
            .max_by(|a, b| loneliness(a.1).total_cmp(&loneliness(b.1)));
        let (Some(rabbit), Some((_, position))) = (calmest, carrot) else {
            self.fail(
                spring,
                "an idle rabbit and a free carrot are left for the eating test",
            );
            return;
        };
        let beside = spring::Float3 {
            x: position.x + CARROT_OFFSET,
            ..position
        };
        if let Err(error) = teleport(spring, rabbit, beside) {
            self.fail(
                spring,
                &format!("a rabbit moves beside a carrot: {error:?}"),
            );
            return;
        }
        self.eat = EatTrial {
            rabbit,
            spot: beside,
            attempts: self.eat.attempts + 1,
            center: center_piece(spring, rabbit),
            ..EatTrial::default()
        };
        self.next(Step::Eating, frame);
    }

    pub(super) fn eating(
        &mut self,
        spring: &spring::Spring<'_, spring::RulesSynced>,
        global: &Global,
        frame: i32,
    ) {
        match self.watch_eating(spring, global, frame - self.since) {
            Outcome::Waiting => {}
            Outcome::Retry(why) if self.eat.attempts < EAT_ATTEMPTS => {
                spring::log_info!(
                    spring,
                    "hunted-test",
                    "eating test: {why}; trying another rabbit"
                );
                self.next(Step::Eat, frame);
            }
            Outcome::Retry(why) => {
                self.fail(
                    spring,
                    &format!("a rabbit eats a carrot standing still: {why}, {EAT_ATTEMPTS} times"),
                );
            }
            Outcome::Rested(range) => {
                self.suite.check(spring,
                    range < 0.05,
                    format!(
                        "an eating rabbit that stands still stops hopping: its body moved {range:.2} over {} frames",
                        POSE_END - POSE_START
                    ),
                );
                self.next(Step::Carrying, frame);
            }
        }
    }

    fn watch_eating(
        &mut self,
        spring: &spring::Spring<'_, spring::RulesSynced>,
        global: &Global,
        elapsed: i32,
    ) -> Outcome {
        let rabbit = self.eat.rabbit;
        let memory = {
            let rabbits = global.rabbits.access();
            rabbits.find(rabbit).map(|index| rabbits.memories[index])
        };
        let Some(memory) = memory else {
            return Outcome::Retry("the rabbit died");
        };
        if memory.carried_carrot >= 0 {
            return Outcome::Retry("the rabbit finished eating early");
        }
        if elapsed >= START_EATING_FRAMES && memory.reserved_carrot < 0 {
            return Outcome::Retry("the rabbit did not start eating");
        }
        if memory.reserved_carrot >= 0 && memory.reserved_eat_time != LONG_EAT_FRAMES {
            let mut rabbits = global.rabbits.access_mut();
            if let Some(index) = rabbits.find(rabbit) {
                rabbits.memories[index].reserved_eat_time = LONG_EAT_FRAMES;
            }
        }
        if elapsed == SETTLE_FRAMES && teleport(spring, rabbit, self.eat.spot).is_err() {
            return Outcome::Retry("the rabbit could not be put back on its spot");
        }
        let speed = spring
            .units_info()
            .get_unit_velocity(rabbit)
            .map_or(f32::MAX, |velocity| velocity.x.hypot(velocity.z));
        if elapsed >= STILL_FROM && speed > STILL_SPEED {
            return Outcome::Retry("the rabbit moved");
        }
        if elapsed < POSE_START {
            return Outcome::Waiting;
        }
        let Some(height) = self.eat.center.and_then(|piece| {
            spring
                .units_pieces()
                .get_unit_piece_position(rabbit, piece)
                .ok()
                .map(|position| position.y)
        }) else {
            return Outcome::Retry("the rabbit's center piece is unreadable");
        };
        let (low, high) = self.eat.hop.unwrap_or((height, height));
        self.eat.hop = Some((low.min(height), high.max(height)));
        if elapsed + 1 < POSE_END {
            return Outcome::Waiting;
        }
        Outcome::Rested(high.max(height) - low.min(height))
    }

    pub(super) fn carrying(
        &mut self,
        spring: &spring::Spring<'_, spring::RulesSynced>,
        global: &Global,
        frame: i32,
    ) {
        let carried = {
            let rabbits = global.rabbits.access();
            rabbits
                .find(self.eat.rabbit)
                .map_or(-1, |index| rabbits.memories[index].carried_carrot)
        };
        if carried < 0 {
            let _ = self.timed_out(spring, frame, "the eating rabbit picks up its carrot");
            return;
        }
        self.suite.check(spring,
            !blocks(spring, carried),
            format!(
                "the rabbit carries the carrot it ate, and the carried carrot neither blocks nor collides: carrot {carried}"
            ),
        );
        self.eat.carried = carried;
        self.next(Step::CarryHome, frame);
    }

    pub(super) fn carry_home(
        &mut self,
        spring: &spring::Spring<'_, spring::RulesSynced>,
        global: &Global,
        frame: i32,
    ) {
        let rabbit = self.eat.rabbit;
        let burrow = crate::units::unit_position(spring, rabbit).and_then(|position| {
            let burrows = global.roster.access().positions(UnitKind::Burrow);
            crate::units::nearest(position, &burrows).map(|(_, burrow, _)| burrow)
        });
        let Some(burrow) = burrow else {
            self.fail(spring, "the carrier and a burrow are there to go home to");
            return;
        };
        self.before.stolen = global.score.access().carrots_stolen;
        let beside = spring::Float3 {
            x: burrow.x + BURROW_OFFSET,
            ..burrow
        };
        if let Err(error) = teleport(spring, rabbit, beside) {
            self.fail(
                spring,
                &format!("the carrier moves beside a burrow: {error:?}"),
            );
            return;
        }
        self.next(Step::CarriedHome, frame);
    }

    pub(super) fn carried_home(
        &mut self,
        spring: &spring::Spring<'_, spring::RulesSynced>,
        global: &Global,
        frame: i32,
    ) {
        let stolen = global.score.access().carrots_stolen - self.before.stolen;
        if stolen < 1 {
            let _ = self.timed_out(
                spring,
                frame,
                "the carrier takes its carrot into the burrow",
            );
            return;
        }
        if self.eat.stolen_at.is_none() {
            self.eat.stolen_at = Some(frame);
        }
        if self
            .eat
            .stolen_at
            .is_some_and(|at| frame - at < SCORE_FRAMES)
        {
            return;
        }
        let carried_gone = global.roster.access().kind(self.eat.carried).is_none();
        let still_carries = {
            let rabbits = global.rabbits.access();
            rabbits
                .find(self.eat.rabbit)
                .is_some_and(|index| rabbits.memories[index].carried_carrot >= 0)
        };
        self.suite.check(spring,
            stolen == 1 && carried_gone && !still_carries,
            format!(
                "a rabbit carrying a carrot into a burrow steals it: stolen +{stolen}, carrot gone {carried_gone}, still carrying {still_carries}"
            ),
        );
        self.rabbit = -1;
        self.next(Step::DropCarrot, frame);
    }
    pub(super) fn begin_drop(
        &mut self,
        spring: &spring::Spring<'_, spring::RulesSynced>,
        global: &Global,
        frame: i32,
    ) {
        let rabbit = self.eat.rabbit;
        let Some(position) = crate::units::unit_position(spring, rabbit) else {
            self.fail(spring, "the carrier remains for the drop trial");
            return;
        };
        let Some(carried) =
            crate::units::create_now(spring, UnitKind::CarrotCarried, position, 0, 0)
        else {
            self.fail(spring, "a carried carrot is created for the drop trial");
            return;
        };
        self.eat.drop_carried = carried;
        self.eat.dropped_before = global.roster.access().count(UnitKind::CarrotDropped);
        crate::unit_scripts::Call::rabbit_attach_carrot(rabbit, carried).run(spring);
        {
            let mut rabbits = global.rabbits.access_mut();
            let index = rabbits.find(rabbit).expect("trial rabbit");
            let memory = &mut rabbits.memories[index];
            memory.carried_carrot = carried;
            memory.carrying_since = frame;
            memory.data.fear = 600.0;
            memory.next_update = frame;
        }
        spring
            .unit_control()
            .set_unit_velocity(rabbit, spring::Float3::new(8.0, 1.0, 2.0))
            .expect("setting the drop trial's initial momentum");
        self.next(Step::CarrotDropped, frame);
    }

    pub(super) fn check_drop(
        &mut self,
        spring: &spring::Spring<'_, spring::RulesSynced>,
        global: &Global,
        frame: i32,
    ) {
        if frame <= self.since {
            return;
        }
        if !self.eat.drop_checked {
            let velocity = spring
                .units_info()
                .get_unit_velocity(self.eat.drop_carried)
                .unwrap_or_default();
            let horizontal = (velocity.x * velocity.x + velocity.z * velocity.z).sqrt();
            let no_longer_carrying = {
                let rabbits = global.rabbits.access();
                rabbits
                    .find(self.eat.rabbit)
                    .is_some_and(|i| rabbits.memories[i].carried_carrot < 0)
            };
            self.suite.check(spring, no_longer_carrying && !blocks(spring, self.eat.rabbit)
                && horizontal > 0.001 && horizontal <= 1.01,
                format!("a frightened carrier drops with bounded momentum and stops blocking: horizontal {horizontal}, released {no_longer_carrying}"));
            self.eat.drop_checked = true;
        }
        if frame - self.since < 18 {
            return;
        }
        let roster = global.roster.access();
        let old_gone = roster.kind(self.eat.drop_carried).is_none();
        let count = roster.count(UnitKind::CarrotDropped);
        drop(roster);
        self.suite.check(spring, old_gone && count == self.eat.dropped_before + 1,
            format!("the thrown carrot becomes exactly one edible dropped carrot: old gone {old_gone}, dropped {count}"));
        self.next(Step::Lit, frame);
    }
}

fn center_piece(spring: &spring::Spring<'_, spring::RulesSynced>, rabbit: i32) -> Option<i32> {
    spring
        .units_pieces()
        .get_unit_piece_map(rabbit)
        .ok()?
        .into_iter()
        .find(|piece| piece.name == "center")
        .map(|piece| piece.piece_num)
}
