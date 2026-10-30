use std::collections::BTreeMap;

/// Coarse point buckets; callers retain their exact distance and ranking tests.
pub struct Grid {
    size: f32,
    cells: BTreeMap<(i32, i32), Vec<usize>>,
    locations: Vec<Option<((i32, i32), usize)>>,
}

impl Grid {
    pub fn new(size: f32, points: impl IntoIterator<Item = (f32, f32)>) -> Self {
        assert!(size.is_finite() && size > 0.0);
        let mut grid = Self {
            size,
            cells: BTreeMap::new(),
            locations: Vec::new(),
        };
        for (index, (x, z)) in points.into_iter().enumerate() {
            grid.insert(index, x, z);
        }
        grid
    }

    /// Unit IDs remain stable as other points are removed. Like Lua's bucket
    /// entries, removal swaps the last member and fixes its saved slot.
    pub fn insert(&mut self, index: usize, x: f32, z: f32) {
        self.remove(index);
        let key = self.cell(x, z);
        let cell = self.cells.entry(key).or_default();
        let slot = cell.len();
        cell.push(index);
        if index >= self.locations.len() {
            self.locations.resize(index + 1, None);
        }
        self.locations[index] = Some((key, slot));
    }

    pub fn remove(&mut self, index: usize) {
        let Some((key, slot)) = self.locations.get_mut(index).and_then(Option::take) else {
            return;
        };
        let cell = self.cells.get_mut(&key).expect("indexed cell");
        cell.swap_remove(slot);
        if let Some(&moved) = cell.get(slot) {
            self.locations[moved] = Some((key, slot));
        }
        if cell.is_empty() {
            self.cells.remove(&key);
        }
    }

    fn cell(&self, x: f32, z: f32) -> (i32, i32) {
        (
            (x / self.size).floor() as i32,
            (z / self.size).floor() as i32,
        )
    }

    /// Visit candidates directly so the hot query has no nested flat-map state.
    pub fn visit_nearby(&self, x: f32, z: f32, mut visit: impl FnMut(usize)) {
        let (x, z) = self.cell(x, z);
        for dx in -1..=1 {
            for dz in -1..=1 {
                if let Some(cell) = self.cells.get(&(x + dx, z + dz)) {
                    for &index in cell {
                        visit(index);
                    }
                }
            }
        }
    }

    /// Includes every point within `size` of the query, without allocating.
    pub fn nearby(&self, x: f32, z: f32) -> impl Iterator<Item = usize> + '_ {
        let (x, z) = self.cell(x, z);
        (-1..=1).flat_map(move |dx| {
            (-1..=1)
                .filter_map(move |dz| self.cells.get(&(x + dx, z + dz)))
                .flatten()
                .copied()
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn incremental_removal_movement_and_reused_ids_match_a_full_scan() {
        let mut grid = Grid::new(1500.0, []);
        let mut points = BTreeMap::new();
        for step in 0..600 {
            let id = (step * 61) % 127;
            if step % 3 == 0 {
                grid.remove(id);
                points.remove(&id);
            } else {
                let p = (
                    ((step * 173) % 9000) as f32 - 4500.0,
                    ((step * 97) % 9000) as f32 - 4500.0,
                );
                grid.insert(id, p.0, p.1);
                points.insert(id, p);
            }
            for (x, z) in [(-1500.0, 0.0), (0.0, 0.0), (1500.0, 1500.0)] {
                let found: Vec<_> = grid.nearby(x, z).collect();
                let unique: std::collections::BTreeSet<_> = found.iter().copied().collect();
                assert_eq!(found.len(), unique.len());
                for id in &found {
                    assert!(points.contains_key(id));
                }
                for (&id, &(px, pz)) in &points {
                    if (px - x).powi(2) + (pz - z).powi(2) <= 1500.0 * 1500.0 {
                        assert!(unique.contains(&id));
                    }
                }
            }
        }
        for &id in points.keys() {
            grid.remove(id);
        }
        assert!(grid.cells.is_empty());
    }

    #[test]
    fn buckets_cover_the_circle_including_negative_coordinates_and_boundaries() {
        let mut points = vec![(-1500.0, 0.0), (0.0, 0.0), (1500.0, 0.0), (1500.001, 0.0)];
        for i in 0..1000 {
            points.push((
                ((i * 61) % 12000) as f32 - 6000.0,
                ((i * 173) % 12000) as f32 - 6000.0,
            ));
        }
        let grid = Grid::new(1500.0, points.iter().copied());
        for (x, z) in [
            (-3000.0, -1500.0),
            (-0.001, 0.0),
            (0.0, 0.0),
            (1500.0, 1500.0),
            (6144.0, 6144.0),
        ] {
            let found: Vec<_> = grid.nearby(x, z).collect();
            let unique: std::collections::BTreeSet<_> = found.iter().copied().collect();
            assert_eq!(unique.len(), found.len());
            for (index, (px, pz)) in points.iter().enumerate() {
                if (x - px).powi(2) + (z - pz).powi(2) <= 1500.0 * 1500.0 {
                    assert!(
                        unique.contains(&index),
                        "missed point {index} near ({x}, {z})"
                    );
                }
            }
        }
    }

    #[test]
    fn empty_and_distant_queries_have_no_candidates() {
        assert_eq!(Grid::new(1500.0, []).nearby(0.0, 0.0).count(), 0);
        assert_eq!(
            Grid::new(1500.0, [(0.0, 0.0)])
                .nearby(6000.0, 6000.0)
                .count(),
            0
        );
    }
}
