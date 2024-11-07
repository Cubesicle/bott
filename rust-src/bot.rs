use std::{ffi::c_void, path::{Path, PathBuf}, str::FromStr, sync::LazyLock, thread};
use anyhow::{Context, Error, Result};
use indexmap::IndexMap;
use parking_lot::Mutex;
use regex::Regex;
use serde::{de::Visitor, Deserialize, Deserializer, Serialize, Serializer};
use strum_macros::Display;
use crate::gd;

#[derive(Display, Eq, PartialEq)]
pub enum Mode {
    Standby,
    Record,
    Replay,
}

#[derive(Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Frame(pub i32);

impl Serialize for Frame {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer
    {
        serializer.serialize_str(&format!("frame-{}", self.0))
    }
}

impl<'de> Deserialize<'de> for Frame {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>
    {
        struct FrameVisitor;
        
        impl<'de> Visitor<'de> for FrameVisitor {
            type Value = Frame;
        
            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("frame-<frame-number>")
            }
            
            fn visit_str<E>(self, v: &str) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            { (|| {
                let re = Regex::new("^frame-([0-9]+)$")?;
                let captures = re.captures(v).context(format!("Failed to parse \"{v}\""))?;
                let frame_number = captures.get(1).map(|m| m.as_str().parse::<i32>().ok()).flatten()
                    .context(format!("Could not get frame number from {v}"))?;

                Ok(Frame(frame_number))
            })().map_err(|e: Error| serde::de::Error::custom(e)) }
        }
        
        deserializer.deserialize_str(FrameVisitor)
    }
}

#[derive(Debug, Eq, Hash, PartialEq)]
pub struct PlayerInput {
    pub button: gd::PlayerButton,
    pub is_player_1: bool,
}

impl Serialize for PlayerInput {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&format!(
            "player-{}-{}",
            if self.is_player_1 { "1" } else { "2" },
            self.button.to_string().to_lowercase()
        ))
    }
}

impl<'de> Deserialize<'de> for PlayerInput {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>
    {
        struct PlayerInputVisitor;

        impl<'de> Visitor<'de> for PlayerInputVisitor {
            type Value = PlayerInput;
        
            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("player-(1 | 2)-(jump | left | right)")
            }
            
            fn visit_str<E>(self, v: &str) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            { (|| {
                let re = Regex::new("^player-(1|2)-(jump|left|right)$")?;
                let captures = re.captures(v).context(format!("Failed to parse \"{v}\""))?;

                let is_player_1 = match captures.get(1) {
                    Some(m) if m.as_str() == "1" => Some(true),
                    Some(m) if m.as_str() == "2" => Some(false),
                    _ => None,
                }.context(format!("Could not get player from \"{v}\""))?;
                
                let button = match captures.get(2) {
                    Some(m) if m.as_str() == "jump" => Some(gd::PlayerButton::Jump),
                    Some(m) if m.as_str() == "left" => Some(gd::PlayerButton::Left),
                    Some(m) if m.as_str() == "right" => Some(gd::PlayerButton::Right),
                    _ => None,
                }.context(format!("Could not get player button from \"{v}\""))?;

                Ok(PlayerInput {
                    button,
                    is_player_1,
                })
            })().map_err(|e: Error| serde::de::Error::custom(e)) }
        }
        
        deserializer.deserialize_str(PlayerInputVisitor)
    }
}

pub type Inputs = IndexMap<Frame, IndexMap<PlayerInput, bool>>;

pub static REPLAY_DIR: LazyLock<PathBuf> = LazyLock::new(|| gd::geode::SAVE_DIR.join("replays"));
pub static MODE: Mutex<Mode> = Mutex::new(Mode::Standby);

static FRAME_STEPPER: Mutex<bool> = Mutex::new(false);
static FRAME_STEPPER_ADVANCE: Mutex<bool> = Mutex::new(false);
static RECORDED_INPUTS: LazyLock<Mutex<Inputs>> = LazyLock::new(|| 
    Mutex::new(IndexMap::new())
);

pub fn is_frame_stepper_on() -> bool {
    *FRAME_STEPPER.lock()
}

pub fn should_frame_stepper_advance() -> bool {
    *FRAME_STEPPER_ADVANCE.lock()
}

pub fn toggle_frame_stepper() {
    let frame_stepper = &mut *FRAME_STEPPER.lock();
    *frame_stepper = !*frame_stepper;
    
    *FRAME_STEPPER_ADVANCE.lock() = false;
}

pub fn set_frame_stepper_advance(advance: bool) {
    *FRAME_STEPPER_ADVANCE.lock() = advance;
}

pub fn record_input(frame: Frame, pressed: bool, input: PlayerInput) {
    thread::spawn(move || {
        let inputs = &mut *RECORDED_INPUTS.lock();
        
        while inputs.last().map(|(k, _)| k > &frame).unwrap_or(false) {
            inputs.pop();
        }
        
        if inputs.is_empty() && !pressed { return; }
        
        let previous_input_state = inputs.iter().rev().find_map(|(k, v)|
            (k != &frame).then(|| v.get(&input)).flatten()
        );
        if previous_input_state == Some(&pressed) {
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
        
        if inputs.first().map(|(k, v)| k == &Frame(1) && v.values().find(|v| **v).is_none()).unwrap_or(false) {
            inputs.shift_remove_index(0);
        }
    });
}

pub fn count_recorded_inputs() -> Option<usize> {
    //RECORDED_INPUTS.try_lock().map(|input_map| {
    //    let mut count = 0usize;
    //    for (_, inputs) in &*input_map {
    //        count += inputs.len();
    //    }
    //    count
    //})
    RECORDED_INPUTS.try_lock().map(|input_map| input_map.len())
}

pub fn handle_frame(
    frame: &Frame,
    this_ptr: *const c_void,
    handle_button: fn(*const c_void, bool, gd::PlayerButton, bool)
) {
    if let Some(input_map) = RECORDED_INPUTS.lock().get(frame) {
        for (input, pressed) in input_map { handle_button(this_ptr, *pressed, input.button, input.is_player_1); }
    }
}

pub fn save_replay(path: &Path) -> Result<()> {
    if !std::fs::exists(REPLAY_DIR.as_path())? {
        std::fs::create_dir(REPLAY_DIR.as_path())?;
    }

    let mut replay_data = toml::Table::new();
    replay_data.insert("inputs".to_string(), toml::Value::Table(toml::Table::try_from(&*RECORDED_INPUTS.lock())?));

    std::fs::write(path, toml::to_string_pretty(&replay_data)?)?;
    
    Ok(())
}

pub fn load_replay(path: &Path) -> Result<()> {
    let replay_data = toml::Table::from_str(&std::fs::read_to_string(path)?)?;
    let mut loaded_inputs = toml::from_str::<Inputs>(
        &toml::to_string(replay_data.get("inputs").context("Could not find inputs in replay file")?)?
    )?;
    
    let mut inputs = RECORDED_INPUTS.lock();
    inputs.clear();
    while let Some((k, v)) = loaded_inputs.shift_remove_index(0) {
        inputs.insert(k, v);
    }

    Ok(())
}

pub fn clear_inputs() {
    thread::spawn(move || RECORDED_INPUTS.lock().clear());
}