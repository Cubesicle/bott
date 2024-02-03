use egui::{Response, Ui, Widget};
use windows::Win32::System::Console::{AllocConsole, FreeConsole};

pub struct Debug { }

impl Debug {
    pub fn new() -> Self { Self {} }
}

impl Widget for Debug {
    fn ui(self, ui: &mut Ui) -> Response {
        ui.heading("Debug");
        let button = ui.button("Toggle console");
        if button.clicked() {
            unsafe { if AllocConsole().is_err() { let _ = FreeConsole(); } }
        }
        button
    }
}