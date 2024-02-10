use std::ffi::CString;
use std::mem::transmute;
use std::sync::atomic::Ordering;
use std::sync::{Once, OnceLock};

use anyhow::{anyhow, Context, Result};
use lazy_static::lazy_static;
use log::info;
use retour::static_detour;
use windows::core::{PCSTR, PCWSTR};
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::Graphics::Gdi::{WindowFromDC, HDC};
use windows::Win32::System::LibraryLoader::{GetModuleHandleW, GetProcAddress};
use windows::Win32::UI::WindowsAndMessaging::{
    CallWindowProcW, DefWindowProcW, SetWindowLongPtrW, GWLP_WNDPROC, WNDPROC,
};

use crate::{bot, gd, gui};

lazy_static! {}
static OLD_WND_PROC: OnceLock<WNDPROC> = OnceLock::new();

static_detour! {
    pub static WGLSwapBuffersHook: extern "stdcall" fn(HDC);
    pub static SchedulerUpdateHook: extern "thiscall" fn(usize, f32);
    pub static HandleButtonHook: extern "thiscall" fn(usize, bool, gd::PlayerButton, bool);
    pub static PostUpdateHook: extern "thiscall" fn(usize, f32);
    pub static PauseGameHook: extern "thiscall" fn(usize, bool);
    pub static ResetLevelHook: extern "thiscall" fn(usize);
}

pub fn load() -> Result<()> {
    unsafe {
        let target = transmute(get_module_symbol_address(
            "opengl32.dll",
            "wglSwapBuffers",
        )?);
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
                SchedulerUpdateHook.call(addr, dt);
            })?
            .enable()?;

        HandleButtonHook
            .initialize(
                transmute(*gd::HANDLE_BUTTON_FN_ADDR),
                |addr, is_held_down, button, is_player_1| {
                    if bot::get_state() == bot::State::Recording
                        && gd::get_play_layer_addr().is_ok()
                    {
                        log::info!(
                        "Holding: {}; Button: {:?}; Player 1? {}; Frame: {};",
                        is_held_down,
                        button,
                        is_player_1,
                        gd::get_current_frame().unwrap_or_default()
                    );
                        bot::add_button_event(
                            gd::get_current_frame().unwrap(),
                            bot::ButtonEvent {
                                is_held_down: is_held_down,
                                button: button,
                                is_player_1: is_player_1,
                            },
                        )
                    }
                    HandleButtonHook.call(
                        addr,
                        is_held_down,
                        button,
                        is_player_1,
                    );
                },
            )?
            .enable()?;

        PostUpdateHook
            .initialize(transmute(*gd::POST_UPDATE_FN_ADDR), |addr, dt| {
                if bot::get_state() == bot::State::Replaying {
                    bot::handle_frame(gd::get_current_frame().unwrap())
                        .unwrap();
                }
                PostUpdateHook.call(addr, dt);
            })?
            .enable()?;

        PauseGameHook
            .initialize(transmute(*gd::PAUSE_GAME_FN_ADDR), |addr, p0| {
                if bot::get_state() == bot::State::Recording {
                    bot::release_all_buttons_at_frame(
                        gd::get_current_frame().unwrap(),
                    );
                }
                PauseGameHook.call(addr, p0);
            })?
            .enable()?;

        ResetLevelHook
            .initialize(transmute(*gd::RESET_LEVEL_FN_ADDR), |addr| {
                ResetLevelHook.call(addr);
                if bot::get_state() == bot::State::Recording {
                    bot::trim_button_events_after_frame(
                        gd::get_current_frame().unwrap(),
                    );
                    bot::release_all_buttons_at_frame(
                        gd::get_current_frame().unwrap(),
                    );
                }
            })?
            .enable()?;
    }

    Ok(())
}

pub fn unload() -> Result<()> {
    unsafe {
        WGLSwapBuffersHook.disable()?;
        SchedulerUpdateHook.disable()?;
        HandleButtonHook.disable()?;
        PostUpdateHook.disable()?;
        PauseGameHook.disable()?;
        ResetLevelHook.disable()?;
    }

    std::thread::sleep(std::time::Duration::from_millis(500));

    let wnd_proc = if let Some(Some(wnd_proc)) = OLD_WND_PROC.get() {
        Ok(*wnd_proc)
    } else {
        Err(anyhow!("Failed to get original window procedure."))
    };
    unsafe {
        SetWindowLongPtrW(
            gui::APP.get_window(),
            GWLP_WNDPROC,
            wnd_proc? as usize as _,
        )
    };

    info!("Hooks unloaded.");
    Ok(())
}

fn wgl_swap_buffers_detour(hdc: HDC) {
    let window = unsafe { WindowFromDC(hdc) };

    static INIT: Once = Once::new();
    INIT.call_once(|| {
        info!("wglSwapBuffers successfully hooked.");

        unsafe {
            gui::APP.init_default(hdc, window, |ctx, t| {
                gui::GUI.write().unwrap().show(ctx, t)
            })
        };

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
        unsafe {
            SetWindowLongPtrW(
                window,
                GWLP_WNDPROC,
                call_wnd_proc_detour as usize as _,
            )
        };
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

    gui::GUI
        .write()
        .unwrap()
        .handle_keydown(msg, wparam, lparam);

    let egui_wants_input = unsafe { gui::APP.wnd_proc(msg, wparam, lparam) };
    if egui_wants_input {
        return unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) };
    }

    unsafe {
        CallWindowProcW(*OLD_WND_PROC.get().unwrap(), hwnd, msg, wparam, lparam)
    }
}

pub fn get_module_symbol_address(module: &str, symbol: &str) -> Result<usize> {
    let module_utf16 = PCWSTR(
        module
            .encode_utf16()
            .chain(std::iter::once(0))
            .collect::<Vec<u16>>()
            .as_ptr(),
    );
    let symbol_cstring = CString::new(symbol).unwrap_or_default();
    let symbol_ansi = PCSTR(symbol_cstring.to_bytes_with_nul().as_ptr());
    let handle = unsafe { GetModuleHandleW(module_utf16) }
        .context(format!("Could not find {}", module))?;
    if let Some(func) = unsafe { GetProcAddress(handle, symbol_ansi) } {
        Ok(func as usize)
    } else {
        Err(anyhow!(
            "Could not get memory address of {} in {}",
            symbol,
            module
        ))
    }
}
