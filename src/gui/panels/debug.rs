use std::sync::atomic::Ordering;

use egui::Ui;

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
        ui.label(format!("Player 1: {:#06x}", unsafe {
            gd::get_player_1_addr().unwrap_or_default()
        }));
        ui.label(format!("Player 2: {:#06x}", unsafe {
            gd::get_player_2_addr().unwrap_or_default()
        }));
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
        ui.add_space(ui.spacing().item_spacing.y);

        ui.heading("Logging");
        ui.add_space(ui.spacing().item_spacing.y);
        egui::ScrollArea::new(egui::Vec2b::new(false, true))
            .max_height(300.0)
            .show(ui, |ui| {
                egui_logger::logger_ui(ui);
            });
    }
}
