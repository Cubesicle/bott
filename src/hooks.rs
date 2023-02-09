use anyhow::{bail, Context, Result};
use detour::static_detour;
use std::ffi::CString;
use windows::core::{PCSTR, PCWSTR};
use windows::Win32::Graphics::Gdi::HDC;
use windows::Win32::System::LibraryLoader::{GetModuleHandleW, GetProcAddress};

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

pub fn unload() -> Result<()> {
    unsafe { WGLSwapBuffersHook.disable()?; }
    Ok(())
}

fn wgl_swap_buffers_detour(hdc: HDC) {
    // println!("buffer be swapping");
    WGLSwapBuffersHook.call(hdc);
}

fn get_module_symbol_address(module: &str, symbol: &str) -> Result<usize> {
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
            Ok(func as usize)
        } else {
            bail!(format!("Could not get memory address of {} in {}", symbol, module))
        }
    }
}
