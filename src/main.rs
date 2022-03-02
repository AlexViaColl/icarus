#![allow(non_upper_case_globals)]

fn main() {
    unsafe {
        let display = XOpenDisplay(std::ptr::null());
        println!("{:?}", display);
        if display.is_null() {
            eprintln!("Cannot open display");
            std::process::exit(1);
        }

        let window = XCreateSimpleWindow(
            display,
            XDefaultRootWindow(display),
            10,
            10,
            100,
            100,
            1,
            0,
            0,
        );
        println!("Window: {}", window);

        XSelectInput(display, window, KeyPressMask);
        XMapWindow(display, window);

        loop {
            let mut event = XEvent { ttype: 0 };
            XNextEvent(display, &mut event);

            match event.ttype {
                KeyPress => {
                    let event = event.xkey;
                    match event.keycode {
                        9 => break,
                        n => println!("Keycode: {}", n),
                    }
                }
                _ => {}
            }
        }

        XCloseDisplay(display);
    };
}

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
    pub fn XNextEvent(display: *mut Display, event: *mut XEvent) -> i32;
}

#[repr(C)]
pub struct Display {
    _data: [u8; 0],
    _marker: core::marker::PhantomData<(*mut u8, core::marker::PhantomPinned)>,
}

pub type Bool = i32;
pub type XID = u64;
pub type CARD32 = XID;
pub type Window = XID;
pub type Time = CARD32;

#[repr(C)]
pub union XEvent {
    pub ttype: i32,
    // pub xany: XAnyEvent,
    pub xkey: XKeyEvent,
    // ...
    pub pad: [i64; 24],
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
