#![allow(non_upper_case_globals)]

#[link(name = "X11")]
extern "C" {
    pub fn XOpenDisplay(display_name: *const i8) -> *mut Display;
    pub fn XCloseDisplay(display: *mut Display) -> i32;
    pub fn XCreateSimpleWindow(
        display: *mut Display,
        parent: Window,
        x: i32,
        y: i32,
        width: u32,
        height: u32,
        border_width: u32,
        border: u64,
        background: u64,
    ) -> Window;
    pub fn XDefaultRootWindow(display: *mut Display) -> Window;
    pub fn XSelectInput(display: *mut Display, window: Window, event_mask: i64) -> i32;
    pub fn XMapWindow(display: *mut Display, window: Window) -> i32;
    pub fn XPending(display: *mut Display) -> i32;
    pub fn XNextEvent(display: *mut Display, event: *mut XEvent) -> i32;
    pub fn XSetErrorHandler(handler: XErrorHandler) -> XErrorHandler;
    pub fn XSetIOErrorHandler(handler: XIOErrorHandler) -> XIOErrorHandler;
    pub fn XStoreName(display: *mut Display, window: Window, window_name: *const i8) -> i32;
    pub fn XLookupKeysym(key_event: *mut XKeyEvent, index: i32) -> KeySym;
}

pub type Bool = i32;
pub type XID = u64;
pub type CARD32 = XID;
pub type Window = XID;
pub type Time = CARD32;
pub type KeySym = XID;

#[repr(C)]
pub struct Display {
    _data: [u8; 0],
    _marker: core::marker::PhantomData<(*mut u8, core::marker::PhantomPinned)>,
}

pub type XErrorHandler = extern "C" fn(display: *mut Display, event: *mut XErrorEvent) -> i32;
pub type XIOErrorHandler = extern "C" fn(display: *mut Display) -> i32;

#[repr(C)]
pub union XEvent {
    pub ttype: i32,
    // pub xany: XAnyEvent,
    pub xkey: XKeyEvent,
    // ...
    pub xerror: XErrorEvent,
    // ...
    pub pad: [i64; 24],
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct XErrorEvent {
    pub ttype: i32,
    pub display: *mut Display,
    pub resourceid: XID,
    pub serial: u64,
    pub error_code: u8,
    pub request_code: u8,
    pub minor_code: u8,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct XKeyEvent {
    pub ttype: i32,
    pub serial: u64,
    pub send_event: Bool,
    pub display: *mut Display,
    pub window: Window,
    pub root: Window,
    pub subwindow: Window,
    pub time: Time,
    pub x: i32,
    pub y: i32,
    pub x_root: i32,
    pub y_root: i32,
    pub state: u32,
    pub keycode: u32,
    pub same_screen: Bool,
}

pub const NoEventMask: i64 = 0;
pub const KeyPressMask: i64 = 1 << 0;

pub const KeyPress: i32 = 2;
pub const KeyRelease: i32 = 3;
