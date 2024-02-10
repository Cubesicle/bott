use std::collections::{HashSet, LinkedList};
use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use std::sync::RwLock;

use anyhow::Result;
use indexmap::{IndexMap, IndexSet};
use lazy_static::lazy_static;

use crate::{gd, hooks};

lazy_static! {
    pub static ref PAUSED: AtomicBool = AtomicBool::new(false);
    static ref STATE: AtomicU8 = AtomicU8::new(0);
    static ref BUTTON_EVENTS: RwLock<IndexMap<u32, RwLock<IndexSet<ButtonEvent>>>> =
        RwLock::new(IndexMap::new());
}

#[derive(Eq, PartialEq)]
pub enum State {
    Standby = 0,
    Recording = 1,
    Replaying = 2,
}

#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub struct ButtonEvent {
    pub is_held_down: bool,
    pub button: gd::PlayerButton,
    pub is_player_1: bool,
}

pub fn get_state() -> State {
    match STATE.load(Ordering::Relaxed) {
        1 => State::Recording,
        2 => State::Replaying,
        _ => State::Standby,
    }
}

pub fn set_state(state: State) {
    STATE.store(state as u8, Ordering::Relaxed);
}

pub fn get_button_event_count() -> usize {
    BUTTON_EVENTS.read().unwrap().len()
}

pub fn add_button_event(frame: u32, button_event: ButtonEvent) {
    if !BUTTON_EVENTS.write().unwrap().contains_key(&frame) {
        BUTTON_EVENTS
            .write()
            .unwrap()
            .insert(frame, RwLock::new(IndexSet::new()));
    }
    BUTTON_EVENTS
        .read()
        .unwrap()
        .get(&frame)
        .unwrap()
        .write()
        .unwrap()
        .insert(button_event);
}

pub fn trim_button_events_after_frame(frame: u32) {
    BUTTON_EVENTS.write().unwrap().retain(|k, _| *k < frame);
}

pub fn release_all_buttons_at_frame(frame: u32) {
    add_button_event(
        frame,
        ButtonEvent {
            is_held_down: false,
            button: gd::PlayerButton::Jump,
            is_player_1: true,
        },
    );
    add_button_event(
        frame,
        ButtonEvent {
            is_held_down: false,
            button: gd::PlayerButton::Jump,
            is_player_1: false,
        },
    );
    add_button_event(
        frame,
        ButtonEvent {
            is_held_down: false,
            button: gd::PlayerButton::Left,
            is_player_1: true,
        },
    );
    add_button_event(
        frame,
        ButtonEvent {
            is_held_down: false,
            button: gd::PlayerButton::Left,
            is_player_1: false,
        },
    );
    add_button_event(
        frame,
        ButtonEvent {
            is_held_down: false,
            button: gd::PlayerButton::Right,
            is_player_1: true,
        },
    );
    add_button_event(
        frame,
        ButtonEvent {
            is_held_down: false,
            button: gd::PlayerButton::Right,
            is_player_1: false,
        },
    );
}

pub fn optimize_button_events() {
    let mut raw_button_events: LinkedList<(u32, ButtonEvent)> =
        LinkedList::new();
    BUTTON_EVENTS.read().unwrap().iter().for_each(|(k, v)| {
        v.read().unwrap().iter().for_each(|v| {
            raw_button_events.push_back((*k, *v));
        })
    });
    BUTTON_EVENTS.write().unwrap().clear();
    let mut pressed_buttons: HashSet<(gd::PlayerButton, bool)> = HashSet::new();
    raw_button_events.iter().for_each(|(k, v)| {
        if (v.is_held_down == true
            && pressed_buttons.insert((v.button, v.is_player_1)) == true)
            || (v.is_held_down == false
                && pressed_buttons.remove(&(v.button, v.is_player_1)) == true)
        {
            add_button_event(*k, *v);
        }
    });
}

pub fn handle_frame(frame: u32) -> Result<()> {
    if let Some(button_events) = BUTTON_EVENTS.read().unwrap().get(&frame) {
        for button_event in button_events.read().unwrap().iter() {
            hooks::HandleButtonHook.call(
                unsafe { gd::get_play_layer_addr()? },
                button_event.is_held_down,
                button_event.button,
                button_event.is_player_1,
            );
        }
    }
    Ok(())
}
