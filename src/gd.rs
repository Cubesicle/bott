use std::mem::transmute;

use anyhow::{anyhow, Result};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;

// https://github.com/Prevter/gd.hpp/tree/main/include/mappings
pub static BASE: Lazy<usize> = Lazy::new(|| unsafe { GetModuleHandleW(None) }.unwrap().0 as usize);
pub static GM_SHARED_STATE_FN_ADDR: Lazy<usize> = Lazy::new(|| *BASE + 0x121540);
pub static POST_UPDATE_FN_ADDR: Lazy<usize> = Lazy::new(|| *BASE + 0x2e7220);
pub static HANDLE_BUTTON_FN_ADDR: Lazy<usize> = Lazy::new(|| *BASE + 0x1b69f0);
pub static PUSH_BUTTON_FN_ADDR: Lazy<usize> = Lazy::new(|| *BASE + 0x2d1d30);
pub static RELEASE_BUTTON_FN_ADDR: Lazy<usize> = Lazy::new(|| *BASE + 0x2d1f70);
pub static PAUSE_GAME_FN_ADDR: Lazy<usize> = Lazy::new(|| *BASE + 0x2eae80);
pub static RESET_LEVEL_FN_ADDR: Lazy<usize> = Lazy::new(|| *BASE + 0x2ea130);
pub static ON_QUIT_FN_ADDR: Lazy<usize> = Lazy::new(|| *BASE + 0x2eb480);
pub static LEVEL_COMPLETE_FN_ADDR: Lazy<usize> = Lazy::new(|| *BASE + 0x2ddb60);
pub static UPDATE_FPS_FN_ADDR: Lazy<usize> = Lazy::new(|| *BASE + 0x12ec60);

pub static MAX_TPS: u8 = 240;
pub static CUSTOM_FPS_TARGET_OFFSET: usize = 0x384;
pub static PLAY_LAYER_OFFSET: usize = 0x198;
pub static TIME_OFFSET: usize = 0x328;
pub static PLAYER_1_OFFSET: usize = 0x878;
pub static PLAYER_2_OFFSET: usize = 0x87C;

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[repr(u32)]
pub enum PlayerButton {
    Jump = 1,
    Left = 2,
    Right = 3,
}

pub unsafe fn get_game_manager_addr() -> Result<usize> {
    let game_manager_addr =
        transmute::<_, unsafe extern "stdcall" fn() -> usize>(*GM_SHARED_STATE_FN_ADDR)();
    if game_manager_addr == 0 {
        return Err(anyhow!("Could not get GameManager address."));
    }
    Ok(game_manager_addr)
}

pub unsafe fn get_mut_fps() -> Result<*mut f32> {
    Ok(transmute::<_, *mut f32>(
        get_game_manager_addr()? + CUSTOM_FPS_TARGET_OFFSET,
    ))
}

pub unsafe fn update_custom_fps() -> Result<()> {
    transmute::<_, unsafe extern "thiscall" fn(usize)>(*UPDATE_FPS_FN_ADDR)(
        get_game_manager_addr()?
    );
    Ok(())
}

pub unsafe fn get_play_layer_addr() -> Result<usize> {
    let play_layer_ptr = (get_game_manager_addr()? + PLAY_LAYER_OFFSET) as *const usize;
    if *play_layer_ptr == 0 {
        return Err(anyhow!("GJBaseGameLayer does not exist."));
    }
    Ok(*play_layer_ptr)
}

pub unsafe fn get_current_frame() -> Result<u32> {
    Ok((*((get_play_layer_addr()? + TIME_OFFSET) as *const f64) * MAX_TPS as f64) as u32)
}

pub unsafe fn get_player_1_addr() -> Result<usize> {
    let ptr = (get_play_layer_addr()? + PLAYER_1_OFFSET) as *const usize;
    if *ptr == 0 {
        return Err(anyhow!("Player 1 does not exist."));
    }
    Ok(*ptr)
}

pub unsafe fn get_player_2_addr() -> Result<usize> {
    let ptr = (get_play_layer_addr()? + PLAYER_2_OFFSET) as *const usize;
    if *ptr == 0 {
        return Err(anyhow!("Player 2 does not exist."));
    }
    Ok(*ptr)
}
