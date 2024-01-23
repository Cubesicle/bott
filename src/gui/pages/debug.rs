use windows::Win32::System::Console::{AllocConsole, FreeConsole};

#[derive(Eq, Hash, PartialEq)]
pub struct Debug { }

impl super::Page for Debug {
    fn ui(&mut self, ui: &mut egui::Ui) {
        ui.heading("Debug");
        if ui.button("Toggle console").clicked() {
            unsafe { if AllocConsole().is_err() { let _ = FreeConsole(); } }
        }
    }
}

impl Default for Debug {
    fn default() -> Self { Self {} }
}
