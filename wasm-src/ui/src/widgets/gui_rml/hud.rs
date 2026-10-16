use alloc::format;

use hunted_common::protocol::status as key;
use hunted_common::weapons::{START_MINES, START_SHELLS};
use springcabal::message;

use super::events::{self, EventKind, Events};
use crate::audio;
use crate::state::Round;

const STREAK_WINDOW_FRAMES: i32 = 120;
const SAMPLE_PERIOD_FRAMES: i32 = 15;
const CARROT_SAMPLE_PHASE: i32 = 7;

#[derive(Default)]
pub struct Status {
    pub frame: Option<i32>,
    pub rabbits: Option<i32>,
    pub kills: Option<i32>,
    pub carrots: Option<i32>,
    pub score: Option<i32>,
    pub time: Option<i32>,
    pub ammo: Option<i32>,
    pub mines: Option<i32>,
    pub flash: Option<f32>,
    pub flare_active: Option<bool>,
    pub game_over: Option<bool>,
}

impl Status {
    pub fn parse(message: &str) -> Option<Self> {
        let mut status = Self::default();
        for (key, value) in message::key_values(message, hunted_common::protocol::STATUS)? {
            let int = || value.parse::<i32>().ok();
            match key {
                key::FRAME => status.frame = int(),
                key::RABBITS => status.rabbits = int(),
                key::KILLS => status.kills = int(),
                key::CARROTS => status.carrots = int(),
                key::SCORE => status.score = int(),
                key::TIME => status.time = int(),
                key::AMMO => status.ammo = int(),
                key::MINES => status.mines = int(),
                key::FLASH => status.flash = value.parse().ok(),
                key::FLARE_ACTIVE => status.flare_active = int().map(|active| active != 0),
                key::GAME_OVER => status.game_over = int().map(|over| over != 0),
                _ => {}
            }
        }
        Some(status)
    }
}

#[derive(Clone, Copy)]
pub struct Bindings {
    pub ammo: u64,
    pub mines: u64,
    pub carrots: u64,
    pub rabbits: u64,
    pub kills: u64,
    pub score: u64,
    pub time: u64,
    pub flash: u64,
    pub game_over: u64,
    pub flare_active: u64,
    pub events: u64,
}

pub struct Hud {
    pub ammo: i32,
    pub mines: i32,
    pub carrots: i32,
    pub rabbits: i32,
    pub kills: i32,
    pub score: i32,
    pub time: i32,
    pub flash: f32,
    status_seen: bool,
    sampled_rabbits: (i32, i32),
    sampled_carrots: (i32, i32),
    streak_frame: i32,
    streak_kills: i32,
    pub events: Events,
}

impl Default for Hud {
    fn default() -> Self {
        Self {
            ammo: START_SHELLS as i32,
            mines: START_MINES as i32,
            carrots: hunted_common::map::CARROT_COUNT,
            rabbits: 0,
            kills: 0,
            score: 0,
            time: 0,
            flash: 0.0,
            status_seen: false,
            sampled_rabbits: (0, -1),
            sampled_carrots: (0, -1),
            streak_frame: -1,
            streak_kills: 0,
            events: Events::default(),
        }
    }
}

impl Hud {
    pub fn apply(&mut self, status: &Status, round: &mut Round) {
        if status.game_over == Some(false) && (round.game_over || round.restart_pending) {
            self.time = 0;
        }
        let first_status = !self.status_seen;
        self.status_seen = true;
        if let Some(frame) = status.frame {
            round.frame = frame;
        }
        if let Some(rabbits) = status.rabbits {
            self.rabbits = rabbits;
        }
        if let Some(kills) = status.kills {
            let delta = kills - self.kills;
            self.kills = kills;
            if delta < 0 {
                self.streak_frame = -1;
                self.streak_kills = 0;
            }
            if !first_status && delta > 0 {
                self.count_streak(delta, round.frame);
            }
        }
        if let Some(carrots) = status.carrots {
            self.carrots = carrots;
        }
        if first_status {
            self.sampled_rabbits.0 = self.rabbits;
            self.sampled_carrots.0 = self.carrots;
        }
        if let Some(score) = status.score {
            self.score = score;
        }
        if let Some(time) = status.time
            && !round.game_over
            && !round.restart_pending
        {
            self.time = time;
        }
        if let Some(ammo) = status.ammo {
            self.ammo = ammo;
        }
        if let Some(mines) = status.mines {
            self.mines = mines;
        }
        if let Some(flash) = status.flash {
            self.flash = flash;
        }
        if let Some(active) = status.flare_active {
            round.flare_active = active;
        }
        match status.game_over {
            Some(false) => {
                round.restart_pending = false;
                round.game_over = false;
            }
            Some(true) if !round.restart_pending => round.game_over = true,
            _ => {}
        }
    }

    pub fn sample(&mut self, frame: i32) {
        if !self.status_seen {
            return;
        }
        let rabbit_period = frame.div_euclid(SAMPLE_PERIOD_FRAMES);
        if rabbit_period != self.sampled_rabbits.1 {
            let delta = self.rabbits - self.sampled_rabbits.0;
            if delta != 0 {
                self.events
                    .show(&format!("Rabbits: {delta:+}"), EventKind::Rabbits);
            }
            self.sampled_rabbits = (self.rabbits, rabbit_period);
        }
        let carrot_period = (frame - CARROT_SAMPLE_PHASE).div_euclid(SAMPLE_PERIOD_FRAMES);
        if carrot_period != self.sampled_carrots.1 {
            let delta = self.carrots - self.sampled_carrots.0;
            if delta != 0 {
                self.events
                    .show(&format!("Carrots: {delta:+}"), EventKind::Carrots);
            }
            self.sampled_carrots = (self.carrots, carrot_period);
        }
    }

    fn count_streak(&mut self, kills: i32, frame: i32) {
        if self.streak_frame < 0 || frame - self.streak_frame > STREAK_WINDOW_FRAMES {
            self.streak_kills = 0;
        }
        let previous_streak = self.streak_kills;
        self.streak_frame = frame;
        self.streak_kills += kills;
        if let Some(streak) = audio::STREAKS
            .iter()
            .find(|streak| self.streak_kills >= streak.kills && previous_streak < streak.kills)
        {
            self.events.show(streak.text, EventKind::KillStreak);
            audio::play_streak_sound(streak);
        }
    }

    pub fn push(&self, bindings: Option<Bindings>) {
        let Some(bindings) = bindings else {
            return;
        };
        set_int(bindings.ammo, self.ammo);
        set_int(bindings.mines, self.mines);
        set_int(bindings.carrots, self.carrots);
        set_int(bindings.rabbits, self.rabbits);
        set_int(bindings.kills, self.kills);
        set_int(bindings.score, self.score);
        set_int(bindings.time, self.time);
        set_percent(bindings.flash, self.flash);
    }
}

pub fn create_bindings(context: u64) -> Option<Bindings> {
    let model = spring::context_create_data_model(context, "hunted").ok()?;
    if !model.success || model.data_model_handle == 0 {
        return None;
    }
    let model = model.data_model_handle;
    let bind_int = |name: &str, initial: i32| {
        let binding = spring::data_model_bind_int(model, name, initial).ok()?;
        (binding.success && binding.variable_handle != 0).then_some(binding.variable_handle)
    };
    let bind_percent = |name: &str, initial: f32| {
        let binding = spring::data_model_bind_percent(model, name, initial).ok()?;
        (binding.success && binding.variable_handle != 0).then_some(binding.variable_handle)
    };
    bind_event(model, "restart", |global| {
        crate::state::request_restart(global);
    })?;
    bind_event(model, "quit", |_| crate::state::request_quit())?;
    let bind_bool = |name: &str| {
        let binding = spring::data_model_bind_bool(model, name, false).ok()?;
        (binding.success && binding.variable_handle != 0).then_some(binding.variable_handle)
    };
    Some(Bindings {
        ammo: bind_int("ammo", START_SHELLS as i32)?,
        mines: bind_int("mines", START_MINES as i32)?,
        carrots: bind_int("carrots", hunted_common::map::CARROT_COUNT)?,
        rabbits: bind_int("rabbits", 0)?,
        kills: bind_int("kills", 0)?,
        score: bind_int("score", 0)?,
        time: bind_int("time", 0)?,
        flash: bind_percent("flash", 0.0)?,
        game_over: bind_bool("game_over")?,
        flare_active: bind_bool("flare_active")?,
        events: events::bind(model)?,
    })
}

fn set_int(handle: u64, value: i32) {
    if handle != 0 {
        let _ = spring::data_model_set_int(handle, value);
    }
}

fn set_percent(handle: u64, value: f32) {
    if handle != 0 {
        let _ = spring::data_model_set_percent(handle, value.clamp(0.0, 100.0));
    }
}

fn bind_event(
    model: u64,
    name: &str,
    handler: impl FnMut(&crate::state::Global) + 'static,
) -> Option<()> {
    let callback = crate::register_callback(handler);
    if spring::data_model_bind_event(model, name, callback, &[])
        .is_ok_and(|event| event.success && event.handle != 0)
    {
        Some(())
    } else {
        crate::unregister_callback(callback);
        None
    }
}
