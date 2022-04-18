#![allow(non_camel_case_types)]

#[link(name = "pulse-simple")]
extern "C" {
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
    pub fn pa_simple_read(s: *mut pa_simple, data: *mut std::ffi::c_void, bytes: usize, error: *mut i32) -> i32;
    pub fn pa_simple_write(s: *mut pa_simple, data: *const std::ffi::c_void, bytes: usize, error: *mut i32) -> i32;
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ptr;

    #[test]
    fn pulseaudio() {
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

        let mut buffer = [0_i16; 44100 * 2];
        for i in 0..44100 {
            buffer[2 * i] = (10000.0 * (2.0 * std::f32::consts::PI * 200.0 * (i as f32 / 44100.0)).sin()) as i16;
            buffer[2 * i + 1] = (10000.0 * (2.0 * std::f32::consts::PI * 200.0 * (i as f32 / 44100.0)).sin()) as i16;
        }
        unsafe { pa_simple_write(s, buffer.as_ptr() as *const std::ffi::c_void, buffer.len() * 2, ptr::null_mut()) };

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
pub struct pa_sample_spec {
    pub format: pa_sample_format_t,
    pub rate: u32,
    pub channels: u8,
}

pub const PA_CHANNELS_MAX: usize = 32;

#[repr(C)]
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
