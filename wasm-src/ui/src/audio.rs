use springcabal::music::MusicLoop;

const MUSIC_FILE: &str = "LuaUI/sounds/Zeus_vs_Bunnies.ogg";
const MUSIC_VOLUME: f32 = 0.15;
const EFFECT_VOLUME: f32 = 20.0;

pub fn music() -> MusicLoop {
    MusicLoop::new(MUSIC_FILE, MUSIC_VOLUME)
}

pub struct Streak {
    pub kills: i32,
    pub text: &'static str,
    sound: &'static str,
}

pub const STREAKS: [Streak; 4] = [
    Streak {
        kills: 35,
        text: "GODLIKE!",
        sound: "sounds/godlike.ogg",
    },
    Streak {
        kills: 20,
        text: "MonsterKill!!!",
        sound: "sounds/monsterkill.ogg",
    },
    Streak {
        kills: 12,
        text: "UltraKill!",
        sound: "sounds/ultrakill.ogg",
    },
    Streak {
        kills: 5,
        text: "Killing Streak!",
        sound: "sounds/killstreak.ogg",
    },
];

pub fn play_streak_sound(spring: &spring::Spring<'_, spring::Ui>, streak: &Streak) {
    let result = spring.sound().play_sound_file(
        streak.sound,
        EFFECT_VOLUME,
        spring::Float3::default(),
        spring::Float3::default(),
        0,
    );
    if !matches!(result, Ok(true)) {
        spring::log_once!(
            spring,
            spring::log_level::WARNING,
            crate::diagnostics::LOG_SECTION,
            "kill-streak sound {} could not be played: {result:?}",
            streak.sound
        );
    }
}
