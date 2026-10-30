use super::*;
use crate::gadgets::weapon_handlers::MINE_TRIGGER_RADIUS;
use crate::status::RESOURCE_PERIOD_FRAMES;
use crate::units;

const LIT_FRAMES: i32 = 120;
const LIT_FEAR: f32 = 150.0;
const RUNNING_SPEED: f32 = 0.5;
const SHOT_OFFSET: f32 = 85.0;
const SHOT_FEAR_JUMP: f32 = 40.0;
const PELLET_FRAMES: i32 = 30;
const MINE_FUSE_FRAMES: i32 = 91;
const FUSE_WAIT: i32 = 5;
const MINE_CLEAR_OF_SHOT: f32 = 500.0;
const MINE_BLAST_RADIUS: f32 = 100.0;
const TRIGGER_OFFSET: f32 = 20.0;
const HILL_GRID: f32 = 128.0;

impl Tracker {
    pub(super) fn spawned(
        &mut self,
        spring: &spring::Spring<'_, spring::RulesSynced>,
        global: &Global,
        frame: i32,
    ) {
        if frame < 10 {
            return;
        }
        let rabbits = global.roster.access().units(UnitKind::Rabbit);
        if rabbits.len() < 5 {
            let _ = self.timed_out(spring, frame, "the first wave spawns");
            return;
        }
        if global.weapons.access().gun_id < 0 {
            let _ = self.timed_out(
                spring,
                frame,
                "the UI's mouse target reaches the rules and spawns the gun",
            );
            return;
        }
        self.suite.check(
            spring,
            crate::gadgets::rabbit_ai::verify_incremental_food_index(),
            "live food buckets preserve target ranks and reservations through removal and ID reuse",
        );
        self.suite.check(
            spring,
            crate::gadgets::rabbit_ai::verify_cached_rabbit_order(),
            "cached rabbit order survives creation, death, kind replacement and ID reuse",
        );
        let roster = global.roster.access();
        let counts = (
            roster.count(UnitKind::Carrot),
            roster.count(UnitKind::Burrow),
            roster.count(UnitKind::Lighthouse),
        );
        let burrows = roster.units(UnitKind::Burrow);
        drop(roster);
        self.suite.check(
            spring,
            counts
                == (
                    map::CARROT_COUNT as usize,
                    map::BURROWS.len(),
                    map::LIGHTHOUSES.len(),
                ),
            format!("the fields, burrows and lighthouses spawn: {counts:?}"),
        );
        let wave = global.waves.access().wave_number;
        self.suite.check(
            spring,
            wave == 1,
            format!(
                "the first wave starts: wave {wave}, {} rabbits",
                rabbits.len()
            ),
        );
        let blocking = burrows
            .iter()
            .filter(|burrow| blocks(spring, **burrow))
            .count();
        self.suite.check(
            spring,
            blocking == 0,
            format!(
                "burrows neither block nor collide: {blocking} of {} do",
                burrows.len()
            ),
        );
        self.suite.check(
            spring,
            global.weapons.access().gun_id >= 0,
            "the UI's mouse target reached the rules and spawned the gun",
        );
        let defs = crate::defs::defs(spring);
        let ally_teams = spring.teams().get_ally_team_list();
        self.suite.check(
            spring,
            ally_teams.as_ref().is_ok_and(|teams| {
                !teams.is_empty()
                    && teams
                        .iter()
                        .all(|&team| spring.game().get_global_los(team) == Ok(1))
            }),
            "all ally teams have global LOS, as in the Lua startup widget",
        );
        let lua_spray = (2000.0 * core::f32::consts::PI / 0xafff as f32).sin();
        self.suite.check(
            spring,
            defs.shotgun_projectiles == 10 && (defs.shotgun_spray_angle - lua_spray).abs() < 1e-6,
            "shotgun pellet count and spread come from the engine weapon definition",
        );
        self.next(Step::Eat, frame);
    }

    pub(super) fn lit(&mut self, ctx: &AddonContext<'_, Global>, frame: i32) {
        let spring = ctx.spring();
        let rabbits = ctx.global().roster.access().units(UnitKind::Rabbit);
        if self.rabbit < 0 {
            let Some(rabbit) = rabbits.first().copied() else {
                self.fail(spring, "a rabbit is there for the flashlight to follow");
                return;
            };
            self.rabbit = rabbit;
            self.before.fear = fear(ctx.global(), rabbit).unwrap_or(0.0);
        }
        for rabbit in rabbits {
            if let Ok(velocity) = spring.units_info().get_unit_velocity(rabbit) {
                self.fastest_rabbit = self.fastest_rabbit.max(velocity.x.hypot(velocity.z));
            }
        }
        if frame - self.since < LIT_FRAMES {
            if let Some(position) = units::unit_position(spring, self.rabbit) {
                send(ctx, protocol::MOVE_GUN, position);
            }
            return;
        }
        self.suite.check(
            spring,
            self.fastest_rabbit > RUNNING_SPEED,
            format!(
                "the rabbits run about: fastest {:.2} elmos a frame",
                self.fastest_rabbit
            ),
        );
        let fear = fear(ctx.global(), self.rabbit);
        self.suite.check(spring,
            fear.is_some_and(|fear| fear > LIT_FEAR),
            format!(
                "the flashlight frightens the rabbit it follows: fear {:.0} -> {fear:?} over {LIT_FRAMES} frames",
                self.before.fear
            ),
        );
        self.next(Step::Fire, frame);
    }

    pub(super) fn fire(&mut self, ctx: &AddonContext<'_, Global>, frame: i32) {
        let spring = ctx.spring();
        if frame % RESOURCE_PERIOD_FRAMES == 0 {
            return;
        }
        let global = ctx.global();
        let Some(position) = units::unit_position(spring, self.rabbit) else {
            self.fail(spring, "the lit rabbit is alive to be shot beside");
            return;
        };
        self.before.shotgun_ammo = global.weapons.access().shotgun_ammo;
        self.before.shots = global.score.access().shots_fired;
        self.before.fear = fear(global, self.rabbit).unwrap_or(0.0);
        let gun = global.weapons.access().gun_id;
        let Some(target) = nearby_shot_target(spring, gun, position) else {
            self.fail(spring, "the shotgun muzzle is available for a nearby shot");
            return;
        };
        self.shot_at = (target.x, target.z);
        send(ctx, protocol::FIRE, target);
        self.next(Step::Fired, frame);
    }

    pub(super) fn fired(
        &mut self,
        spring: &spring::Spring<'_, spring::RulesSynced>,
        global: &Global,
        frame: i32,
    ) {
        let (ammo, gun) = {
            let weapons = global.weapons.access();
            (weapons.shotgun_ammo, weapons.gun_id)
        };
        let shots = global.score.access().shots_fired;
        self.suite.check(
            spring,
            shots == self.before.shots + 1 && ammo == self.before.shotgun_ammo - 1.0,
            format!(
                "firing spends a shell and counts the shot: ammo {} -> {ammo}, shots {} -> {shots}",
                self.before.shotgun_ammo, self.before.shots
            ),
        );
        self.suite.check(
            spring,
            global.session.access().status_dirty,
            "firing sends the HUD a new status this frame",
        );
        let kick = spring
            .units_info()
            .get_unit_velocity(gun)
            .map(|velocity| velocity.y);
        self.suite.check(
            spring,
            kick.as_ref().is_ok_and(|kick| *kick > 0.0),
            format!("the recoil kicks the gun up: vertical speed {kick:?}"),
        );
        let (x, z) = self.shot_at;
        self.suite.check(
            spring,
            has_scare(global, x, z, Area::SHOTGUN),
            "the shot scares rabbits within 300 with strength 150",
        );
        self.next(Step::Scared, frame);
    }

    pub(super) fn scared(
        &mut self,
        spring: &spring::Spring<'_, spring::RulesSynced>,
        global: &Global,
        frame: i32,
    ) {
        let fear = fear(global, self.rabbit);
        self.suite.check(spring,
            fear.is_some_and(|fear| fear >= self.before.fear + SHOT_FEAR_JUMP),
            format!(
                "a shot {SHOT_OFFSET:.0} elmos away frightens a rabbit at once: fear {:.0} -> {fear:?}",
                self.before.fear
            ),
        );
        self.next(Step::ShootField, frame);
    }

    pub(super) fn shoot_field(&mut self, ctx: &AddonContext<'_, Global>, frame: i32) {
        let spring = ctx.spring();
        let target = free_carrots(ctx.global())
            .first()
            .and_then(|(carrot, _)| Some((*carrot, units::unit_position(spring, *carrot)?)));
        let Some((carrot, position)) = target else {
            self.fail(spring, "a carrot is left to shoot at");
            return;
        };
        self.shot_carrot = carrot;
        self.shot_at = (position.x, position.z);
        send(ctx, protocol::FIRE, position);
        self.next(Step::FieldShot, frame);
    }

    pub(super) fn field_shot(
        &mut self,
        spring: &spring::Spring<'_, spring::RulesSynced>,
        global: &Global,
        frame: i32,
    ) {
        if frame - self.since < PELLET_FRAMES {
            return;
        }
        let standing = global.roster.access().kind(self.shot_carrot) == Some(UnitKind::Carrot);
        let shot_down = self.shot_down.len();
        self.suite.check(spring,
            standing && shot_down == 0,
            format!(
                "shots into a field leave its carrots, the burrows and the gun alone: carrot shot at stands {standing}, {shot_down} destroyed by weapons"
            ),
        );
        self.next(Step::PlaceMine, frame);
    }

    pub(super) fn place_mine(&mut self, ctx: &AddonContext<'_, Global>, frame: i32) {
        let spring = ctx.spring();
        if frame % RESOURCE_PERIOD_FRAMES == 0 || (frame + 1) % RESOURCE_PERIOD_FRAMES == 0 {
            return;
        }
        let global = ctx.global();
        let (shot_x, shot_z) = self.shot_at;
        let shot = spring::Float3::new(shot_x, 0.0, shot_z);
        let spot = free_carrots(global)
            .into_iter()
            .find(|(_, position)| distance_2d(*position, shot) >= MINE_CLEAR_OF_SHOT);
        let rabbit = global
            .roster
            .access()
            .units(UnitKind::Rabbit)
            .first()
            .copied();
        let (Some((_, position)), Some(rabbit)) = (spot, rabbit) else {
            self.fail(
                spring,
                "a free carrot clear of the shots and a rabbit are left for the mine",
            );
            return;
        };
        let beside = spring::Float3 {
            x: position.x + TRIGGER_OFFSET,
            ..position
        };
        if let Err(error) = teleport(spring, rabbit, beside) {
            self.fail(
                spring,
                &format!("a rabbit moves beside the mine: {error:?}"),
            );
            return;
        }
        self.blast_carrots = global
            .roster
            .access()
            .positions(UnitKind::Carrot)
            .into_iter()
            .filter(|(_, carrot)| distance_2d(*carrot, position) <= MINE_BLAST_RADIUS)
            .map(|(carrot, _)| carrot)
            .collect();
        {
            let weapons = global.weapons.access();
            self.before.mine_ammo = weapons.mine_ammo;
            self.before.mines = weapons.mines.len();
        }
        send(ctx, protocol::PLACE_MINE, position);
        self.next(Step::MinePlaced, frame);
    }

    pub(super) fn mine_placed(
        &mut self,
        spring: &spring::Spring<'_, spring::RulesSynced>,
        global: &Global,
        frame: i32,
    ) {
        if frame - self.since < 2 {
            return;
        }
        let (mines, ammo, mine) = {
            let weapons = global.weapons.access();
            (
                weapons.mines.len(),
                weapons.mine_ammo,
                weapons.mines.last().copied(),
            )
        };
        let in_roster = global.roster.access().count(UnitKind::Mine);
        self.suite.check(spring,
            mines == self.before.mines + 1
                && in_roster == mines
                && ammo == self.before.mine_ammo - 1.0,
            format!(
                "placing a mine spends one and arms it: {mines} armed, {in_roster} in play, ammo {} -> {ammo}",
                self.before.mine_ammo
            ),
        );
        self.suite.check(
            spring,
            global.session.access().status_dirty,
            "placing a mine sends the HUD a new status this frame",
        );
        self.mine = mine.map(|mine| (mine.unit_id, mine.x, mine.z));
        self.next(Step::FuseLit, frame);
        self.fuse_lit(spring, global, frame);
    }

    pub(super) fn fuse_lit(
        &mut self,
        spring: &spring::Spring<'_, spring::RulesSynced>,
        global: &Global,
        frame: i32,
    ) {
        let unit = self.mine.map_or(-1, |mine| mine.0);
        let lit = global
            .weapons
            .access()
            .mines
            .iter()
            .any(|mine| mine.unit_id == unit && mine.fuse.is_some());
        if !lit {
            let _ = self.timed_out(spring, frame, "a rabbit beside the mine lights its fuse");
            return;
        }
        let waited = frame - self.since;
        match self.hill_y {
            None => self.suite.check(spring,
                waited <= FUSE_WAIT,
                format!("the rabbit beside the mine lights its fuse: after {waited} frames"),
            ),
            Some(y) => self.suite.check(spring,
                waited <= FUSE_WAIT && y > MINE_TRIGGER_RADIUS,
                format!(
                    "a rabbit on raised ground (y {y:.0}) lights the mine's fuse: after {waited} frames"
                ),
            ),
        };
        self.fuse_lit = frame;
        self.next(Step::Detonated, frame);
    }

    pub(super) fn detonated(
        &mut self,
        spring: &spring::Spring<'_, spring::RulesSynced>,
        global: &Global,
        frame: i32,
    ) {
        let Some((unit, x, z)) = self.mine else {
            return;
        };
        let armed = global
            .weapons
            .access()
            .mines
            .iter()
            .any(|mine| mine.unit_id == unit);
        if armed {
            let _ = self.timed_out(spring, frame, "the mine goes off");
            return;
        }
        if self.hill_y.is_some() {
            self.next(Step::Kill, frame);
            return;
        }
        self.suite.check(
            spring,
            frame - self.fuse_lit == MINE_FUSE_FRAMES,
            format!(
                "the mine goes off {} frames after its fuse is lit (expected {MINE_FUSE_FRAMES})",
                frame - self.fuse_lit
            ),
        );
        self.suite.check(
            spring,
            has_scare(global, x, z, Area::MINE),
            "the mine scares rabbits within 250 with strength 400",
        );
        let blasted = self
            .blast_carrots
            .iter()
            .filter(|carrot| self.shot_down.contains(carrot))
            .count();
        self.suite.check(spring,
            !self.blast_carrots.is_empty() && blasted == 0,
            format!(
                "a mine going off in a field leaves its carrots: {blasted} of {} within its blast destroyed",
                self.blast_carrots.len()
            ),
        );
        self.next(Step::HillMine, frame);
    }

    pub(super) fn hill_mine(&mut self, ctx: &AddonContext<'_, Global>, frame: i32) {
        let spring = ctx.spring();
        let global = ctx.global();
        let Some(hill) = highest_ground(spring) else {
            self.fail(spring, "the map's ground heights are readable");
            return;
        };
        let rabbit = global
            .roster
            .access()
            .units(UnitKind::Rabbit)
            .first()
            .copied();
        let Some(rabbit) = rabbit else {
            self.fail(spring, "a rabbit is left to put on the hill");
            return;
        };
        if let Err(error) = spring.unit_control().set_unit_position(rabbit, hill) {
            self.fail(
                spring,
                &format!("the rabbit moves onto the hill: {error:?}"),
            );
            return;
        }
        self.before.mines = {
            let mut weapons = global.weapons.access_mut();
            weapons.mine_ammo = weapons.mine_ammo.max(1.0);
            weapons.mines.len()
        };
        send(ctx, protocol::PLACE_MINE, hill);
        self.hill_y = Some(hill.y);
        self.next(Step::HillMinePlaced, frame);
    }

    pub(super) fn hill_mine_placed(
        &mut self,
        spring: &spring::Spring<'_, spring::RulesSynced>,
        global: &Global,
        frame: i32,
    ) {
        if frame - self.since < 2 {
            return;
        }
        let mine = {
            let weapons = global.weapons.access();
            weapons
                .mines
                .last()
                .filter(|_| weapons.mines.len() > self.before.mines)
                .map(|mine| (mine.unit_id, mine.x, mine.z))
        };
        if mine.is_none() {
            self.fail(spring, "a mine is placed on the hill");
            return;
        }
        self.mine = mine;
        self.next(Step::FuseLit, frame);
        self.fuse_lit(spring, global, frame);
    }
}

pub(super) fn blocks(spring: &spring::Spring<'_, spring::RulesSynced>, unit: i32) -> bool {
    spring
        .units_info()
        .get_unit_blocking(unit)
        .map_or(true, |state| {
            state.is_blocking || state.is_solid_object_collidable
        })
}

fn highest_ground(spring: &spring::Spring<'_, spring::RulesSynced>) -> Option<spring::Float3> {
    let size = 2.0 * map::CENTER;
    let cells = (size / HILL_GRID) as usize;
    let points: Vec<[f32; 2]> = (0..cells * cells)
        .map(|cell| {
            let at = |index: usize| (index as f32 + 0.5) * HILL_GRID;
            [at(cell % cells), at(cell / cells)]
        })
        .collect();
    let heights = spring.terrain().ground_heights(&points).ok()?;
    points
        .iter()
        .zip(heights)
        .max_by(|a, b| a.1.total_cmp(&b.1))
        .map(|(&[x, z], y)| spring::Float3 { x, y, z })
}

// This case measures fear from a nearby shot, not a pellet hitting its subject.
// Pick among the same four diagonal offsets by their distance from the actual
// muzzle-to-target segment. A fixed diagonal can put the rabbit on that segment
// when the UI first spawns the gun a frame earlier or later.
fn nearby_shot_target(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    gun: i32,
    rabbit: spring::Float3,
) -> Option<spring::Float3> {
    let flare = spring
        .units_pieces()
        .get_unit_piece_map(gun)
        .ok()?
        .into_iter()
        .find(|piece| piece.name == "flare")?
        .piece_num;
    let muzzle = spring
        .units_pieces()
        .get_unit_piece_pos_dir(gun, flare)
        .ok()?
        .position;
    let mut best = None;
    let mut clearance = -1.0;
    for (dx, dz) in [
        (SHOT_OFFSET, -SHOT_OFFSET),
        (SHOT_OFFSET, SHOT_OFFSET),
        (-SHOT_OFFSET, -SHOT_OFFSET),
        (-SHOT_OFFSET, SHOT_OFFSET),
    ] {
        let target = spring::Float3 {
            x: rabbit.x + dx,
            z: rabbit.z + dz,
            ..rabbit
        };
        let direction = [
            target.x - muzzle.x,
            target.y - muzzle.y,
            target.z - muzzle.z,
        ];
        let offset = [
            rabbit.x - muzzle.x,
            rabbit.y - muzzle.y,
            rabbit.z - muzzle.z,
        ];
        let length_squared = direction.iter().map(|x| x * x).sum::<f32>();
        let along = (offset
            .iter()
            .zip(direction)
            .map(|(a, b)| a * b)
            .sum::<f32>()
            / length_squared.max(0.001))
        .clamp(0.0, 1.0);
        let distance_squared = offset
            .iter()
            .zip(direction)
            .map(|(a, b)| (a - along * b).powi(2))
            .sum::<f32>();
        if distance_squared > clearance {
            clearance = distance_squared;
            best = Some(target);
        }
    }
    best
}
