use springcabal::defs::DefCache;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum UnitKind {
    Rabbit,
    Carrot,
    CarrotCarried,
    CarrotDropped,
    Burrow,
    Lighthouse,
    Mine,
    Shotgun,
}

impl UnitKind {
    pub const ALL: [UnitKind; 8] = [
        UnitKind::Rabbit,
        UnitKind::Carrot,
        UnitKind::CarrotCarried,
        UnitKind::CarrotDropped,
        UnitKind::Burrow,
        UnitKind::Lighthouse,
        UnitKind::Mine,
        UnitKind::Shotgun,
    ];

    pub const fn def_name(self) -> &'static str {
        match self {
            UnitKind::Rabbit => "rabbit",
            UnitKind::Carrot => "carrot",
            UnitKind::CarrotCarried => "carrot_carried",
            UnitKind::CarrotDropped => "carrot_dropped",
            UnitKind::Burrow => "burrow",
            UnitKind::Lighthouse => "lighthouse",
            UnitKind::Mine => "mine",
            UnitKind::Shotgun => "shotgun",
        }
    }

    pub const fn index(self) -> usize {
        self as usize
    }

    pub const fn is_carrot(self) -> bool {
        matches!(
            self,
            UnitKind::Carrot | UnitKind::CarrotCarried | UnitKind::CarrotDropped
        )
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Defs {
    unit_ids: [i32; UnitKind::ALL.len()],
    pub rabbit_speed: f32,
    pub shotgun_speed: f32,
    pub shotgun_weapon: Option<i32>,
    pub shotgun_projectile_speed: f32,
    pub shotgun_projectiles: i32,
    pub shotgun_spray_angle: f32,
}

impl Defs {
    fn resolve() -> Self {
        let unit_ids = UnitKind::ALL
            .map(|kind| spring::get_unit_def_id_by_name(kind.def_name()).unwrap_or(-1));
        let speed =
            |kind: UnitKind| spring::get_unit_def_speed(unit_ids[kind.index()]).unwrap_or(0.0);
        let shotgun_weapon = spring::weapon_defs::get_weapon_def_id("shotgun").ok();
        let shotgun_projectile_speed = shotgun_weapon
            .and_then(|id| spring::weapon_defs::get_weapon_def_by_id(id).ok())
            .filter(|def| def.exists)
            .map_or(2000.0 / spring::consts::GAME_SPEED as f32, |def| {
                def.info.projectile_speed
            });
        let projectile_params = shotgun_weapon
            .and_then(|id| spring::weapon_defs::get_weapon_def_projectile_params(id).ok());
        Self {
            shotgun_projectiles: projectile_params
                .as_ref()
                .map_or(0, |params| params.projectiles),
            shotgun_spray_angle: projectile_params
                .as_ref()
                .map_or(0.0, |params| params.spray_angle),
            unit_ids,
            rabbit_speed: speed(UnitKind::Rabbit),
            shotgun_speed: speed(UnitKind::Shotgun),
            shotgun_weapon,
            shotgun_projectile_speed,
        }
    }

    pub fn unit_def(&self, kind: UnitKind) -> i32 {
        self.unit_ids[kind.index()]
    }

    pub fn kind_of(&self, unit_def: i32) -> Option<UnitKind> {
        if unit_def < 0 {
            return None;
        }
        UnitKind::ALL
            .into_iter()
            .find(|kind| self.unit_ids[kind.index()] == unit_def)
    }
}

static DEFS: DefCache<Defs> = DefCache::new(Defs::resolve);

pub fn defs() -> Defs {
    *DEFS.get()
}

pub fn kind_of(unit_def: i32) -> Option<UnitKind> {
    defs().kind_of(unit_def)
}
