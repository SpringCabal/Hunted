use std::collections::BTreeMap;

/// Coarse point buckets; callers retain their exact distance and ranking tests.
pub struct Grid {
    size: f32,
    cells: BTreeMap<(i32, i32), Vec<usize>>,
}

impl Grid {
    pub fn new(size: f32, points: impl IntoIterator<Item = (f32, f32)>) -> Self {
        assert!(size.is_finite() && size > 0.0);
        let mut grid = Self {
            size,
            cells: BTreeMap::new(),
        };
        for (index, (x, z)) in points.into_iter().enumerate() {
            grid.cells.entry(grid.cell(x, z)).or_default().push(index);
        }
        grid
    }

    fn cell(&self, x: f32, z: f32) -> (i32, i32) {
        (
            (x / self.size).floor() as i32,
            (z / self.size).floor() as i32,
        )
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
