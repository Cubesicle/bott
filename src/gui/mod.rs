use egui_opengl_internal::OpenGLApp;
use std::{collections::HashMap, hash::Hash};
use windows::Win32::UI::Input::KeyboardAndMouse::{VIRTUAL_KEY, VK_RSHIFT, GetAsyncKeyState};

mod pages;

#[derive(Eq, Hash, PartialEq)]
enum Keybinds {
    ToggleGUI
}

pub struct RBotGUI {
    open: bool,
    current_page: pages::Pages,
    pages: Option<HashMap<pages::Pages, Box<dyn pages::Page>>>,
    keybinds: Option<HashMap<Keybinds, VIRTUAL_KEY>>,
}

impl RBotGUI {
    pub const fn new() -> Self {
        Self {
            open: true,
            current_page: pages::Pages::Record,
            pages: None,
            keybinds: None,
        }
    }

    pub fn init(&mut self) {
        self.pages = Some(HashMap::new());
        self.pages.as_mut().unwrap().insert(pages::Pages::Record, Box::new(pages::Record::default()));
        self.pages.as_mut().unwrap().insert(pages::Pages::Replay, Box::new(pages::Replay::default()));
        self.pages.as_mut().unwrap().insert(pages::Pages::Debug, Box::new(pages::Debug::default()));

        self.keybinds = Some(HashMap::from([
            (Keybinds::ToggleGUI, VK_RSHIFT)
        ]));
    }

    pub fn name(&self) -> &'static str { "rBot" }

    pub fn show(&mut self, ctx: &egui::Context, _: &mut i32) {
        let inner_margin = 2.0;
        let outer_margin = 8.0;

        egui::Window::new(self.name())
            .open(&mut self.open)
            .show(ctx, |ui| {
                egui::TopBottomPanel::top("header").show_inside(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.selectable_value(
                            &mut self.current_page,
                            pages::Pages::Record,
                            "Record"
                        );
                        ui.selectable_value(
                            &mut self.current_page,
                            pages::Pages::Replay,
                            "Replay"
                        );
                        ui.selectable_value(
                            &mut self.current_page,
                            pages::Pages::Debug,
                            "Debug"
                        );
                        ui.menu_button("Eject", |ui| {
                            ui.label("Do you want to eject the DLL?");
                            if ui.button("No").clicked() {
                                ui.close_menu();
                            }
                            if ui.button("Yes").clicked() {                
                                unsafe { crate::EXITING = true; }
                            }
                        });
                    });
                    ui.add_space(inner_margin);
                });
                egui::TopBottomPanel::bottom("footer").min_height(0.0).show_inside(ui, |ui| {
                    ui.add_space(outer_margin);
                    ui.vertical_centered(|ui| {
                        ui.label("Made with ❤ by Cubesicle.");
                    });
                });
                egui::CentralPanel::default().show_inside(ui, |ui| {
                    egui::ScrollArea::both().show(ui, |ui| {
                        match &self.current_page {
                            page => {
                                self.pages.as_mut().unwrap().get_mut(page).unwrap().ui(ui);
                            }
                        }
                    });
                });
            });
    }

    pub fn detect_keybinds(&mut self) {
        unsafe {
            let key = self.keybinds.as_ref().unwrap().get(&Keybinds::ToggleGUI).unwrap().0 as i32;
            if GetAsyncKeyState(key) & 0x01 == 1 {
                self.open = !self.open;
            }
        }
    }
}

pub static mut APP: OpenGLApp<i32> = OpenGLApp::new();
pub static mut GUI: RBotGUI = RBotGUI::new();
