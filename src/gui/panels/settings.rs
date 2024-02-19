use std::sync::atomic::Ordering;

use crate::{bot, gd};

#[derive(Default)]
pub struct Settings {}

impl super::Panel for Settings {
    fn ui(&mut self, ui: &mut egui::Ui) {
        ui.heading("Settings");
        ui.add_space(ui.spacing().item_spacing.y);

        ui.label("Mode:");
        ui.horizontal(|ui| {
            if ui
                .add(egui::RadioButton::new(
                    bot::get_state() == bot::State::Standby,
                    "Standby",
                ))
                .clicked()
            {
                bot::set_state(bot::State::Standby);
            }
            if ui
                .add(egui::RadioButton::new(
                    bot::get_state() == bot::State::Recording,
                    "Record",
                ))
                .clicked()
            {
                bot::set_state(bot::State::Recording);
            }
            if ui
                .add(egui::RadioButton::new(
                    bot::get_state() == bot::State::Replaying,
                    "Replay",
                ))
                .clicked()
            {
                bot::set_state(bot::State::Replaying);
            }
        });
        ui.add_space(ui.spacing().item_spacing.y);

        if let Ok(fps) = unsafe { gd::get_mut_fps() } {
            ui.horizontal(|ui| {
                ui.label("FPS cap:");
                if ui
                    .add(egui::DragValue::new(unsafe { &mut *fps }).clamp_range(60..=u32::MAX))
                    .changed()
                {
                    let _ = unsafe { gd::update_custom_fps() };
                }
                if ui.button("Default").clicked() {
                    unsafe { *fps = gd::MAX_TPS as f32 };
                    let _ = unsafe { gd::update_custom_fps() };
                }
            });
            ui.add_space(ui.spacing().item_spacing.y);
        }

        ui.label("*Recommended");
        if ui
            .add(egui::Checkbox::new(
                &mut bot::LOCK_DELTA_TIME.load(Ordering::Relaxed),
                "Lock delta time",
            ))
            .clicked()
        {
            bot::LOCK_DELTA_TIME.store(
                !bot::LOCK_DELTA_TIME.load(Ordering::Relaxed),
                Ordering::Relaxed,
            );
        }
        ui.add_space(ui.spacing().item_spacing.y);

        ui.label(format!(
            "Current frame: {}",
            unsafe { gd::get_current_frame() }
                .map(|f| f.to_string())
                .unwrap_or("N/A".to_string())
        ));
        ui.label(format!(
            "Button events in memory: {}",
            bot::get_button_event_count()
        ));
        if ui.button("Remove excess button events").clicked() {
            bot::optimize_button_events();
        }
    }
}
