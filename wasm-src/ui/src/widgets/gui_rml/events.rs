use alloc::string::String;
use alloc::vec::Vec;

const LIFETIME_SECONDS: f32 = 1.5;
const MAX_EVENTS: usize = 8;
const FIELD_FLOAT: u8 = 2;
const FIELD_STRING: u8 = 3;
const FIELD_PERCENT: u8 = 6;
const EVENT_FIELDS: [(&str, u8); 5] = [
    ("text", FIELD_STRING),
    ("kind", FIELD_STRING),
    ("top", FIELD_PERCENT),
    ("opacity", FIELD_FLOAT),
    ("size", FIELD_FLOAT),
];

#[derive(Clone, Copy)]
pub enum EventKind {
    Rabbits,
    Carrots,
    KillStreak,
}

impl EventKind {
    fn name(self) -> &'static str {
        match self {
            Self::Rabbits => "event-rabbits",
            Self::Carrots => "event-carrots",
            Self::KillStreak => "event-kill-streak",
        }
    }
}

struct Event {
    text: String,
    kind: EventKind,
    age_seconds: f32,
}

pub struct Events {
    events: Vec<Event>,
    shown: bool,
}

impl Default for Events {
    fn default() -> Self {
        Self {
            events: Vec::new(),
            shown: true,
        }
    }
}

impl Events {
    pub fn show(&mut self, text: &str, kind: EventKind) {
        self.events.push(Event {
            text: text.into(),
            kind,
            age_seconds: 0.0,
        });
        if self.events.len() > MAX_EVENTS {
            self.events.remove(0);
        }
    }

    pub fn invalidate(&mut self) {
        self.shown = true;
    }

    pub fn update(&mut self, delta_seconds: f32) -> Option<Vec<spring::rml_ui::RmlDataValue>> {
        for event in &mut self.events {
            event.age_seconds += delta_seconds;
        }
        self.events
            .retain(|event| event.age_seconds < LIFETIME_SECONDS);
        if self.events.is_empty() && !self.shown {
            return None;
        }
        self.shown = !self.events.is_empty();
        let mut values = Vec::with_capacity(self.events.len() * EVENT_FIELDS.len());
        for event in self.events.iter().rev() {
            let age = (event.age_seconds / LIFETIME_SECONDS).clamp(0.0, 1.0);
            let top = 72.0 - age * 62.0;
            let opacity = if age < 0.15 {
                age / 0.15
            } else {
                1.0 - ((age - 0.15) / 0.85).clamp(0.0, 1.0)
            };
            let size = 28.0 + age * 12.0;
            values.extend([
                string_value(&event.text),
                string_value(event.kind.name()),
                spring::rml_ui::RmlDataValue {
                    type_: FIELD_PERCENT,
                    float_value: top,
                    ..Default::default()
                },
                spring::rml_ui::RmlDataValue {
                    type_: FIELD_FLOAT,
                    float_value: opacity,
                    ..Default::default()
                },
                spring::rml_ui::RmlDataValue {
                    type_: FIELD_FLOAT,
                    float_value: size,
                    ..Default::default()
                },
            ]);
        }
        Some(values)
    }
}

pub fn bind(spring: &spring::Spring<'_, spring::Ui>, model: u64) -> Option<u64> {
    let fields = EVENT_FIELDS.map(|(name, type_)| spring::rml_ui::RmlDataFieldDef {
        name: name.into(),
        type_,
    });
    let rows = spring
        .rml_ui()
        .data_model_bind_rows(model, "events", &fields)
        .ok()?;
    (rows.success && rows.rows_handle != 0).then_some(rows.rows_handle)
}

fn string_value(text: &str) -> spring::rml_ui::RmlDataValue {
    spring::rml_ui::RmlDataValue {
        type_: FIELD_STRING,
        string_value: text.into(),
        ..Default::default()
    }
}
