use crate::xcb_sys::*;

use std::ffi::c_void;
use std::ptr;

pub struct Xcb {
    pub connection: *mut xcb_connection_t,
    pub screen: *mut xcb_screen_t,
    pub window: xcb_window_t,
    pub atom_wm_protocols: xcb_intern_atom_reply_t,
    pub atom_wm_delete_window: xcb_intern_atom_reply_t,
}

pub struct XcbConfig {
    pub width: u32,
    pub height: u32,
    pub show: bool,
    pub title: String,
    pub class: String,
}
impl Default for XcbConfig {
    fn default() -> Self {
        Self {
            width: 1280,
            height: 720,
            show: true,
            //fullscreen: false,
            title: String::from("Vulkan Example"),
            class: String::from("vulkanExample"),
        }
    }
}

impl Xcb {
    pub fn init() -> Self {
        Self::with_config(XcbConfig::default())
    }
    pub fn with_config(config: XcbConfig) -> Self {
        unsafe {
            let mut scr = 0;
            let connection = xcb_connect(ptr::null(), &mut scr);
            assert!(!connection.is_null());
            if xcb_connection_has_error(connection) != 0 {
                println!("Could not establish connection to the X Server!");
                std::process::exit(1);
            }

            let setup = xcb_get_setup(connection);
            //println!("{}", cstr_to_string(xcb_setup_vendor(setup)));
            let mut iter = xcb_setup_roots_iterator(setup);
            while scr > 0 {
                scr -= 1;
                xcb_screen_next(&mut iter);
            }
            let screen = iter.data;

            let window = xcb_generate_id(connection);
            let value_mask = XCB_CW_BACK_PIXEL | XCB_CW_EVENT_MASK;
            let value_list = [
                (*screen).black_pixel,
                XCB_EVENT_MASK_KEY_RELEASE
                    | XCB_EVENT_MASK_KEY_PRESS
                    | XCB_EVENT_MASK_EXPOSURE
                    | XCB_EVENT_MASK_STRUCTURE_NOTIFY
                    | XCB_EVENT_MASK_POINTER_MOTION
                    | XCB_EVENT_MASK_BUTTON_PRESS
                    | XCB_EVENT_MASK_BUTTON_RELEASE,
            ];

            xcb_create_window(
                connection,
                XCB_COPY_FROM_PARENT,
                window,
                (*screen).root,
                0,
                0,
                config.width as u16,
                config.height as u16,
                0,
                XCB_WINDOW_CLASS_INPUT_OUTPUT,
                (*screen).root_visual,
                value_mask,
                value_list.as_ptr() as *const c_void,
            );

            // Get atoms
            let atom_wm_protocols = *xcb_intern_atom_reply(
                connection,
                xcb_intern_atom(connection, 1, "WM_PROTOCOLS".len() as u16, b"WM_PROTOCOLS\0".as_ptr() as *const i8),
                ptr::null_mut(),
            );
            let atom_wm_delete_window = *xcb_intern_atom_reply(
                connection,
                xcb_intern_atom(
                    connection,
                    0,
                    "WM_DELETE_WINDOW".len() as u16,
                    b"WM_DELETE_WINDOW\0".as_ptr() as *const i8,
                ),
                ptr::null_mut(),
            );

            // Change window name
            xcb_change_property(
                connection,
                XCB_PROP_MODE_REPLACE,
                window,
                XCB_ATOM_WM_NAME,
                XCB_ATOM_STRING,
                8,
                config.title.len() as u32,
                config.title.as_ptr() as *const c_void,
            );

            // Change window class
            let wm_class = format!("{}\0{}\0", config.class, config.title);
            xcb_change_property(
                connection,
                XCB_PROP_MODE_REPLACE,
                window,
                XCB_ATOM_WM_CLASS,
                XCB_ATOM_STRING,
                8,
                wm_class.len() as u32,
                wm_class.as_ptr() as *const c_void,
            );

            // Change WM_PROTOCOLS
            xcb_change_property(
                connection,
                XCB_PROP_MODE_REPLACE,
                window,
                atom_wm_protocols.atom,
                4,
                32,
                1,
                &atom_wm_delete_window.atom as *const _ as *const c_void,
            );

            if config.show {
                xcb_map_window(connection, window);
            }
            xcb_flush(connection);

            Self {
                connection,
                screen,
                window,
                atom_wm_protocols,
                atom_wm_delete_window,
            }
        }
    }
    pub fn next_event(&self) -> Option<*mut xcb_generic_event_t> {
        let event = unsafe { xcb_poll_for_event(self.connection) };
        if event.is_null() {
            None
        } else {
            Some(event)
        }
    }
}
