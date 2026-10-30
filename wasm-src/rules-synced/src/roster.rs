use alloc::collections::BTreeMap;
use alloc::vec::Vec;

use crate::defs::UnitKind;

pub struct Roster {
    by_kind: [BTreeMap<i32, spring::Float3>; UnitKind::ALL.len()],
    kinds: BTreeMap<i32, UnitKind>,
    rabbit_revision: u64,
    food_grid: hunted_common::spatial::Grid,
    food: Vec<Option<(UnitKind, spring::Float3)>>,
    pub desirables: hunted_common::desired::Desirables,
}

impl Default for Roster {
    fn default() -> Self {
        Self {
            by_kind: Default::default(),
            kinds: BTreeMap::new(),
            rabbit_revision: 0,
            desirables: Default::default(),
            food_grid: hunted_common::spatial::Grid::new(1500.0, []),
            food: Vec::new(),
        }
    }
}

impl Roster {
    pub fn track(&mut self, unit: i32, kind: UnitKind, position: spring::Float3) {
        if let Some(old) = self.kinds.insert(unit, kind) {
            self.by_kind[old.index()].remove(&unit);
            if old == UnitKind::Rabbit {
                self.rabbit_revision += 1;
            }
            if is_static_target(old) {
                self.desirables.remove_unit(unit);
                self.unindex_food(unit);
            }
        }
        self.by_kind[kind.index()].insert(unit, position);
        if kind == UnitKind::Rabbit {
            self.rabbit_revision += 1;
        }
        if is_static_target(kind) {
            self.desirables.insert_unit(unit);
            self.index_food(unit, kind, position);
        }
    }

    pub fn untrack(&mut self, unit: i32) {
        if let Some(kind) = self.kinds.remove(&unit) {
            self.by_kind[kind.index()].remove(&unit);
            if kind == UnitKind::Rabbit {
                self.rabbit_revision += 1;
            }
            if is_static_target(kind) {
                self.desirables.remove_unit(unit);
                self.unindex_food(unit);
            }
        }
    }

    pub fn begin_field(&mut self, field: usize) {
        self.desirables.begin_field(field);
    }

    pub fn add_field_carrot(&mut self, field: usize, unit: i32) {
        self.desirables.add_original_carrot(field, unit);
    }

    fn index_food(&mut self, unit: i32, kind: UnitKind, position: spring::Float3) {
        if !matches!(
            kind,
            UnitKind::Carrot | UnitKind::CarrotDropped | UnitKind::Mine
        ) {
            return;
        }
        let index = unit as usize;
        if index >= self.food.len() {
            self.food.resize(index + 1, None);
        }
        self.food[index] = Some((kind, position));
        self.food_grid.insert(index, position.x, position.z);
    }

    fn unindex_food(&mut self, unit: i32) {
        let index = unit as usize;
        self.food_grid.remove(index);
        if let Some(entry) = self.food.get_mut(index) {
            *entry = None;
        }
    }

    pub fn visit_food(&self, x: f32, z: f32, mut visit: impl FnMut(i32, UnitKind, spring::Float3)) {
        self.food_grid.visit_nearby(x, z, |index| {
            let (kind, position) = self.food[index].expect("indexed food");
            visit(index as i32, kind, position);
        });
    }

    pub fn food_position(&self, unit: i32) -> Option<spring::Float3> {
        self.food
            .get(usize::try_from(unit).ok()?)
            .copied()
            .flatten()
            .map(|(_, p)| p)
    }

    pub fn visit_kind(&self, kind: UnitKind, mut visit: impl FnMut(i32, spring::Float3)) {
        for (&unit, &position) in &self.by_kind[kind.index()] {
            visit(unit, position);
        }
    }

    pub fn rabbit_revision(&self) -> u64 {
        self.rabbit_revision
    }

    pub fn kind(&self, unit: i32) -> Option<UnitKind> {
        self.kinds.get(&unit).copied()
    }

    pub fn units(&self, kind: UnitKind) -> Vec<i32> {
        self.by_kind[kind.index()].keys().copied().collect()
    }

    pub fn count(&self, kind: UnitKind) -> usize {
        self.by_kind[kind.index()].len()
    }

    pub fn positions(&self, kind: UnitKind) -> Vec<(i32, spring::Float3)> {
        self.by_kind[kind.index()]
            .iter()
            .map(|(unit, position)| (*unit, *position))
            .collect()
    }

    pub fn carrot_count(&self) -> i32 {
        UnitKind::ALL
            .into_iter()
            .filter(|kind| kind.is_carrot())
            .map(|kind| self.count(kind) as i32)
            .sum()
    }
}

fn is_static_target(kind: UnitKind) -> bool {
    matches!(
        kind,
        UnitKind::Carrot | UnitKind::CarrotDropped | UnitKind::Mine | UnitKind::Burrow
    )
}
