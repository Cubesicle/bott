use egui::{Response, Ui, Widget};

use crate::bot;

pub struct Record {}

impl Record {
    pub fn new() -> Self {
        Self {}
    }
}

impl Widget for Record {
    fn ui(self, ui: &mut Ui) -> Response {
        let heading = ui.heading("Record");
        if ui
            .checkbox(
                &mut (bot::get_state() == bot::State::Recording),
                "Enabled",
            )
            .clicked()
        {
            if bot::get_state() != bot::State::Recording {
                bot::set_state(bot::State::Recording)
            } else {
                bot::set_state(bot::State::Standby);
            }
        }
        if ui.button("Remove excess button events").clicked() {
            bot::optimize_button_events();
        }
        ui.label(format!(
            "Button events recorded: {}",
            bot::get_button_event_count()
        ));
        heading
    }
}
