#[link(name = "X11")]
extern "C" {}
use std::ffi::CString;
use std::mem::MaybeUninit;
use x11::xlib::{
    Display, Window, XBlackPixelOfScreen, XCreateSimpleWindow, XDefaultScreenOfDisplay,
    XGetWindowAttributes, XMapWindow, XOpenDisplay, XRootWindowOfScreen, XWindowAttributes,
};

pub enum Event {
    Resized { width: u32, height: u32 },
}

pub trait SizedWindow {
    fn size(&self) -> (u32, u32);
}
#[derive(Debug)]
pub struct ScreensaverWindow {
    pub dpy: *mut Display,
    pub root_window_id: Window,
}

impl ScreensaverWindow {
    pub fn new() -> Result<Self, ()> {
        unsafe {
            let xscreensaver_id_str = std::env::var("XSCREENSAVER_WINDOW")
                .ok()
                .unwrap_or_default()
                .split_whitespace()
                .next()
                .unwrap_or("")
                .trim_start_matches("0x")
                .to_string();
            let xscreensaver_id = Window::from_str_radix(&xscreensaver_id_str, 16).ok();
            let display_num = CString::new("DISPLAY").expect("can create CString");
            let dpy = XOpenDisplay(libc::getenv(display_num.as_ptr()));

            match xscreensaver_id {
                // We got our window from xscreensaver
                Some(root_window_id) => {
                    x11::xlib::XSelectInput(
                        dpy,
                        root_window_id,
                        x11::xlib::ExposureMask
                            | x11::xlib::KeyPressMask
                            | x11::xlib::StructureNotifyMask,
                    );
                    Ok(ScreensaverWindow {
                        dpy,
                        root_window_id,
                    })
                }
                // We create our own window for development
                None => {
                    let height = 800;
                    let width = 1200;
                    let screen = XDefaultScreenOfDisplay(dpy);

                    let win = XCreateSimpleWindow(
                        dpy,
                        XRootWindowOfScreen(screen),
                        0,
                        0,
                        width,
                        height,
                        10,
                        XBlackPixelOfScreen(screen),
                        XBlackPixelOfScreen(screen),
                    );

                    x11::xlib::XSelectInput(
                        dpy,
                        win,
                        x11::xlib::ExposureMask
                            | x11::xlib::KeyPressMask
                            | x11::xlib::StructureNotifyMask,
                    );
                    XMapWindow(dpy, win);
                    Ok(ScreensaverWindow {
                        dpy,
                        root_window_id: win,
                    })
                }
            }
        }
    }

    pub fn process_events(&self) -> Vec<Event> {
        let mut result: Vec<Event> = Vec::new();
        let mut cur_xevent = x11::xlib::XEvent { pad: [0; 24] };
        while unsafe { x11::xlib::XPending(self.dpy) } > 0 {
            unsafe { (x11::xlib::XNextEvent)(self.dpy, &mut cur_xevent) };
            let cur_event_type = cur_xevent.get_type();
            match cur_event_type {
                x11::xlib::ConfigureNotify => {
                    let e = x11::xlib::XConfigureEvent::from(cur_xevent);
                    result.push(Event::Resized {
                        width: e.width as u32,
                        height: e.height as u32,
                    });
                }
                x11::xlib::KeyPress => {}
                _ => {}
            }
        }
        result
    }
}

impl SizedWindow for ScreensaverWindow {
    fn size(&self) -> (u32, u32) {
        let mut attrs = MaybeUninit::<XWindowAttributes>::uninit();
        unsafe { XGetWindowAttributes(self.dpy, self.root_window_id, attrs.as_mut_ptr()) };
        let attrs = unsafe { attrs.assume_init() };
        (attrs.width as u32, attrs.height as u32)
    }
}
/// Allows wgpu to create a surface from ScreensaverWindow
unsafe impl raw_window_handle::HasRawWindowHandle for ScreensaverWindow {
    fn raw_window_handle(&self) -> raw_window_handle::RawWindowHandle {
        let mut handle = raw_window_handle::XlibHandle::empty();
        handle.display = self.dpy as *mut libc::c_void;
        handle.window = self.root_window_id;
        raw_window_handle::RawWindowHandle::Xlib(handle)
    }
}
