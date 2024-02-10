use std::mem::transmute;

use anyhow::{anyhow, Result};
use lazy_static::lazy_static;
use windows::Win32::System::LibraryLoader::GetModuleHandleW;

// Note: PlayLayer derives from GJBaseGameLayer.
// GameManager.m_playLayer = GameManager + 0x198
// GJBaseGameLayer.m_gameState = GJBaseGameLayer +
// GJBaseGameLayer.m_player1 = GJBaseGameLayer + 0x878
// GJBaseGameLayer.m_player2 = GJBaseGameLayer + 0x87C

lazy_static! {
    pub static ref BASE: usize =
        unsafe { GetModuleHandleW(None) }.unwrap().0 as usize;
    pub static ref GM_SHARED_STATE_FN_ADDR: usize = *BASE + 0x121540;
    pub static ref POST_UPDATE_FN_ADDR: usize = *BASE + 0x2e7220;
    pub static ref HANDLE_BUTTON_FN_ADDR: usize = *BASE + 0x1b69f0;
    pub static ref PAUSE_GAME_FN_ADDR: usize = *BASE + 0x2eae80;
    pub static ref RESET_LEVEL_FN_ADDR: usize = *BASE + 0x2ea130;
}
pub static MAX_TPS: u8 = 240;
pub static PLAY_LAYER_OFFSET: usize = 0x198;
pub static CURRENT_FRAME_OFFSET: usize = 0x328;
pub static PLAYER_1_OFFSET: usize = 0x878;
pub static PLAYER_2_OFFSET: usize = 0x87C;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[repr(u32)]
pub enum PlayerButton {
    Jump = 1,
    Left = 2,
    Right = 3,
}

pub unsafe fn get_game_manager_addr() -> Result<usize> {
    let game_manager_addr = transmute::<_, unsafe extern "stdcall" fn() -> usize>(
        *GM_SHARED_STATE_FN_ADDR,
    )();
    if game_manager_addr == 0 {
        return Err(anyhow!("Could not get GameManager address."));
    }
    Ok(game_manager_addr)
}

pub unsafe fn get_play_layer_addr() -> Result<usize> {
    let play_layer_ptr =
        (get_game_manager_addr()? + PLAY_LAYER_OFFSET) as *const usize;
    if *play_layer_ptr == 0 {
        return Err(anyhow!("GJBaseGameLayer does not exist."));
    }
    Ok(*play_layer_ptr)
}

pub unsafe fn get_current_frame() -> Result<u32> {
    Ok(
        (*((get_play_layer_addr()? + CURRENT_FRAME_OFFSET) as *const f64)
            * MAX_TPS as f64) as u32,
    )
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
