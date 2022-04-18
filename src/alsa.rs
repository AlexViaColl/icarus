#![allow(non_camel_case_types)]

#[link(name = "asound")]
extern "C" {
    pub fn snd_pcm_open(pcmp: *mut *mut snd_pcm_t, name: *const i8, stream: snd_pcm_stream_t, mode: i32) -> i32;
    pub fn snd_pcm_close(pcm: *mut snd_pcm_t) -> i32;

    pub fn snd_pcm_hw_params_malloc(ptr: *mut *mut snd_pcm_hw_params_t) -> i32;
    pub fn snd_pcm_hw_params_free(obj: *mut snd_pcm_hw_params_t);

    pub fn snd_pcm_hw_params_any(pcm: *mut snd_pcm_t, params: *mut snd_pcm_hw_params_t) -> i32;
    pub fn snd_pcm_hw_params_set_access(
        pcm: *mut snd_pcm_t,
        params: *mut snd_pcm_hw_params_t,
        access: snd_pcm_access_t,
    ) -> i32;
    pub fn snd_pcm_hw_params_set_format(
        pcm: *mut snd_pcm_t,
        params: *mut snd_pcm_hw_params_t,
        format: snd_pcm_format_t,
    ) -> i32;
    pub fn snd_pcm_hw_params_set_channels(pcm: *mut snd_pcm_t, params: *mut snd_pcm_hw_params_t, val: u32) -> i32;
    pub fn snd_pcm_hw_params_set_rate(
        pcm: *mut snd_pcm_t,
        params: *mut snd_pcm_hw_params_t,
        value: u32,
        dir: i32,
    ) -> i32;
    pub fn snd_pcm_hw_params_set_periods(
        pcm: *mut snd_pcm_t,
        params: *mut snd_pcm_hw_params_t,
        val: u32,
        dir: i32,
    ) -> i32;
    pub fn snd_pcm_hw_params_set_period_time(
        pcm: *mut snd_pcm_t,
        params: *mut snd_pcm_hw_params_t,
        val: u32,
        dir: i32,
    ) -> i32;

    pub fn snd_pcm_hw_params(pcm: *mut snd_pcm_t, params: *mut snd_pcm_hw_params_t) -> i32;

    pub fn snd_pcm_writei(
        pcm: *mut snd_pcm_t,
        buffer: *const std::ffi::c_void,
        size: snd_pcm_uframes_t,
    ) -> snd_pcm_sframes_t;

    pub fn snd_pcm_drain(pcm: *mut snd_pcm_t) -> i32;
}

pub type snd_pcm_sframes_t = i64;
pub type snd_pcm_uframes_t = u64;

#[repr(C)]
pub struct snd_pcm_t_ {
    _data: [u8; 0],
    _marker: core::marker::PhantomData<(*mut u8, core::marker::PhantomPinned)>,
}

#[repr(transparent)]
pub struct snd_pcm_t(*mut snd_pcm_t_);
impl Default for snd_pcm_t {
    fn default() -> Self {
        Self(std::ptr::null_mut())
    }
}

#[repr(C)]
pub struct snd_pcm_hw_params_t_ {
    _data: [u8; 0],
    _marker: core::marker::PhantomData<(*mut u8, core::marker::PhantomPinned)>,
}

#[repr(transparent)]
pub struct snd_pcm_hw_params_t(*mut snd_pcm_hw_params_t_);
impl Default for snd_pcm_hw_params_t {
    fn default() -> Self {
        Self(std::ptr::null_mut())
    }
}

#[repr(C)]
pub enum snd_pcm_stream_t {
    SND_PCM_STREAM_PLAYBACK,
    SND_PCM_STREAM_CAPTURE,
}
pub use snd_pcm_stream_t::*;

#[repr(C)]
pub enum snd_pcm_access_t {
    SND_PCM_ACCESS_MMAP_INTERLEAVED,
    SND_PCM_ACCESS_MMAP_NONINTERLEAVED,
    SND_PCM_ACCESS_MMAP_COMPLEX,
    SND_PCM_ACCESS_RW_INTERLEAVED,
    SND_PCM_ACCESS_RW_NONINTERLEAVED,
}
pub use snd_pcm_access_t::*;

#[repr(C)]
pub enum snd_pcm_format_t {
    SND_PCM_FORMAT_UNKNOWN,
    SND_PCM_FORMAT_S8,
    SND_PCM_FORMAT_U8,
    SND_PCM_FORMAT_S16_LE,
    SND_PCM_FORMAT_S16_BE,
    SND_PCM_FORMAT_U16_LE,
    SND_PCM_FORMAT_U16_BE,
    SND_PCM_FORMAT_S24_LE,
    SND_PCM_FORMAT_S24_BE,
    SND_PCM_FORMAT_U24_LE,
    SND_PCM_FORMAT_U24_BE,
    SND_PCM_FORMAT_S32_LE,
    SND_PCM_FORMAT_S32_BE,
    SND_PCM_FORMAT_U32_LE,
    SND_PCM_FORMAT_U32_BE,
    SND_PCM_FORMAT_FLOAT_LE,
    SND_PCM_FORMAT_FLOAT_BE,
    SND_PCM_FORMAT_FLOAT64_LE,
    SND_PCM_FORMAT_FLOAT64_BE,
    SND_PCM_FORMAT_IEC958_SUBFRAME_LE,
    SND_PCM_FORMAT_IEC958_SUBFRAME_BE,
    SND_PCM_FORMAT_MU_LAW,
    SND_PCM_FORMAT_A_LAW,
    SND_PCM_FORMAT_IMA_ADPCM,
    SND_PCM_FORMAT_MPEG,
    SND_PCM_FORMAT_GSM,
    SND_PCM_FORMAT_S20_LE,
    SND_PCM_FORMAT_S20_BE,
    SND_PCM_FORMAT_U20_LE,
    SND_PCM_FORMAT_U20_BE,
    SND_PCM_FORMAT_SPECIAL,
    SND_PCM_FORMAT_S24_3LE,
    SND_PCM_FORMAT_S24_3BE,
    SND_PCM_FORMAT_U24_3LE,
    SND_PCM_FORMAT_U24_3BE,
    SND_PCM_FORMAT_S20_3LE,
    SND_PCM_FORMAT_S20_3BE,
    SND_PCM_FORMAT_U20_3LE,
    SND_PCM_FORMAT_U20_3BE,
    SND_PCM_FORMAT_S18_3LE,
    SND_PCM_FORMAT_S18_3BE,
    SND_PCM_FORMAT_U18_3LE,
    SND_PCM_FORMAT_U18_3BE,
    SND_PCM_FORMAT_S16,
    SND_PCM_FORMAT_U16,
    SND_PCM_FORMAT_S24,
    SND_PCM_FORMAT_U24,
    SND_PCM_FORMAT_S32,
    SND_PCM_FORMAT_U32,
    SND_PCM_FORMAT_FLOAT,
    SND_PCM_FORMAT_FLOAT64,
    SND_PCM_FORMAT_IEC958_SUBFRAME,
    SND_PCM_FORMAT_S20,
    SND_PCM_FORMAT_U20,
}
pub use snd_pcm_format_t::*;

#[cfg(test)]
mod tests {
    use super::*;
    use std::ptr;

    #[test]
    fn alsa() {
        let mut pcm = ptr::null_mut();
        unsafe { snd_pcm_open(&mut pcm, b"default\0".as_ptr() as *const i8, SND_PCM_STREAM_PLAYBACK, 0) };

        let mut hw_params = ptr::null_mut();
        unsafe { snd_pcm_hw_params_malloc(&mut hw_params) };

        unsafe { snd_pcm_hw_params_set_access(pcm, hw_params, SND_PCM_ACCESS_RW_INTERLEAVED) };
        unsafe { snd_pcm_hw_params_set_format(pcm, hw_params, SND_PCM_FORMAT_S16_LE) };
        unsafe { snd_pcm_hw_params_set_channels(pcm, hw_params, 1) };
        unsafe { snd_pcm_hw_params_set_rate(pcm, hw_params, 48000, 0) };
        unsafe { snd_pcm_hw_params_set_periods(pcm, hw_params, 10, 0) };
        unsafe { snd_pcm_hw_params_set_period_time(pcm, hw_params, 100000, 0) }; // 0.1 seconds

        unsafe { snd_pcm_hw_params(pcm, hw_params) };

        unsafe { snd_pcm_hw_params_free(hw_params) };

        let mut samples = [0_i16; 48000];

        for i in 0..48000 {
            samples[i] = (10000.0 * (2.0 * std::f32::consts::PI * 200.0 * (i as f32 / 48000.0)).sin()) as i16;
        }

        unsafe { snd_pcm_writei(pcm, samples.as_ptr() as *const std::ffi::c_void, 48000) };

        unsafe { snd_pcm_drain(pcm) };

        unsafe { snd_pcm_close(pcm) };
    }
}
