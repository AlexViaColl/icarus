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

    // Proplist
    pub fn pa_proplist_to_string(p: *const pa_proplist) -> *mut i8;
}

opaque!(pa_context, pa_context_);

pub type pa_context_notify_cb_t = extern "C" fn(c: *mut pa_context, userdata: *mut c_void);
pub type pa_context_success_cb_t = extern "C" fn(c: *mut pa_context, success: i32, userdata: *mut c_void);
pub type pa_context_subscribe_cb_t =
    extern "C" fn(c: *mut pa_context, t: pa_subscription_event_type_t, idx: u32, userdata: *mut c_void);

#[repr(C)]
#[derive(Debug)]
pub enum pa_context_state_t {
    PA_CONTEXT_UNCONNECTED,
    PA_CONTEXT_CONNECTING,
    PA_CONTEXT_AUTHORIZING,
    PA_CONTEXT_SETTING_NAME,
    PA_CONTEXT_READY,
    PA_CONTEXT_FAILED,
    PA_CONTEXT_TERMINATED,
}
pub use pa_context_state_t::*;

#[repr(C)]
pub enum pa_context_flags_t {
    PA_CONTEXT_NOFLAGS = 0x0000,
    PA_CONTEXT_NOAUTOSPAWN = 0x0001,
    PA_CONTEXT_NOFAIL = 0x0002,
}

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
pub use pa_subscription_mask_t::*;

bitflag_struct!(pa_subscription_event_type_t: pa_subscription_event_type_t_bits);
bitflag_enum!(pa_subscription_event_type_t_bits {
    PA_SUBSCRIPTION_EVENT_SINK = 0x0000,
    PA_SUBSCRIPTION_EVENT_SOURCE = 0x0001,
    PA_SUBSCRIPTION_EVENT_SINK_INPUT = 0x0002,
    PA_SUBSCRIPTION_EVENT_SOURCE_OUTPUT = 0x0003,
    PA_SUBSCRIPTION_EVENT_MODULE = 0x0004,
    PA_SUBSCRIPTION_EVENT_CLIENT = 0x0005,
    PA_SUBSCRIPTION_EVENT_SAMPLE_CACHE = 0x0006,
    PA_SUBSCRIPTION_EVENT_SERVER = 0x0007,
    PA_SUBSCRIPTION_EVENT_AUTOLOAD = 0x0008,
    PA_SUBSCRIPTION_EVENT_CARD = 0x0009,
    PA_SUBSCRIPTION_EVENT_FACILITY_MASK = 0x000F,
    //PA_SUBSCRIPTION_EVENT_NEW = 0x0000,
    PA_SUBSCRIPTION_EVENT_CHANGE = 0x0010,
    PA_SUBSCRIPTION_EVENT_REMOVE = 0x0020,
    PA_SUBSCRIPTION_EVENT_TYPE_MASK = 0x0030,
});
//#[repr(C)]
//#[derive(Debug)]
//pub enum pa_subscription_event_type_t {
//    PA_SUBSCRIPTION_EVENT_SINK = 0x0000,
//    PA_SUBSCRIPTION_EVENT_SOURCE = 0x0001,
//    PA_SUBSCRIPTION_EVENT_SINK_INPUT = 0x0002,
//    PA_SUBSCRIPTION_EVENT_SOURCE_OUTPUT = 0x0003,
//    PA_SUBSCRIPTION_EVENT_MODULE = 0x0004,
//    PA_SUBSCRIPTION_EVENT_CLIENT = 0x0005,
//    PA_SUBSCRIPTION_EVENT_SAMPLE_CACHE = 0x0006,
//    PA_SUBSCRIPTION_EVENT_SERVER = 0x0007,
//    PA_SUBSCRIPTION_EVENT_AUTOLOAD = 0x0008,
//    PA_SUBSCRIPTION_EVENT_CARD = 0x0009,
//    PA_SUBSCRIPTION_EVENT_FACILITY_MASK = 0x000F,
//    //PA_SUBSCRIPTION_EVENT_NEW = 0x0000,
//    PA_SUBSCRIPTION_EVENT_CHANGE = 0x0010,
//    PA_SUBSCRIPTION_EVENT_REMOVE = 0x0020,
//    PA_SUBSCRIPTION_EVENT_TYPE_MASK = 0x0030,
//}

#[repr(C)]
pub struct pa_spawn_api {
    pub prefork: extern "C" fn(),
    pub postfork: extern "C" fn(),
    pub atfork: extern "C" fn(),
}

opaque!(pa_mainloop, pa_mainloop_);
#[repr(C)]
pub struct pa_mainloop_api {
    pub userdata: *mut c_void,
    pub io_new: extern "C" fn(
        a: *mut pa_mainloop_api,
        fd: i32,
        events: pa_io_event_flags_t,
        cb: pa_io_event_cb_t,
        userdata: *mut c_void,
    ) -> *mut pa_io_event,
    pub io_enable: extern "C" fn(e: *mut pa_io_event, events: pa_io_event_flags_t),
    pub io_free: extern "C" fn(e: *mut pa_io_event),
    pub io_set_destroy: extern "C" fn(e: *mut pa_io_event, cb: pa_io_event_destroy_cb_t),
    pub time_new: extern "C" fn(
        a: *mut pa_mainloop_api,
        tv: *const timeval,
        cb: pa_time_event_cb_t,
        userdata: *mut c_void,
    ) -> *mut pa_time_event,
    pub time_restart: extern "C" fn(e: *mut pa_time_event, tv: *const timeval),
    pub time_free: extern "C" fn(e: *mut pa_time_event),
    pub time_set_destroy: extern "C" fn(e: *mut pa_time_event, cb: pa_time_event_destroy_cb_t),
    pub defer_new:
        extern "C" fn(a: *mut pa_mainloop_api, cb: pa_defer_event_cb_t, userdata: *mut c_void) -> *mut pa_defer_event,
    pub defer_enable: extern "C" fn(e: *mut pa_defer_event, b: i32),
    pub defer_free: extern "C" fn(e: *mut pa_defer_event),
    pub defer_set_destroy: extern "C" fn(e: *mut pa_defer_event, cb: pa_defer_event_destroy_cb_t),
    pub quit: extern "C" fn(a: *mut pa_mainloop_api, retval: i32),
}

opaque!(pa_io_event, pa_io_event_);
#[repr(C)]
pub enum pa_io_event_flags_t {
    PA_IO_EVENT_NULL = 0,
    PA_IO_EVENT_INPUT = 1,
    PA_IO_EVENT_OUTPUT = 2,
    PA_IO_EVENT_HANGUP = 4,
    PA_IO_EVENT_ERROR = 8,
}
pub type pa_io_event_cb_t = extern "C" fn(
    a: *mut pa_mainloop_api,
    e: *mut pa_io_event,
    fd: i32,
    events: pa_io_event_flags_t,
    userdata: *mut c_void,
);
pub type pa_io_event_destroy_cb_t = extern "C" fn(a: *mut pa_mainloop_api, e: *mut pa_io_event, userdata: *mut c_void);

opaque!(pa_time_event, pa_time_event_);
pub type pa_time_event_cb_t =
    extern "C" fn(a: *mut pa_mainloop_api, e: *mut pa_time_event, tv: *const timeval, userdata: *mut c_void);
pub type pa_time_event_destroy_cb_t =
    extern "C" fn(a: *mut pa_mainloop_api, e: *mut pa_time_event, userdata: *mut c_void);

opaque!(pa_defer_event, pa_defer_event_);
pub type pa_defer_event_cb_t = extern "C" fn(a: *mut pa_mainloop_api, e: *mut pa_defer_event, userdata: *mut c_void);
pub type pa_defer_event_destroy_cb_t =
    extern "C" fn(a: *mut pa_mainloop_api, e: *mut pa_defer_event, userdata: *mut c_void);

opaque!(pa_threaded_mainloop, pa_threaded_mainloop_);

opaque!(pa_operation, pa_operation_);
pub type pa_sink_info_cb_t =
    extern "C" fn(c: *mut pa_context, i: *const pa_sink_info, eol: i32, user_data: *mut c_void);
pub type pa_sink_input_info_cb_t =
    extern "C" fn(c: *mut pa_context, i: *const pa_sink_input_info, eol: i32, userdata: *mut c_void);
pub type pa_source_info_cb_t =
    extern "C" fn(c: *mut pa_context, i: *const pa_source_info, eol: i32, userdata: *mut c_void);
pub type pa_source_output_info_cb_t =
    extern "C" fn(c: *mut pa_context, i: *const pa_source_output_info, eol: i32, userdata: *mut c_void);
pub type pa_server_info_cb_t = extern "C" fn(c: *mut pa_context, i: *const pa_server_info, userdata: *mut c_void);
pub type pa_module_info_cb_t =
    extern "C" fn(c: *mut pa_context, i: *const pa_module_info, eol: i32, userdata: *mut c_void);
pub type pa_client_info_cb_t =
    extern "C" fn(c: *mut pa_context, i: *const pa_client_info, eol: i32, userdata: *mut c_void);

#[repr(C)]
pub struct pa_sink_info {
    pub name: *const i8,
    pub index: u32,
    pub description: *const i8,
    pub sample_spec: pa_sample_spec,
    pub channel_map: pa_channel_map,
    pub owner_module: u32,
    pub volume: pa_cvolume,
    pub mute: i32,
    pub monitor_source: u32,
    pub monitor_source_name: *const i8,
    pub latency: pa_usec_t,
    pub driver: *const i8,
    pub flags: pa_sink_flags_t,
    pub proplist: *mut pa_proplist,
    pub configured_latency: pa_usec_t,
    pub base_volume: pa_volume_t,
    pub state: pa_sink_state_t,
    pub n_volume_steps: u32,
    pub card: u32,
    pub n_ports: u32,
    pub ports: *mut *mut pa_sink_port_info,
    pub active_port: *mut pa_sink_port_info,
    pub n_formats: u8,
    pub formats: *mut *mut pa_format_info,
}

#[repr(C)]
pub struct pa_sink_input_info {
    pub index: u32,
    pub name: *const i8,
    pub owner_module: u32,
    pub client: u32,
    pub sink: u32,
    pub sample_spec: pa_sample_spec,
    pub channel_map: pa_channel_map,
    pub volume: pa_cvolume,
    pub buffer_usec: pa_usec_t,
    pub sink_usec: pa_usec_t,
    pub resample_method: *const i8,
    pub driver: *const i8,
    pub mute: i32,
    pub proplist: *mut pa_proplist,
    pub corked: i32,
    pub has_volume: i32,
    pub volume_writable: i32,
    pub format: *mut pa_format_info,
}

#[repr(C)]
pub struct pa_source_info {
    pub name: *const i8,
    pub index: u32,
    pub description: *const i8,
    pub sample_spec: pa_sample_spec,
    pub channel_map: pa_channel_map,
    pub owner_module: u32,
    pub volume: pa_cvolume,
    pub mute: i32,
    pub monitor_of_sink: u32,
    pub monitor_of_sink_name: *const i8,
    pub latency: pa_usec_t,
    pub driver: *const i8,
    pub flags: pa_source_flags_t,
    pub proplist: *mut pa_proplist,
    pub configured_latency: pa_usec_t,
    pub base_volume: pa_volume_t,
    pub state: pa_source_state_t,
    pub n_volume_steps: u32,
    pub card: u32,
    pub n_ports: u32,
    pub ports: *mut *mut pa_source_port_info,
    pub active_port: *mut pa_source_port_info,
    pub n_formats: u8,
    pub formats: *mut *mut pa_format_info,
}

#[repr(C)]
pub struct pa_source_output_info {
    pub index: u32,
    pub name: *const i8,
    pub owner_module: u32,
    pub client: u32,
    pub source: u32,
    pub sample_spec: pa_sample_spec,
    pub channel_map: pa_channel_map,
    pub buffer_usec: pa_usec_t,
    pub source_usec: pa_usec_t,
    pub resample_method: *const i8,
    pub driver: *const i8,
    pub proplist: *mut pa_proplist,
    pub corked: i32,
    pub volume: pa_cvolume,
    pub mute: i32,
    pub has_volume: i32,
    pub volume_writable: i32,
    pub format: *mut pa_format_info,
}

#[repr(C)]
pub struct pa_server_info {
    pub user_name: *const i8,           /*< User name of the daemon process */
    pub host_name: *const i8,           /*< Host name the daemon is running on */
    pub server_version: *const i8,      /*< Version string of the daemon */
    pub server_name: *const i8,         /*< Server package name (usually "pulseaudio") */
    pub sample_spec: pa_sample_spec,    /*< Default sample specification */
    pub default_sink_name: *const i8,   /*< Name of default sink. */
    pub default_source_name: *const i8, /*< Name of default source. */
    pub cookie: u32,                    /*< A random cookie for identifying this instance of PulseAudio. */
    pub channel_map: pa_channel_map,    /*< Default channel map. \since 0.9.15 */
}
impl std::fmt::Debug for pa_server_info {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("pa_server_info")
            .field("user_name", &unsafe { cstr_to_string(self.user_name) })
            .field("host_name", &unsafe { cstr_to_string(self.host_name) })
            .field("server_version", &unsafe { cstr_to_string(self.server_version) })
            .field("server_name", &unsafe { cstr_to_string(self.server_name) })
            .field("sample_spec", &self.sample_spec)
            .field("default_sink_name", &unsafe { cstr_to_string(self.default_sink_name) })
            .field("default_source_name", &unsafe { cstr_to_string(self.default_source_name) })
            .field("cookie", &self.cookie)
            //.field("channel_map", &self.channel_map)
            .finish()
    }
}

#[repr(C)]
pub struct pa_module_info {
    pub index: u32,
    pub name: *const i8,
    pub argument: *const i8,
    pub n_used: u32,
    pub auto_unload: i32,
    pub proplist: *mut pa_proplist,
}

#[repr(C)]
pub struct pa_client_info {
    pub index: u32,
    pub name: *const i8,
    pub owner_module: u32,
    pub driver: *const i8,
    pub proplist: *mut pa_proplist,
}

opaque!(pa_proplist, pa_proplist_);

#[repr(C)]
pub struct pa_cvolume {
    pub channels: u8,
    pub values: [pa_volume_t; PA_CHANNELS_MAX],
}
pub type pa_volume_t = u32;

#[repr(C)]
pub struct pa_sink_port_info {
    pub name: *const i8,
    pub description: *const i8,
    pub priority: u32,
    pub available: i32,
    pub availability_group: *const i8,
    pub ttype: u32,
}

#[repr(C)]
pub struct pa_source_port_info {
    pub name: *const i8,
    pub description: *const i8,
    pub priority: u32,
    pub available: i32,
    pub available_group: *const i8,
    pub ttype: u32,
}

#[repr(C)]
pub struct pa_format_info {
    pub encoding: pa_encoding_t,
    pub plist: *mut pa_proplist,
}

#[repr(C)]
pub enum pa_encoding_t {
    PA_ENCODING_ANY,
    PA_ENCODING_PCM,
    PA_ENCODING_AC3_IEC61937,
    PA_ENCODING_EAC3_IEC61937,
    PA_ENCODING_MPEG_IEC61937,
    PA_ENCODING_DTS_IEC61937,
    PA_ENCODING_MPEG2_AAC_IEC61937,
    PA_ENCODING_TRUEHD_IEC61937,
    PA_ENCODING_DTSHD_IEC61937,
    PA_ENCODING_MAX,
    PA_ENCODING_INVALID = -1,
}

#[repr(C)]
pub enum pa_sink_flags_t {
    PA_SINK_NOFLAGS = 0x0000,
    PA_SINK_HW_VOLUME_CTRL = 0x0001,
    PA_SINK_LATENCY = 0x0002,
    PA_SINK_HARDWARE = 0x0004,
    PA_SINK_NETWORK = 0x0008,
    PA_SINK_HW_MUTE_CTRL = 0x0010,
    PA_SINK_DECIBEL_VOLUME = 0x0020,
    PA_SINK_FLAT_VOLUME = 0x0040,
    PA_SINK_DYNAMIC_LATENCY = 0x0080,
    PA_SINK_SET_FORMATS = 0x0100,
    PA_SINK_SHARE_VOLUME_WITH_MASTER = 0x1000000,
    PA_SINK_DEFERRED_VOLUME = 0x2000000,
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

    extern "C" fn state_changed_callback(c: *mut pa_context, userdata: *mut c_void) {
        unsafe {
            let state = pa_context_get_state(c);
            println!("[state_changed] {:?}", state);
            match state {
                PA_CONTEXT_READY => {
                    pa_context_subscribe(c, PA_SUBSCRIPTION_MASK_ALL, success_callback, ptr::null_mut());
                    pa_context_set_subscribe_callback(c, subscribe_callback, ptr::null_mut());

                    pa_context_get_server_info(c, server_info_callback, ptr::null_mut());
                    pa_context_get_client_info_list(c, client_info_callback, ptr::null_mut());
                    pa_context_get_module_info_list(c, module_info_callback, ptr::null_mut());
                    pa_context_get_sink_info_list(c, sink_info_callback, ptr::null_mut());
                    pa_context_get_sink_input_info_list(c, sink_input_info_callback, ptr::null_mut());
                    pa_context_get_source_info_list(c, source_info_callback, ptr::null_mut());
                    pa_context_get_source_output_info_list(c, source_output_info_callback, ptr::null_mut());

                    pa_context_move_sink_input_by_name(
                        c,
                        394,
                        //b"alsa_output.pci-0000_0a_00.1.hdmi-stereo\0".as_ptr() as *const i8,
                        b"alsa_output.pci-0000_0c_00.4.analog-stereo\0".as_ptr() as *const i8,
                        success_callback,
                        ptr::null_mut(),
                    );
                }
                PA_CONTEXT_FAILED | PA_CONTEXT_TERMINATED => {}
                _ => {}
            }
        }
    }

    extern "C" fn subscribe_callback(
        c: *mut pa_context,
        t: pa_subscription_event_type_t,
        idx: u32,
        userdata: *mut c_void,
    ) {
        let obj: pa_subscription_event_type_t = (t.value & PA_SUBSCRIPTION_EVENT_FACILITY_MASK).into();
        let ttype: pa_subscription_event_type_t = (t.value & PA_SUBSCRIPTION_EVENT_TYPE_MASK).into();
        println!("[subscribe_info] event type: {:?}, obj: {:?}, idx: {}", ttype, obj, idx);
    }

    extern "C" fn success_callback(c: *mut pa_context, success: i32, userdata: *mut c_void) {
        println!("[success_info] success: {}", success);
    }

    extern "C" fn sink_info_callback(c: *mut pa_context, i: *const pa_sink_info, eol: i32, userdata: *mut c_void) {
        if eol == 1 {
            return;
        }
        unsafe {
            println!("[sink_info] index: {}, name: {}", (*i).index, cstr_to_string((*i).name));
        }
    }

    extern "C" fn sink_input_info_callback(
        c: *mut pa_context,
        i: *const pa_sink_input_info,
        eol: i32,
        userdata: *mut c_void,
    ) {
        if eol == 1 {
            return;
        }
        unsafe {
            println!(
                "[sink_input_info] index: {}, name: {}, driver: {}, client: {}",
                (*i).index,
                cstr_to_string((*i).name),
                cstr_to_string((*i).driver),
                (*i).client,
            );
        }
    }

    extern "C" fn source_info_callback(c: *mut pa_context, i: *const pa_source_info, eol: i32, userdata: *mut c_void) {
        if eol == 1 {
            return;
        }
        unsafe {
            println!("[source_info] index: {}, name: {}", (*i).index, cstr_to_string((*i).name));
        }
    }

    extern "C" fn source_output_info_callback(
        c: *mut pa_context,
        i: *const pa_source_output_info,
        eol: i32,
        userdata: *mut c_void,
    ) {
        if eol == 1 {
            return;
        }
        unsafe {
            println!("[source_output_info] index: {}, name: {}", (*i).index, cstr_to_string((*i).name));
        }
    }

    extern "C" fn server_info_callback(c: *mut pa_context, i: *const pa_server_info, userdata: *mut c_void) {
        unsafe {
            println!("[server_info] name: {}", cstr_to_string((*i).server_name));
        }
    }

    extern "C" fn module_info_callback(c: *mut pa_context, i: *const pa_module_info, eol: i32, userdata: *mut c_void) {
        if eol == 1 {
            return;
        }
        unsafe {
            println!(
                "[module_info] index: {}, name: {}, argument: {}",
                (*i).index,
                cstr_to_string((*i).name),
                if (*i).argument.is_null() {
                    String::from("no argument")
                } else {
                    cstr_to_string((*i).argument)
                }
            );
            println!("[module_info] proplist:\n{}", cstr_to_string(pa_proplist_to_string((*i).proplist)));
        }
    }

    extern "C" fn client_info_callback(c: *mut pa_context, i: *const pa_client_info, eol: i32, userdata: *mut c_void) {
        if eol == 1 {
            return;
        }
        unsafe {
            println!("[client_info] index: {}, name: {}", (*i).index, cstr_to_string((*i).name));
            //println!("[client_info] proplist:\n{}", cstr_to_string(pa_proplist_to_string((*i).proplist)));
        }
    }

    #[test]
    fn mainloop() {
        unsafe {
            let mainloop = pa_mainloop_new();
            assert!(!mainloop.is_null());

            let api = pa_mainloop_get_api(mainloop);
            assert!(!api.is_null());

            let c = pa_context_new(api, b"mainloop test\0".as_ptr() as *const i8);
            assert!(!c.is_null());

            pa_context_set_state_callback(c, state_changed_callback, ptr::null_mut());

            let res = pa_context_connect(c, ptr::null(), 0, ptr::null());
            assert!(res >= 0);

            // Option 1
            //pa_mainloop_run(mainloop, ptr::null_mut());

            // Option 2
            //loop {
            //    if pa_mainloop_iterate(mainloop, 0, ptr::null_mut()) < 0 {
            //        break;
            //    }
            //}

            // Option 3
            let mut n = 0;
            loop {
                if pa_mainloop_prepare(mainloop, 1000 /*1000 microseconds = 1ms*/) < 0 {
                    break;
                }
                if pa_mainloop_poll(mainloop) < 0 {
                    break;
                }
                if pa_mainloop_dispatch(mainloop) < 0 {
                    break;
                }

                // Exit after 1000 ms
                if n > 1000 {
                    pa_mainloop_quit(mainloop, 1);
                }
                n += 1;
            }

            pa_mainloop_free(mainloop);
        }
    }

    #[test]
    fn introspection() {
        unsafe {
            let mainloop = pa_threaded_mainloop_new();
            assert!(!mainloop.is_null());

            let api = pa_threaded_mainloop_get_api(mainloop);
            assert!(!api.is_null());

            let c = pa_context_new(api, b"introspection test\0".as_ptr() as *const i8);
            assert!(!c.is_null());

            pa_context_set_state_callback(c, state_changed_callback, ptr::null_mut());

            let res = pa_context_connect(c, ptr::null(), 0, ptr::null());
            assert!(res >= 0);

            pa_threaded_mainloop_start(mainloop);

            std::thread::sleep(std::time::Duration::from_millis(100));

            pa_threaded_mainloop_stop(mainloop);
            pa_context_disconnect(c);
            pa_threaded_mainloop_free(mainloop);
        }
    }

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
