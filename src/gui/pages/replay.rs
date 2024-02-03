use egui::{Response, Ui, Widget};

pub struct Replay {}

impl Replay {
    pub fn new() -> Self { Self {} }
}

impl Widget for Replay {
    fn ui(self, ui: &mut Ui) -> Response {
        ui.heading("Replay")
    }
}