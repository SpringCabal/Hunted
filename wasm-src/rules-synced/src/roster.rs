use alloc::collections::BTreeMap;
use alloc::vec::Vec;

use crate::defs::UnitKind;

#[derive(Default)]
pub struct Roster {
    by_kind: [BTreeMap<i32, spring::Float3>; UnitKind::ALL.len()],
    kinds: BTreeMap<i32, UnitKind>,
    static_revision: u64,
}

impl Roster {
    pub fn track(&mut self, unit: i32, kind: UnitKind, position: spring::Float3) {
        if let Some(old) = self.kinds.insert(unit, kind) {
            self.by_kind[old.index()].remove(&unit);
            if is_static_target(old) {
                self.static_revision += 1;
            }
        }
        self.by_kind[kind.index()].insert(unit, position);
        if is_static_target(kind) {
            self.static_revision += 1;
        }
    }

    pub fn untrack(&mut self, unit: i32) {
        if let Some(kind) = self.kinds.remove(&unit) {
            self.by_kind[kind.index()].remove(&unit);
            if is_static_target(kind) {
                self.static_revision += 1;
            }
        }
    }

    pub fn static_revision(&self) -> u64 {
        self.static_revision
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
