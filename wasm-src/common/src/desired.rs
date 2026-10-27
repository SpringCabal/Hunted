use std::collections::BTreeMap;

/// The Lua AI appends areas and units to one list and swap-removes them.
/// Its first entry wins equal desirability, independently of engine unit IDs.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Key {
    Unit(i32),
    Field(usize),
}

#[derive(Clone, Default)]
pub struct Desirables {
    order: Vec<Key>,
    indices: BTreeMap<Key, usize>,
    original_fields: BTreeMap<i32, usize>,
    fields: Vec<Vec<i32>>,
}

impl Desirables {
    fn insert(&mut self, key: Key) {
        assert!(!self.indices.contains_key(&key), "duplicate desirable");
        self.indices.insert(key, self.order.len());
        self.order.push(key);
    }

    fn remove(&mut self, key: Key) {
        let Some(index) = self.indices.remove(&key) else {
            return;
        };
        self.order.swap_remove(index);
        if let Some(moved) = self.order.get(index) {
            self.indices.insert(*moved, index);
        }
    }

    pub fn insert_unit(&mut self, unit: i32) {
        self.insert(Key::Unit(unit));
    }

    pub fn begin_field(&mut self, field: usize) {
        if self.fields.len() <= field {
            self.fields.resize_with(field + 1, Vec::new);
        }
        assert!(
            self.fields[field].is_empty(),
            "field still has original carrots"
        );
        self.insert(Key::Field(field));
    }

    pub fn add_original_carrot(&mut self, field: usize, unit: i32) {
        assert!(self.indices.contains_key(&Key::Unit(unit)));
        assert!(self.indices.contains_key(&Key::Field(field)));
        assert!(self.original_fields.insert(unit, field).is_none());
        self.fields[field].push(unit);
    }

    pub fn remove_unit(&mut self, unit: i32) {
        // UnitDestroyed visits the Lua gadget list in reverse layer order:
        // Rabbit AI (10) removes its unit before Field Spawner (15) removes
        // an exhausted field. Both swap-removals affect subsequent tie order.
        self.remove(Key::Unit(unit));
        if let Some(field) = self.original_fields.remove(&unit) {
            self.fields[field].retain(|id| *id != unit);
            if self.fields[field].is_empty() {
                self.remove(Key::Field(field));
            }
        }
    }

    pub fn rank(&self, key: Key) -> usize {
        self.indices.get(&key).copied().expect("tracked desirable")
    }

    /// Live unit desirables in the Lua list's current order, including swaps.
    pub fn ordered_units(&self) -> impl Iterator<Item = i32> + '_ {
        self.order.iter().filter_map(|key| match key {
            Key::Unit(unit) => Some(*unit),
            Key::Field(_) => None,
        })
    }

    pub fn fields(&self) -> &[Vec<i32>] {
        &self.fields
    }
}

/// First list entry wins a rounded desirability tie, even if a later point
/// is slightly nearer. Distances are squared; entries outside the area lose.
pub fn best_falloff(
    distances: impl Iterator<Item = f32>,
    radius: f32,
    proximity: f32,
    edge: f32,
) -> Option<(usize, f32, f32)> {
    let mut nearest = radius * radius;
    let mut best = None;
    let mut best_magnitude = 0.0;
    for (index, square) in distances.enumerate() {
        if square >= nearest {
            continue;
        }
        nearest = square;
        let magnitude = proximity * (1.0 - square.sqrt() / radius) + edge;
        if magnitude > best_magnitude {
            best_magnitude = magnitude;
            best = Some((index, square, magnitude));
        }
    }
    best
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unit_iteration_keeps_both_swaps_when_a_field_is_exhausted() {
        let mut desirables = Desirables::default();
        desirables.begin_field(0);
        desirables.insert_unit(10);
        desirables.add_original_carrot(0, 10);
        desirables.begin_field(1);
        desirables.insert_unit(20);
        desirables.insert_unit(30);
        assert_eq!(desirables.ordered_units().collect::<Vec<_>>(), [10, 20, 30]);
        desirables.remove_unit(10);
        assert_eq!(desirables.ordered_units().collect::<Vec<_>>(), [20, 30]);
        desirables.insert_unit(40);
        desirables.remove_unit(20);
        assert_eq!(desirables.ordered_units().collect::<Vec<_>>(), [40, 30]);
    }

    #[test]
    fn rounded_falloff_ties_retain_the_earlier_target() {
        let distances = [50.00001_f32.powi(2), 50.0_f32.powi(2)];
        for radius in [1500.0, 9000.0] {
            assert_eq!(
                2.0 * (1.0 - distances[0].sqrt() / radius) + 0.1,
                2.0 * (1.0 - distances[1].sqrt() / radius) + 0.1
            );
            assert_eq!(
                best_falloff(distances.into_iter(), radius, 2.0, 0.1)
                    .unwrap()
                    .0,
                0
            );
        }
        assert_eq!(
            best_falloff([9000.0_f32.powi(2)].into_iter(), 9000.0, 2.0, 1.0),
            None
        );
    }

    #[test]
    fn equal_desirability_uses_creation_order_then_swap_removal() {
        let mut targets = Desirables::default();
        targets.begin_field(0);
        targets.insert_unit(900);
        targets.add_original_carrot(0, 900);
        targets.insert_unit(3);
        targets.insert_unit(700);
        assert!(targets.rank(Key::Unit(900)) < targets.rank(Key::Unit(3)));
        targets.remove_unit(900);
        assert_eq!(targets.order, [Key::Unit(3), Key::Unit(700)]);
    }

    #[test]
    fn extra_carrots_do_not_keep_a_field_alive_and_reused_ids_do_not_rejoin_it() {
        let mut targets = Desirables::default();
        targets.begin_field(0);
        for unit in [10, 20] {
            targets.insert_unit(unit);
            targets.add_original_carrot(0, unit);
        }
        targets.insert_unit(30); // A given/cloned carrot inside the same bounds.
        targets.remove_unit(10);
        targets.insert_unit(10); // Engine IDs can be reused for a new carrot.
        assert_eq!(targets.fields(), &[vec![20]]);
        targets.remove_unit(20);
        assert!(targets.fields()[0].is_empty());
        assert!(!targets.indices.contains_key(&Key::Field(0)));
        assert!(targets.indices.contains_key(&Key::Unit(10)));
        assert!(targets.indices.contains_key(&Key::Unit(30)));
    }

    #[test]
    fn restarting_rebuilds_fields_and_their_original_members() {
        let mut targets = Desirables::default();
        for unit in [40, 50] {
            targets.begin_field(0);
            targets.insert_unit(unit);
            targets.add_original_carrot(0, unit);
            targets.remove_unit(unit);
            assert!(targets.order.is_empty());
        }
    }
}
