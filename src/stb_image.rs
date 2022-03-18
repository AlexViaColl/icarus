#![allow(non_camel_case_types)]

#[link(name = "stb_image")]
extern "C" {
    pub fn stbi_load(
        filename: *const i8,
        x: *mut i32,
        y: *mut i32,
        channels: *mut i32,
        desired_channels: i32,
    ) -> *mut stbi_uc;
    pub fn stbi_load_from_memory(
        buffer: *const stbi_uc,
        len: i32,
        x: *mut i32,
        y: *mut i32,
        comp: *mut i32,
        req_comp: i32,
    ) -> *mut stbi_uc;
    pub fn stbi_image_free(retval_from_stbi_load: *mut std::ffi::c_void);
}

pub type stbi_uc = u8;
