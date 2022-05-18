use crate::input::{ButtonId, InputState, KeyId};
use crate::x11_sys as x11;

use std::ptr;

pub struct Config {
    pub width: u32,
    pub height: u32,
    pub app_name: String,
}

pub struct Platform {
    pub dpy: *mut x11::Display,
    pub window: x11::Window,

    pub window_width: u32,
    pub window_height: u32,
}

impl Platform {
    pub fn init(config: Config) -> Self {
        unsafe {
            x11::XInitThreads();
            let dpy = x11::XOpenDisplay(ptr::null());
            assert!(!dpy.is_null());

            let screen = x11::XDefaultScreen(dpy);
            let root = x11::XRootWindow(dpy, screen);
            let window_width = config.width;
            let window_height = config.height;
            let window =
                x11::XCreateSimpleWindow(dpy, root, 0, 0, window_width, window_height, 1, 0, /*BG_COLOR*/ 0);

            let mut app_name = config.app_name;
            app_name.push(0 as char);
            assert_ne!(x11::XStoreName(dpy, window, app_name.as_ptr() as *const i8), 0);
            let mask = x11::KeyPressMask
                | x11::KeyReleaseMask
                | x11::ButtonPressMask
                | x11::ButtonReleaseMask
                | x11::PointerMotionMask
                | x11::ExposureMask
                | x11::StructureNotifyMask;
            assert_ne!(x11::XSelectInput(dpy, window, mask), 0);
            assert_ne!(
                x11::XSetClassHint(
                    dpy,
                    window,
                    &mut x11::XClassHint {
                        res_name: app_name.as_ptr() as *mut i8,
                        res_class: b"Icarus\0".as_ptr() as *mut i8,
                    }
                ),
                0
            );
            assert_ne!(x11::XMapWindow(dpy, window), 0);
            Self {
                dpy,
                window,
                window_width,
                window_height,
            }
        }
    }

    pub fn process_messages(&mut self, input: &mut InputState) {
        input.reset_transitions();
        unsafe {
            while x11::XPending(self.dpy) > 0 {
                let mut event = x11::XEvent::default();
                x11::XNextEvent(self.dpy, &mut event);
                match event.ttype {
                    x11::MotionNotify => {
                        let event = event.xmotion;
                        input.set_mouse_pos(event.x as f32, event.y as f32);
                    }
                    x11::KeyPress | x11::KeyRelease => {
                        #[allow(unused_variables)]
                        let keysym = x11::XLookupKeysym(&mut event.xkey, 0);
                        let event = event.xkey;
                        //println!("KeySym: 0x{:04x} / KeyCode: 0x{:04x}", keysym, event.keycode);

                        let is_down = event.ttype == x11::KeyPress;
                        input.set_key(KeyId::Any, is_down);
                        match keysym {
                            x11::XK_Escape => input.set_key(KeyId::Esc, is_down),
                            x11::XK_Return => input.set_key(KeyId::Enter, is_down),
                            x11::XK_space => input.set_key(KeyId::Space, is_down),
                            x11::XK_a => input.set_key(KeyId::A, is_down),
                            x11::XK_d => input.set_key(KeyId::D, is_down),
                            x11::XK_m => input.set_key(KeyId::M, is_down),
                            x11::XK_p => input.set_key(KeyId::P, is_down),
                            x11::XK_r => input.set_key(KeyId::R, is_down),
                            x11::XK_s => input.set_key(KeyId::S, is_down),
                            x11::XK_w => input.set_key(KeyId::W, is_down),
                            x11::XK_Down => input.set_key(KeyId::Down, is_down),
                            x11::XK_Up => input.set_key(KeyId::Up, is_down),
                            x11::XK_Left => input.set_key(KeyId::Left, is_down),
                            x11::XK_Right => input.set_key(KeyId::Right, is_down),
                            _n => {} // println!("Keycode: {}", n),
                        }
                    }
                    x11::ButtonPress | x11::ButtonRelease => {
                        let event = event.xbutton;
                        let is_down = event.ttype == x11::ButtonPress;
                        match event.button {
                            x11::Button1 => input.set_button(ButtonId::Left, is_down, event.x, event.y),
                            x11::Button3 => input.set_button(ButtonId::Right, is_down, event.x, event.y),
                            x11::Button2 => input.set_button(ButtonId::Middle, is_down, event.x, event.y),
                            _ => {}
                        }
                        //println!("{:?}", event);
                    }
                    x11::ConfigureNotify => {
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
