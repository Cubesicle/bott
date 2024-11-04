use std::{ffi::c_void, sync::LazyLock, thread};
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
    button: gd::PlayerButton,
    is_player_1: bool,
}

impl PlayerInput {
    pub fn new(button: gd::PlayerButton, is_player_1: bool) -> Self {
        PlayerInput {
            button,
            is_player_1,
        }
    }
}

pub static MODE: Mutex<Mode> = Mutex::new(Mode::Standby);
pub static RECORDED_INPUTS: LazyLock<Mutex<IndexMap<i32, IndexMap<PlayerInput, bool>>>> = LazyLock::new(|| 
    Mutex::new(IndexMap::new())
);

pub fn record_input(frame: i32, pressed: bool, input: PlayerInput) {
    thread::spawn(move || {
        let inputs = &mut *RECORDED_INPUTS.lock();
        
        while inputs.last().map(|(k, _)| k > &frame).unwrap_or(false) {
            inputs.pop();
        }
        
        if inputs.is_empty() && !pressed { return; }
        
        let is_last_input_pressed = inputs.iter().rev().find_map(|(k, v)|
            (k != &frame).then(|| v.get(&input)).flatten()
        );
        if is_last_input_pressed == Some(&pressed) {
            if let Some(input_map) = inputs.get_mut(&frame) {
                if let Some(input_pressed) = input_map.get(&input) {
                    if input_pressed != &pressed {
                        input_map.shift_remove(&input);
                        if input_map.is_empty() { inputs.shift_remove(&frame); }
                    }
                }
            }
        } else if let Some(input_map) = inputs.get_mut(&frame) {
            input_map.insert(input, pressed);
        } else {
            let mut input_map = IndexMap::<PlayerInput, bool>::new();
            input_map.insert(input, pressed);

            inputs.insert(frame, input_map);
        };
        
        if inputs.first().map(|(k, v)| k == &1 && v.values().find(|v| **v).is_none()).unwrap_or(false) {
            inputs.shift_remove_index(0);
        }
    });
}

pub fn handle_frame(
    frame: i32,
    this_ptr: *const c_void,
    handle_button: fn(*const c_void, bool, gd::PlayerButton, bool)
) {
    RECORDED_INPUTS.lock().get(&frame).and_then(|input_map| Some(input_map.iter().for_each(|(input, pressed)| {
        handle_button(this_ptr, *pressed, input.button, input.is_player_1);
    })));
}