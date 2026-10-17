mod events;
mod hud;
#[cfg(feature = "e2e")]
mod tests;

use core::cell::RefCell;

use spring::ResultExt;
use spring_addons::ui::Widget;

use crate::diagnostics;
use crate::state::Global;

use hud::{Bindings, Hud, Status};

const CONTEXT_NAME: &str = "hunted-wasm";
const DOCUMENT: &str = "LuaUI/wasm/hunted.rml";
const CHECK_PERIOD: i32 = 30;
struct Rml {
    context: u64,
    document: u64,
    bindings: Option<Bindings>,
    check_ticks: i32,
    hud: Hud,
    force_started: bool,
    #[cfg(feature = "e2e")]
    test_stage: u8,
    #[cfg(feature = "headless-integration")]
    status_confirmed: bool,
}

impl Default for Rml {
    fn default() -> Self {
        Self {
            context: 0,
            document: 0,
            bindings: None,
            check_ticks: CHECK_PERIOD,
            hud: Hud::default(),
            force_started: false,
            #[cfg(feature = "e2e")]
            test_stage: 0,
            #[cfg(feature = "headless-integration")]
            status_confirmed: false,
        }
    }
}

impl Rml {
    fn update(&mut self, spring: &spring::Spring<'_, spring::Ui>, global: &Global, dt: f32) {
        let round = *global.round.access();
        let cursor = if round.game_over {
            "cursornormal"
        } else {
            "none"
        };
        let _ = spring.unsynced_ctrl().set_mouse_cursor(cursor, 1.0);
        self.initialize(spring);
        if let Ok(frame) = spring.game().current_frame() {
            self.hud.sample(frame);
        }
        if let Some(values) = self.hud.events.update(dt.max(0.0))
            && let Some(bindings) = self.bindings
        {
            spring
                .rml_ui()
                .data_model_set_rows(bindings.events, &values)
                .log_err_once(spring, "updating Hunted floating messages");
        }
        if round.restart_pending && self.hud.time != 0 {
            self.hud.time = 0;
            self.hud.push(spring, self.bindings);
        }
        self.show_round(spring, global);
        #[cfg(feature = "e2e")]
        self.verify_bindings(spring, global);
        if self.context != 0 {
            let _ = spring.rml_ui().context_update(self.context);
        }
    }

    fn draw(
        &mut self,
        spring: &spring::Spring<'_, spring::Ui>,
        global: &Global,
        width: i32,
        height: i32,
    ) {
        self.initialize(spring);
        *global.view.access_mut() = crate::state::View { width, height };
        if self.context == 0 {
            return;
        }
        let _ = spring
            .rml_ui()
            .context_set_dimensions(self.context, width, height);
        let _ = spring.rml_ui().context_render(self.context);
    }

    fn on_status(
        &mut self,
        spring: &spring::Spring<'_, spring::Ui>,
        global: &Global,
        message: &str,
    ) {
        let Some(status) = Status::parse(message) else {
            return;
        };
        self.hud
            .apply(spring, &status, &mut global.round.access_mut());
        self.hud.push(spring, self.bindings);
        self.show_round(spring, global);
        #[cfg(feature = "headless-integration")]
        if !self.status_confirmed
            && let Some(frame) = status.frame
        {
            self.status_confirmed = true;
            springcabal::message::send_to_rules(
                spring,
                &springcabal::message::Encoder::new(hunted_common::protocol::TEST_STATUS_SEEN)
                    .field(frame)
                    .finish(),
            );
        }
        diagnostics::status_received(
            spring,
            status.frame.unwrap_or(-1),
            status.carrots.unwrap_or(0),
            status.ammo.unwrap_or(0),
            status.game_over.unwrap_or(false),
        );
    }

    fn show_round(&mut self, spring: &spring::Spring<'_, spring::Ui>, global: &Global) {
        if let Some(bindings) = self.bindings {
            let round = *global.round.access();
            let _ = spring
                .rml_ui()
                .data_model_set_bool(bindings.flare_active, round.flare_active);
            let _ = spring
                .rml_ui()
                .data_model_set_bool(bindings.game_over, round.game_over);
        }
    }

    fn initialize(&mut self, spring: &spring::Spring<'_, spring::Ui>) {
        if self.context != 0 {
            self.check_ticks += 1;
            if self.check_ticks < CHECK_PERIOD {
                return;
            }
            self.check_ticks = 0;
            match context_is_live(spring, self.context, self.document) {
                Some(true) => return,
                Some(false) => {
                    let _ = spring.rml_ui().remove_context(self.context);
                    self.reset_handles(spring);
                    return;
                }
                None => return,
            }
        }

        match spring.rml_ui().is_ready() {
            Ok(true) => {}
            Ok(false) => return,
            Err(error) => {
                spring::log_once!(
                    spring,
                    spring::log_level::WARNING,
                    crate::diagnostics::LOG_SECTION,
                    "RmlUi readiness query failed (error code {})",
                    error.code
                );
                return;
            }
        }
        if !self.force_started {
            self.force_started = true;
            let _ = spring.messages().send_commands("forcestart", "");
        }
        let Some((context, bindings, document)) = create_document(spring) else {
            return;
        };
        self.context = context;
        self.document = document;
        self.bindings = Some(bindings);
        self.check_ticks = 0;
        self.hud.push(spring, self.bindings);
        diagnostics::info(spring, "Hunted RmlUi HUD initialized");
    }

    fn reset_handles(&mut self, spring: &spring::Spring<'_, spring::Ui>) {
        self.context = 0;
        self.document = 0;
        self.bindings = None;
        self.hud.events.invalidate();
        self.check_ticks = CHECK_PERIOD;
        diagnostics::debug(
            spring,
            "RmlUi Hunted handles were stale; rebuilding the HUD",
        );
    }
}

fn create_document(spring: &spring::Spring<'_, spring::Ui>) -> Option<(u64, Bindings, u64)> {
    let context = match spring.rml_ui().create_context(CONTEXT_NAME) {
        Ok(context) => context,
        Err(error) => {
            spring::log_once!(
                spring,
                spring::log_level::WARNING,
                crate::diagnostics::LOG_SECTION,
                "RmlUi context creation failed (error code {})",
                error.code
            );
            return None;
        }
    };
    if !context.success || context.context_handle == 0 {
        spring::log_once!(
            spring,
            spring::log_level::WARNING,
            crate::diagnostics::LOG_SECTION,
            "RmlUi context creation returned no context"
        );
        return None;
    }
    let context = context.context_handle;
    let document = load_document(spring, context);
    if document.is_none() {
        let _ = spring.rml_ui().remove_context(context);
    }
    document.map(|(bindings, document)| (context, bindings, document))
}

fn load_document(spring: &spring::Spring<'_, spring::Ui>, context: u64) -> Option<(Bindings, u64)> {
    let _ = spring.rml_ui().context_enable_mouse_cursor(context, false);
    let _ = spring.unsynced_ctrl().set_mouse_cursor("none", 1.0);
    let Some(bindings) = hud::create_bindings(spring, context) else {
        spring::log_once!(
            spring,
            spring::log_level::WARNING,
            crate::diagnostics::LOG_SECTION,
            "RmlUi Hunted data-model creation failed; retrying HUD initialization"
        );
        return None;
    };
    let document = match spring.rml_ui().context_load_document(context, DOCUMENT) {
        Ok(document) => document,
        Err(error) => {
            spring::log_once!(
                spring,
                spring::log_level::WARNING,
                crate::diagnostics::LOG_SECTION,
                "RmlUi Hunted document load failed (error code {})",
                error.code
            );
            return None;
        }
    };
    if !document.success || document.document_handle == 0 {
        spring::log_once!(
            spring,
            spring::log_level::WARNING,
            crate::diagnostics::LOG_SECTION,
            "RmlUi Hunted document load returned no document"
        );
        return None;
    }
    let document = document.document_handle;
    for (id, markup) in [
        (
            "endgame",
            include_str!("../../../../LuaUI/wasm/endgame.rml"),
        ),
        ("events", include_str!("../../../../LuaUI/wasm/events.rml")),
    ] {
        let element = spring
            .rml_ui()
            .element_get_element_by_id(document, id)
            .ok()?;
        if !element.exists {
            return None;
        }
        if spring
            .rml_ui()
            .element_set_inner_rml(element.element_handle, markup)
            != Ok(true)
        {
            return None;
        }
    }
    match spring.rml_ui().document_show(document, Default::default()) {
        Ok(true) => Some((bindings, document)),
        Ok(false) => {
            spring::log_once!(
                spring,
                spring::log_level::WARNING,
                crate::diagnostics::LOG_SECTION,
                "RmlUi Hunted document show returned false"
            );
            None
        }
        Err(error) => {
            spring::log_once!(
                spring,
                spring::log_level::WARNING,
                crate::diagnostics::LOG_SECTION,
                "RmlUi Hunted document show failed (error code {})",
                error.code
            );
            None
        }
    }
}

fn context_is_live(
    spring: &spring::Spring<'_, spring::Ui>,
    context: u64,
    document: u64,
) -> Option<bool> {
    let found = spring.rml_ui().get_context(CONTEXT_NAME).ok()?;
    if !found.exists || found.context_handle != context || document == 0 {
        return Some(false);
    }
    let document_context = spring.rml_ui().document_get_context(document).ok()?;
    Some(document_context.exists && document_context.context_handle == context)
}

#[derive(Default)]
pub struct GuiRmlWidget {
    rml: RefCell<Rml>,
}

impl GuiRmlWidget {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Widget<Global> for GuiRmlWidget {
    fn name(&self) -> &'static str {
        "GuiRml"
    }

    fn update(&self, ctx: &spring_addons::AddonContext<'_, Global, spring::Ui>, dt: f32) {
        let spring = ctx.spring();
        self.rml.borrow_mut().update(spring, ctx.global(), dt);
    }

    fn draw_screen(
        &self,
        ctx: &spring_addons::AddonContext<'_, Global, spring::Ui>,
        width: i32,
        height: i32,
    ) {
        let spring = ctx.spring();
        self.rml
            .borrow_mut()
            .draw(spring, ctx.global(), width, height);
    }

    fn shutdown(&self, _ctx: &spring_addons::AddonContext<'_, Global, spring::Ui>) {
        let spring = _ctx.spring();
        let mut rml = self.rml.borrow_mut();
        if rml.context != 0 {
            let _ = spring.rml_ui().remove_context(rml.context);
            rml.reset_handles(spring);
        }
    }

    fn recv_from_synced(
        &self,
        ctx: &spring_addons::AddonContext<'_, Global, spring::Ui>,
        message: &[u8],
    ) {
        let spring = ctx.spring();
        if let Ok(text) = core::str::from_utf8(message) {
            self.rml.borrow_mut().on_status(spring, ctx.global(), text);
        }
    }
}
