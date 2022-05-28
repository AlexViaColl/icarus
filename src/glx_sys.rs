#![allow(non_snake_case)]

use crate::gl_sys::{GLboolean, GLdouble, GLint, GLubyte, GLuint};
use crate::opaque;
use crate::x11_sys::{Bool, Display, Font, Pixmap, VisualID, Window, XVisualInfo, XID};

use std::ffi::c_void;

#[link(name = "GLX")]
extern "C" {
    // 1.0
    pub fn glXChooseVisual(dpy: *mut Display, screen: i32, attriblist: *mut i32) -> *mut XVisualInfo;
    pub fn glXCreateContext(
        dpy: *mut Display,
        vis: *mut XVisualInfo,
        shareList: GLXContext,
        direct: Bool,
    ) -> GLXContext;
    pub fn glXDestroyContext(dpy: *mut Display, ctx: GLXContext);
    pub fn glXMakeCurrent(dpy: *mut Display, drawable: GLXDrawable, ctx: GLXContext) -> Bool;
    pub fn glXCopyContext(dpy: *mut Display, src: GLXContext, dst: GLXContext, mask: u64);
    pub fn glXSwapBuffers(dpy: *mut Display, drawable: GLXDrawable);
    pub fn glXCreateGLXPixmap(dpy: *mut Display, visual: *mut XVisualInfo, pixmap: Pixmap) -> GLXPixmap;
    pub fn glXDestroyGLXPixmap(dpy: *mut Display, pixmap: GLXPixmap);
    pub fn glXQueryExtension(dpy: *mut Display, errorb: *mut i32, event: *mut i32) -> Bool;
    pub fn glXQueryVersion(dpy: *mut Display, maj: *mut i32, min: *mut i32) -> Bool;
    pub fn glXIsDirect(dpy: *mut Display, ctx: GLXContext) -> Bool;
    pub fn glXGetConfig(dpy: *mut Display, visual: *mut XVisualInfo, attrib: i32, value: *mut i32) -> i32;
    pub fn glXGetCurrentContext() -> GLXContext;
    pub fn glXGetCurrentDrawable() -> GLXDrawable;
    pub fn glXWaitGL();
    pub fn glXWaitX();
    pub fn glXUseXFont(font: Font, first: i32, count: i32, list: i32);

    // 1.1
    pub fn glXQueryExtensionsString(dpy: *mut Display, screen: i32) -> *const i8;
    pub fn glXQueryServerString(dpy: *mut Display, screen: i32, name: i32) -> *const i8;
    pub fn glXGetClientString(dpy: *mut Display, name: i32) -> *const i8;

    // 1.2
    pub fn glXGetCurrentDisplay() -> *mut Display;

    // 1.3
    pub fn glXChooseFBConfig(
        dpy: *mut Display,
        screen: i32,
        attribList: *const i32,
        nitems: *mut i32,
    ) -> *mut GLXFBConfig;
    pub fn glXGetFBConfigAttrib(dpy: *mut Display, config: GLXFBConfig, attribute: i32, value: *mut i32) -> i32;
    pub fn glXGetFBConfigs(dpy: *mut Display, screen: i32, nelements: *mut i32) -> *mut GLXFBConfig;
    pub fn glXGetVisualFromFBConfig(dpy: *mut Display, config: GLXFBConfig) -> *mut XVisualInfo;
    pub fn glXCreateWindow(dpy: *mut Display, config: GLXFBConfig, win: Window, attribList: *const i32) -> GLXWindow;
    pub fn glXDestroyWindow(dpy: *mut Display, window: GLXWindow);
    pub fn glXCreatePixmap(dpy: *mut Display, config: GLXFBConfig, pixmap: Pixmap, attribList: *const i32)
        -> GLXPixmap;
    pub fn glXDestroyPixmap(dpy: *mut Display, pixmap: GLXPixmap);
    pub fn glXCreatePbuffer(dpy: *mut Display, config: GLXFBConfig, attribList: *const i32) -> GLXPbuffer;
    pub fn glXDestroyPbuffer(dpy: *mut Display, pbuf: GLXPbuffer);
    pub fn glXQueryDrawable(dpy: *mut Display, draw: GLXDrawable, attribute: i32, value: *mut u32);
    pub fn glXCreateNewContext(
        dpy: *mut Display,
        config: GLXFBConfig,
        renderType: i32,
        shareList: GLXContext,
        direct: Bool,
    ) -> GLXContext;
    pub fn glXMakeContextCurrent(dpy: *mut Display, draw: GLXDrawable, read: GLXDrawable, ctx: GLXContext) -> Bool;
    pub fn glXGetCurrentReadDrawable() -> GLXDrawable;
    pub fn glXQueryContext(dpy: *mut Display, ctx: GLXContext, attribute: i32, value: *mut i32) -> i32;
    pub fn glXSelectEvent(dpy: *mut Display, drawable: GLXDrawable, mask: u64);
    pub fn glXGetSelectedEvent(dpy: *mut Display, drawable: GLXDrawable, mask: *mut u64);

    // 1.4
    pub fn glXGetProcAddres(procname: *const GLubyte) -> *mut c_void;

    // Extensions:
    // GLX_ARB_create_context
    pub fn glXCreateContextAttribsARB(
        dpy: *mut Display,
        config: GLXFBConfig,
        share_context: GLXContext,
        direct: Bool,
        attrib_list: *const i32,
    ) -> GLXContext;

    // GLX_NV_vertex_array_range
    // pub fn glXAllocateMemoryNV(...);
    // pub fn glXFreeMemoryNV(...);

    // GLX_ARB_render_texture
    // GLX_NV_float_buffer
    // GLX_MESA_swap_frame_usage
    // GLX_MESA_swap_control
    // GLX_EXT_texture_from_pixmap
}

opaque!(GLXContext, __GLXcontextRec);
pub type GLXPixmap = XID;
pub type GLXDrawable = XID;
pub type GLXFBConfigID = XID;
pub type GLXContextID = XID;
pub type GLXWindow = XID;
pub type GLXPbuffer = XID;
pub type GLXFBConfig = *mut __GLXFBConfigRec;

pub const GLX_VENDOR: i32 = 1;
pub const GLX_VERSION: i32 = 2;
pub const GLX_EXTENSIONS: i32 = 3;

pub const GLX_USE_GL: i32 = 1;
pub const GLX_BUFFER_SIZE: i32 = 2;
pub const GLX_LEVEL: i32 = 3;
pub const GLX_RGBA: i32 = 4;
pub const GLX_DOUBLEBUFFER: i32 = 5;
pub const GLX_STEREO: i32 = 6;
pub const GLX_AUX_BUFFERS: i32 = 7;
pub const GLX_RED_SIZE: i32 = 8;
pub const GLX_GREEN_SIZE: i32 = 9;
pub const GLX_BLUE_SIZE: i32 = 10;
pub const GLX_ALPHA_SIZE: i32 = 11;
pub const GLX_DEPTH_SIZE: i32 = 12;
pub const GLX_STENCIL_SIZE: i32 = 13;
pub const GLX_ACCUM_RED_SIZE: i32 = 14;
pub const GLX_ACCUM_GREEN_SIZE: i32 = 15;
pub const GLX_ACCUM_BLUE_SIZE: i32 = 16;
pub const GLX_ACCUM_ALPHA_SIZE: i32 = 17;

pub const GLX_X_VISUAL_TYPE: i32 = 0x22;

pub const GLX_DRAWABLE_TYPE: i32 = 0x8010;
pub const GLX_RENDER_TYPE: i32 = 0x8011;
pub const GLX_X_RENDERABLE: i32 = 0x8012;
pub const GLX_FBCONFIG_ID: i32 = 0x8013;

pub const GLX_WINDOW_BIT: i32 = 0x00000001;
pub const GLX_PIXMAP_BIT: i32 = 0x00000002;
pub const GLX_PBUFFER_BIT: i32 = 0x00000004;
pub const GLX_AUX_BUFFERS_BIT: i32 = 0x00000010;

pub const GLX_RGBA_BIT: i32 = 0x00000001;
pub const GLX_COLOR_INDEX_BIT: i32 = 0x00000002;

pub const GLX_SLOW_CONFIG: i32 = 0x8001;
pub const GLX_TRUE_COLOR: i32 = 0x8002;
pub const GLX_DIRECT_COLOR: i32 = 0x8003;
pub const GLX_PSEUDO_COLOR: i32 = 0x8004;
pub const GLX_STATIC_COLOR: i32 = 0x8005;
pub const GLX_GRAY_SCALE: i32 = 0x8006;
pub const GLX_STATIC_GRAY: i32 = 0x8007;
pub const GLX_TRANSPARENT_RGB: i32 = 0x8008;
pub const GLX_TRANSPARENT_INDEX: i32 = 0x8009;

#[repr(C)]
#[derive(Debug)]
pub struct __GLXvisualConfigRec {
    pub vid: VisualID,
    pub class: i32,
    pub rgba: Bool,
    pub redSize: i32,
    pub greenSize: i32,
    pub blueSize: i32,
    pub alphaSize: i32,
    pub redMask: u64,
    pub greenMask: u64,
    pub blueMask: u64,
    pub alphaMask: u64,
    pub accumRedSize: i32,
    pub accumGreenSize: i32,
    pub accumBlueSize: i32,
    pub accumAlphaSize: i32,
    pub doubleBuffer: Bool,
    pub stereo: Bool,
    pub bufferSize: i32,
    pub depthSize: i32,
    pub stencilSize: i32,
    pub auxBuffers: i32,
    pub level: i32,
    pub visualRating: i32,
    pub transparentPixel: i32,
    pub transparentRed: i32,
    pub transparentGreen: i32,
    pub transparentBlue: i32,
    pub transparentAlpha: i32,
    pub transparentIndex: i32,
    pub multiSampleSize: i32,
    pub nMultiSampleBuffers: i32,
    pub visualSelectGroup: i32,
}

#[repr(C)]
#[derive(Debug)]
pub struct __GLXFBConfigRec {
    pub visualType: i32,
    pub transparentType: i32,
    pub transparentRed: i32,
    pub transparentGreen: i32,
    pub transparentBlue: i32,
    pub transparentAlpha: i32,
    pub transparentIndex: i32,
    pub visualCaveat: i32,
    pub associatedVisualId: i32,
    pub screen: i32,
    pub drawableType: i32,
    pub renderType: i32,
    pub maxPbufferWidth: i32,
    pub maxPbufferHeight: i32,
    pub maxPbufferPixels: i32,
    pub optimalPbufferWidth: i32,
    pub optimalPbufferHeight: i32,
    pub visualSelectGroup: i32,
    pub id: u32,
    pub rgbMode: GLboolean,
    pub colorIndexMode: GLboolean,
    pub doubleBufferMode: GLboolean,
    pub stereoMode: GLboolean,
    pub haveAccumBuffer: GLboolean,
    pub haveDepthBuffer: GLboolean,
    pub haveStencilBuffer: GLboolean,
    pub accumRedBits: GLint,
    pub accumGreenBits: GLint,
    pub accumBlueBits: GLint,
    pub accumAlphaBits: GLint,
    pub depthBits: GLint,
    pub stencilBits: GLint,
    pub indexBits: GLint,
    pub redBits: GLint,
    pub greenBits: GLint,
    pub blueBits: GLint,
    pub alphaBits: GLint,
    pub redMask: GLuint,
    pub greenMask: GLuint,
    pub blueMask: GLuint,
    pub alphaMask: GLuint,
    pub multiSampleSize: GLuint,
    pub nMultiSampleBuffers: GLuint,
    pub maxAuxBuffers: GLint,
    pub level: GLint,
    pub extendedRange: GLboolean,
    pub minRed: GLdouble,
    pub maxRed: GLdouble,
    pub minGreen: GLdouble,
    pub maxGreen: GLdouble,
    pub minBlue: GLdouble,
    pub maxBlue: GLdouble,
    pub minAlpha: GLdouble,
    pub maxAlpha: GLdouble,
}

pub const GLX_CONTEXT_DEBUG_BIT_ARB: i32 = 0x00000001;
pub const GLX_CONTEXT_FORWARD_COMPATIBLE_BIT_ARB: i32 = 0x00000002;
pub const GLX_CONTEXT_MAJOR_VERSION_ARB: i32 = 0x2091;
pub const GLX_CONTEXT_MINOR_VERSION_ARB: i32 = 0x2092;
pub const GLX_CONTEXT_FLAGS_ARG: i32 = 0x2094;

pub const GLX_CONTEXT_CORE_PROFILE_BIT_ARB: i32 = 0x00000001;
pub const GLX_CONTEXT_COMPATIBILITY_PROFILE_BIT_ARG: i32 = 0x00000002;
pub const GLX_CONTEXT_PROFILE_MASK_ARB: i32 = 0x9126;

pub const GLX_CONTEXT_ES_PROFILE_BIT_EXT: i32 = 0x00000004;
pub const GLX_CONTEXT_ES2_PROFILE_BIT_EXT: i32 = 0x00000004;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gl_sys::*;
    use crate::string_util::*;
    use crate::x11_sys::*;

    use std::ptr;

    #[test]
    #[ignore]
    fn gles2_context() {
        unsafe {
            let dpy = XOpenDisplay(ptr::null());
            #[rustfmt::skip]
            let attribs = [
                GLX_X_RENDERABLE,   True,
                GLX_DRAWABLE_TYPE,  GLX_WINDOW_BIT,
                GLX_RENDER_TYPE,    GLX_RGBA_BIT,
                GLX_X_VISUAL_TYPE,  GLX_TRUE_COLOR,
                GLX_RED_SIZE,       8,
                GLX_GREEN_SIZE,     8,
                GLX_BLUE_SIZE,      8,
                GLX_ALPHA_SIZE,     8,
                GLX_DEPTH_SIZE,     24,
                GLX_STENCIL_SIZE,   8,
                GLX_DOUBLEBUFFER,   True,
                //GLX_SAMPLE_BUFFERS, 1,
                //GLX_SAMPLES,        4,
                0,
            ];
            let mut fbcount = 0;
            let fbconfigs = glXChooseFBConfig(dpy, XDefaultScreen(dpy), attribs.as_ptr(), &mut fbcount);
            assert!(!fbconfigs.is_null());
            println!("Found {} matching FB configs.", fbcount);
            let fbconfig = *fbconfigs.offset(0);
            XFree(fbconfigs as *mut c_void);

            let vi = glXGetVisualFromFBConfig(dpy, fbconfig);
            assert!(!vi.is_null());
            println!("Chosen visual ID = 0x{:x}", (*vi).visualid);

            let window = XCreateWindow(
                dpy,
                XRootWindow(dpy, (*vi).screen),
                0,
                0,
                800,
                600,
                0,
                (*vi).depth,
                InputOutput,
                (*vi).visual,
                CWBorderPixel | CWColormap | CWEventMask,
                &mut XSetWindowAttributes {
                    colormap: XCreateColormap(dpy, XRootWindow(dpy, (*vi).screen), (*vi).visual, AllocNone),
                    event_mask: StructureNotifyMask,
                    ..XSetWindowAttributes::default()
                },
            );
            assert!(window != 0);
            XFree(vi as *mut c_void);

            //let window = XCreateSimpleWindow(dpy, XDefaultRootWindow(dpy), 0, 0, 800, 600, 0, 0, 0);

            #[rustfmt::skip]
            let context_attribs = [
                GLX_CONTEXT_MAJOR_VERSION_ARB, 2,
                GLX_CONTEXT_MINOR_VERSION_ARB, 0,
                GLX_CONTEXT_PROFILE_MASK_ARB, GLX_CONTEXT_ES_PROFILE_BIT_EXT,
                0,
            ];
            let ctx = glXCreateContextAttribsARB(dpy, fbconfig, GLXContext::default(), 1, context_attribs.as_ptr());
            assert!(ctx != GLXContext::default());
            assert!(glXMakeCurrent(dpy, window, ctx) != 0);
            XSync(dpy, False);

            let mut value = 0;
            glGetIntegerv(GL_MAJOR_VERSION, &mut value);
            println!("GL_MAJOR_VERSION = {}", value);
            glGetIntegerv(GL_MINOR_VERSION, &mut value);
            println!("GL_MINOR_VERSION = {}", value);

            println!("{}", cstr_to_string(glGetString(GL_VERSION)));

            glGetIntegerv(GL_CONTEXT_PROFILE_MASK, &mut value);
            println!("GL_CONTEXT_PROFILE_MASK = 0x{:x}", value);
            glGetIntegerv(GL_CONTEXT_FLAGS, &mut value);
            println!("GL_CONTEXT_FLAGS = 0x{:x}", value);
            //XMapWindow(dpy, window);
        }
    }

    #[test]
    fn gles3_context() {}

    #[test]
    fn gl46_context() {}

    #[test]
    #[ignore]
    fn it_works() {
        unsafe {
            let dpy = XOpenDisplay(ptr::null());
            let window = XCreateSimpleWindow(dpy, XDefaultRootWindow(dpy), 0, 0, 800, 600, 0, 0, 0);

            let mut major = 0;
            let mut minor = 0;
            assert!(glXQueryVersion(dpy, &mut major, &mut minor) != 0);
            println!("GLX {}.{}", major, minor);

            let mut error_base = 0;
            let mut event_base = 0;
            assert!(glXQueryExtension(dpy, &mut error_base, &mut event_base) != 0);
            println!("glXQueryExtension: error_base {}, event_base: {}", error_base, event_base);

            //println!("glXQueryExtensionsString: {}", cstr_to_string(glXQueryExtensionsString(dpy, 0)));

            println!("glXQueryServerString(GLX_VENDOR): {}", cstr_to_string(glXQueryServerString(dpy, 0, GLX_VENDOR)));
            println!(
                "glXQueryServerString(GLX_VERSION): {}",
                cstr_to_string(glXQueryServerString(dpy, 0, GLX_VERSION))
            );
            //println!(
            //    "glXQueryServerString(GLX_EXTENSIONS): {}",
            //    cstr_to_string(glXQueryServerString(dpy, 0, GLX_EXTENSIONS))
            //);

            println!("glXGetClientSctring(GLX_VENDOR): {}", cstr_to_string(glXGetClientString(dpy, GLX_VENDOR)));
            println!("glXGetClientSctring(GLX_VERSION): {}", cstr_to_string(glXGetClientString(dpy, GLX_VERSION)));

            let mut visualinfos_count = 0;
            let visualinfos = XGetVisualInfo(
                dpy,
                VisualScreenMask,
                &mut XVisualInfo {
                    screen: XDefaultScreen(dpy),
                    ..XVisualInfo::default()
                },
                &mut visualinfos_count,
            );
            println!("#Visual Infos: {}", visualinfos_count); // 132 on my machine

            macro_rules! attr {
                ($x: expr) => {{
                    ($x, stringify!($x))
                }};
            }
            for i in 0..visualinfos_count {
                let vi = visualinfos.offset(i as isize);
                //println!("{:#?}", *vi);

                for attr in [
                    attr!(GLX_USE_GL),
                    attr!(GLX_BUFFER_SIZE),
                    attr!(GLX_LEVEL),
                    attr!(GLX_RGBA),
                    attr!(GLX_DRAWABLE_TYPE),
                    attr!(GLX_DOUBLEBUFFER),
                    attr!(GLX_STEREO),
                    attr!(GLX_AUX_BUFFERS),
                    attr!(GLX_RED_SIZE),
                    attr!(GLX_GREEN_SIZE),
                    attr!(GLX_BLUE_SIZE),
                    attr!(GLX_ALPHA_SIZE),
                    attr!(GLX_DEPTH_SIZE),
                    attr!(GLX_STENCIL_SIZE),
                    attr!(GLX_ACCUM_RED_SIZE),
                    attr!(GLX_ACCUM_GREEN_SIZE),
                    attr!(GLX_ACCUM_BLUE_SIZE),
                    attr!(GLX_ACCUM_ALPHA_SIZE),
                ] {
                    let mut value = 0;
                    glXGetConfig(dpy, vi, attr.0, &mut value);
                    //println!("{}: {}", attr.1, value);
                }
                break;
            }

            let mut fbconfigs_count = 0;
            let fbconfigs = glXGetFBConfigs(dpy, 0, &mut fbconfigs_count);
            println!("#GLXFBConfigs: {}", fbconfigs_count); // 215 on my machine
            for i in 0..fbconfigs_count {
                let fbconfig = *fbconfigs.offset(i as isize);
                //println!("{:#x?}", *fbconfig);

                let _vi = glXGetVisualFromFBConfig(dpy, fbconfig);

                let mut value = 0;
                glXGetFBConfigAttrib(dpy, fbconfig, GLX_FBCONFIG_ID, &mut value);
                //println!("FBConfig ID: {:x}", value);
                #[rustfmt::skip]
                //println!( "Visual ID: {:x}", if vi.is_null() { 0 } else { (*vi).visualid });

                break;
            }

            XFree(fbconfigs as *mut c_void);
            XFree(visualinfos as *mut c_void);

            //let mut glx_attribs = [
            //    //GLX_RGBA,
            //    GLX_DOUBLEBUFFER, 1,
            //    GLX_RED_SIZE, 8,
            //    GLX_GREEN_SIZE, 8,
            //    GLX_BLUE_SIZE, 8,
            //    GLX_ALPHA_SIZE, 8,
            //    0,
            //];
            let mut fbconfigs_count = 0;
            let fbconfigs = glXChooseFBConfig(
                dpy,
                XDefaultScreen(dpy),
                /*glx_attribs.as_mut_ptr()*/ ptr::null_mut(),
                &mut fbconfigs_count,
            );
            let vi = glXGetVisualFromFBConfig(dpy, *fbconfigs.offset(0));
            XFree(fbconfigs as *mut c_void);
            //let vi = glXChooseVisual(dpy, 0, glx_attribs.as_mut_ptr());
            assert!(!vi.is_null());
            let ctx = glXCreateContext(dpy, vi, GLXContext::default(), 1);
            assert!(ctx != GLXContext::default());

            assert!(glXMakeCurrent(dpy, window, ctx) != 0);

            XMapWindow(dpy, window);

            println!("GL_VENDOR: {}", cstr_to_string(glGetString(GL_VENDOR)));
            println!("GL_RENDERER: {}", cstr_to_string(glGetString(GL_RENDERER)));
            println!("GL_VERSION: {}", cstr_to_string(glGetString(GL_VERSION)));
            println!("GL_SHADING_LANGUAGE_VERSION: {}", cstr_to_string(glGetString(GL_SHADING_LANGUAGE_VERSION)));
            //println!("GL_EXTENSIONS: {}", cstr_to_string(glGetString(GL_EXTENSIONS)));

            //glClearColor(1.0, 0.0, 0.0, 1.0);
            //glClear(GL_COLOR_BUFFER_BIT);

            //glBegin(GL_TRIANGLES);
            //glColor3f(0.68, 0.84, 0.0);
            //glVertex2f(-0.5, -0.5);
            //glVertex2f(0.5, -0.5);
            //glVertex2f(0.0, 0.5);
            //glEnd();

            glXSwapBuffers(dpy, window);

            assert!(glXMakeCurrent(dpy, 0, GLXContext::default()) != 0);
            glXDestroyContext(dpy, ctx);

            XCloseDisplay(dpy);
        }
    }
}
