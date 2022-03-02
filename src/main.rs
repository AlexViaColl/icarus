#![allow(non_upper_case_globals)]
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

        let root = XDefaultRootWindow(display);
        let window = XCreateSimpleWindow(display, root, 0, 0, 100, 100, 1, 0, 0);

        assert_ne!(
            XStoreName(display, window, b"Icarus\0".as_ptr() as *const i8),
            0
        );
        assert_ne!(XSelectInput(display, window, KeyPressMask), 0);
        assert_ne!(XMapWindow(display, window), 0);

        let mut running = true;
        while running {
            while XPending(display) > 0 {
                let mut event = XEvent { ttype: 0 };
                assert_ne!(XNextEvent(display, &mut event), 0);
                match event.ttype {
                    KeyPress => {
                        let event = event.xkey;
                        match event.keycode {
                            9 => running = false,
                            n => println!("Keycode: {}", n),
                        }
                    }
                    _ => {}
                }
            }
        }

        assert_ne!(XCloseDisplay(display), 0);
    };
}
