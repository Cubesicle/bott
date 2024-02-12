use std::sync::atomic::Ordering;

use egui::Ui;
use windows::Win32::System::Console::{AllocConsole, FreeConsole};

use crate::bot;
use crate::gd::{self, get_current_frame};

#[derive(Default)]
pub struct Debug {}

impl super::Page for Debug {
    fn ui(&mut self, ui: &mut Ui) {
        ui.heading("Debug");
        ui.add_space(ui.spacing().item_spacing.y);

        let button = ui.button("Toggle console");
        if button.clicked() {
            unsafe {
                if AllocConsole().is_err() {
                    let _ = FreeConsole();
                }
            }
        }
        ui.add_space(ui.spacing().item_spacing.y);

        ui.label(format!(
            "Current frame: {}",
            if unsafe { gd::get_play_layer_addr() }.is_err() {
                0
            } else {
                unsafe { get_current_frame().unwrap() }
            }
        ));
        ui.add_space(ui.spacing().item_spacing.y);

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
            bot::PAUSED.store(!bot::PAUSED.load(Ordering::Relaxed), Ordering::Relaxed);
        }
        //logger_ui(ui);
    }
}
