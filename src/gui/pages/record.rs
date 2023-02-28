#[derive(Eq, Hash, PartialEq)]
pub struct Record {}

impl super::Page for Record {
    fn ui(&mut self, ui: &mut egui::Ui) {
        ui.heading("Record");
    }
}

impl Default for Record {
    fn default() -> Self { Self {} }
}