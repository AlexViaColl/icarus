#![allow(non_camel_case_types)]

use crate::opaque;

use std::ffi::c_void;

#[link(name = "xcb")]
extern "C" {
    pub fn xcb_connect(displayname: *const i8, screenp: *mut i32) -> *mut xcb_connection_t;
    pub fn xcb_disconnect(c: *mut xcb_connection_t);
    pub fn xcb_connection_has_error(c: *mut xcb_connection_t) -> i32;
    pub fn xcb_get_setup(c: *mut xcb_connection_t) -> *const xcb_setup_t;
    pub fn xcb_setup_roots_iterator(r: *const xcb_setup_t) -> xcb_screen_iterator_t;
    pub fn xcb_screen_next(i: *mut xcb_screen_iterator_t);
    pub fn xcb_generate_id(c: *mut xcb_connection_t) -> u32;
    pub fn xcb_create_window(
        c: *mut xcb_connection_t,
        depth: u8,
        wid: xcb_window_t,
        parent: xcb_window_t,
        x: i16,
        y: i16,
        width: u16,
        height: u16,
        border_width: u16,
        class: u16,
        visual: xcb_visualid_t,
        value_mask: u32,
        value_list: *const c_void,
    ) -> xcb_void_cookie_t;
    pub fn xcb_destroy_window(c: *mut xcb_connection_t, window: xcb_window_t) -> xcb_void_cookie_t;
    pub fn xcb_map_window(c: *mut xcb_connection_t, window: xcb_window_t) -> xcb_void_cookie_t;
    pub fn xcb_intern_atom(
        c: *mut xcb_connection_t,
        only_if_exists: u8,
        name_len: u16,
        name: *const i8,
    ) -> xcb_intern_atom_cookie_t;
    pub fn xcb_intern_atom_reply(
        c: *mut xcb_connection_t,
        cookie: xcb_intern_atom_cookie_t,
        e: *mut *mut xcb_generic_error_t,
    ) -> *mut xcb_intern_atom_reply_t;
    pub fn xcb_change_property(
        c: *mut xcb_connection_t,
        mode: u8,
        window: xcb_window_t,
        property: xcb_atom_t,
        ttype: xcb_atom_t,
        format: u8,
        data_len: u32,
        data: *const c_void,
    ) -> xcb_void_cookie_t;
    pub fn xcb_flush(c: *mut xcb_connection_t) -> i32;
    pub fn xcb_poll_for_event(c: *mut xcb_connection_t) -> *mut xcb_generic_event_t;
    pub fn xcb_get_keyboard_mapping(
        c: *mut xcb_connection_t,
        first_keycode: xcb_keycode_t,
        count: u8,
    ) -> xcb_get_keyboard_mapping_cookie_t;
    pub fn xcb_get_keyboard_mapping_reply(
        c: *mut xcb_connection_t,
        cookie: xcb_get_keyboard_mapping_cookie_t,
        e: *mut *mut xcb_generic_error_t,
    ) -> *mut xcb_get_keyboard_mapping_reply_t;

}

#[link(name = "xcb-keysyms")]
extern "C" {
    // xcb_keysyms.h
    pub fn xcb_key_symbols_alloc(c: *mut xcb_connection_t) -> *mut xcb_key_symbols_t;
    pub fn xcb_key_symbols_free(syms: *mut xcb_key_symbols_t);
    pub fn xcb_key_symbols_get_keysym(syms: *mut xcb_key_symbols_t, keycode: xcb_keycode_t, col: i32) -> xcb_keysym_t;
}

opaque!(xcb_connection_t, xcb_connection_t_);

#[repr(C)]
#[derive(Debug)]
pub struct xcb_setup_t {
    pub status: u8,
    pub pad0: u8,
    pub protocol_major_version: u16,
    pub protocol_minor_version: u16,
    pub length: u16,
    pub release_number: u32,
    pub resource_id_base: u32,
    pub resource_id_mask: u32,
    pub motion_buffer_size: u32,
    pub vendor_len: u16,
    pub maximum_request_length: u16,
    pub roots_len: u8,
    pub pixmap_formats_len: u8,
    pub image_byte_order: u8,
    pub bitmap_format_bit_order: u8,
    pub bitmap_format_scanline_unit: u8,
    pub bitmap_format_scanline_pad: u8,
    pub min_keycode: xcb_keycode_t,
    pub max_keycode: xcb_keycode_t,
    pub pad1: [u8; 4],
}
pub type xcb_keycode_t = u8;

#[repr(C)]
pub struct xcb_screen_iterator_t {
    pub data: *mut xcb_screen_t,
    pub rem: i32,
    pub index: i32,
}
#[repr(C)]
#[derive(Debug)]
pub struct xcb_screen_t {
    pub root: xcb_window_t,
    pub default_colormap: xcb_colormap_t,
    pub white_pixel: u32,
    pub black_pixel: u32,
    pub current_input_masks: u32,
    pub width_in_pixels: u16,
    pub height_in_pixels: u16,
    pub width_in_millimeters: u16,
    pub height_in_millimeters: u16,
    pub min_installed_maps: u16,
    pub max_installed_maps: u16,
    pub root_visual: xcb_visualid_t,
    pub backing_stores: u8,
    pub save_unders: u8,
    pub root_depth: u8,
    pub allowed_depths_len: u8,
}
pub type xcb_window_t = u32;
pub type xcb_colormap_t = u32;
pub type xcb_visualid_t = u32;
pub type xcb_atom_t = u32;
pub type xcb_timestamp_t = u32;
pub type xcb_keysym_t = u32;

#[repr(C)]
pub struct xcb_void_cookie_t {
    pub sequence: u32,
}
#[repr(C)]
pub struct xcb_intern_atom_cookie_t {
    pub sequence: u32,
}
#[repr(C)]
pub struct xcb_intern_atom_reply_t {
    pub response_type: u8,
    pub pad0: u8,
    pub sequence: u16,
    pub length: u32,
    pub atom: xcb_atom_t,
}
#[repr(C)]
pub struct xcb_get_keyboard_mapping_cookie_t {
    pub sequence: u32,
}
#[repr(C)]
#[derive(Debug)]
pub struct xcb_get_keyboard_mapping_reply_t {
    pub response_type: u8,
    pub keysyms_per_keycode: u8,
    sequence: u16,
    length: u32,
    pad0: [u8; 24],
}
#[repr(C)]
pub struct xcb_generic_error_t {
    pub response_type: u8,
    pub error_code: u8,
    pub sequence: u16,
    pub resource_id: u32,
    pub minor_code: u16,
    pub major_code: u8,
    pub pad0: u8,
    pub pad: [u32; 5],
    pub full_sequence: u32,
}
#[repr(C)]
pub struct xcb_generic_event_t {
    pub response_type: u8,
    pub pad0: u8,
    pub sequence: u16,
    pub pad: [u32; 7],
    pub full_sequence: u32,
}
#[repr(C)]
#[derive(Copy, Clone)]
pub struct xcb_client_message_event_t {
    pub response_type: u8,
    pub format: u8,
    pub sequence: u16,
    pub window: xcb_window_t,
    pub ttype: xcb_atom_t,
    pub data: xcb_client_message_data_t,
}
#[repr(C)]
#[derive(Copy, Clone)]
pub union xcb_client_message_data_t {
    pub data8: [u8; 20],
    pub data16: [u16; 10],
    pub data32: [u32; 5],
}
#[repr(C)]
pub struct xcb_key_press_event_t {
    pub response_type: u8,
    pub detail: xcb_keycode_t,
    pub sequence: u16,
    pub time: xcb_timestamp_t,
    pub root: xcb_window_t,
    pub event: xcb_window_t,
    pub child: xcb_window_t,
    pub root_x: i16,
    pub root_y: i16,
    pub event_x: i16,
    pub event_y: i16,
    pub state: u16,
    pub same_screen: u8,
    pub pad0: u8,
}

opaque!(xcb_key_symbols_t, _XCBKeySymbols);

pub const XCB_EVENT_MASK_NO_EVENT: u32 = 0;
pub const XCB_EVENT_MASK_KEY_PRESS: u32 = 1;
pub const XCB_EVENT_MASK_KEY_RELEASE: u32 = 2;
pub const XCB_EVENT_MASK_BUTTON_PRESS: u32 = 4;
pub const XCB_EVENT_MASK_BUTTON_RELEASE: u32 = 8;
pub const XCB_EVENT_MASK_ENTER_WINDOW: u32 = 16;
pub const XCB_EVENT_MASK_LEAVE_WINDOW: u32 = 32;
pub const XCB_EVENT_MASK_POINTER_MOTION: u32 = 64;
pub const XCB_EVENT_MASK_POINTER_MOTION_HINT: u32 = 128;
pub const XCB_EVENT_MASK_BUTTON_1_MOTION: u32 = 256;
pub const XCB_EVENT_MASK_BUTTON_2_MOTION: u32 = 512;
pub const XCB_EVENT_MASK_BUTTON_3_MOTION: u32 = 1024;
pub const XCB_EVENT_MASK_BUTTON_4_MOTION: u32 = 2048;
pub const XCB_EVENT_MASK_BUTTON_5_MOTION: u32 = 4096;
pub const XCB_EVENT_MASK_BUTTON_MOTION: u32 = 8192;
pub const XCB_EVENT_MASK_KEYMAP_STATE: u32 = 16384;
pub const XCB_EVENT_MASK_EXPOSURE: u32 = 32768;
pub const XCB_EVENT_MASK_VISIBILITY_CHANGE: u32 = 65536;
pub const XCB_EVENT_MASK_STRUCTURE_NOTIFY: u32 = 131072;
pub const XCB_EVENT_MASK_RESIZE_REDIRECT: u32 = 262144;
pub const XCB_EVENT_MASK_SUBSTRUCTURE_NOTIFY: u32 = 524288;
pub const XCB_EVENT_MASK_SUBSTRUCTURE_REDIRECT: u32 = 1048576;
pub const XCB_EVENT_MASK_FOCUS_CHANGE: u32 = 2097152;
pub const XCB_EVENT_MASK_PROPERTY_CHANGE: u32 = 4194304;
pub const XCB_EVENT_MASK_COLOR_MAP_CHANGE: u32 = 8388608;
pub const XCB_EVENT_MASK_OWNER_GRAB_BUTTON: u32 = 16777216;

pub const XCB_KEY_PRESS: u8 = 2;
pub const XCB_KEY_RELEASE: u8 = 3;
pub const XCB_BUTTON_PRESS: u8 = 4;
pub const XCB_BUTTON_RELEASE: u8 = 5;
pub const XCB_MOTION_NOTIFY: u8 = 6;
pub const XCB_ENTER_NOTIFY: u8 = 7;
pub const XCB_LEAVE_NOTIFY: u8 = 8;
pub const XCB_FOCUS_IN: u8 = 9;
pub const XCB_FOCUS_OUT: u8 = 10;
pub const XCB_KEYMAP_NOTIFY: u8 = 11;
pub const XCB_EXPOSE: u8 = 12;
pub const XCB_GRAPHICS_EXPOSURE: u8 = 13;
pub const XCB_NO_EXPOSURE: u8 = 14;
pub const XCB_VISIBILITY_NOTIFY: u8 = 15;
pub const XCB_CREATE_NOTIFY: u8 = 16;
pub const XCB_DESTROY_NOTIFY: u8 = 17;
pub const XCB_UNMAP_NOTIFY: u8 = 18;
pub const XCB_MAP_NOTIFY: u8 = 19;
pub const XCB_MAP_REQUEST: u8 = 20;
pub const XCB_REPARENT_NOTIFY: u8 = 21;
pub const XCB_CONFIGURE_NOTIFY: u8 = 22;
pub const XCB_CONFIGURE_REQUEST: u8 = 23;
pub const XCB_GRAVITY_NOTIFY: u8 = 24;
pub const XCB_RESIZE_REQUEST: u8 = 25;
pub const XCB_CIRCULATE_NOTIFY: u8 = 26;
pub const XCB_CIRCULATE_REQUEST: u8 = 27;
pub const XCB_PROPERTY_NOTIFY: u8 = 28;
pub const XCB_SELECTION_CLEAR: u8 = 29;
pub const XCB_SELECTION_REQUEST: u8 = 30;
pub const XCB_SELECTION_NOTIFY: u8 = 31;
pub const XCB_COLORMAP_NOTIFY: u8 = 32;
pub const XCB_CLIENT_MESSAGE: u8 = 33;
pub const XCB_MAPPING_NOTIFY: u8 = 34;

pub const XCB_WINDOW_CLASS_COPY_FROM_PARENT: u16 = 0;
pub const XCB_WINDOW_CLASS_INPUT_OUTPUT: u16 = 1;
pub const XCB_WINDOW_CLASS_INPUT_ONLY: u16 = 2;

pub const XCB_CW_BACK_PIXMAP: u32 = 1;
pub const XCB_CW_BACK_PIXEL: u32 = 2;
pub const XCB_CW_BORDER_PIXMAP: u32 = 4;
pub const XCB_CW_BORDER_PIXEL: u32 = 8;
pub const XCB_CW_BIT_GRAVITY: u32 = 16;
pub const XCB_CW_WIN_GRAVITY: u32 = 32;
pub const XCB_CW_BACKING_STORE: u32 = 64;
pub const XCB_CW_BACKING_PLANES: u32 = 128;
pub const XCB_CW_BACKING_PIXEL: u32 = 256;
pub const XCB_CW_OVERRIDE_REDIRECT: u32 = 512;
pub const XCB_CW_SAVE_UNDER: u32 = 1024;
pub const XCB_CW_EVENT_MASK: u32 = 2048;
pub const XCB_CW_DONT_PROPAGATE: u32 = 4096;
pub const XCB_CW_COLORMAP: u32 = 8192;
pub const XCB_CW_CURSOR: u32 = 16384;

pub const XCB_COPY_FROM_PARENT: u8 = 0;

pub const XCB_PROP_MODE_REPLACE: u8 = 0;
pub const XCB_PROP_MODE_PREPEND: u8 = 1;
pub const XCB_PROP_MODE_APPEND: u8 = 2;

pub const XCB_ATOM_STRING: u32 = 31;
pub const XCB_ATOM_WM_NAME: u32 = 39;
pub const XCB_ATOM_WM_CLASS: u32 = 67;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::x11::*;
    use std::ptr;

    #[test]
    #[ignore]
    fn xcb() {
        unsafe {
            let mut scr = 0;
            let c = xcb_connect(ptr::null(), &mut scr);
            assert_ne!(c, ptr::null_mut());
            assert_eq!(xcb_connection_has_error(c), 0);

            let setup = xcb_get_setup(c);
            //println!("{:#?}", *setup);
            let mut iter = xcb_setup_roots_iterator(setup);
            loop {
                //println!("Screen: {}", scr);
                if scr <= 0 {
                    break;
                }
                scr -= 1;
                xcb_screen_next(&mut iter);
            }
            let screen = iter.data;

            //println!("{:#?}", *screen);

            let window = xcb_generate_id(c);
            //println!("window: {}", window);
            let value_list = [
                (*screen).black_pixel,
                XCB_EVENT_MASK_KEY_PRESS
                    | XCB_EVENT_MASK_KEY_RELEASE
                    | XCB_EVENT_MASK_EXPOSURE
                    | XCB_EVENT_MASK_STRUCTURE_NOTIFY
                    | XCB_EVENT_MASK_POINTER_MOTION
                    | XCB_EVENT_MASK_BUTTON_PRESS
                    | XCB_EVENT_MASK_BUTTON_RELEASE,
            ];
            xcb_create_window(
                c,
                XCB_COPY_FROM_PARENT,
                window,
                (*screen).root,
                0,
                0,
                1280,
                720,
                0,
                XCB_WINDOW_CLASS_INPUT_OUTPUT,
                (*screen).root_visual,
                XCB_CW_BACK_PIXEL | XCB_CW_EVENT_MASK,
                value_list.as_ptr() as *const c_void,
            );

            let atom_name = b"WM_PROTOCOLS\0";
            let cookie = xcb_intern_atom(c, 1, atom_name.len() as u16, atom_name.as_ptr() as *const i8);
            let atom_wm_protocols = xcb_intern_atom_reply(c, cookie, ptr::null_mut());

            let atom_name = b"WM_DELETE_WINDOW\0";
            let cookie = xcb_intern_atom(c, 0, atom_name.len() as u16, atom_name.as_ptr() as *const i8);
            let atom_wm_delete_window = xcb_intern_atom_reply(c, cookie, ptr::null_mut());

            xcb_change_property(
                c,
                XCB_PROP_MODE_REPLACE,
                window,
                (*atom_wm_protocols).atom,
                4,
                32,
                1,
                &(*atom_wm_delete_window).atom as *const _ as *const c_void,
            );
            let title = b"XCB Test\0";
            xcb_change_property(
                c,
                XCB_PROP_MODE_REPLACE,
                window,
                XCB_ATOM_WM_NAME,
                XCB_ATOM_STRING,
                8,
                title.len() as u32,
                title.as_ptr() as *const c_void,
            );

            xcb_map_window(c, window);
            xcb_flush(c);
            let mut quit = false;
            while !quit {
                loop {
                    let event = xcb_poll_for_event(c);
                    if event.is_null() {
                        break;
                    }

                    match (*event).response_type & 0x7f {
                        XCB_CLIENT_MESSAGE => {
                            let event = *(event as *mut xcb_client_message_event_t);
                            if event.data.data32[0] == (*atom_wm_delete_window).atom {
                                quit = true;
                            } else {
                                println!("[XCB_CLIENT_MESSAGE] type: {}, data: {:x?}", event.ttype, event.data.data32);
                            }
                        }
                        XCB_KEY_PRESS => {
                            // xcbcommon-keysyms.h
                            // xcb_get_keyboard_mapping
                            let event = event as *mut xcb_key_press_event_t;
                            println!("keycode: {}", (*event).detail);

                            let key_symbols = xcb_key_symbols_alloc(c);
                            let keysym = xcb_key_symbols_get_keysym(key_symbols, (*event).detail, 0);
                            println!("keysym: 0x{:04x}", keysym);
                            xcb_key_symbols_free(key_symbols);

                            if keysym == XK_Escape as xcb_keysym_t {
                                quit = true;
                            }

                            let min_keycode = (*setup).min_keycode;
                            let max_keycode = (*setup).max_keycode;
                            let keyboard_mapping = xcb_get_keyboard_mapping_reply(
                                c,
                                xcb_get_keyboard_mapping(c, min_keycode, max_keycode - (min_keycode - 1)),
                                ptr::null_mut(),
                            );
                            let keysyms = keyboard_mapping.offset(1) as *mut xcb_keysym_t;
                            for keycode_idx in 0..(*keyboard_mapping).length as isize {
                                for keysym_idx in 0..(*keyboard_mapping).keysyms_per_keycode as isize {
                                    let keysym = keysyms.offset(
                                        keysym_idx + keycode_idx * (*keyboard_mapping).keysyms_per_keycode as isize,
                                    );
                                    if *keysym != 0 {
                                        //println!(
                                        //    "keycode: {} - keysym: 0x{:08x}",
                                        //    (*setup).min_keycode as isize + keycode_idx,
                                        //    *keysym
                                        //);
                                        break;
                                    }
                                }
                            }
                            //println!("{:#?}", *keyboard_mapping);
                            //quit = true;
                        }
                        XCB_EXPOSE => {
                            //println!("XCB_EXPOSE");
                        }
                        _ => {}
                    }
                }
            }

            xcb_destroy_window(c, window);
            xcb_disconnect(c);
        }
    }
}
