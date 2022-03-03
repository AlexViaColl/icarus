#![allow(non_upper_case_globals)]
#![allow(non_snake_case)]
use icarus::*;

extern "C" fn error_handler(_display: *mut Display, _event: *mut XErrorEvent) -> i32 {
    println!("An error ocurred!");
    0
}
extern "C" fn error_io_handler(_display: *mut Display) -> i32 {
    panic!("A fatal I/O error ocurred!");
}

fn main() {
    unsafe {
        let display = XOpenDisplay(std::ptr::null());
        if display.is_null() {
            eprintln!("Cannot open display");
            std::process::exit(1);
        }

        let _orig_err_handler = XSetErrorHandler(error_handler);
        let _orig_err_io_handler = XSetIOErrorHandler(error_io_handler);

        let screen = XDefaultScreen(display);
        let root = XRootWindow(display, screen);
        let window = XCreateSimpleWindow(display, root, 0, 0, 100, 100, 1, 0, 0xFFFFFFFF);

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
                        let gc = XDefaultGC(display, screen);
                        XFillRectangle(display, window, gc, 20, 20, 10, 10);
                    }
                    _ => {}
                }
            }
        }

        XCloseDisplay(display);
    };
}
