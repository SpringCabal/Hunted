use alloc::collections::BTreeSet;
use alloc::vec::Vec;
use core::cell::{Cell, RefCell};

use hunted_common::lighthouse;
use spring_addons::AddonContext;
use spring_addons::event::UnitDestroyedEvent;
use spring_addons::ui::Widget;

use crate::defs::defs;
use crate::diagnostics;
use crate::state::Global;
use crate::state::Point3;

const FLASHLIGHT_REVEAL_RADIUS: f32 = 172.5;
const ALL_UNITS: i32 = -1;

#[derive(Clone, Copy)]
pub struct Beam {
    pub x: f32,
    pub z: f32,
    pub size: f32,
}

impl Beam {
    fn reveal_radius(&self) -> f32 {
        90.0 + 1.1 * self.size / 2.0
    }
}

#[derive(Default)]
pub struct Visibility {
    rabbits: BTreeSet<i32>,
    revealed: BTreeSet<i32>,
    lighthouses: Vec<i32>,
    pub beams: Vec<Beam>,
}

impl Visibility {
    pub fn add_rabbit(&mut self, unit_id: i32) {
        let _ = spring::unsynced_ctrl::set_unit_no_draw(unit_id, true);
        let _ = spring::unsynced_ctrl::set_unit_no_select(unit_id, true);
        let _ = spring::unsynced_ctrl::set_unit_no_minimap(unit_id, true);
        self.rabbits.insert(unit_id);
    }

    pub fn add_lighthouse(&mut self, unit_id: i32) {
        self.lighthouses.push(unit_id);
    }

    pub fn remove(&mut self, unit_id: i32) {
        self.rabbits.remove(&unit_id);
        self.revealed.remove(&unit_id);
        self.lighthouses.retain(|lighthouse| *lighthouse != unit_id);
    }

    pub fn update(&mut self, flashlight: Option<Point3>, flare_active: bool, frame: i32) {
        self.beams = self
            .lighthouses
            .iter()
            .filter_map(|unit| beam(*unit))
            .collect();
        let revealed = if flare_active {
            self.rabbits.clone()
        } else {
            let mut revealed = BTreeSet::new();
            let areas = flashlight
                .map(|target| (target.x, target.z, FLASHLIGHT_REVEAL_RADIUS))
                .into_iter()
                .chain(
                    self.beams
                        .iter()
                        .map(|beam| (beam.x, beam.z, beam.reveal_radius())),
                );
            for (x, z, radius) in areas {
                let Ok(units) = spring::get_units_in_cylinder(x, z, radius, ALL_UNITS) else {
                    continue;
                };
                revealed.extend(units.into_iter().filter(|unit| self.rabbits.contains(unit)));
            }
            revealed
        };
        let mut failures = 0;
        for (unit, drawn) in self
            .revealed
            .difference(&revealed)
            .map(|unit| (*unit, false))
            .chain(
                revealed
                    .difference(&self.revealed)
                    .map(|unit| (*unit, true)),
            )
        {
            if spring::unsynced_ctrl::set_unit_no_draw(unit, !drawn).is_err() {
                failures += 1;
            }
        }
        diagnostics::visibility(
            frame,
            self.rabbits.len() as i32,
            revealed.len() as i32,
            failures,
            flare_active,
        );
        self.revealed = revealed;
    }
}

fn beam(lighthouse: i32) -> Option<Beam> {
    let position = spring::units_info::get_unit_position(
        lighthouse,
        spring::units_info::GetUnitPositionOptions::default(),
    )
    .ok()?;
    let param = |name: &str| spring::get_unit_rules_param_f32(lighthouse, name).ok();
    Some(Beam {
        x: param(lighthouse::PARAM_X).unwrap_or(position.x),
        z: param(lighthouse::PARAM_Z).unwrap_or(position.z),
        size: param(lighthouse::PARAM_SIZE).unwrap_or(lighthouse::BEAM_SIZE),
    })
}

#[derive(Default)]
pub struct GfxRabbitVisibilityWidget {
    visibility: RefCell<Visibility>,
    last_pass: Cell<Option<Pass>>,
}

type Pass = (i32, Option<(f32, f32)>, bool);

impl GfxRabbitVisibilityWidget {
    pub fn new() -> Self {
        Self::default()
    }

    fn add(&self, unit: i32, def: i32) {
        let defs = defs();
        if Some(def) == defs.rabbit {
            self.visibility.borrow_mut().add_rabbit(unit);
        } else if Some(def) == defs.lighthouse {
            self.visibility.borrow_mut().add_lighthouse(unit);
        }
    }
}

impl Widget<Global> for GfxRabbitVisibilityWidget {
    fn name(&self) -> &'static str {
        "GfxRabbitVisibility"
    }

    fn init(&self, _ctx: &AddonContext<'_, Global>) {
        for unit in spring::get_all_units().unwrap_or_default() {
            if let Ok(def) = spring::units_info::get_unit_def_id(unit) {
                self.add(unit, def);
            }
        }
    }

    fn unit_created(
        &self,
        _ctx: &AddonContext<'_, Global>,
        unit: i32,
        def: i32,
        _team: i32,
        _builder: i32,
    ) {
        self.add(unit, def);
    }

    fn unit_destroyed(&self, _ctx: &AddonContext<'_, Global>, event: &UnitDestroyedEvent) {
        self.visibility.borrow_mut().remove(event.unit_id);
    }

    fn update(&self, ctx: &AddonContext<'_, Global>, _dt: f32) {
        let global = ctx.global();
        let round = *global.round.access();
        let flashlight = global.lights.access().flashlight;
        let pass = (
            spring::game::frame().unwrap_or(0) as i32,
            flashlight.map(|target| (target.x, target.z)),
            round.flare_active,
        );
        if self.last_pass.replace(Some(pass)) == Some(pass) {
            return;
        }
        let mut visibility = self.visibility.borrow_mut();
        visibility.update(flashlight, round.flare_active, round.frame);
        global
            .lights
            .access_mut()
            .beams
            .clone_from(&visibility.beams);
    }
}
