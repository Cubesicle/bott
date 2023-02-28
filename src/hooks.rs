use crate::gui;

use anyhow::{anyhow, Context, Result};
use log::info;
use retour::static_detour;
use std::ffi::CString;
use std::mem::transmute;
use std::os::raw::c_void;
use std::sync::Once;
use windows::Win32::Foundation::{HWND, WPARAM, LPARAM, LRESULT};
use windows::core::{PCSTR, PCWSTR};
use windows::Win32::Graphics::Gdi::{HDC, WindowFromDC};
use windows::Win32::System::LibraryLoader::{GetModuleHandleW, GetProcAddress};
use windows::Win32::UI::WindowsAndMessaging::{GWLP_WNDPROC, WNDPROC, SetWindowLongPtrA, CallWindowProcW};

static mut OLD_WND_PROC: Option<WNDPROC> = None;

static_detour! {
    static WGLSwapBuffersHook: extern "system" fn(HDC);
}

type FnWGLSwapBuffers = extern "system" fn(HDC);

pub fn load() -> Result<()> {
    unsafe {
        let address = get_module_symbol_address("opengl32.dll", "wglSwapBuffers")?;
        let target: FnWGLSwapBuffers = std::mem::transmute(address);
        WGLSwapBuffersHook
            .initialize(target, wgl_swap_buffers_detour)?
            .enable()?;
    }

    Ok(())
}

pub fn unload() -> Result<()>{
    unsafe {
        WGLSwapBuffersHook.disable()?;
    }

    std::thread::sleep(std::time::Duration::from_millis(500));

    unsafe {
        let wnd_proc = if let Some(wnd_proc) = OLD_WND_PROC.unwrap_or_default() {
            Ok(wnd_proc)
        } else {
            Err(anyhow!("Failed to get original window procedure."))
        };
        let _: Option<WNDPROC> = Some(transmute(SetWindowLongPtrA(
            gui::APP.get_window(),
            GWLP_WNDPROC,
            wnd_proc? as usize as _,
        )));
    }

    Ok(())
}

fn wgl_swap_buffers_detour(hdc: HDC) {
    unsafe {
        static INIT: Once = Once::new();
        INIT.call_once(|| {
            info!("wglSwapBuffers successfully hooked.");

            let window = WindowFromDC(hdc);
            gui::APP.init_default(hdc, window, |ctx, t| gui::GUI.show(ctx, t));

            OLD_WND_PROC = Some(transmute(SetWindowLongPtrA(
                window,
                GWLP_WNDPROC,
                call_wnd_proc_detour as usize as _,
            )));
        });

        gui::APP.render(hdc);
        WGLSwapBuffersHook.call(hdc);
    }
}

unsafe extern "stdcall" fn call_wnd_proc_detour(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    static INIT: Once = Once::new();
    INIT.call_once(|| {
        info!("CallWindowProcW successfully hooked.");
    });

    gui::GUI.detect_keybinds(); 

    let egui_wants_input = gui::APP.wnd_proc(msg, wparam, lparam);
    if egui_wants_input {
        return LRESULT(1);
    }

    CallWindowProcW(OLD_WND_PROC.unwrap(), hwnd, msg, wparam, lparam)
}

pub fn get_module_symbol_address(module: &str, symbol: &str) -> Result<*const c_void> {
    let module_utf16 = module
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect::<Vec<u16>>();
    let symbol_ansi = CString::new(symbol).unwrap_or_default();
    unsafe {
        let handle = GetModuleHandleW(PCWSTR::from_raw(module_utf16.as_ptr())).context(format!("Could not find {}", module))?;
        if let Some(func) = GetProcAddress(
            handle,
            PCSTR::from_raw(symbol_ansi.to_bytes_with_nul().as_ptr()),
        ) {
            Ok(func as *const c_void)
        } else {
            Err(anyhow!("Could not get memory address of {} in {}", symbol, module))
        }
    }
}
