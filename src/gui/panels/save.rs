use egui::Ui;

use crate::{bot, gui};

#[derive(Default)]
pub struct Save {
    file_name: String,
    save_button_msg: Option<String>,
}

impl super::Panel for Save {
    fn ui(&mut self, ui: &mut Ui) {
        ui.heading("Save");
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
            let save_button = ui.button("Save");
            if save_button.clicked() {
                self.save_button_msg = Some(
                    bot::save_replay(self.file_name.as_str())
                        .err()
                        .map(|e| e.to_string())
                        .unwrap_or("Replay saved.".to_string()),
                );
            }
            gui::RBotGUI::show_message_tooltip(ui, &mut self.save_button_msg, &save_button);
        });
    }
}
