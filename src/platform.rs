use crate::input::{InputState, KeyId};
use crate::x11::*;

use std::ptr;

pub struct Config {
    pub width: u32,
    pub height: u32,
    pub app_name: String,
}

pub struct Platform {
    pub dpy: *mut Display,
    pub window: Window,

    pub window_width: u32,
    pub window_height: u32,
}

impl Platform {
    pub fn init(config: Config) -> Self {
        unsafe {
            XInitThreads();
            let dpy = XOpenDisplay(ptr::null());
            assert!(!dpy.is_null());

            let screen = XDefaultScreen(dpy);
            let root = XRootWindow(dpy, screen);
            let window_width = config.width;
            let window_height = config.height;
            let window =
                XCreateSimpleWindow(dpy, root, 0, 0, window_width, window_height, 1, 0, /*BG_COLOR*/ 0 as u64);

            let mut app_name = config.app_name;
            app_name.push(0 as char);
            assert_ne!(XStoreName(dpy, window, app_name.as_ptr() as *const i8), 0);
            let mask = KeyPressMask | KeyReleaseMask | ExposureMask | StructureNotifyMask;
            assert_ne!(XSelectInput(dpy, window, mask), 0);
            assert_ne!(
                XSetClassHint(
                    dpy,
                    window,
                    &mut XClassHint {
                        res_name: app_name.as_ptr() as *mut i8,
                        res_class: app_name.as_ptr() as *mut i8,
                    }
                ),
                0
            );
            assert_ne!(XMapWindow(dpy, window), 0);
            Self {
                dpy,
                window,
                window_width,
                window_height,
            }
        }
    }

    pub fn process_messages(&mut self, input: &mut InputState) {
        unsafe {
            while XPending(self.dpy) > 0 {
                let mut event = XEvent::default();
                XNextEvent(self.dpy, &mut event);
                match event.ttype {
                    KeyPress | KeyRelease => {
                        #[allow(unused_variables)]
                        let keysym = XLookupKeysym(&mut event.xkey, 0);
                        let event = event.xkey;
                        //println!("KeySym: 0x{:04x} / KeyCode: 0x{:04x}", keysym, event.keycode);

                        let is_down = event.ttype == KeyPress;
                        input.set_key(KeyId::Any, is_down);
                        match keysym {
                            XK_Escape => input.set_key(KeyId::Esc, is_down),
                            XK_a => input.set_key(KeyId::A, is_down),
                            XK_d => input.set_key(KeyId::D, is_down),
                            XK_p => input.set_key(KeyId::P, is_down),
                            XK_s => input.set_key(KeyId::S, is_down),
                            XK_w => input.set_key(KeyId::W, is_down),
                            XK_Down => input.set_key(KeyId::Down, is_down),
                            XK_Up => input.set_key(KeyId::Up, is_down),
                            XK_Left => input.set_key(KeyId::Left, is_down),
                            XK_Right => input.set_key(KeyId::Right, is_down),
                            _n => {} // println!("Keycode: {}", n),
                        }
                    }
                    ConfigureNotify => {
                        let event = event.xconfigure;
                        if event.width as u32 != self.window_width || event.height as u32 != self.window_height {
                            self.window_width = event.width as u32;
                            self.window_height = event.height as u32;
                            // println!("ConfigureNotify ({}, {})", window_width, window_height);
                            //recreate_swapchain(&mut vk_ctx);
                        }
                    }
                    _ => {}
                }
            }
        }
    }
}
