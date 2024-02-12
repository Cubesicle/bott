use egui::Ui;

use crate::{bot, gui};

#[derive(Default)]
pub struct Record {
    file_name: String,
}

impl super::Page for Record {
    fn ui(&mut self, ui: &mut Ui) {
        ui.heading("Record");
        ui.add_space(ui.spacing().item_spacing.y);

        if ui
            .checkbox(&mut (bot::get_state() == bot::State::Recording), "Enabled")
            .clicked()
        {
            if bot::get_state() != bot::State::Recording {
                bot::set_state(bot::State::Recording);
            } else {
                bot::set_state(bot::State::Standby);
            }
        }
        ui.add_space(ui.spacing().item_spacing.y);

        ui.horizontal(|ui| {
            ui.add(
                egui::TextEdit::singleline(&mut self.file_name)
                    .hint_text("File name")
                    .desired_width(60.0)
                    .clip_text(false),
            );
            ui.add_space(-ui.spacing().item_spacing.x);
            ui.label(".csv");
            if ui.button("Save").clicked() {
                let message = bot::save_replay(self.file_name.as_str())
                    .err()
                    .map(|e| e.to_string())
                    .unwrap_or("Replay saved.".to_string());
                unsafe { gui::GUI.open_popup(message.as_str()) };
            }
        });
        ui.add_space(ui.spacing().item_spacing.y);

        if ui.button("Remove excess button events").clicked() {
            bot::optimize_button_events();
        }
        ui.add_space(ui.spacing().item_spacing.y);

        ui.label(format!(
            "Button events recorded: {}",
            bot::get_button_event_count()
        ));
    }
}
