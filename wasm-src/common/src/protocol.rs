pub const STATUS: &str = "HUNTED_STATUS";

pub const MOVE_GUN: &str = "movegun";
pub const FIRE: &str = "shotgun";
pub const PLACE_MINE: &str = "mine";
pub const RESTART: &str = "restart";

pub const TEST_STATUS_SEEN: &str = "HUNTED_TEST|status_seen";

pub mod status {
    pub const FRAME: &str = "frame";
    pub const RABBITS: &str = "rabbits";
    pub const KILLS: &str = "kills";
    pub const CARROTS: &str = "carrots";
    pub const SCORE: &str = "score";
    pub const TIME: &str = "time";
    pub const AMMO: &str = "ammo";
    pub const MINES: &str = "mines";
    pub const FLASH: &str = "flash";
    pub const FLARE_ACTIVE: &str = "flare_active";
    pub const GAME_OVER: &str = "game_over";
}

#[cfg(test)]
mod tests {
    use super::status;

    #[test]
    fn status_keys_are_distinct() {
        use status::*;
        let keys = [
            FRAME,
            RABBITS,
            KILLS,
            CARROTS,
            SCORE,
            TIME,
            AMMO,
            MINES,
            FLASH,
            FLARE_ACTIVE,
            GAME_OVER,
        ];
        for (index, key) in keys.iter().enumerate() {
            assert!(!keys[..index].contains(key), "{key} repeats");
        }
    }
}
