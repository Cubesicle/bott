#[derive(Eq, Hash, PartialEq)]
pub struct Debug { }

impl super::Page for Debug {
    fn ui(&mut self, ui: &mut egui::Ui) {
        egui::ScrollArea::both().show(ui, |ui| {
            ui.heading("Debug");
            ui.separator();
            egui_logger::logger_ui(ui);
        });
    }
}

impl Default for Debug {
    fn default() -> Self { Self {} }
}