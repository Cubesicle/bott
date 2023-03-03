#[derive(Eq, Hash, PartialEq)]
pub struct Debug { }

impl super::Page for Debug {
    fn ui(&mut self, ui: &mut egui::Ui) {
        ui.heading("Debug");
        if ui.button("lols").hovered() {
            log::info!("wow");
        }
        ui.separator();
        egui_logger::logger_ui(ui);
    }
}

impl Default for Debug {
    fn default() -> Self { Self {} }
}
