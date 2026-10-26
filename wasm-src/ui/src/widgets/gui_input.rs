use core::cell::Cell;

use hunted_common::protocol;
use spring_addons::event::{EventResult, KeyEvent};
use spring_addons::ui::Widget;
use springcabal::menu;
use springcabal::message::Encoder;

use crate::diagnostics;
use crate::state::{self, Global, Point3};

pub struct GuiInputWidget {
    mouse: Cell<Option<(i32, i32)>>,
    benchmark_no_input: Cell<bool>,
    last_target: Cell<Option<Point3>>,
    last_sent_frame: Cell<i32>,
}

impl Default for GuiInputWidget {
    fn default() -> Self {
        Self {
            mouse: Cell::new(None),
            benchmark_no_input: Cell::new(false),
            last_target: Cell::new(None),
            last_sent_frame: Cell::new(-1),
        }
    }
}

impl GuiInputWidget {
    pub fn new() -> Self {
        Self::default()
    }

    fn update_target(&self, spring: &spring::Spring<'_, spring::Ui>, global: &Global) {
        if self.benchmark_no_input.get() {
            return;
        }
        let Some((x, y)) = self.mouse_position(spring, global) else {
            return;
        };
        self.mouse.set(Some((x, y)));
        let Some(target) = ground_at(spring, x, y) else {
            return;
        };
        let current_frame = spring.game().current_frame().unwrap_or(0);
        let target_changed = self
            .last_target
            .get()
            .is_none_or(|old| (target.x - old.x).abs() > 1.0 || (target.z - old.z).abs() > 1.0);
        if !target_changed || current_frame == self.last_sent_frame.get() {
            return;
        }
        self.last_sent_frame.set(current_frame);
        global.lights.access_mut().flashlight = Some(target);
        diagnostics::mouse_target(spring, x, y, target);
        if send_target(spring, protocol::MOVE_GUN, target) {
            self.last_target.set(Some(target));
        }
    }

    fn handle_mouse_press(
        &self,
        spring: &spring::Spring<'_, spring::Ui>,
        global: &Global,
        x: i32,
        y: i32,
        button: i32,
    ) -> bool {
        if self.benchmark_no_input.get() {
            return false;
        }
        diagnostics::mouse_press(spring, x, y, button);
        if global.round.access().game_over {
            return true;
        }
        self.mouse.set(Some((x, y)));
        let command = match button {
            spring::mouse::LEFT => protocol::FIRE,
            spring::mouse::RIGHT => protocol::PLACE_MINE,
            _ => return false,
        };
        let Some(target) = ground_at(spring, x, y).or(self.last_target.get()) else {
            return true;
        };
        global.lights.access_mut().flashlight = Some(target);
        send_target(spring, command, target);
        true
    }

    fn mouse_position(
        &self,
        spring: &spring::Spring<'_, spring::Ui>,
        global: &Global,
    ) -> Option<(i32, i32)> {
        let (x, y) = spring
            .input()
            .get_mouse_state()
            .ok()
            .filter(|mouse| !mouse.offscreen && mouse.x.is_finite() && mouse.y.is_finite())
            .map(|mouse| (mouse.x.round() as i32, mouse.y.round() as i32))
            .or(self.mouse.get())?;
        let view = *global.view.access();
        (x >= 0 && x < view.width && y >= 0 && y < view.height).then_some((x, y))
    }
}

fn send_target(spring: &spring::Spring<'_, spring::Ui>, command: &str, target: Point3) -> bool {
    springcabal::message::send_to_rules(
        spring,
        &Encoder::new(command)
            .field(target.x)
            .field(target.y)
            .field(target.z)
            .finish(),
    )
}

fn ground_at(
    spring: &spring::Spring<'_, spring::Ui>,
    screen_x: i32,
    screen_y: i32,
) -> Option<Point3> {
    let ground = spring
        .camera()
        .ground_at(screen_x as f32, screen_y as f32)
        .ok()
        .flatten()?;
    Some(Point3 {
        x: ground.x,
        y: ground.y,
        z: ground.z,
    })
}

impl Widget<Global> for GuiInputWidget {
    fn name(&self) -> &'static str {
        "GuiInput"
    }

    fn init(&self, ctx: &spring_addons::AddonContext<'_, Global, spring::Ui>) {
        self.benchmark_no_input.set(
            ctx.spring()
                .game()
                .get_mod_option("benchmark_no_input")
                .is_ok_and(|value| value.value == "1"),
        );
        if self.benchmark_no_input.get() {
            spring::log_notice!(ctx.spring(), "HuntedBenchmark", "mouse_input=off");
        }
    }

    fn update(&self, ctx: &spring_addons::AddonContext<'_, Global, spring::Ui>, _dt: f32) {
        let spring = ctx.spring();
        self.update_target(spring, ctx.global());
    }

    fn mouse_move(
        &self,
        _ctx: &spring_addons::AddonContext<'_, Global, spring::Ui>,
        x: i32,
        y: i32,
        _dx: i32,
        _dy: i32,
        _button: i32,
    ) -> EventResult {
        self.mouse.set(Some((x, y)));
        EventResult::Ignored
    }

    fn mouse_press(
        &self,
        ctx: &spring_addons::AddonContext<'_, Global, spring::Ui>,
        x: i32,
        y: i32,
        button: i32,
    ) -> EventResult {
        let spring = ctx.spring();
        self.handle_mouse_press(spring, ctx.global(), x, y, button)
            .into()
    }

    fn key_press(
        &self,
        ctx: &spring_addons::AddonContext<'_, Global, spring::Ui>,
        event: &KeyEvent<'_>,
    ) -> EventResult {
        let spring = ctx.spring();
        if menu::is_quit_to_menu(event) {
            state::request_quit(spring);
            return EventResult::Handled;
        }
        if event.is_repeat {
            return EventResult::Ignored;
        }
        if event.matches("q") && ctx.global().round.access().game_over {
            state::request_quit(spring);
            return EventResult::Handled;
        }
        if !event.matches("r") && !event.matches("enter") {
            return EventResult::Ignored;
        }
        state::request_restart(spring, ctx.global()).into()
    }
}
