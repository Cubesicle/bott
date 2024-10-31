use std::{ffi::c_void, mem::transmute};

mod bot;
mod gd;
mod gui;

#[no_mangle]
pub extern "C" fn gui_run(ctx: *const c_void) {
    gui::run(unsafe { transmute::<_, &egui::Context>(ctx) });
}

#[no_mangle]
pub extern "C" fn gui_is_showing() -> bool {
    gui::is_showing()
}

#[no_mangle]
pub extern "C" fn show_gui(show: bool) {
    gui::show(show);
}