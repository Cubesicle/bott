#[derive(Eq, Hash, PartialEq)]
pub struct Replay {}

impl super::Page for Replay {
    fn ui(&mut self, ui: &mut egui::Ui) {
        ui.heading("Replay");
    }
}

impl Default for Replay {
    fn default() -> Self { Self {} }
}