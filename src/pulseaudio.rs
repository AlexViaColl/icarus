#![allow(non_camel_case_types)]

use crate::string_util::cstr_to_string;
use crate::{bitflag_enum, bitflag_struct, opaque};

use std::ffi::c_void;

pub type time_t = i64;
pub type suseconds_t = i64;
#[repr(C)]
pub struct timeval {
    pub tv_sec: time_t,
    pub tv_usec: suseconds_t,
}

#[link(name = "pulse-simple")]
extern "C" {
    // Simple API
    pub fn pa_simple_new(
        server: *const i8,
        name: *const i8,
        dir: pa_stream_direction_t,
        dev: *const i8,
        stream_name: *const i8,
        ss: *const pa_sample_spec,
        map: *const pa_channel_map,
        attr: *const pa_buffer_attr,
        error: *mut i32,
    ) -> *mut pa_simple;
    pub fn pa_simple_free(s: *mut pa_simple);
    pub fn pa_simple_drain(s: *mut pa_simple, error: *mut i32) -> i32;
    pub fn pa_simple_flush(s: *mut pa_simple, error: *mut i32) -> i32;
    pub fn pa_simple_get_latency(s: *mut pa_simple, error: *mut i32) -> pa_usec_t;
    pub fn pa_simple_read(s: *mut pa_simple, data: *mut c_void, bytes: usize, error: *mut i32) -> i32;
    pub fn pa_simple_write(s: *mut pa_simple, data: *const c_void, bytes: usize, error: *mut i32) -> i32;
}

#[link(name = "pulse")]
extern "C" {
    // Context
    pub fn pa_context_new(mainloop: *mut pa_mainloop_api, name: *const i8) -> *mut pa_context;
    //pub fn pa_context_new_with_proplist(
    //    mainloop: *mut pa_mainloop_api,
    //    name: *const i8,
    //    proplist: *const pa_proplist,
    //) -> *mut pa_context;
    pub fn pa_context_set_state_callback(c: *mut pa_context, cb: pa_context_notify_cb_t, userdata: *mut c_void);
    pub fn pa_context_get_state(c: *mut pa_context) -> pa_context_state_t;
    pub fn pa_context_connect(
        c: *mut pa_context,
        server: *const i8,
        flags: u32, //pa_context_flags_t,
        api: *const pa_spawn_api,
    ) -> i32;
    pub fn pa_context_disconnect(c: *mut pa_context);

    // Main Loop
    pub fn pa_mainloop_new() -> *mut pa_mainloop;
    pub fn pa_mainloop_free(m: *mut pa_mainloop);
    pub fn pa_mainloop_get_api(m: *mut pa_mainloop) -> *mut pa_mainloop_api;
    pub fn pa_mainloop_prepare(m: *mut pa_mainloop, timeout: i32) -> i32;
    pub fn pa_mainloop_poll(m: *mut pa_mainloop) -> i32;
    pub fn pa_mainloop_dispatch(m: *mut pa_mainloop) -> i32;
    pub fn pa_mainloop_iterate(m: *mut pa_mainloop, block: i32, retval: *mut i32) -> i32;
    pub fn pa_mainloop_run(m: *mut pa_mainloop, retval: *mut i32) -> i32;
    pub fn pa_mainloop_quit(m: *mut pa_mainloop, retval: i32);

    pub fn pa_threaded_mainloop_new() -> *mut pa_threaded_mainloop;
    pub fn pa_threaded_mainloop_free(m: *mut pa_threaded_mainloop);
    pub fn pa_threaded_mainloop_start(m: *mut pa_threaded_mainloop);
    pub fn pa_threaded_mainloop_stop(m: *mut pa_threaded_mainloop);
    pub fn pa_threaded_mainloop_lock(m: *mut pa_threaded_mainloop);
    pub fn pa_threaded_mainloop_unlock(m: *mut pa_threaded_mainloop);
    pub fn pa_threaded_mainloop_wait(m: *mut pa_threaded_mainloop);
    pub fn pa_threaded_mainloop_signal(m: *mut pa_threaded_mainloop, wait_for_accept: i32);
    pub fn pa_threaded_mainloop_accept(m: *mut pa_threaded_mainloop);
    pub fn pa_threaded_mainloop_get_retval(m: *mut pa_threaded_mainloop) -> i32;
    pub fn pa_threaded_mainloop_get_api(m: *mut pa_threaded_mainloop) -> *mut pa_mainloop_api;

    // Event Subscription / Notifications
    pub fn pa_context_subscribe(
        c: *mut pa_context,
        m: pa_subscription_mask_t,
        cb: pa_context_success_cb_t,
        userdata: *mut c_void,
    ) -> *mut pa_operation;
    pub fn pa_context_set_subscribe_callback(c: *mut pa_context, cb: pa_context_subscribe_cb_t, userdata: *mut c_void);

    // Introspection
    // Sinks
    pub fn pa_context_get_sink_info_list(
        c: *mut pa_context,
        cb: pa_sink_info_cb_t,
        userdata: *mut c_void,
    ) -> *mut pa_operation;
    pub fn pa_context_get_sink_input_info_list(
        c: *mut pa_context,
        cb: pa_sink_input_info_cb_t,
        userdata: *mut c_void,
    ) -> *mut pa_operation;
    pub fn pa_context_move_sink_input_by_name(
        c: *mut pa_context,
        idx: u32,
        sink_name: *const i8,
        cb: pa_context_success_cb_t,
        userdata: *mut c_void,
    ) -> *mut pa_operation;
    pub fn pa_context_move_sink_input_by_index(
        c: *mut pa_context,
        idx: u32,
        sink_idx: u32,
        cb: pa_context_success_cb_t,
        userdata: *mut c_void,
    ) -> *mut pa_operation;

    // Sources
    pub fn pa_context_get_source_info_list(
        c: *mut pa_context,
        cb: pa_source_info_cb_t,
        userdata: *mut c_void,
    ) -> *mut pa_operation;
    pub fn pa_context_get_source_output_info_list(
        c: *mut pa_context,
        cb: pa_source_output_info_cb_t,
        userdata: *mut c_void,
    ) -> *mut pa_operation;

    // Server
    pub fn pa_context_get_server_info(
        c: *mut pa_context,
        cb: pa_server_info_cb_t,
        userdata: *mut c_void,
    ) -> *mut pa_operation;

    // Modules
    pub fn pa_context_get_module_info_list(
        c: *mut pa_context,
        cb: pa_module_info_cb_t,
        userdata: *mut c_void,
    ) -> *mut pa_operation;

    // Messages
    // Clients
    pub fn pa_context_get_client_info_list(
        c: *mut pa_context,
        cb: pa_client_info_cb_t,
        userdata: *mut c_void,
    ) -> *mut pa_operation;

    // Cards
    // Sink Inputs
    // Source Outputs
    // Statistics

}

opaque!(pa_context, pa_context_);
#[repr(C)]
pub enum pa_subscription_mask_t {
    PA_SUBSCRIPTION_MASK_NULL = 0x0000,
    PA_SUBSCRIPTION_MASK_SINK = 0x0001,
    PA_SUBSCRIPTION_MASK_SOURCE = 0x0002,
    PA_SUBSCRIPTION_MASK_SINK_INPUT = 0x0004,
    PA_SUBSCRIPTION_MASK_SOURCE_OUTPUT = 0x0008,
    PA_SUBSCRIPTION_MASK_MODULE = 0x0010,
    PA_SUBSCRIPTION_MASK_CLIENT = 0x0020,
    PA_SUBSCRIPTION_MASK_SAMPLE_CACHE = 0x0040,
    PA_SUBSCRIPTION_MASK_SERVER = 0x0080,
    PA_SUBSCRIPTION_MASK_AUTOLOAD = 0x0100,
    PA_SUBSCRIPTION_MASK_CARD = 0x0200,
    PA_SUBSCRIPTION_MASK_ALL = 0x02ff,
}

#[repr(C)]
pub enum pa_source_flags_t {
    PA_SOURCE_NOFLAGS = 0x0000,
    PA_SOURCE_HW_VOLUME_CTRL = 0x0001,
    PA_SOURCE_LATENCY = 0x0002,
    PA_SOURCE_HARDWARE = 0x0004,
    PA_SOURCE_NETWORK = 0x0008,
    PA_SOURCE_HW_MUTE_CTRL = 0x0010,
    PA_SOURCE_DECIBEL_VOLUME = 0x0020,
    PA_SOURCE_DYNAMIC_LATENCY = 0x0040,
    PA_SOURCE_FLAT_VOLUME = 0x0080,
    PA_SOURCE_SHARE_VOLUME_WITH_MASTER = 0x1000000,
    PA_SOURCE_DEFERRED_VOLUME = 0x2000000,
}

#[repr(C)]
pub enum pa_sink_state_t {
    PA_SINK_INVALID_STATE = -1,
    PA_SINK_RUNNING = 0,
    PA_SINK_IDLE = 1,
    PA_SINK_SUSPENDED = 2,
    PA_SINK_INIT = -2,
    PA_SINK_UNLINKED = -3,
}

#[repr(C)]
pub enum pa_source_state_t {
    PA_SOURCE_INVALID_STATE = -1,
    PA_SOURCE_RUNNING = 0,
    PA_SOURCE_IDLE = 1,
    PA_SOURCE_SUSPENDED = 2,
    PA_SOURCE_INIT = -2,
    PA_SOURCE_UNLINKED = -3,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ptr;

    #[test]
    fn simple() {
        let ss = pa_sample_spec {
            format: PA_SAMPLE_S16LE,
            rate: 44100,
            channels: 2,
        };
        let s = unsafe {
            pa_simple_new(
                ptr::null(),
                b"Foo\0".as_ptr() as *const i8,
                PA_STREAM_PLAYBACK,
                ptr::null(),
                b"playback\0".as_ptr() as *const i8,
                &ss,
                ptr::null(),
                ptr::null(),
                ptr::null_mut(),
            )
        };
        if s.is_null() {
            return;
        }

        let mut buffer = [0_i16; 44100 * 2];
        for i in 0..44100 {
            buffer[2 * i] = (10000.0 * (2.0 * std::f32::consts::PI * 200.0 * (i as f32 / 44100.0)).sin()) as i16;
            buffer[2 * i + 1] = (10000.0 * (2.0 * std::f32::consts::PI * 200.0 * (i as f32 / 44100.0)).sin()) as i16;
        }
        unsafe { pa_simple_write(s, buffer.as_ptr() as *const c_void, buffer.len() * 2, ptr::null_mut()) };

        unsafe { pa_simple_drain(s, ptr::null_mut()) };
        unsafe { pa_simple_free(s) };
    }
}

pub type pa_usec_t = u64;

#[repr(C)]
pub struct pa_simple_ {
    _data: [u8; 0],
    _marker: core::marker::PhantomData<(*mut u8, core::marker::PhantomPinned)>,
}

#[repr(transparent)]
pub struct pa_simple(*mut pa_simple_);
impl Default for pa_simple {
    fn default() -> Self {
        Self(std::ptr::null_mut())
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct pa_sample_spec {
    pub format: pa_sample_format_t,
    pub rate: u32,
    pub channels: u8,
}

pub const PA_CHANNELS_MAX: usize = 32;

#[repr(C)]
#[derive(Debug)]
pub struct pa_channel_map {
    pub channels: u8,
    pub map: [pa_channel_position_t; PA_CHANNELS_MAX],
}

#[repr(C)]
pub struct pa_buffer_attr {
    pub maxlength: u32,
    pub tlength: u32,
    pub prebuf: u32,
    pub minreq: u32,
    pub fragsize: u32,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum pa_stream_direction_t {
    PA_STREAM_NODIRECTION,
    PA_STREAM_PLAYBACK,
    PA_STREAM_RECORD,
    PA_STREAM_UPLOAD,
}
pub use pa_stream_direction_t::*;

#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum pa_sample_format_t {
    PA_SAMPLE_U8 = 0,
    PA_SAMPLE_ALAW,
    PA_SAMPLE_ULAW,
    PA_SAMPLE_S16LE,
    PA_SAMPLE_S16BE,
    PA_SAMPLE_FLOAT32LE,
    PA_SAMPLE_FLOAT32BE,
    PA_SAMPLE_S32LE,
    PA_SAMPLE_S32BE,
    PA_SAMPLE_S24LE,
    PA_SAMPLE_S24BE,
    PA_SAMPLE_S24_32LE,
    PA_SAMPLE_S24_32BE,
    PA_SAMPLE_MAX,
    PA_SAMPLE_INVALID = -1,
}
pub use pa_sample_format_t::*;

#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum pa_channel_position_t {
    PA_CHANNEL_POSITION_INVALID = -1,
    PA_CHANNEL_POSITION_MONO = 0,

    PA_CHANNEL_POSITION_FRONT_LEFT,
    PA_CHANNEL_POSITION_FRONT_RIGHT,
    PA_CHANNEL_POSITION_FRONT_CENTER,

    //PA_CHANNEL_POSITION_LEFT = PA_CHANNEL_POSITION_FRONT_LEFT,
    //PA_CHANNEL_POSITION_RIGHT = PA_CHANNEL_POSITION_FRONT_RIGHT,
    //PA_CHANNEL_POSITION_CENTER = PA_CHANNEL_POSITION_FRONT_CENTER,
    PA_CHANNEL_POSITION_REAR_CENTER,
    PA_CHANNEL_POSITION_REAR_LEFT,
    PA_CHANNEL_POSITION_REAR_RIGHT,
    PA_CHANNEL_POSITION_LFE,
    //PA_CHANNEL_POSITION_SUBWOOFER = PA_CHANNEL_POSITION_LFE,
    //PA_CHANNEL_POSITION_FRONT_LEFT_OF_CENTER,
    //PA_CHANNEL_POSITION_FRONT_RIGHT_OF_CENTER,
    PA_CHANNEL_POSITION_SIDE_LEFT,
    PA_CHANNEL_POSITION_SIDE_RIGHT,

    PA_CHANNEL_POSITION_AUX0,
    PA_CHANNEL_POSITION_AUX1,
    PA_CHANNEL_POSITION_AUX2,
    PA_CHANNEL_POSITION_AUX3,
    PA_CHANNEL_POSITION_AUX4,
    PA_CHANNEL_POSITION_AUX5,
    PA_CHANNEL_POSITION_AUX6,
    PA_CHANNEL_POSITION_AUX7,
    PA_CHANNEL_POSITION_AUX8,
    PA_CHANNEL_POSITION_AUX9,
    PA_CHANNEL_POSITION_AUX10,
    PA_CHANNEL_POSITION_AUX11,
    PA_CHANNEL_POSITION_AUX12,
    PA_CHANNEL_POSITION_AUX13,
    PA_CHANNEL_POSITION_AUX14,
    PA_CHANNEL_POSITION_AUX15,
    PA_CHANNEL_POSITION_AUX16,
    PA_CHANNEL_POSITION_AUX17,
    PA_CHANNEL_POSITION_AUX18,
    PA_CHANNEL_POSITION_AUX19,
    PA_CHANNEL_POSITION_AUX20,
    PA_CHANNEL_POSITION_AUX21,
    PA_CHANNEL_POSITION_AUX22,
    PA_CHANNEL_POSITION_AUX23,
    PA_CHANNEL_POSITION_AUX24,
    PA_CHANNEL_POSITION_AUX25,
    PA_CHANNEL_POSITION_AUX26,
    PA_CHANNEL_POSITION_AUX27,
    PA_CHANNEL_POSITION_AUX28,
    PA_CHANNEL_POSITION_AUX29,
    PA_CHANNEL_POSITION_AUX30,
    PA_CHANNEL_POSITION_AUX31,

    PA_CHANNEL_POSITION_TOP_CENTER,
    PA_CHANNEL_POSITION_TOP_FRONT_LEFT,
    PA_CHANNEL_POSITION_TOP_FRONT_RIGHT,
    PA_CHANNEL_POSITION_TOP_FRONT_CENTER,
    PA_CHANNEL_POSITION_TOP_REAR_LEFT,
    PA_CHANNEL_POSITION_TOP_REAR_RIGHT,
    PA_CHANNEL_POSITION_TOP_REAR_CENTER,

    PA_CHANNEL_POSITION_MAX,
}
pub use pa_channel_position_t::*;
