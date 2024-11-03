use std::{ffi::{c_int, c_void}, mem::transmute};

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

#[no_mangle]
pub extern "C" fn bot_is_recording() -> bool {
    *bot::MODE.lock() == bot::Mode::Record
}

#[no_mangle]
pub extern "C" fn bot_is_replaying() -> bool {
    *bot::MODE.lock() == bot::Mode::Replay
}

#[no_mangle]
pub extern "C" fn bot_record_input(frame: c_int, pressed: bool, button: c_int, is_player_1: bool) {
    let button = match button {
        1 => gd::PlayerButton::Jump,
        2 => gd::PlayerButton::Left,
        3 => gd::PlayerButton::Right,
        _ => return,
    };

    bot::record_input(frame, bot::PlayerInput::new(
        pressed,
        button,
        is_player_1,
    ));
}