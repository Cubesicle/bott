use egui::Ui;

use crate::{bot, gui};

#[derive(Default)]
pub struct Load {
    selected_file_name: String,
    filter: String,
    load_button_msg: Option<String>,
    delete_button_msg: Option<String>,
}

impl super::Panel for Load {
    fn ui(&mut self, ui: &mut Ui) {
        ui.heading("Load");
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
                    if let Ok(list) = std::fs::read_dir(bot::REPLAYS_DIR.as_path()) {
                        egui::ScrollArea::new([false, true]).show(ui, |ui| {
                            for f in list
                                .filter_map(|f| f.ok())
                                .map(|f| f.file_name().to_str().unwrap_or_default().to_string())
                                .filter(|f| self.filter.is_empty() || f.contains(&self.filter))
                            {
                                ui.selectable_value(&mut self.selected_file_name, f.clone(), f);
                            }
                        });
                    }
                })
                .response;
            ui.add_space(-ui.spacing().item_spacing.x);
            let load_button = ui.button("Load");
            if load_button.clicked() {
                self.load_button_msg = Some(
                    bot::load_replay(self.selected_file_name.as_str())
                        .err()
                        .map(|e| e.to_string())
                        .unwrap_or("Replay loaded.".to_string()),
                );
            }
            gui::BottGUI::show_message_tooltip(ui, &mut self.load_button_msg, &load_button);
            ui.menu_button("Delete", |ui| {
                if ui.button("nuh uh").clicked() {
                    ui.close_menu();
                }
                let yes_button = ui.button("yes plz");
                if yes_button.clicked() {
                    if self.selected_file_name.is_empty() {
                        self.delete_button_msg = Some("File name is empty.".to_string());
                    } else if let Err(e) =
                        std::fs::remove_file(bot::REPLAYS_DIR.join(&self.selected_file_name))
                    {
                        self.delete_button_msg = Some(e.to_string());
                    } else {
                        self.selected_file_name = String::new();
                        ui.close_menu()
                    }
                }
                gui::BottGUI::show_message_tooltip(ui, &mut self.delete_button_msg, &yes_button);
            });
        });
    }
}
