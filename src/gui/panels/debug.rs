use std::sync::atomic::Ordering;

use egui::Ui;
use windows::Win32::System::Console::{AllocConsole, FreeConsole};

use crate::{bot, gd};

#[derive(Default)]
pub struct Debug {}

impl super::Panel for Debug {
    fn ui(&mut self, ui: &mut Ui) {
        ui.heading("Debug");
        ui.add_space(ui.spacing().item_spacing.y);

        ui.label(format!("GameManager: {:#06x}", unsafe {
            gd::get_game_manager_addr().unwrap_or_default()
        }));
        ui.label(format!("PlayLayer: {:#06x}", unsafe {
            gd::get_play_layer_addr().unwrap_or_default()
        }));
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
