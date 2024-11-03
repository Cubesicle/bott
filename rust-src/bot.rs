use std::{collections::HashSet, sync::LazyLock};
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
    println!("{}: {:?}", frame, input);
    let inputs = &mut *RECORDED_INPUTS.lock();
    
    while inputs.last().map(|(k, _)| k >= &frame).unwrap_or(false) {
        inputs.pop();
    }
    
    if let Some(input_set) = inputs.get_mut(&frame) {
        input_set.insert(input);
    } else {
        let mut input_set = HashSet::<PlayerInput>::new();
        input_set.insert(input);

        inputs.insert(frame, input_set);
    };
}