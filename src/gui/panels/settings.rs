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
                    .add(egui::DragValue::new(unsafe { &mut *fps }).clamp_range(60..=1000))
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

        if ui
            .add(egui::Checkbox::new(
                &mut bot::ALLOW_FRAME_SKIPPING.load(Ordering::Relaxed),
                "Allow frame skipping",
            ))
            .clicked()
        {
            bot::ALLOW_FRAME_SKIPPING.store(
                !bot::ALLOW_FRAME_SKIPPING.load(Ordering::Relaxed),
                Ordering::Relaxed,
            );
        }
        ui.add_space(ui.spacing().item_spacing.y);

        ui.label(format!("Current frame: {}", unsafe {
            gd::get_current_frame().unwrap_or(0)
        }));
        ui.label(format!(
            "Button events in memory: {}",
            bot::get_button_event_count()
        ));
        if ui.button("Remove excess button events").clicked() {
            bot::optimize_button_events();
        }
    }
}
