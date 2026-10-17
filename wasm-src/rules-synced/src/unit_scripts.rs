#[derive(Clone, Copy)]
pub struct Call {
    pub unit: i32,
    pub function: &'static str,
    arguments: [f32; 2],
    argument_count: usize,
}

impl Call {
    fn new(unit: i32, function: &'static str, arguments: &[f32]) -> Self {
        let mut values = [0.0; 2];
        values[..arguments.len()].copy_from_slice(arguments);
        Self {
            unit,
            function,
            arguments: values,
            argument_count: arguments.len(),
        }
    }

    pub fn arguments(&self) -> &[f32] {
        &self.arguments[..self.argument_count]
    }

    pub fn shotgun_fire(gun: i32) -> Self {
        Self::new(gun, "Fire", &[])
    }

    pub fn shotgun_set_pitch(gun: i32, pitch: f32) -> Self {
        Self::new(gun, "SetPitch", &[pitch])
    }

    pub fn rabbit_attach_carrot(rabbit: i32, carrot: i32) -> Self {
        Self::new(rabbit, "AttachCarrot", &[carrot as f32])
    }

    pub fn rabbit_drop_carrot(rabbit: i32, carrot: i32) -> Self {
        Self::new(rabbit, "DropCarrot", &[carrot as f32])
    }

    pub fn carrot_be_dropped(carrot: i32) -> Self {
        Self::new(carrot, "BeDropped", &[])
    }

    pub fn carrot_score(carrot: i32) -> Self {
        Self::new(carrot, "ScoreCarrot", &[])
    }

    pub fn carrot_consume(carrot: i32) -> Self {
        Self::new(carrot, "ConsumeCarrot", &[])
    }

    pub fn carrot_start_being_stolen(carrot: i32, progress: i32) -> Self {
        Self::new(carrot, "StartBeingStolen", &[progress as f32])
    }

    pub fn carrot_stop_being_stolen(carrot: i32, progress: i32) -> Self {
        Self::new(carrot, "StopBeingStolen", &[progress as f32])
    }

    pub fn mine_start_fuse(mine: i32) -> Self {
        Self::new(mine, "StartFuse", &[])
    }

    pub fn run(self, spring: &spring::Spring<'_, spring::RulesSynced>) {
        crate::cus::call(spring, self);
    }

    pub fn queue(self) {
        crate::with_addon_context(|ctx| ctx.commands().run(move |spring| self.run(spring)));
    }
}

pub fn lighthouse_beam(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    unit: i32,
) -> Option<(f32, f32)> {
    use hunted_common::lighthouse::{PARAM_X, PARAM_Z};
    Some((
        spring
            .rules_params()
            .get_unit_rules_param_f32(unit, PARAM_X)
            .ok()?,
        spring
            .rules_params()
            .get_unit_rules_param_f32(unit, PARAM_Z)
            .ok()?,
    ))
}
