use std::collections::HashSet;
use std::ffi::OsString;

use egui::Ui;

use crate::{bot, gui};

#[derive(Default)]
pub struct Replay {
    file_names: HashSet<OsString>,
    selected_file_name: String,
    filter: String,
}

impl super::Page for Replay {
    fn ui(&mut self, ui: &mut Ui) {
        ui.heading("Replay");
        ui.add_space(ui.spacing().item_spacing.y);

        if ui
            .checkbox(&mut (bot::get_state() == bot::State::Replaying), "Enabled")
            .clicked()
        {
            if bot::get_state() != bot::State::Replaying {
                bot::set_state(bot::State::Replaying);
            } else {
                bot::set_state(bot::State::Standby);
            }
        }
        ui.add_space(ui.spacing().item_spacing.y);

        ui.horizontal(|ui| {
            egui::ComboBox::from_label("")
                .selected_text(self.selected_file_name.clone())
                .show_ui(ui, |ui| {
                    let text_box =
                        ui.add(egui::TextEdit::singleline(&mut self.filter).hint_text("Filter"));
                    if text_box.clicked() {
                        self.selected_file_name = String::new();
                    }
                    text_box.request_focus();
                    ui.add_space(ui.spacing().item_spacing.x);
                    for s in self.file_names.iter().filter(|f| {
                        self.filter.is_empty()
                            || f.to_str().unwrap_or_default().contains(&self.filter)
                    }) {
                        ui.selectable_value(
                            &mut self.selected_file_name,
                            s.to_str().unwrap_or_default().to_string(),
                            s.to_str().unwrap_or_default().to_string(),
                        );
                    }
                });
            ui.add_space(-ui.spacing().item_spacing.x);
            if ui.button("Load").clicked() {
                let message = bot::load_replay(self.selected_file_name.as_str())
                    .err()
                    .map(|e| e.to_string())
                    .unwrap_or("Replay loaded.".to_string());
                unsafe { gui::GUI.open_popup(message.as_str()) };
            }
            if ui.button("Scan for files").clicked() {
                self.file_names.clear();
                match std::fs::read_dir(bot::REPLAYS_DIR.as_path()) {
                    Ok(thing) => {
                        for f in thing.filter_map(|f| f.ok()).map(|f| f.file_name()) {
                            unsafe { gui::GUI.open_popup(format!("Found {:?}.", f).as_str()) }
                            self.file_names.insert(f);
                        }
                    }
                    Err(e) => unsafe { gui::GUI.open_popup(e.to_string().as_str()) },
                }
            }
        });
    }
}
