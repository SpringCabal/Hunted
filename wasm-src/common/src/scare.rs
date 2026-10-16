use crate::map;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Area {
    pub radius: f32,
    pub edge: f32,
    pub proximity: f32,
}

const FLASHLIGHT_REACH: f32 = 3100.0;
const FLASHLIGHT_FADE: f32 = 300.0;

impl Area {
    pub const SHOTGUN: Area = Area {
        radius: 300.0,
        edge: 0.0,
        proximity: 150.0,
    };

    pub const MINE: Area = Area {
        radius: 250.0,
        edge: 0.0,
        proximity: 400.0,
    };

    pub const LIGHTHOUSE: Area = Area {
        radius: 250.0,
        edge: 0.1,
        proximity: 1.6,
    };

    pub const FLASHLIGHT: Area = Area {
        radius: 200.0,
        edge: 0.1,
        proximity: 1.8,
    };

    pub fn flashlight(x: f32, z: f32) -> Area {
        let distance = (x - map::CENTER).hypot(z - map::CENTER);
        let strength = ((FLASHLIGHT_REACH - distance) / FLASHLIGHT_FADE).clamp(0.0, 1.0);
        Area {
            edge: Self::FLASHLIGHT.edge * strength,
            proximity: Self::FLASHLIGHT.proximity * strength,
            ..Self::FLASHLIGHT
        }
    }

    pub fn magnitude(&self, distance_sq: f32) -> Option<f32> {
        (distance_sq < self.radius * self.radius)
            .then(|| self.proximity * (1.0 - distance_sq.sqrt() / self.radius) + self.edge)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Scary {
    pub x: f32,
    pub z: f32,
    pub magnitude: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Source {
    pub x: f32,
    pub z: f32,
    pub area: Area,
}

impl Source {
    pub fn felt_at(&self, x: f32, z: f32) -> Option<Scary> {
        let distance_sq = (x - self.x).powi(2) + (z - self.z).powi(2);
        self.area.magnitude(distance_sq).map(|magnitude| Scary {
            x: self.x,
            z: self.z,
            magnitude,
        })
    }
}

pub fn scariest(sources: &[Source], x: f32, z: f32, factor: f32) -> Option<Scary> {
    let mut best: Option<Scary> = None;
    for source in sources {
        let Some(mut scary) = source.felt_at(x, z) else {
            continue;
        };
        scary.magnitude *= factor;
        if scary.magnitude > best.map_or(0.0, |best| best.magnitude) {
            best = Some(scary);
        }
    }
    best
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(left: f32, right: f32) -> bool {
        (left - right).abs() < 1e-4
    }

    #[test]
    fn magnitude_falls_off_to_the_edge_value() {
        let area = Area::LIGHTHOUSE;
        assert!(close(area.magnitude(0.0).unwrap(), 1.7));
        assert!(close(area.magnitude(125.0 * 125.0).unwrap(), 0.9));
        assert_eq!(area.magnitude(250.0 * 250.0), None);
    }

    #[test]
    fn mines_scare_harder_but_closer_than_the_shotgun() {
        assert!(close(Area::MINE.magnitude(0.0).unwrap(), 400.0));
        assert!(close(Area::SHOTGUN.magnitude(0.0).unwrap(), 150.0));
        assert!(Area::SHOTGUN.magnitude(260.0 * 260.0).is_some());
        assert_eq!(Area::MINE.magnitude(260.0 * 260.0), None);
    }

    #[test]
    fn the_flashlight_is_full_strength_inside_the_map_and_off_at_the_edge() {
        let centre = Area::flashlight(map::CENTER, map::CENTER);
        assert_eq!(centre, Area::FLASHLIGHT);
        assert!(close(centre.magnitude(0.0).unwrap(), 1.9));

        let fading = Area::flashlight(map::CENTER + 2950.0, map::CENTER);
        assert!(close(fading.proximity, 0.9));
        assert!(close(fading.edge, 0.05));

        let edge = Area::flashlight(map::CENTER + 3100.0, map::CENTER);
        assert!(close(edge.magnitude(0.0).unwrap(), 0.0));
    }

    #[test]
    fn scariest_picks_the_strongest_positive_source() {
        let near = Source {
            x: 100.0,
            z: 0.0,
            area: Area::LIGHTHOUSE,
        };
        let far = Source {
            x: 200.0,
            z: 0.0,
            area: Area::LIGHTHOUSE,
        };
        let best = scariest(&[far, near], 0.0, 0.0, 1.0).unwrap();
        assert_eq!((best.x, best.z), (100.0, 0.0));
        let halved = scariest(&[near], 0.0, 0.0, 0.5).unwrap();
        assert!(close(
            halved.magnitude,
            0.5 * near.felt_at(0.0, 0.0).unwrap().magnitude
        ));
        assert_eq!(scariest(&[near], 0.0, 0.0, 0.0), None);
        assert_eq!(scariest(&[near], 1000.0, 0.0, 1.0), None);
    }
}
