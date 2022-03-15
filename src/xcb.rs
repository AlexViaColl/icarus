#![allow(non_camel_case_types)]

use std::ffi::c_void;

#[link(name = "xcb")]
extern "C" {
    pub fn xcb_connect(display_name: *const i8, screenp: *mut i32) -> *mut xcb_connection_t;
    pub fn xcb_disconnect(c: *mut xcb_connection_t);
    pub fn xcb_get_setup(c: *mut xcb_connection_t) -> *mut xcb_setup_t;
    pub fn xcb_setup_roots_iterator(r: *const xcb_setup_t) -> xcb_screen_iterator_t;
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
    pub fn xcb_map_window(c: *mut xcb_connection_t, window: xcb_window_t) -> xcb_void_cookie_t;
    pub fn xcb_flush(c: *mut xcb_connection_t) -> i32;
}

macro_rules! opaque {
    ($name: ident) => {
        #[repr(C)]
        pub struct $name {
            _data: [u8; 0],
            _marker: core::marker::PhantomData<(*mut u8, core::marker::PhantomPinned)>,
        }
    };
}

opaque!(xcb_connection_t);

pub type xcb_window_t = u32;
pub type xcb_visualid_t = u32;
pub type xcb_colormap_t = u32;
pub type xcb_keycode_t = u8;

#[repr(C)]
pub struct xcb_void_cookie_t {
    pub sequence: u32,
}

#[repr(C)]
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

#[repr(C)]
pub struct xcb_screen_iterator_t {
    pub data: *mut xcb_screen_t,
    pub rem: i32,
    pub index: i32,
}

#[repr(C)]
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

pub const XCB_COPY_FROM_PARENT: u8 = 0;

pub const XCB_WINDOW_CLASS_INPUT_OUTPUT: u16 = 1;

pub const XCB_CW_BACK_PIXEL: u32 = 2;
pub const XCB_CW_EVENT_MASK: u32 = 2048;

// typedef enum xcb_event_mask_t {
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
pub const XCB_EVENT_MASK_OWNER_GRAB_BUTTON: u32 = 1677721;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn xcb_init() {
        let mut screen_num = 0;
        let connection = unsafe { xcb_connect(std::ptr::null(), &mut screen_num) };
        println!("connection: {:?}, screen: {}", connection, screen_num);
        assert_ne!(connection, std::ptr::null_mut());

        let setup = unsafe { xcb_get_setup(connection) };
        let iter = unsafe { xcb_setup_roots_iterator(setup) };
        let screen = iter.data;

        let window = unsafe { xcb_generate_id(connection) };
        unsafe {
            xcb_create_window(
                connection,
                XCB_COPY_FROM_PARENT,
                window,
                (*screen).root,
                0,
                0,
                800,
                600,
                0,
                XCB_WINDOW_CLASS_INPUT_OUTPUT,
                (*screen).root_visual,
                XCB_CW_BACK_PIXEL | XCB_CW_EVENT_MASK,
                [(*screen).black_pixel, XCB_EVENT_MASK_EXPOSURE].as_ptr() as *const c_void,
            )
        };

        unsafe { xcb_map_window(connection, window) };
        unsafe { xcb_flush(connection) };

        loop {}

        unsafe { xcb_disconnect(connection) };
    }
}
