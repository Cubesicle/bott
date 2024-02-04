use std::ffi::CString;
use std::mem::transmute;
use std::os::raw::c_void;
use std::sync::Once;

use anyhow::{anyhow, Context, Result};
use log::info;
use retour::static_detour;
use windows::core::{PCSTR, PCWSTR};
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::Graphics::Gdi::{WindowFromDC, HDC};
use windows::Win32::System::LibraryLoader::{GetModuleHandleW, GetProcAddress};
use windows::Win32::UI::WindowsAndMessaging::{
    CallWindowProcW, DefWindowProcW, SetWindowLongPtrW, GWLP_WNDPROC, WNDPROC,
};

use crate::gui;

static mut OLD_WND_PROC: Option<WNDPROC> = None;

static_detour! {
    static WGLSwapBuffersHook: extern "system" fn(HDC);
}

pub fn load() -> Result<()> {
    let address = get_module_symbol_address("opengl32.dll", "wglSwapBuffers")?;
    let target = unsafe { std::mem::transmute(address) };
    unsafe {
        WGLSwapBuffersHook
            .initialize(target, wgl_swap_buffers_detour)?
            .enable()?
    };

    Ok(())
}

pub fn unload() -> Result<()> {
    unsafe { WGLSwapBuffersHook.disable() }?;

    std::thread::sleep(std::time::Duration::from_millis(500));

    let wnd_proc =
        if let Some(wnd_proc) = unsafe { OLD_WND_PROC.unwrap_or_default() } {
            Ok(wnd_proc)
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
                gui::GUI.lock().show(ctx, t)
            })
        };

        unsafe {
            OLD_WND_PROC = Some(transmute(SetWindowLongPtrW(
                window,
                GWLP_WNDPROC,
                call_wnd_proc_detour as usize as _,
            )))
        };
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

extern "system" fn call_wnd_proc_detour(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    static INIT: Once = Once::new();
    INIT.call_once(|| {
        info!("CallWindowProcW successfully hooked.");
    });

    gui::GUI.lock().handle_keydown(msg, wparam, lparam);

    let egui_wants_input = unsafe { gui::APP.wnd_proc(msg, wparam, lparam) };
    if egui_wants_input {
        return unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) };
    }

    unsafe { CallWindowProcW(OLD_WND_PROC.unwrap(), hwnd, msg, wparam, lparam) }
}

pub fn get_module_symbol_address(
    module: &str,
    symbol: &str,
) -> Result<*const c_void> {
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
        Ok(func as *const c_void)
    } else {
        Err(anyhow!(
            "Could not get memory address of {} in {}",
            symbol,
            module
        ))
    }
}
