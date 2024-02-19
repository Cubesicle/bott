use std::collections::{HashSet, LinkedList};
use std::fs::{self, File};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use std::sync::RwLock;

use anyhow::{ensure, Result};
use indexmap::IndexMap;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};

use crate::gd::PlayerButton;
use crate::{gd, hooks};

lazy_static! {
    pub static ref PAUSED: AtomicBool = AtomicBool::new(false);
    pub static ref LOCK_DELTA_TIME: AtomicBool = AtomicBool::new(true);
    pub static ref REPLAYS_DIR: PathBuf = crate::EXE_PATH.parent().unwrap().join("bott");
    static ref STATE: AtomicU8 = AtomicU8::new(0);
    static ref BUTTON_EVENTS: RwLock<IndexMap<u32, RwLock<LinkedList<ButtonEvent>>>> =
        RwLock::new(IndexMap::new());
}

#[derive(Eq, PartialEq)]
pub enum State {
    Standby = 0,
    Recording = 1,
    Replaying = 2,
}

#[derive(Eq, Hash, PartialEq)]
pub struct ButtonEvent {
    button: PlayerButton,
    is_held_down: bool,
    is_player_1: bool,
}

impl ButtonEvent {
    pub fn new(button: PlayerButton, is_held_down: bool, is_player_1: bool) -> Self {
        Self {
            button,
            is_held_down,
            is_player_1,
        }
    }
}

#[derive(Deserialize, Serialize)]
struct ButtonEventWithFrame {
    frame: u32,
    button: PlayerButton,
    is_held_down: bool,
    is_player_1: bool,
}

impl ButtonEventWithFrame {
    pub fn new(frame: u32, button: PlayerButton, is_held_down: bool, is_player_1: bool) -> Self {
        Self {
            frame,
            button,
            is_held_down,
            is_player_1,
        }
    }
}

type UnmappedButtonEvents = LinkedList<ButtonEventWithFrame>;

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
            .insert(frame, RwLock::new(LinkedList::new()));
    }
    BUTTON_EVENTS
        .read()
        .unwrap()
        .get(&frame)
        .unwrap()
        .write()
        .unwrap()
        .push_back(button_event);
}

pub fn truncate_button_events_at_frame(frame: u32) {
    while BUTTON_EVENTS
        .read()
        .unwrap()
        .last()
        .map(|(k, _)| *k >= frame)
        .unwrap_or(false)
    {
        BUTTON_EVENTS.write().unwrap().pop();
    }
}

pub fn release_all_buttons_at_frame(frame: u32) {
    add_button_event(frame, ButtonEvent::new(PlayerButton::Jump, false, true));
    add_button_event(frame, ButtonEvent::new(PlayerButton::Jump, false, false));
    add_button_event(frame, ButtonEvent::new(PlayerButton::Left, false, true));
    add_button_event(frame, ButtonEvent::new(PlayerButton::Left, false, false));
    add_button_event(frame, ButtonEvent::new(PlayerButton::Right, false, true));
    add_button_event(frame, ButtonEvent::new(PlayerButton::Right, false, false));
}

pub fn optimize_button_events() {
    let mut button_events = UnmappedButtonEvents::new();
    dump_unmapped_optimized(&mut button_events);
    load_unmapped(&button_events);
}

pub fn handle_frame(frame: u32) -> Result<()> {
    if let Some(button_events) = BUTTON_EVENTS.read().unwrap().get(&frame) {
        for button_event in button_events.read().unwrap().iter() {
            log::debug!(
                "Bot: {} {:?} {} {}",
                frame,
                button_event.button,
                button_event.is_held_down,
                button_event.is_player_1,
            );
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

pub fn save_replay(file_name: &str) -> Result<()> {
    ensure!(
        !file_name.to_string().trim().is_empty(),
        "File name is empty."
    );
    let replay_file_path = REPLAYS_DIR.join(file_name.to_string() + ".csv");
    let _ = fs::create_dir(REPLAYS_DIR.as_path());
    let mut wtr = csv::Writer::from_writer(File::create_new(replay_file_path)?);
    let mut unmapped_button_events = UnmappedButtonEvents::default();
    dump_unmapped_optimized(&mut unmapped_button_events);
    for b in unmapped_button_events {
        wtr.serialize(b)?;
    }
    Ok(())
}

pub fn load_replay(file_name: &str) -> Result<()> {
    ensure!(
        !file_name.to_string().trim().is_empty(),
        "File name is empty."
    );
    let replay_file_path = REPLAYS_DIR.join(file_name.to_string());
    let mut rdr = csv::Reader::from_reader(File::open(replay_file_path)?);
    let mut unmapped_button_events = UnmappedButtonEvents::default();
    for b in rdr.deserialize() {
        unmapped_button_events.push_back(b?);
    }
    load_unmapped(&unmapped_button_events);
    Ok(())
}

fn dump_unmapped_optimized(unmapped_button_events: &mut UnmappedButtonEvents) {
    let mut pressed_buttons = HashSet::<(PlayerButton, bool)>::new();
    for (k, v) in BUTTON_EVENTS.read().unwrap().iter() {
        for v in v.read().unwrap().iter() {
            if (v.is_held_down == true && pressed_buttons.insert((v.button, v.is_player_1)) == true)
                || (v.is_held_down == false
                    && pressed_buttons.remove(&(v.button, v.is_player_1)) == true)
            {
                unmapped_button_events.push_back(ButtonEventWithFrame::new(
                    *k,
                    v.button,
                    v.is_held_down,
                    v.is_player_1,
                ));
            }
        }
    }
}

fn load_unmapped(unmapped_button_events: &UnmappedButtonEvents) {
    *BUTTON_EVENTS.write().unwrap() = IndexMap::new();
    for b in unmapped_button_events {
        add_button_event(
            b.frame,
            ButtonEvent::new(b.button, b.is_held_down, b.is_player_1),
        );
    }
}
