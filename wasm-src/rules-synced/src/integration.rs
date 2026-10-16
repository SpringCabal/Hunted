mod rabbits;
mod round;
mod weapons;

use alloc::collections::BTreeSet;
use alloc::format;
use alloc::vec::Vec;
use core::cell::RefCell;

use hunted_common::protocol;
use hunted_common::scare::Area;
use spring_addons::AddonContext;
use spring_addons::rules::{Gadget, UnitDestroyedEvent};
use springcabal::check::Suite;
use springcabal::message::{self, Encoder};

use crate::defs::{self, UnitKind};
use crate::gadgets::ui_input;
use crate::map;
use crate::state::Global;

const EXPECTED_CHECKS: u32 = 35;
const TIMEOUT: i32 = 200;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Step {
    Spawned,
    Eat,
    Eating,
    Carrying,
    CarryHome,
    CarriedHome,
    Lit,
    Fire,
    Fired,
    Scared,
    ShootField,
    FieldShot,
    PlaceMine,
    MinePlaced,
    FuseLit,
    Detonated,
    HillMine,
    HillMinePlaced,
    Kill,
    Killed,
    CarrotTick,
    CarrotScored,
    Flare,
    FlareStarted,
    RestartMidHunt,
    Lose,
    Lost,
    RestartTwice,
    Restarted,
    NewHunt,
    Done,
}

pub struct IntegrationGadget {
    tracker: RefCell<Tracker>,
}

impl Default for IntegrationGadget {
    fn default() -> Self {
        Self {
            tracker: RefCell::new(Tracker {
                suite: Suite::new(
                    "[Hunted-Rust-Test]",
                    "HUNTED INTEGRATION TESTS",
                    EXPECTED_CHECKS,
                ),
                step: Step::Spawned,
                since: 0,
                rabbit: -1,
                shot_at: (0.0, 0.0),
                shot_carrot: -1,
                mine: None,
                blast_carrots: Vec::new(),
                fuse_lit: 0,
                hill_y: None,
                fastest_rabbit: 0.0,
                eat: rabbits::EatTrial::default(),
                before: Before::default(),
                shot_down: BTreeSet::new(),
                ui_status: None,
            }),
        }
    }
}

impl Gadget<Global> for IntegrationGadget {
    fn name(&self) -> &'static str {
        "Integration"
    }

    fn game_frame(&self, ctx: &AddonContext<'_, Global>, frame: i32) {
        self.tracker.borrow_mut().update(ctx, frame);
    }

    fn unit_destroyed(&self, ctx: &AddonContext<'_, Global>, event: &UnitDestroyedEvent) {
        let protected = matches!(
            defs::kind_of(event.unit_def_id),
            Some(kind) if kind.is_carrot() || matches!(kind, UnitKind::Burrow | UnitKind::Shotgun)
        );
        if protected && event.attacker_id >= 0 && !ctx.global().session.access().restart_in_progress
        {
            self.tracker.borrow_mut().shot_down.insert(event.unit_id);
        }
    }

    fn handle_lua_msg(
        &self,
        _ctx: &AddonContext<'_, Global>,
        _player_id: i32,
        _script: i32,
        _mode: i32,
        data: &[u8],
    ) {
        let Some(frame) = core::str::from_utf8(data)
            .ok()
            .and_then(|text| message::fields(text, protocol::TEST_STATUS_SEEN))
            .and_then(|mut fields| fields.i32())
        else {
            return;
        };
        self.tracker.borrow_mut().ui_status.get_or_insert(frame);
    }
}

#[derive(Default)]
struct Before {
    fear: f32,
    shotgun_ammo: f32,
    mine_ammo: f32,
    shots: i32,
    kills: i32,
    score: i32,
    mines: usize,
    gun: i32,
    carrots: i32,
    stolen: i32,
}

struct Tracker {
    suite: Suite,
    step: Step,
    since: i32,
    rabbit: i32,
    shot_at: (f32, f32),
    shot_carrot: i32,
    mine: Option<(i32, f32, f32)>,
    blast_carrots: Vec<i32>,
    fuse_lit: i32,
    hill_y: Option<f32>,
    fastest_rabbit: f32,
    eat: rabbits::EatTrial,
    before: Before,
    shot_down: BTreeSet<i32>,
    ui_status: Option<i32>,
}

impl Tracker {
    fn next(&mut self, step: Step, frame: i32) {
        self.step = step;
        self.since = frame;
    }

    fn timed_out(&mut self, frame: i32, what: &str) -> bool {
        if frame - self.since <= TIMEOUT {
            return false;
        }
        self.fail(&format!("{what} within {TIMEOUT} frames"));
        true
    }

    fn fail(&mut self, what: &str) {
        self.suite.check(false, what);
        self.suite.finish();
        self.step = Step::Done;
    }

    fn update(&mut self, ctx: &AddonContext<'_, Global>, frame: i32) {
        let global = ctx.global();
        match self.step {
            Step::Spawned => self.spawned(global, frame),
            Step::Lit => self.lit(ctx, frame),
            Step::Fire => self.fire(ctx, frame),
            Step::Fired => self.fired(global, frame),
            Step::Scared => self.scared(global, frame),
            Step::ShootField => self.shoot_field(ctx, frame),
            Step::FieldShot => self.field_shot(global, frame),
            Step::PlaceMine => self.place_mine(ctx, frame),
            Step::MinePlaced => self.mine_placed(global, frame),
            Step::FuseLit => self.fuse_lit(global, frame),
            Step::Detonated => self.detonated(global, frame),
            Step::HillMine => self.hill_mine(ctx, frame),
            Step::HillMinePlaced => self.hill_mine_placed(global, frame),
            Step::Kill => self.kill(global, frame),
            Step::Killed => self.killed(global, frame),
            Step::CarrotTick => self.carrot_tick(global, frame),
            Step::CarrotScored => self.carrot_scored(global, frame),
            Step::Eat => self.eat(global, frame),
            Step::Eating => self.eating(global, frame),
            Step::Carrying => self.carrying(global, frame),
            Step::CarryHome => self.carry_home(global, frame),
            Step::CarriedHome => self.carried_home(global, frame),
            Step::Flare => self.flare(global, frame),
            Step::FlareStarted => self.flare_started(global, frame),
            Step::RestartMidHunt => self.restart_mid_hunt(ctx, frame),
            Step::Lose => self.lose(global, frame),
            Step::Lost => self.lost(global, frame),
            Step::RestartTwice => self.restart_twice(ctx, frame),
            Step::Restarted => self.restarted(global, frame),
            Step::NewHunt => self.new_hunt(global, frame),
            Step::Done => {}
        }
    }

    fn check_nothing_shot_down(&mut self, what: &str) {
        let shot_down = self.shot_down.len();
        self.suite.check(
            shot_down == 0,
            format!("{what}: {shot_down} carrots, burrows or guns destroyed by weapons"),
        );
    }
}

fn fear(global: &Global, rabbit: i32) -> Option<f32> {
    let rabbits = global.rabbits.access();
    rabbits
        .find(rabbit)
        .map(|index| rabbits.memories[index].data.fear)
}

fn has_scare(global: &Global, x: f32, z: f32, area: Area) -> bool {
    global
        .scares
        .access()
        .0
        .iter()
        .any(|scare| scare.area == area && (scare.x - x).abs() < 1.0 && (scare.z - z).abs() < 1.0)
}

fn send(ctx: &AddonContext<'_, Global>, command: &str, position: spring::Float3) {
    let message = Encoder::new(command)
        .field(position.x)
        .field(position.y)
        .field(position.z)
        .finish();
    ui_input::receive(ctx, message.as_bytes());
}

fn distance_2d(a: spring::Float3, b: spring::Float3) -> f32 {
    (a.x - b.x).hypot(a.z - b.z)
}

fn free_carrots(global: &Global) -> Vec<(i32, spring::Float3)> {
    let carrots = global.roster.access().positions(UnitKind::Carrot);
    let rabbits = global.rabbits.access();
    carrots
        .into_iter()
        .filter(|(carrot, _)| rabbits.reserving(*carrot).is_none())
        .collect()
}

fn teleport(unit: i32, position: spring::Float3) -> Result<(), spring::ApiError> {
    spring::set_unit_position(unit, position)?;
    spring::set_unit_velocity(unit, spring::Float3::default())?;
    spring::set_unit_move_goal(unit, position, 16.0, 0.0, true)?;
    Ok(())
}
