#[macro_export]
macro_rules! errbox {
    ($s:expr) => {
        let msg_utf16: Vec<u16> = format!("{}", $s)
            .encode_utf16()
            .chain(std::iter::once(0))
            .collect::<Vec<u16>>();
        unsafe {
            windows::Win32::UI::WindowsAndMessaging::MessageBoxW(
                None,
                windows::core::PCWSTR::from_raw(msg_utf16.as_ptr()),
                windows::core::w!("Error"),
                windows::Win32::UI::WindowsAndMessaging::MB_ICONERROR,
            )
        };
    };
}
