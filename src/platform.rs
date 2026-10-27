//! Platform glue the windowing library does not cover: the global mouse
//! position (needed to drag docked borderless windows), window shapes from
//! region.txt, and keeping the secondary windows out of the taskbar.

use winit::window::Window;

/// Global cursor position in screen pixels, if the platform can tell.
pub fn global_cursor() -> Option<(i32, i32)> {
    imp::global_cursor()
}

/// Restrict the window to the visible pixels of `mask` (row-major,
/// `w`x`h` skin pixels, each drawn `scale` times larger). `None` removes the shape.
pub fn set_shape(window: &Window, mask: Option<(&[bool], i32, i32, i32)>) {
    imp::set_shape(window, mask)
}

/// Mark a window as an auxiliary window of `owner`: no taskbar entry and
/// stacked with its owner.
pub fn make_auxiliary(window: &Window, owner: &Window) {
    imp::make_auxiliary(window, owner)
}

/// Make a pop-up (menu, tooltip) that never takes focus from the player.
pub fn make_popup(window: &Window) {
    imp::make_popup(window)
}

/// Horizontal runs of visible pixels, scaled: (x, y, w, h).
pub fn mask_runs(mask: &[bool], w: i32, h: i32, scale: i32) -> Vec<(i32, i32, i32, i32)> {
    let mut out = Vec::new();
    for y in 0..h {
        let mut x = 0;
        while x < w {
            if mask[(y * w + x) as usize] {
                let start = x;
                while x < w && mask[(y * w + x) as usize] {
                    x += 1;
                }
                out.push((start * scale, y * scale, (x - start) * scale, scale));
            } else {
                x += 1;
            }
        }
    }
    out
}

#[cfg(all(unix, not(target_os = "macos")))]
mod imp {
    use std::cell::RefCell;

    use winit::raw_window_handle::{HasWindowHandle, RawWindowHandle};
    use winit::window::Window;
    use x11rb::connection::Connection;
    use x11rb::protocol::shape::{self, ConnectionExt as _};
    use x11rb::protocol::xproto::{
        AtomEnum, ClipOrdering, ConnectionExt as _, PropMode, Rectangle,
    };
    use x11rb::rust_connection::RustConnection;
    use x11rb::wrapper::ConnectionExt as _;

    thread_local! {
        static CONN: RefCell<Option<Option<(RustConnection, u32)>>> = const { RefCell::new(None) };
    }

    fn with_conn<T>(f: impl FnOnce(&RustConnection, u32) -> Option<T>) -> Option<T> {
        CONN.with(|c| {
            let mut c = c.borrow_mut();
            if c.is_none() {
                *c = Some(x11rb::connect(None).ok().map(|(conn, screen)| {
                    let root = conn.setup().roots[screen].root;
                    (conn, root)
                }));
            }
            match c.as_ref().unwrap() {
                Some((conn, root)) => f(conn, *root),
                None => None,
            }
        })
    }

    fn xid(window: &Window) -> Option<u32> {
        match window.window_handle().ok()?.as_raw() {
            RawWindowHandle::Xlib(h) => Some(h.window as u32),
            RawWindowHandle::Xcb(h) => Some(h.window.get()),
            _ => None,
        }
    }

    pub fn global_cursor() -> Option<(i32, i32)> {
        std::env::var_os("DISPLAY")?;
        with_conn(|conn, root| {
            let r = conn.query_pointer(root).ok()?.reply().ok()?;
            Some((r.root_x as i32, r.root_y as i32))
        })
    }

    pub fn set_shape(window: &Window, mask: Option<(&[bool], i32, i32, i32)>) {
        let Some(id) = xid(window) else { return };
        with_conn(|conn, _| {
            match mask {
                Some((m, w, h, scale)) => {
                    let rects: Vec<Rectangle> = super::mask_runs(m, w, h, scale)
                        .into_iter()
                        .map(|(x, y, w, h)| Rectangle {
                            x: x as i16,
                            y: y as i16,
                            width: w as u16,
                            height: h as u16,
                        })
                        .collect();
                    conn.shape_rectangles(
                        shape::SO::SET,
                        shape::SK::BOUNDING,
                        ClipOrdering::UNSORTED,
                        id,
                        0,
                        0,
                        &rects,
                    )
                    .ok()?;
                }
                None => {
                    conn.shape_mask(shape::SO::SET, shape::SK::BOUNDING, id, 0, 0, x11rb::NONE)
                        .ok()?;
                }
            }
            conn.flush().ok()
        });
    }

    pub fn make_popup(_window: &Window) {
        // Override-redirect windows never receive focus.
    }

    pub fn make_auxiliary(window: &Window, owner: &Window) {
        let (Some(id), Some(owner_id)) = (xid(window), xid(owner)) else {
            return;
        };
        with_conn(|conn, _| {
            let atom = |name: &[u8]| {
                conn.intern_atom(false, name)
                    .ok()?
                    .reply()
                    .ok()
                    .map(|r| r.atom)
            };
            let state = atom(b"_NET_WM_STATE")?;
            let skip_taskbar = atom(b"_NET_WM_STATE_SKIP_TASKBAR")?;
            let skip_pager = atom(b"_NET_WM_STATE_SKIP_PAGER")?;
            conn.change_property32(
                PropMode::REPLACE,
                id,
                state,
                AtomEnum::ATOM,
                &[skip_taskbar, skip_pager],
            )
            .ok()?;
            conn.change_property32(
                PropMode::REPLACE,
                id,
                AtomEnum::WM_TRANSIENT_FOR,
                AtomEnum::WINDOW,
                &[owner_id],
            )
            .ok()?;
            conn.flush().ok()
        });
    }
}

#[cfg(windows)]
mod imp {
    use windows_sys::Win32::Foundation::POINT;
    use windows_sys::Win32::Graphics::Gdi::{
        CombineRgn, CreateRectRgn, DeleteObject, RGN_OR, SetWindowRgn,
    };
    use windows_sys::Win32::UI::WindowsAndMessaging::GetCursorPos;
    use winit::raw_window_handle::{HasWindowHandle, RawWindowHandle};
    use winit::window::Window;

    pub fn global_cursor() -> Option<(i32, i32)> {
        let mut p = POINT { x: 0, y: 0 };
        // SAFETY: GetCursorPos writes into the provided POINT.
        if unsafe { GetCursorPos(&mut p) } != 0 {
            Some((p.x, p.y))
        } else {
            None
        }
    }

    fn hwnd(window: &Window) -> Option<isize> {
        match window.window_handle().ok()?.as_raw() {
            RawWindowHandle::Win32(h) => Some(h.hwnd.get()),
            _ => None,
        }
    }

    pub fn set_shape(window: &Window, mask: Option<(&[bool], i32, i32, i32)>) {
        let Some(h) = hwnd(window) else { return };
        // SAFETY: plain GDI region calls on a window handle owned by this process.
        unsafe {
            match mask {
                Some((m, w, hh, scale)) => {
                    let rgn = CreateRectRgn(0, 0, 0, 0);
                    for (x, y, rw, rh) in super::mask_runs(m, w, hh, scale) {
                        let r = CreateRectRgn(x, y, x + rw, y + rh);
                        CombineRgn(rgn, rgn, r, RGN_OR);
                        DeleteObject(r as _);
                    }
                    SetWindowRgn(h as _, rgn, 1);
                }
                None => {
                    SetWindowRgn(h as _, std::ptr::null_mut(), 1);
                }
            }
        }
    }

    pub fn make_auxiliary(_window: &Window, _owner: &Window) {
        // Done at creation time with the owner-window attribute.
    }

    pub fn make_popup(window: &Window) {
        use windows_sys::Win32::UI::WindowsAndMessaging::{
            GWL_EXSTYLE, GetWindowLongPtrW, SetWindowLongPtrW, WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW,
        };
        let Some(h) = hwnd(window) else { return };
        // SAFETY: style change on a window owned by this process.
        unsafe {
            let ex = GetWindowLongPtrW(h as _, GWL_EXSTYLE);
            SetWindowLongPtrW(
                h as _,
                GWL_EXSTYLE,
                ex | (WS_EX_NOACTIVATE | WS_EX_TOOLWINDOW) as isize,
            );
        }
    }
}

#[cfg(target_os = "macos")]
mod imp {
    use winit::window::Window;

    pub fn global_cursor() -> Option<(i32, i32)> {
        use objc2_app_kit::NSEvent;
        let p = NSEvent::mouseLocation();
        // AppKit's origin is the bottom-left of the primary screen.
        let screen_h = primary_screen_height()?;
        Some((p.x.round() as i32, (screen_h - p.y).round() as i32))
    }

    fn primary_screen_height() -> Option<f64> {
        let mtm = objc2_foundation::MainThreadMarker::new()?;
        let screens = objc2_app_kit::NSScreen::screens(mtm);
        let first = screens.firstObject()?;
        Some(first.frame().size.height)
    }

    pub fn set_shape(_window: &Window, _mask: Option<(&[bool], i32, i32, i32)>) {}

    pub fn make_auxiliary(_window: &Window, _owner: &Window) {}

    pub fn make_popup(_window: &Window) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn runs() {
        let m = [true, true, false, true, false, false];
        assert_eq!(mask_runs(&m, 3, 2, 2), vec![(0, 0, 4, 2), (0, 2, 2, 2)]);
    }
}
