const MAP_SIZE: f32 = 6144.0;
const EDGE_MARGIN: f32 = 1.0;
pub const CENTER: f32 = MAP_SIZE / 2.0;

pub fn clamp_to_map(coordinate: f32) -> f32 {
    coordinate.clamp(EDGE_MARGIN, MAP_SIZE - EDGE_MARGIN)
}

pub const CARROT_COUNT: i32 = 100;

pub const BURROWS: &[(f32, f32)] = &[
    (3600.0, 1350.0),
    (4740.0, 1940.0),
    (4650.0, 3340.0),
    (4920.0, 4620.0),
    (3680.0, 5270.0),
    (2190.0, 4820.0),
    (1000.0, 3660.0),
    (800.0, 2170.0),
    (2050.0, 1250.0),
];

pub const LIGHTHOUSES: &[(f32, f32)] = &[
    (2560.0, 2500.0),
    (3130.0, 4320.0),
    (3980.0, 3810.0),
    (3630.0, 2670.0),
    (2200.0, 3650.0),
];

pub const FIELD_SPECS: &[(f32, f32, f32, f32, usize, usize)] = &[
    (2650.0, 2125.0, 2930.0, 2380.0, 4, 5),
    (2320.0, 3050.0, 2505.0, 3550.0, 8, 3),
    (3230.0, 3900.0, 3890.0, 4190.0, 4, 11),
    (3745.0, 2360.0, 3870.0, 2570.0, 4, 3),
];

pub fn carrot_position(index: i32) -> Option<(f32, f32)> {
    let mut index = usize::try_from(index).ok()?;
    for &(top_x, top_z, bot_x, bot_z, rows, cols) in FIELD_SPECS {
        let count = rows * cols;
        if index < count {
            let row = index / cols;
            let col = index % cols;
            let col_space = (bot_x - top_x) / (cols.saturating_sub(1).max(1) as f32);
            let row_space = (bot_z - top_z) / (rows.saturating_sub(1).max(1) as f32);
            return Some((
                top_x + col as f32 * col_space,
                top_z + row as f32 * row_space,
            ));
        }
        index -= count;
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_fields_hold_100_carrots() {
        let total: usize = FIELD_SPECS.iter().map(|field| field.4 * field.5).sum();
        assert_eq!(total as i32, CARROT_COUNT);
        assert!(carrot_position(99).is_some());
        assert_eq!(carrot_position(100), None);
        assert_eq!(carrot_position(-1), None);
    }

    #[test]
    fn carrots_fill_each_field_row_by_row_corner_to_corner() {
        let (top_x, top_z, bottom_x, bottom_z, rows, cols) = FIELD_SPECS[0];
        assert_eq!(carrot_position(0), Some((top_x, top_z)));
        assert_eq!(carrot_position(cols as i32 - 1), Some((bottom_x, top_z)));
        assert_eq!(
            carrot_position(cols as i32),
            Some((top_x, top_z + (bottom_z - top_z) / (rows - 1) as f32))
        );
        assert_eq!(
            carrot_position((rows * cols) as i32 - 1),
            Some((bottom_x, bottom_z))
        );
        let second = FIELD_SPECS[1];
        assert_eq!(
            carrot_position((rows * cols) as i32),
            Some((second.0, second.1))
        );
    }

    #[test]
    fn every_carrot_lies_inside_its_field() {
        let mut index = 0;
        for &(top_x, top_z, bottom_x, bottom_z, rows, cols) in FIELD_SPECS {
            for _ in 0..rows * cols {
                let (x, z) = carrot_position(index).unwrap();
                assert!((top_x..=bottom_x).contains(&x) && (top_z..=bottom_z).contains(&z));
                index += 1;
            }
        }
    }

    #[test]
    fn clamping_keeps_positions_inside_the_map() {
        assert_eq!(clamp_to_map(-50.0), EDGE_MARGIN);
        assert_eq!(clamp_to_map(10_000.0), MAP_SIZE - EDGE_MARGIN);
        assert_eq!(clamp_to_map(CENTER), CENTER);
    }
}
