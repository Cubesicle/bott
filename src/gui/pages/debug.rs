use std::sync::atomic::Ordering;

use egui::{Response, Ui, Widget};
use windows::Win32::System::Console::{AllocConsole, FreeConsole};

use crate::bot;
use crate::gd::{self, get_current_frame};

pub struct Debug {}

impl Debug {
    pub fn new() -> Self {
        Self {}
    }
}

impl Widget for Debug {
    fn ui(self, ui: &mut Ui) -> Response {
        let heading = ui.heading("Debug");
        let button = ui.button("Toggle console");
        if button.clicked() {
            unsafe {
                if AllocConsole().is_err() {
                    let _ = FreeConsole();
                }
            }
        }
        ui.label(format!(
            "Current frame: {}",
            if unsafe { gd::get_play_layer_addr() }.is_err() {
                0
            } else {
                unsafe { get_current_frame().unwrap() }
            }
        ));
        if ui
            .button(format!(
                "{}",
                if bot::PAUSED.load(Ordering::Relaxed) {
                    "Play"
                } else {
                    "Pause"
                }
            ))
            .clicked()
        {
            bot::PAUSED
                .store(!bot::PAUSED.load(Ordering::Relaxed), Ordering::Relaxed);
        }
        //logger_ui(ui);
        heading
    }
}
