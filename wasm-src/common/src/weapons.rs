pub const START_SHELLS: f32 = 100.0;
pub const START_MINES: f32 = 5.0;
pub const MAX_AMMO: f32 = 100.0;
const _: () = assert!(START_SHELLS <= MAX_AMMO && START_MINES <= MAX_AMMO);
