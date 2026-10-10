//! Stable desktop approximation. No Explorer reparenting, hooks, or polling.
use tauri::{AppHandle, Manager, PhysicalPosition, PhysicalSize, WebviewWindow};

pub fn fit_work_area(window: &WebviewWindow) -> Result<(), String> {
    #[cfg(windows)]
    {
        use windows_sys::Win32::Graphics::Gdi::{
            GetMonitorInfoW, MonitorFromWindow, MONITORINFO, MONITOR_DEFAULTTOPRIMARY,
        };
        let hwnd = window.hwnd().map_err(|e| e.to_string())?.0;
        let mut info = MONITORINFO {
            cbSize: std::mem::size_of::<MONITORINFO>() as u32,
            ..Default::default()
        };
        // rcWork is physical pixels and excludes the taskbar on any edge. Tauri/Wry
        // remains per-monitor DPI aware; CSS uses logical pixels inside this area.
        unsafe {
            let monitor = MonitorFromWindow(hwnd, MONITOR_DEFAULTTOPRIMARY);
            if GetMonitorInfoW(monitor, &mut info) == 0 {
                return Err(std::io::Error::last_os_error().to_string());
            }
        }
        let rect = info.rcWork;
        window
            .set_position(PhysicalPosition::new(rect.left, rect.top))
            .map_err(|e| e.to_string())?;
        window
            .set_size(PhysicalSize::new(
                (rect.right - rect.left) as u32,
                (rect.bottom - rect.top) as u32,
            ))
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

fn finish_native_mode(window: &WebviewWindow, editing: bool) -> Result<(), String> {
    #[cfg(windows)]
    {
        use windows_sys::Win32::UI::WindowsAndMessaging::{
            GetWindowLongPtrW, SetLayeredWindowAttributes, SetWindowPos, ShowWindow,
            GWL_EXSTYLE, HWND_TOP, LWA_ALPHA, SW_RESTORE, SW_SHOWNOACTIVATE,
            SWP_NOMOVE, SWP_NOSIZE, WS_EX_LAYERED,
        };
        let hwnd = window.hwnd().map_err(|e| e.to_string())?.0;
        // Show Desktop can suppress a window without updating Tao's cached visible
        // flag. Explicit restoration makes a user-requested edit recover reliably.
        unsafe {
            if editing {
                ShowWindow(hwnd, SW_RESTORE);
                if SetWindowPos(hwnd, HWND_TOP, 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE) == 0 {
                    return Err(std::io::Error::last_os_error().to_string());
                }
            } else {
                // The click-through style adds WS_EX_LAYERED. Initialize its alpha;
                // an uninitialized layered HWND can disappear until its next repaint.
                if GetWindowLongPtrW(hwnd, GWL_EXSTYLE) as u32 & WS_EX_LAYERED != 0
                    && SetLayeredWindowAttributes(hwnd, 0, 255, LWA_ALPHA) == 0
                {
                    return Err(std::io::Error::last_os_error().to_string());
                }
                ShowWindow(hwnd, SW_SHOWNOACTIVATE);
            }
        }
    }
    Ok(())
}

pub fn apply(app: &AppHandle, editing: bool) -> Result<(), String> {
    let window = app
        .get_webview_window("main")
        .ok_or("Board window unavailable")?;
    if editing {
        window
            .set_always_on_bottom(false)
            .map_err(|e| e.to_string())?;
        window.set_focusable(true).map_err(|e| e.to_string())?;
        window
            .set_ignore_cursor_events(false)
            .map_err(|e| e.to_string())?;
        window.unminimize().map_err(|e| e.to_string())?;
        fit_work_area(&window)?;
        window.show().map_err(|e| e.to_string())?;
        window.set_focus().map_err(|e| e.to_string())?;
    } else {
        // Ignore input BEFORE lowering the window. No focus request occurs on this path.
        window
            .set_ignore_cursor_events(true)
            .map_err(|e| e.to_string())?;
        window.set_focusable(false).map_err(|e| e.to_string())?;
        window
            .set_always_on_bottom(true)
            .map_err(|e| e.to_string())?;
        window.show().map_err(|e| e.to_string())?;
    }
    finish_native_mode(&window, editing)
}
