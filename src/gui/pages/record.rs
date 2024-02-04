use egui::{Response, Ui, Widget};

pub struct Record {}

impl Record {
    pub fn new() -> Self {
        Self {}
    }
}

impl Widget for Record {
    fn ui(self, ui: &mut Ui) -> Response {
        ui.heading("Record")
    }
}
