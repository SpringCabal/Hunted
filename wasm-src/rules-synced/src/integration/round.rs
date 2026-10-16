use super::*;
use crate::units;

impl Tracker {
    pub(super) fn kill(&mut self, global: &Global, frame: i32) {
        if frame % 30 != 10 {
            return;
        }
        self.before.kills = global.score.access().rabbits_killed;
        self.before.score = global.score.access().score;
        self.destroy_a_rabbit(global);
        self.next(Step::Killed, frame);
    }

    pub(super) fn killed(&mut self, global: &Global, frame: i32) {
        let score = *global.score.access();
        self.suite.check(
            score.rabbits_killed == self.before.kills + 1 && score.score == self.before.score + 50,
            format!(
                "a kill counts and scores 50: kills {} -> {}, score {} -> {}",
                self.before.kills, score.rabbits_killed, self.before.score, score.score
            ),
        );
        self.suite.check(
            global.session.access().status_dirty,
            "a kill sends the HUD a new status this frame",
        );
        self.next(Step::CarrotTick, frame);
    }

    pub(super) fn carrot_tick(&mut self, global: &Global, frame: i32) {
        if frame % 30 != 29 {
            return;
        }
        self.before.score = global.score.access().score;
        self.next(Step::CarrotScored, frame);
    }

    pub(super) fn carrot_scored(&mut self, global: &Global, frame: i32) {
        if frame - self.since < 2 {
            return;
        }
        let score = *global.score.access();
        self.suite.check(
            score.score == self.before.score + score.carrots,
            format!(
                "each carrot in play scores a point a second: {} -> {} with {} carrots",
                self.before.score, score.score, score.carrots
            ),
        );
        self.next(Step::Flare, frame);
    }

    pub(super) fn flare(&mut self, global: &Global, frame: i32) {
        if frame % 30 != 10 {
            return;
        }
        let kills = global.score.access().rabbits_killed;
        {
            let mut flare = global.flare.access_mut();
            flare.required_kills = kills - flare.previous_kills + 1;
        }
        {
            let mut weapons = global.weapons.access_mut();
            weapons.shotgun_ammo = 50.0;
            weapons.mine_ammo = 2.0;
        }
        self.destroy_a_rabbit(global);
        self.next(Step::FlareStarted, frame);
    }

    pub(super) fn flare_started(&mut self, global: &Global, frame: i32) {
        if frame - self.since < 2 {
            return;
        }
        let active = global.flare.access().active;
        let (shells, mines) = {
            let weapons = global.weapons.access();
            (weapons.shotgun_ammo, weapons.mine_ammo)
        };
        self.suite.check(
            active && shells == 70.0 && mines == 7.0,
            format!(
                "the flare starts after the required kills and adds 20 shells and 5 mines: active {active}, ammo 50 -> {shells}, mines 2 -> {mines}"
            ),
        );
        self.next(Step::RestartMidHunt, frame);
    }

    pub(super) fn restart_mid_hunt(&mut self, ctx: &AddonContext<'_, Global>, frame: i32) {
        let accepted = ui_input::receive(ctx, protocol::RESTART.as_bytes());
        self.suite.check(!accepted, "a restart mid-hunt is refused");
        self.next(Step::Lose, frame);
    }

    pub(super) fn lose(&mut self, global: &Global, frame: i32) {
        if frame % 30 != 10 {
            return;
        }
        self.before.carrots = global.roster.access().carrot_count();
        spring::commands::run(|| {
            crate::with_global(|global| {
                let carrots: Vec<i32> = {
                    let roster = global.roster.access();
                    [
                        UnitKind::Carrot,
                        UnitKind::CarrotCarried,
                        UnitKind::CarrotDropped,
                    ]
                    .into_iter()
                    .flat_map(|kind| roster.units(kind))
                    .collect()
                };
                for carrot in carrots {
                    units::destroy_now(carrot, false, -1);
                }
            });
        });
        self.next(Step::Lost, frame);
    }

    pub(super) fn lost(&mut self, global: &Global, frame: i32) {
        let game_over = global.session.access().game_over;
        let carrots = global.score.access().carrots;
        self.suite.check(
            game_over && carrots == 0,
            format!(
                "losing the last carrot ends the hunt: {} carrots -> {carrots}, game over {game_over}",
                self.before.carrots
            ),
        );
        self.next(Step::RestartTwice, frame);
    }

    pub(super) fn restart_twice(&mut self, ctx: &AddonContext<'_, Global>, frame: i32) {
        if frame % 30 != 10 {
            return;
        }
        self.before.gun = ctx.global().weapons.access().gun_id;
        self.shot_at = (1234.0, 2345.0);
        ctx.global().weapons.access_mut().target = crate::state::Target {
            x: self.shot_at.0,
            y: 0.0,
            z: self.shot_at.1,
            valid: true,
        };
        let first = ui_input::receive(ctx, protocol::RESTART.as_bytes());
        let second = ui_input::receive(ctx, protocol::RESTART.as_bytes());
        self.suite.check(
            first && !second,
            format!("a finished hunt restarts once when asked twice: {first}, {second}"),
        );
        self.next(Step::Restarted, frame);
    }

    pub(super) fn restarted(&mut self, global: &Global, frame: i32) {
        let score = *global.score.access();
        let session = *global.session.access();
        let roster = global.roster.access();
        let (rabbits, guns) = (
            roster.count(UnitKind::Rabbit),
            roster.units(UnitKind::Shotgun),
        );
        drop(roster);
        let gun = global.weapons.access().gun_id;
        let target = global.weapons.access().target;
        self.suite.check(
            target.valid && (target.x, target.z) == self.shot_at,
            "a stationary cursor keeps its flashlight target after restart",
        );
        self.suite.check(
            score.rabbits_killed == 0
                && score.score == 0
                && score.shots_fired == 0
                && rabbits == 0
                && !session.game_over
                && !session.restart_in_progress
                && guns == [gun]
                && gun == self.before.gun,
            format!(
                "the restart clears the round and keeps the gun: kills {}, score {}, shots {}, rabbits {rabbits}, gun {} -> {gun}",
                score.rabbits_killed, score.score, score.shots_fired, self.before.gun
            ),
        );
        self.next(Step::NewHunt, frame);
    }

    pub(super) fn new_hunt(&mut self, global: &Global, frame: i32) {
        if frame - self.since < 20 {
            return;
        }
        let carrots = global.roster.access().count(UnitKind::Carrot);
        let wave = global.waves.access().wave_number;
        self.suite.check(
            carrots == map::CARROT_COUNT as usize && wave == 1,
            format!(
                "the new hunt replants the fields and starts the first wave: {carrots} carrots, wave {wave}"
            ),
        );
        self.suite.check(
            self.ui_status.is_some(),
            format!(
                "the UI received the rules' status: first confirmed frame {:?}",
                self.ui_status
            ),
        );
        self.check_nothing_shot_down("no weapon destroyed a carrot, a burrow or the gun");
        self.suite.finish();
        self.next(Step::Done, frame);
    }

    fn destroy_a_rabbit(&mut self, global: &Global) {
        let rabbit = global
            .roster
            .access()
            .units(UnitKind::Rabbit)
            .first()
            .copied();
        match rabbit {
            Some(rabbit) => units::destroy(rabbit, false, -1),
            None => self.fail("a rabbit is left to kill"),
        }
    }
}
