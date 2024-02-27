use std::ffi::CString;
use std::mem::transmute;
use std::sync::atomic::Ordering;
use std::sync::{Once, OnceLock};

use anyhow::{Context, Result};
use log::info;
use retour::static_detour;
use windows::core::{HSTRING, PCSTR};
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::Graphics::Gdi::{WindowFromDC, HDC};
use windows::Win32::System::LibraryLoader::{GetModuleHandleW, GetProcAddress};
use windows::Win32::UI::WindowsAndMessaging::{
    CallWindowProcW, DefWindowProcW, SetWindowLongPtrW, GWLP_WNDPROC, WNDPROC,
};

use crate::{bot, gd, gui};

static OLD_WND_PROC: OnceLock<WNDPROC> = OnceLock::new();

static_detour! {
    pub static WGLSwapBuffersHook: extern "stdcall" fn(HDC);
    pub static SchedulerUpdateHook: extern "thiscall" fn(usize, f32);
    pub static HandleButtonHook: extern "thiscall" fn(usize, bool, gd::PlayerButton, bool);
    pub static PushButtonHook: extern "thiscall" fn(usize, gd::PlayerButton);
    pub static ReleaseButtonHook: extern "thiscall" fn(usize, gd::PlayerButton);
    pub static PostUpdateHook: extern "thiscall" fn(usize, f32);
    pub static PauseGameHook: extern "thiscall" fn(usize, bool);
    pub static ResetLevelHook: extern "thiscall" fn(usize);
    pub static OnQuitHook: extern "thiscall" fn(usize);
    pub static LevelCompleteHook: extern "thiscall" fn(usize);
}

pub fn load() -> Result<()> {
    unsafe {
        let target = transmute(get_module_symbol_address("opengl32.dll", "wglSwapBuffers")?);
        WGLSwapBuffersHook
            .initialize(target, wgl_swap_buffers_detour)?
            .enable()?;

        let target = transmute(get_module_symbol_address(
            "libcocos2d.dll",
            "?update@CCScheduler@cocos2d@@UAEXM@Z",
        )?);
        SchedulerUpdateHook
            .initialize(target, |addr, dt| {
                if bot::PAUSED.load(Ordering::Relaxed) {
                    return;
                }
                if bot::LOCK_DELTA_TIME.load(Ordering::Relaxed) {
                    SchedulerUpdateHook.call(addr, 1.0 / gd::MAX_TPS as f32);
                    return;
                }
                SchedulerUpdateHook.call(addr, dt);
            })?
            .enable()?;

        HandleButtonHook
            .initialize(
                transmute(*gd::HANDLE_BUTTON_FN_ADDR),
                |addr, is_held_down, button, is_player_1| {
                    if bot::get_state() == bot::State::Replaying {
                        return;
                    }
                    HandleButtonHook.call(addr, is_held_down, button, is_player_1);
                },
            )?
            .enable()?;

        PushButtonHook
            .initialize(transmute(*gd::PUSH_BUTTON_FN_ADDR), |addr, button| {
                button_detour(addr, button, true);
                PushButtonHook.call(addr, button);
            })?
            .enable()?;

        ReleaseButtonHook
            .initialize(transmute(*gd::RELEASE_BUTTON_FN_ADDR), |addr, button| {
                button_detour(addr, button, false);
                ReleaseButtonHook.call(addr, button);
            })?
            .enable()?;

        PostUpdateHook
            .initialize(transmute(*gd::POST_UPDATE_FN_ADDR), |addr, dt| {
                if bot::get_state() == bot::State::Replaying {
                    bot::handle_frame(gd::get_current_frame().unwrap()).unwrap();
                }
                PostUpdateHook.call(addr, dt);
            })?
            .enable()?;

        PauseGameHook
            .initialize(transmute(*gd::PAUSE_GAME_FN_ADDR), |addr, p0| {
                info!("Paused.");
                if bot::get_state() == bot::State::Recording {
                    bot::release_all_buttons_at_frame(gd::get_current_frame().unwrap());
                }
                PauseGameHook.call(addr, p0);
            })?
            .enable()?;

        ResetLevelHook
            .initialize(transmute(*gd::RESET_LEVEL_FN_ADDR), |addr| {
                info!("Reset.");
                ResetLevelHook.call(addr);
                if bot::get_state() == bot::State::Recording {
                    bot::truncate_button_events_at_frame(gd::get_current_frame().unwrap());
                    bot::release_all_buttons_at_frame(gd::get_current_frame().unwrap());
                }
            })?
            .enable()?;

        OnQuitHook
            .initialize(transmute(*gd::ON_QUIT_FN_ADDR), |addr| {
                bot::optimize_button_events();
                bot::set_state(bot::State::Standby);
                OnQuitHook.call(addr);
            })?
            .enable()?;

        LevelCompleteHook
            .initialize(transmute(*gd::LEVEL_COMPLETE_FN_ADDR), |addr| {
                bot::optimize_button_events();
                bot::set_state(bot::State::Standby);
                LevelCompleteHook.call(addr);
            })?
            .enable()?;
    }

    Ok(())
}

pub fn unload() -> Result<()> {
    unsafe {
        let _ = WGLSwapBuffersHook.disable();
        let _ = SchedulerUpdateHook.disable();
        let _ = HandleButtonHook.disable();
        let _ = PushButtonHook.disable();
        let _ = ReleaseButtonHook.disable();
        let _ = PostUpdateHook.disable();
        let _ = PauseGameHook.disable();
        let _ = ResetLevelHook.disable();
        let _ = OnQuitHook.disable();
        let _ = LevelCompleteHook.disable();
    }

    std::thread::sleep(std::time::Duration::from_millis(500));

    let wnd_proc = OLD_WND_PROC
        .get()
        .unwrap_or(&None)
        .context("Failed to get original window procedure.");
    unsafe { SetWindowLongPtrW(gui::APP.get_window(), GWLP_WNDPROC, wnd_proc? as usize as _) };

    info!("Hooks unloaded.");
    Ok(())
}

fn wgl_swap_buffers_detour(hdc: HDC) {
    let window = unsafe { WindowFromDC(hdc) };

    static INIT: Once = Once::new();
    INIT.call_once(|| {
        info!("wglSwapBuffers successfully hooked.");

        unsafe { gui::APP.init_default(hdc, window, |ctx, t| gui::GUI.show(ctx, t)) };

        OLD_WND_PROC
            .set(unsafe {
                transmute(SetWindowLongPtrW(
                    window,
                    GWLP_WNDPROC,
                    call_wnd_proc_detour as usize as _,
                ))
            })
            .expect("Window procedure could not be set.");
    });

    if !unsafe { gui::APP.get_window().eq(&window) } {
        unsafe { SetWindowLongPtrW(window, GWLP_WNDPROC, call_wnd_proc_detour as usize as _) };
    }

    unsafe { gui::APP.render(hdc) };
    WGLSwapBuffersHook.call(hdc);
}

extern "stdcall" fn call_wnd_proc_detour(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    static INIT: Once = Once::new();
    INIT.call_once(|| {
        info!("CallWindowProcW successfully hooked.");
    });

    unsafe { gui::GUI.handle_keydown(msg, wparam, lparam) };

    let egui_wants_input = unsafe { gui::APP.wnd_proc(msg, wparam, lparam) };
    if egui_wants_input {
        return unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) };
    }

    unsafe { CallWindowProcW(*OLD_WND_PROC.get().unwrap(), hwnd, msg, wparam, lparam) }
}

unsafe fn button_detour(player_addr: usize, button: gd::PlayerButton, is_held_down: bool) {
    if bot::get_state() != bot::State::Recording || !gd::get_play_layer_addr().is_ok() {
        return;
    }
    if button != gd::PlayerButton::Jump && !bot::RECORD_PLATFORMER.load(Ordering::Relaxed) {
        return;
    }
    let is_player_1 = player_addr == gd::get_player_1_addr().unwrap();
    if !is_player_1 && !bot::RECORD_PLAYER_2.load(Ordering::Relaxed) {
        return;
    }
    let frame = gd::get_current_frame().unwrap();
    bot::add_button_event(
        frame,
        bot::ButtonEvent::new(button, is_held_down, is_player_1),
    );
    info!(
        "User: {} {:?} {} {}",
        frame, button, is_held_down, is_player_1
    );
}

fn get_module_symbol_address(module: &str, symbol: &str) -> Result<usize> {
    let symbol_cstring = CString::new(symbol).unwrap_or_default();
    let symbol_ansi = PCSTR(symbol_cstring.to_bytes_with_nul().as_ptr());
    let handle = unsafe { GetModuleHandleW(&HSTRING::from(module)) }
        .context(format!("Could not find {}.", module))?;
    unsafe { GetProcAddress(handle, symbol_ansi) }
        .map(|f| f as usize)
        .context(format!(
            "Could not get memory address of {} in {}.",
            symbol, module
        ))
}
