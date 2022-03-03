#![allow(non_upper_case_globals)]
#![allow(non_snake_case)]
use icarus::*;

use std::process;
use std::ptr;

const BG_COLOR: u64 = 0x00000000;

extern "C" fn error_handler(_display: *mut Display, _event: *mut XErrorEvent) -> i32 {
    println!("An error ocurred!");
    0
}
extern "C" fn error_io_handler(_display: *mut Display) -> i32 {
    panic!("A fatal I/O error ocurred!");
}

fn main() {
    unsafe {
        let mut extension_count = 0;
        vkEnumerateInstanceExtensionProperties(ptr::null(), &mut extension_count, ptr::null_mut());
        println!("extension count: {}", extension_count);

        let mut instance = ptr::null_mut();
        let result = vkCreateInstance(
            &VkInstanceCreateInfo {
                sType: VK_STRUCTURE_TYPE_INSTANCE_CREATE_INFO,
                pNext: ptr::null(),
                flags: 0,
                pApplicationInfo: &VkApplicationInfo {
                    sType: VK_STRUCTURE_TYPE_APPLICATION_INFO,
                    pNext: ptr::null(),
                    pApplicationName: b"Hello Triangle\0".as_ptr() as *const i8,
                    applicationVersion: 0,
                    pEngineName: b"No Engine\0".as_ptr() as *const i8,
                    engineVersion: 0,
                    apiVersion: 0,
                },
                enabledLayerCount: 0,
                ppEnabledLayerNames: ptr::null(),
                enabledExtensionCount: 0,
                ppEnabledExtensionNames: ptr::null(),
            },
            ptr::null(),
            &mut instance,
        );
        println!("vkCreateInfo result: {:?}", result);
        process::exit(1);

        let display = XOpenDisplay(std::ptr::null());
        if display.is_null() {
            eprintln!("Cannot open display");
            process::exit(1);
        }

        let _orig_err_handler = XSetErrorHandler(error_handler);
        let _orig_err_io_handler = XSetIOErrorHandler(error_io_handler);

        let screen = XDefaultScreen(display);
        let root = XRootWindow(display, screen);
        let window = XCreateSimpleWindow(display, root, 0, 0, 100, 100, 1, 0, BG_COLOR);

        assert_ne!(
            XStoreName(display, window, b"Icarus\0".as_ptr() as *const i8),
            0
        );
        assert_ne!(
            XSelectInput(display, window, KeyPressMask | ExposureMask),
            0
        );
        assert_ne!(XMapWindow(display, window), 0);

        let mut running = true;
        while running {
            while XPending(display) > 0 {
                let mut event = XEvent { pad: [0; 24] };
                XNextEvent(display, &mut event);
                match event.ttype {
                    KeyPress => {
                        let keysym = XLookupKeysym(&mut event.xkey, 0);
                        let event = event.xkey;
                        println!("KeySym: {} / KeyCode: {}", keysym, event.keycode);
                        match event.keycode {
                            9 => running = false,
                            n => println!("Keycode: {}", n),
                        }
                    }
                    Expose => {
                        // let gc = XDefaultGC(display, screen);
                        // XFillRectangle(display, window, gc, 20, 20, 10, 10);
                    }
                    _ => {}
                }
            }
        }

        XCloseDisplay(display);
    };
}
