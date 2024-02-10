use egui::{Response, Ui, Widget};

use crate::bot;

pub struct Replay {}

impl Replay {
    pub fn new() -> Self {
        Self {}
    }
}

impl Widget for Replay {
    fn ui(self, ui: &mut Ui) -> Response {
        let heading = ui.heading("Replay");
        if ui
            .checkbox(
                &mut (bot::get_state() == bot::State::Replaying),
                "Enabled",
            )
            .clicked()
        {
            if bot::get_state() != bot::State::Replaying {
                bot::set_state(bot::State::Replaying)
            } else {
                bot::set_state(bot::State::Standby);
            }
        }
        heading
    }
}
