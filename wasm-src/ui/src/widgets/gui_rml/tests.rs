use super::*;

impl Rml {
    pub(super) fn verify_bindings(
        &mut self,
        spring: &spring::Spring<'_, spring::Ui>,
        global: &Global,
    ) {
        let frame = spring.game().current_frame().unwrap_or(0);
        if self.document == 0 || frame < 182 {
            return;
        }
        match self.test_stage {
            0 => {
                let bindings = self.bindings.unwrap();
                let incomplete_row = spring::rml_ui::RmlDataValue {
                    type_: 3,
                    string_value: "incomplete".into(),
                    ..Default::default()
                };
                assert!(
                    spring
                        .rml_ui()
                        .data_model_set_rows(bindings.events, &[incomplete_row])
                        .is_err()
                );
                self.hud
                    .events
                    .show("binding <&>", events::EventKind::Rabbits);
                let values = self.hud.events.update(0.0).unwrap();
                assert!(
                    spring
                        .rml_ui()
                        .data_model_set_rows(bindings.events, &values)
                        .is_ok_and(|ok| ok)
                );
                self.test_stage = 1;
            }
            1 if frame >= 187 => {
                let row = spring
                    .rml_ui()
                    .element_query_selector(self.document, ".event-line.event-rabbits")
                    .unwrap();
                assert!(row.exists, "the bound event row is visible");
                let text = spring
                    .rml_ui()
                    .element_get_inner_rml(row.element_handle)
                    .unwrap();
                assert!(text.contains("binding"), "the event text is bound");
                let empty = self.hud.events.update(2.0).unwrap();
                assert!(
                    spring
                        .rml_ui()
                        .data_model_set_rows(self.bindings.unwrap().events, &empty)
                        .is_ok()
                );
                self.test_stage = 2;
            }
            2 if frame >= 193 => {
                let row = spring
                    .rml_ui()
                    .element_query_selector(self.document, ".event-line")
                    .unwrap();
                assert!(
                    !row.exists
                        || !spring
                            .rml_ui()
                            .element_is_visible(row.element_handle)
                            .unwrap()
                );
                let restart = spring
                    .rml_ui()
                    .element_get_element_by_id(self.document, "endgame-restart")
                    .unwrap();
                assert!(restart.exists, "the restart button exists");
                assert!(
                    spring
                        .rml_ui()
                        .element_click(restart.element_handle)
                        .is_ok()
                );
                assert!(
                    global.round.access().restart_pending,
                    "RML dispatches the restart action"
                );
                self.test_stage = 3;
            }
            3 if frame >= 203 => {
                assert!(!global.round.access().game_over && !global.round.access().restart_pending);
                assert!(
                    self.hud.time <= 1 && self.hud.carrots == hunted_common::map::CARROT_COUNT,
                    "restart HUD: time={}, carrots={}",
                    self.hud.time,
                    self.hud.carrots
                );
                spring::log_notice!(
                    spring,
                    crate::diagnostics::LOG_SECTION,
                    "HUNTED HUD BINDINGS PASSED"
                );
                self.test_stage = 4;
            }
            _ => {}
        }
    }
}
