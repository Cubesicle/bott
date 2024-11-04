use std::{collections::HashSet, ffi::c_void, sync::LazyLock, thread};
use indexmap::IndexMap;
use parking_lot::Mutex;
use strum_macros::Display;
use crate::gd;

#[derive(Display, Eq, PartialEq)]
pub enum Mode {
    Standby,
    Record,
    Replay,
}

#[derive(Debug, Eq, Hash, PartialEq)]
pub struct PlayerInput {
    pressed: bool,
    button: gd::PlayerButton,
    is_player_1: bool,
}

impl PlayerInput {
    pub fn new(pressed: bool, button: gd::PlayerButton, is_player_1: bool) -> Self {
        PlayerInput {
            pressed,
            button,
            is_player_1,
        }
    }
}

pub static MODE: Mutex<Mode> = Mutex::new(Mode::Standby);
pub static RECORDED_INPUTS: LazyLock<Mutex<IndexMap<i32, HashSet<PlayerInput>>>> = LazyLock::new(|| 
    Mutex::new(IndexMap::new())
);

pub fn record_input(frame: i32, input: PlayerInput) {
    thread::spawn(move || {
        let inputs = &mut *RECORDED_INPUTS.lock();
        
        while inputs.last().map(|(k, _)| k >= &frame).unwrap_or(false) {
            inputs.pop();
        }
        
        let is_last_input_pressed = inputs.iter().rev().find_map(|(_, v)|
            v.iter().find(|PlayerInput { pressed: _, button, is_player_1 }|
                button == &input.button && is_player_1 == &input.is_player_1
            ).map(|input| input.pressed)
        );
        if is_last_input_pressed == Some(input.pressed) {
            return;
        }
        
        if let Some(input_set) = inputs.get_mut(&frame) {
            input_set.insert(input);
        } else {
            let mut input_set = HashSet::<PlayerInput>::new();
            input_set.insert(input);

            inputs.insert(frame, input_set);
        };
    });
}

pub fn handle_frame(
    frame: i32,
    this_ptr: *const c_void,
    handle_button: fn(*const c_void, bool, gd::PlayerButton, bool)
) {
    RECORDED_INPUTS.lock().get(&frame).and_then(|input_set| Some(input_set.iter().for_each(|input| {
        handle_button(this_ptr, input.pressed, input.button, input.is_player_1);
    })));
}