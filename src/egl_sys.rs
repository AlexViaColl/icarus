#![allow(non_camel_case_types)]

use std::ffi::c_void;
use std::ptr;

pub type EGLBoolean = u32;
pub type EGLint = i32;
pub type EGLenum = u32;
pub type EGLDisplay = *mut c_void;
pub type EGLConfig = *mut c_void;
pub type EGLSurface = *mut c_void;
pub type EGLContext = *mut c_void;

pub type __eglMustCastToProperFunctionPointerType = extern "C" fn();

pub type EGLNativeDisplayType = *mut crate::x11_sys::Display;
pub type EGLNativePixmapType = crate::x11_sys::Pixmap;
pub type EGLNativeWindowType = crate::x11_sys::Window;

pub type EGLClientBuffer = *mut c_void;
pub type EGLSync = *mut c_void;
pub type EGLAttrib = isize;
pub type EGLTime = u64;
pub type EGLImage = *mut c_void;

#[link(name = "EGL")]
extern "C" {
    // 1.0
    pub fn eglChooseConfig(
        dpy: EGLDisplay,
        attrib_list: *const EGLint,
        configs: *mut EGLConfig,
        config_size: EGLint,
        num_config: *mut EGLint,
    ) -> EGLBoolean;
    pub fn eglCopyBuffers(dpy: EGLDisplay, surface: EGLSurface, target: EGLNativePixmapType) -> EGLBoolean;
    pub fn eglCreateContext(
        dpy: EGLDisplay,
        config: EGLConfig,
        share_context: EGLContext,
        attrib_list: *const EGLint,
    ) -> EGLContext;
    pub fn eglCreatePbufferSurface(dpy: EGLDisplay, config: EGLConfig, attrib_list: *const EGLint) -> EGLSurface;
    pub fn eglCreatePixmapSurface(
        dpy: EGLDisplay,
        config: EGLConfig,
        pixmap: EGLNativePixmapType,
        attrib_list: *const EGLint,
    ) -> EGLSurface;
    pub fn eglCreateWindowSurface(
        dpy: EGLDisplay,
        config: EGLConfig,
        win: EGLNativeWindowType,
        attrib_list: *const EGLint,
    ) -> EGLSurface;
    pub fn eglDestroyContext(dpy: EGLDisplay, ctx: EGLContext) -> EGLBoolean;
    pub fn eglDestroySurface(dpy: EGLDisplay, surface: EGLSurface) -> EGLBoolean;
    pub fn eglGetConfigAttrib(dpy: EGLDisplay, config: EGLConfig, attribute: EGLint, value: *mut EGLint) -> EGLBoolean;
    pub fn eglGetConfigs(
        dpy: EGLDisplay,
        configs: *mut EGLConfig,
        config_size: EGLint,
        num_config: *mut EGLint,
    ) -> EGLBoolean;
    pub fn eglGetCurrentDisplay() -> EGLDisplay;
    pub fn eglGetCurrentSurface(readdraw: EGLint) -> EGLSurface;
    pub fn eglGetDisplay(display_id: EGLNativeDisplayType) -> EGLDisplay;
    pub fn eglGetError() -> EGLint;
    pub fn eglGetProcAddress(procname: *const i8) -> __eglMustCastToProperFunctionPointerType;
    pub fn eglInitialize(dpy: EGLDisplay, major: *mut EGLint, minor: *mut EGLint) -> EGLBoolean;
    pub fn eglMakeCurrent(dpy: EGLDisplay, draw: EGLSurface, read: EGLSurface, ctx: EGLContext) -> EGLBoolean;
    pub fn eglQueryContext(dpy: EGLDisplay, ctx: EGLContext, attribute: EGLint, value: *mut EGLint) -> EGLBoolean;
    pub fn eglQueryString(dpy: EGLDisplay, name: EGLint) -> *const i8;
    pub fn eglQuerySurface(dpy: EGLDisplay, surface: EGLSurface, attribute: EGLint, value: *mut EGLint) -> EGLBoolean;
    pub fn eglSwapBuffers(dpy: EGLDisplay, surface: EGLSurface) -> EGLBoolean;
    pub fn eglTerminate(dpy: EGLDisplay) -> EGLBoolean;
    pub fn eglWaitGL() -> EGLBoolean;
    pub fn eglWaitNative(engine: EGLint) -> EGLBoolean;

    // 1.1
    pub fn eglBindTexImage(dpy: EGLDisplay, surface: EGLSurface, buffer: EGLint) -> EGLBoolean;
    pub fn eglReleaseTexImage(dpy: EGLDisplay, surface: EGLSurface, buffer: EGLint) -> EGLBoolean;
    pub fn eglSurfaceAttrib(dpy: EGLDisplay, surface: EGLSurface, attribute: EGLint, value: EGLint) -> EGLBoolean;
    pub fn eglSwapInterval(dpy: EGLDisplay, interval: EGLint) -> EGLBoolean;

    // 1.2
    pub fn eglBindAPI(api: EGLenum) -> EGLBoolean;
    pub fn eglQueryAPI() -> EGLenum;
    pub fn eglCreatePbufferFromClientBuffer(
        dpy: EGLDisplay,
        buftype: EGLenum,
        buffer: EGLClientBuffer,
        config: EGLConfig,
        attrib_list: *const EGLint,
    ) -> EGLSurface;
    pub fn eglReleaseThread() -> EGLBoolean;
    pub fn eglWaitClient() -> EGLBoolean;

    // 1.3

    // 1.4
    pub fn eglGetCurrentContext() -> EGLContext;

    // 1.5
    pub fn eglCreateSync(dpy: EGLDisplay, ttype: EGLenum, attrib_list: *const EGLAttrib) -> EGLSync;
    pub fn eglDestroySync(dpy: EGLDisplay, sync: EGLSync) -> EGLBoolean;
    pub fn eglClientWaitSync(dpy: EGLDisplay, sync: EGLSync, flags: EGLint, timeout: EGLTime) -> EGLint;
    pub fn eglGetSyncAttrib(dpy: EGLDisplay, sync: EGLSync, attribute: EGLint, value: *mut EGLAttrib) -> EGLBoolean;
    pub fn eglCreateImage(
        dpy: EGLDisplay,
        ctx: EGLContext,
        target: EGLenum,
        buffer: EGLClientBuffer,
        attrib_list: *const EGLAttrib,
    ) -> EGLImage;
    pub fn eglDestroyImage(dpy: EGLDisplay, iamge: EGLImage) -> EGLBoolean;
    pub fn eglGetPlatformDisplay(
        platform: EGLenum,
        native_display: *mut c_void,
        attrib_list: *const EGLAttrib,
    ) -> EGLDisplay;
    pub fn eglCreatePlatformWindowSurface(
        dpy: EGLDisplay,
        config: EGLConfig,
        native_window: *mut c_void,
        attrib_list: *const EGLAttrib,
    ) -> EGLSurface;
    pub fn eglCreatePlatformPixmapSurface(
        dpy: EGLDisplay,
        config: EGLConfig,
        native_pixmap: *mut c_void,
        attrib_list: *const EGLAttrib,
    ) -> EGLSurface;
    pub fn eglWaitSync(dpy: EGLDisplay, sync: EGLSync, flags: EGLint) -> EGLBoolean;
}

pub const EGL_DONT_CARE: EGLint = -1;
pub const EGL_FALSE: EGLint = 0;
pub const EGL_TRUE: EGLint = 1;

pub const EGL_DEFAULT_DISPLAY: EGLNativeDisplayType = ptr::null_mut();

pub const EGL_OPENGL_ES_API: EGLenum = 0x30A0;
pub const EGL_OPENVG_API: EGLenum = 0x30A0;
pub const EGL_OPENGL_API: EGLenum = 0x30A2;

// eglGetConfigAttrib
pub const EGL_BUFFER_SIZE: EGLint = 0x3020;
pub const EGL_ALPHA_SIZE: EGLint = 0x3021;
pub const EGL_BLUE_SIZE: EGLint = 0x3022;
pub const EGL_GREEN_SIZE: EGLint = 0x3023;
pub const EGL_RED_SIZE: EGLint = 0x3024;
pub const EGL_DEPTH_SIZE: EGLint = 0x3025;
pub const EGL_STENCIL_SIZE: EGLint = 0x3026;
pub const EGL_CONFIG_CAVEAT: EGLint = 0x3027;
pub const EGL_CONFIG_ID: EGLint = 0x3028;
pub const EGL_LEVEL: EGLint = 0x3029;
pub const EGL_MAX_PBUFFER_HEIGHT: EGLint = 0x302A;
pub const EGL_MAX_PBUFFER_PIXELS: EGLint = 0x302B;
pub const EGL_MAX_PBUFFER_WIDTH: EGLint = 0x302C;
pub const EGL_NATIVE_RENDERABLE: EGLint = 0x302D;
pub const EGL_NATIVE_VISUAL_ID: EGLint = 0x302E;
pub const EGL_NATIVE_VISUAL_TYPE: EGLint = 0x302F;
pub const EGL_SAMPLES: EGLint = 0x3031;
pub const EGL_SAMPLE_BUFFERS: EGLint = 0x3032;
pub const EGL_SURFACE_TYPE: EGLint = 0x3033;
pub const EGL_TRANSPARENT_TYPE: EGLint = 0x3034;
pub const EGL_TRANSPARENT_BLUE_VALUE: EGLint = 0x3035;
pub const EGL_TRANSPARENT_GREEN_VALUE: EGLint = 0x3036;
pub const EGL_TRANSPARENT_RED_VALUE: EGLint = 0x3037;
pub const EGL_NONE: EGLint = 0x3038;
pub const EGL_BIND_TO_TEXTURE_RGB: EGLint = 0x3039;
pub const EGL_BIND_TO_TEXTURE_RGBA: EGLint = 0x303A;
pub const EGL_MIN_SWAP_INTERVAL: EGLint = 0x303B;
pub const EGL_MAX_SWAP_INTERVAL: EGLint = 0x303C;
pub const EGL_LUMINANCE_SIZE: EGLint = 0x303D;
pub const EGL_ALPHA_MASK_SIZE: EGLint = 0x303E;
pub const EGL_COLOR_BUFFER_TYPE: EGLint = 0x303F;
pub const EGL_RENDERABLE_TYPE: EGLint = 0x3040;
pub const EGL_MATCH_NATIVE_PIXMAP: EGLint = 0x3041; // Can be specified in eglChooseConfig but not in eglGetConfigAttrib
pub const EGL_CONFORMANT: EGLint = 0x3042;

pub const EGL_OPENGL_BIT: EGLint = 0x0008;
pub const EGL_OPENGL_ES_BIT: EGLint = 0x0001;
pub const EGL_OPENGL_ES2_BIT: EGLint = 0x0004;
pub const EGL_OPENVG_BIT: EGLint = 0x0002;

// eglCreateContext
pub const EGL_CONTEXT_MAJOR_VERSION: EGLint = 0x3098;
pub const EGL_CONTEXT_MINOR_VERSION: EGLint = 0x30FB;
pub const EGL_CONTEXT_OPENGL_PROFILE_MASK: EGLint = 0x30FD;
pub const EGL_CONTEXT_OPENGL_CORE_PROFILE_BIT: EGLint = 0x00000001;
pub const EGL_CONTEXT_OPENGL_COMPATIBILITY_PROFILE_BIT: EGLint = 0x00000002;
pub const EGL_CONTEXT_OPENGL_DEBUG: EGLint = 0x31B0;
pub const EGL_CONTEXT_OPENGL_FORWARD_COMPATIBLE: EGLint = 0x31B1;
pub const EGL_CONTEXT_OPENGL_ROBUST_ACCESS: EGLint = 0x31B2;

pub const EGL_PLATFORM_X11_EXT: EGLenum = 0x31D5;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::x11_sys::*;
    use std::mem;

    #[test]
    #[ignore]
    fn it_works() {
        unsafe {
            let dpy = eglGetDisplay(EGL_DEFAULT_DISPLAY);
            assert!(!dpy.is_null());
            let mut major = 0;
            let mut minor = 0;
            assert_ne!(eglInitialize(dpy, &mut major, &mut minor), 0);
            println!("EGL {}.{}", major, minor);

            assert_ne!(eglBindAPI(EGL_OPENGL_ES_API), 0);

            let mut configs_count = 0;
            assert_ne!(eglGetConfigs(dpy, ptr::null_mut(), 0, &mut configs_count), 0);
            println!("Configs count = {}", configs_count);

            let mut configs = vec![ptr::null_mut(); configs_count as usize];
            assert_ne!(
                eglGetConfigs(
                    dpy,
                    configs.as_mut_ptr(),
                    mem::size_of::<EGLConfig>() as i32 * configs_count,
                    &mut configs_count
                ),
                0
            );

            macro_rules! attr {
                ($x: expr) => {{
                    ($x, stringify!($x))
                }};
            }
            for i in 0..configs_count as usize {
                for attr in [
                    attr!(EGL_CONFIG_ID),
                    attr!(EGL_CONFORMANT),
                    attr!(EGL_NATIVE_VISUAL_ID),
                    attr!(EGL_NATIVE_VISUAL_TYPE),
                    attr!(EGL_SURFACE_TYPE),
                ] {
                    let mut value = 0;
                    eglGetConfigAttrib(dpy, configs[i], attr.0, &mut value);
                    println!("{} = {:x}", attr.1, value);
                }
                println!();
            }

            let config = configs[0];

            #[rustfmt::skip]
            let ctx = eglCreateContext(dpy, config, ptr::null_mut(), [
                EGL_CONTEXT_MAJOR_VERSION, 1,
                EGL_CONTEXT_MINOR_VERSION, 0,
                //EGL_CONTEXT_OPENGL_PROFILE_MASK, EGL_CONTEXT_OPENGL_CORE_PROFILE_BIT, // or EGL_CONTEXT_OPENGL_COMPATIBILITY_PROFILE_BIT
                //EGL_CONTEXT_OPENGL_DEBUG, EGL_TRUE,
                //EGL_CONTEXT_OPENGL_FORWARD_COMPATIBLE, EGL_TRUE,
                //EGL_CONTEXT_OPENGL_ROBUST_ACCESS, EGL_TRUE,
                //EGL_CONTEXT_OPENGL_RESET_NOTIFICATION_STRATEGY, EGL_LOSE_CONTEXT_ON_RESET,
                EGL_NONE,
            ].as_ptr());
            assert!(!ctx.is_null());

            let display = XOpenDisplay(ptr::null());
            let _window = XCreateSimpleWindow(display, XDefaultRootWindow(display), 0, 0, 800, 600, 0, 0, 0);
            //let surface = eglCreateWindowSurface(dpy, config, window, ptr::null());
            //assert!(!surface.is_null());

            //assert_ne!(eglMakeCurrent(dpy, surface, surface, ctx), 0);

            assert_ne!(eglDestroyContext(dpy, ctx), 0);
            assert_ne!(eglTerminate(dpy), 0);
        }
    }
}
