#![allow(non_upper_case_globals)]
#![allow(dead_code)]

use std::ffi::c_void;
use std::ptr;

#[link(name = "X11")]
extern "C" {
    pub fn XInitThreads() -> Status;
    pub fn XOpenDisplay(display_name: *const i8) -> *mut Display;
    pub fn XCloseDisplay(display: *mut Display) -> i32;
    pub fn XCreateSimpleWindow(
        display: *mut Display,
        parent: Window,
        x: i32,
        y: i32,
        width: u32,
        height: u32,
        border_width: u32,
        border: u64,
        background: u64,
    ) -> Window;
    pub fn XDefaultRootWindow(display: *mut Display) -> Window;
    pub fn XRootWindow(display: *mut Display, screen_number: i32) -> Window;
    pub fn XDefaultScreen(display: *mut Display) -> i32;
    pub fn XSelectInput(display: *mut Display, window: Window, event_mask: i64) -> i32;
    pub fn XMapWindow(display: *mut Display, window: Window) -> i32;
    pub fn XPending(display: *mut Display) -> i32;
    pub fn XNextEvent(display: *mut Display, event: *mut XEvent) -> i32;
    pub fn XSetErrorHandler(handler: XErrorHandler) -> XErrorHandler;
    pub fn XSetIOErrorHandler(handler: XIOErrorHandler) -> XIOErrorHandler;
    pub fn XStoreName(display: *mut Display, window: Window, window_name: *const i8) -> i32;
    pub fn XLookupKeysym(key_event: *mut XKeyEvent, index: i32) -> KeySym;
    pub fn XDefaultGC(display: *mut Display, screen_number: i32) -> GC;
    pub fn XFillRectangle(display: *mut Display, d: Drawable, gc: GC, x: i32, y: i32, width: u32, height: u32) -> i32;
    pub fn XGetClassHint(display: *mut Display, w: Window, class_hints_return: *mut XClassHint) -> Status;
    pub fn XSetClassHint(display: *mut Display, w: Window, class_hints: *mut XClassHint) -> i32;
    pub fn XFree(data: *mut c_void) -> i32;
    pub fn XGetVisualInfo(
        display: *mut Display,
        vinfo_mask: i64,
        vinfo_template: *mut XVisualInfo,
        nitems_return: *mut i32,
    ) -> *mut XVisualInfo;
}

pub type XPointer = *mut i8;
pub type Status = i32;
pub type Bool = i32;
pub type XID = u64;
pub type CARD32 = XID;
pub type Window = XID;
pub type Time = CARD32;
pub type KeySym = XID;
pub type Drawable = XID;
pub type Font = XID;
pub type Pixmap = XID;

#[repr(C)]
pub struct Display {
    _data: [u8; 0],
    _marker: core::marker::PhantomData<(*mut u8, core::marker::PhantomPinned)>,
}

#[repr(C)]
#[derive(Debug)]
pub struct XVisualInfo {
    pub visual: *mut Visual,
    pub visualid: VisualID,
    pub screen: i32,
    pub depth: i32,
    pub class: i32,
    pub red_mask: u64,
    pub green_mask: u64,
    pub blue_mask: u64,
    pub colormap_size: i32,
    pub bits_per_rgb: i32, // log base 2 of the number of distinct color values (individually) of red, green, and blue.
}
impl Default for XVisualInfo {
    fn default() -> Self {
        Self {
            visual: ptr::null_mut(),
            visualid: 0,
            screen: 0,
            depth: 0,
            class: 0,
            red_mask: 0,
            green_mask: 0,
            blue_mask: 0,
            colormap_size: 0,
            bits_per_rgb: 0,
        }
    }
}
#[repr(C)]
pub struct Visual {
    pub ext_data: *mut XExtData,
    pub visualid: VisualID,
    pub class: i32,
    pub red_mask: u64,
    pub green_mask: u64,
    pub blue_mask: u64,
    pub bits_per_rgb: i32,
    pub map_entries: i32,
}
pub type VisualID = u64;
#[repr(C)]
pub struct XExtData {
    pub number: i32,
    pub next: *mut XExtData,
    pub free_private: extern "C" fn(extension: *mut XExtData) -> i32,
    pub private_data: XPointer,
}

#[repr(C)]
#[derive(Debug)]
pub struct _GC {
    _data: [u8; 0],
    _marker: core::marker::PhantomData<(*mut u8, core::marker::PhantomPinned)>,
}
pub type GC = *mut _GC;

#[repr(C)]
pub struct XClassHint {
    pub res_name: *mut i8,
    pub res_class: *mut i8,
}

pub type XErrorHandler = extern "C" fn(display: *mut Display, event: *mut XErrorEvent) -> i32;
pub type XIOErrorHandler = extern "C" fn(display: *mut Display) -> i32;

#[repr(C)]
pub union XEvent {
    pub ttype: i32,
    pub xany: XAnyEvent,
    pub xkey: XKeyEvent,
    pub xbutton: XButtonEvent,
    pub xmotion: XMotionEvent,
    // ...
    pub xconfigure: XConfigureEvent,
    // ...
    pub xerror: XErrorEvent,
    // ...
    pub pad: [i64; 24],
}

impl Default for XEvent {
    fn default() -> Self {
        Self {
            pad: [0; 24],
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct XAnyEvent {
    pub ttype: i32,
    pub serial: u64,
    pub send_event: Bool,
    pub display: *mut Display,
    pub window: Window,
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct XKeyEvent {
    pub ttype: i32,
    pub serial: u64,
    pub send_event: Bool,
    pub display: *mut Display,
    pub window: Window,
    pub root: Window,
    pub subwindow: Window,
    pub time: Time,
    pub x: i32,
    pub y: i32,
    pub x_root: i32,
    pub y_root: i32,
    pub state: u32,
    pub keycode: u32,
    pub same_screen: Bool,
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct XButtonEvent {
    pub ttype: i32,
    pub serial: u64,
    pub send_event: Bool,
    pub display: *mut Display,
    pub window: Window,
    pub root: Window,
    pub subwindow: Window,
    pub time: Time,
    pub x: i32,
    pub y: i32,
    pub x_root: i32,
    pub y_root: i32,
    pub state: u32,
    pub button: u32,
    pub same_screen: Bool,
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct XMotionEvent {
    pub ttype: i32,
    pub serial: u64,
    pub send_event: Bool,
    pub display: *mut Display,
    pub window: Window,
    pub root: Window,
    pub subwindow: Window,
    pub time: Time,
    pub x: i32,
    pub y: i32,
    pub x_root: i32,
    pub y_root: i32,
    pub state: u32,
    pub is_hint: i8,
    pub same_screen: Bool,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct XConfigureEvent {
    pub ttype: i32,
    pub serial: u64,
    pub send_event: Bool,
    pub display: *mut Display,
    pub event: Window,
    pub window: Window,
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
    pub border_width: i32,
    pub above: Window,
    pub override_redirect: Bool,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct XErrorEvent {
    pub ttype: i32,
    pub display: *mut Display,
    pub resourceid: XID,
    pub serial: u64,
    pub error_code: u8,
    pub request_code: u8,
    pub minor_code: u8,
}

pub const NoEventMask: i64 = 0;
pub const KeyPressMask: i64 = 1 << 0;
pub const KeyReleaseMask: i64 = 1 << 1;
pub const ButtonPressMask: i64 = 1 << 2;
pub const ButtonReleaseMask: i64 = 1 << 3;
pub const EnterWindowMask: i64 = 1 << 4;
pub const LeaveWindowMask: i64 = 1 << 5;
pub const PointerMotionMask: i64 = 1 << 6;
// ...
pub const ExposureMask: i64 = 1 << 15;
pub const StructureNotifyMask: i64 = 1 << 17;

pub const KeyPress: i32 = 2;
pub const KeyRelease: i32 = 3;
pub const ButtonPress: i32 = 4;
pub const ButtonRelease: i32 = 5;
pub const MotionNotify: i32 = 6;
// ...
pub const Expose: i32 = 12;
pub const ConfigureNotify: i32 = 22;

pub const VisualNoMask: i64 = 0x0;
pub const VisualIDMask: i64 = 0x1;
pub const VisualScreenMask: i64 = 0x2;
pub const VisualDepthMask: i64 = 0x4;

pub const Button1: u32 = 1;
pub const Button2: u32 = 2;
pub const Button3: u32 = 3;
pub const Button4: u32 = 4;
pub const Button5: u32 = 5;

pub const XK_BackSpace: KeySym = 0xff08;
pub const XK_Tab: KeySym = 0xff09;
pub const XK_Linefeed: KeySym = 0xff0a; /* Linefeed, LF */
pub const XK_Clear: KeySym = 0xff0b;
pub const XK_Return: KeySym = 0xff0d; /* Return, enter */
pub const XK_Pause: KeySym = 0xff13; /* Pause, hold */
pub const XK_Scroll_Lock: KeySym = 0xff14;
pub const XK_Sys_Req: KeySym = 0xff15;
pub const XK_Escape: KeySym = 0xff1b;
pub const XK_Delete: KeySym = 0xffff; /* Delete, rubout */

pub const XK_Home: KeySym = 0xff50;
pub const XK_Left: KeySym = 0xff51; /* Move left, left arrow */
pub const XK_Up: KeySym = 0xff52; /* Move up, up arrow */
pub const XK_Right: KeySym = 0xff53; /* Move right, right arrow */
pub const XK_Down: KeySym = 0xff54; /* Move down, down arrow */
pub const XK_Prior: KeySym = 0xff55; /* Prior, previous */
pub const XK_Page_Up: KeySym = 0xff55;
pub const XK_Next: KeySym = 0xff56; /* Next */
pub const XK_Page_Down: KeySym = 0xff56;
pub const XK_End: KeySym = 0xff57; /* EOL */
pub const XK_Begin: KeySym = 0xff58; /* BOL */

pub const XK_Select: KeySym = 0xff60; /* Select, mark */
pub const XK_Print: KeySym = 0xff61;
pub const XK_Execute: KeySym = 0xff62; /* Execute, run, do */
pub const XK_Insert: KeySym = 0xff63; /* Insert, insert here */
pub const XK_Undo: KeySym = 0xff65;
pub const XK_Redo: KeySym = 0xff66; /* Redo, again */
pub const XK_Menu: KeySym = 0xff67;
pub const XK_Find: KeySym = 0xff68; /* Find, search */
pub const XK_Cancel: KeySym = 0xff69; /* Cancel, stop, abort, exit */
pub const XK_Help: KeySym = 0xff6a; /* Help */
pub const XK_Break: KeySym = 0xff6b;
pub const XK_Mode_switch: KeySym = 0xff7e; /* Character set switch */
pub const XK_script_switch: KeySym = 0xff7e; /* Alias for mode_switch */
pub const XK_Num_Lock: KeySym = 0xff7f;

pub const XK_KP_Space: KeySym = 0xff80; /* Space */
pub const XK_KP_Tab: KeySym = 0xff89;
pub const XK_KP_Enter: KeySym = 0xff8d; /* Enter */
pub const XK_KP_F1: KeySym = 0xff91; /* PF1, KP_A, ... */
pub const XK_KP_F2: KeySym = 0xff92;
pub const XK_KP_F3: KeySym = 0xff93;
pub const XK_KP_F4: KeySym = 0xff94;
pub const XK_KP_Home: KeySym = 0xff95;
pub const XK_KP_Left: KeySym = 0xff96;
pub const XK_KP_Up: KeySym = 0xff97;
pub const XK_KP_Right: KeySym = 0xff98;
pub const XK_KP_Down: KeySym = 0xff99;
pub const XK_KP_Prior: KeySym = 0xff9a;
pub const XK_KP_Page_Up: KeySym = 0xff9a;
pub const XK_KP_Next: KeySym = 0xff9b;
pub const XK_KP_Page_Down: KeySym = 0xff9b;
pub const XK_KP_End: KeySym = 0xff9c;
pub const XK_KP_Begin: KeySym = 0xff9d;
pub const XK_KP_Insert: KeySym = 0xff9e;
pub const XK_KP_Delete: KeySym = 0xff9f;
pub const XK_KP_Equal: KeySym = 0xffbd; /* Equals */
pub const XK_KP_Multiply: KeySym = 0xffaa;
pub const XK_KP_Add: KeySym = 0xffab;
pub const XK_KP_Separator: KeySym = 0xffac; /* Separator, often comma */
pub const XK_KP_Subtract: KeySym = 0xffad;
pub const XK_KP_Decimal: KeySym = 0xffae;
pub const XK_KP_Divide: KeySym = 0xffaf;

pub const XK_KP_0: KeySym = 0xffb0;
pub const XK_KP_1: KeySym = 0xffb1;
pub const XK_KP_2: KeySym = 0xffb2;
pub const XK_KP_3: KeySym = 0xffb3;
pub const XK_KP_4: KeySym = 0xffb4;
pub const XK_KP_5: KeySym = 0xffb5;
pub const XK_KP_6: KeySym = 0xffb6;
pub const XK_KP_7: KeySym = 0xffb7;
pub const XK_KP_8: KeySym = 0xffb8;
pub const XK_KP_9: KeySym = 0xffb9;

pub const XK_space: KeySym = 0x0020; /* U+0020 SPACE */
pub const XK_exclam: KeySym = 0x0021; /* U+0021 EXCLAMATION MARK */
pub const XK_quotedbl: KeySym = 0x0022; /* U+0022 QUOTATION MARK */
pub const XK_numbersign: KeySym = 0x0023; /* U+0023 NUMBER SIGN */
pub const XK_dollar: KeySym = 0x0024; /* U+0024 DOLLAR SIGN */
pub const XK_percent: KeySym = 0x0025; /* U+0025 PERCENT SIGN */
pub const XK_ampersand: KeySym = 0x0026; /* U+0026 AMPERSAND */
pub const XK_apostrophe: KeySym = 0x0027; /* U+0027 APOSTROPHE */
pub const XK_quoteright: KeySym = 0x0027; /* deprecated */
pub const XK_parenleft: KeySym = 0x0028; /* U+0028 LEFT PARENTHESIS */
pub const XK_parenright: KeySym = 0x0029; /* U+0029 RIGHT PARENTHESIS */
pub const XK_asterisk: KeySym = 0x002a; /* U+002A ASTERISK */
pub const XK_plus: KeySym = 0x002b; /* U+002B PLUS SIGN */
pub const XK_comma: KeySym = 0x002c; /* U+002C COMMA */
pub const XK_minus: KeySym = 0x002d; /* U+002D HYPHEN-MINUS */
pub const XK_period: KeySym = 0x002e; /* U+002E FULL STOP */
pub const XK_slash: KeySym = 0x002f; /* U+002F SOLIDUS */
pub const XK_0: KeySym = 0x0030; /* U+0030 DIGIT ZERO */
pub const XK_1: KeySym = 0x0031; /* U+0031 DIGIT ONE */
pub const XK_2: KeySym = 0x0032; /* U+0032 DIGIT TWO */
pub const XK_3: KeySym = 0x0033; /* U+0033 DIGIT THREE */
pub const XK_4: KeySym = 0x0034; /* U+0034 DIGIT FOUR */
pub const XK_5: KeySym = 0x0035; /* U+0035 DIGIT FIVE */
pub const XK_6: KeySym = 0x0036; /* U+0036 DIGIT SIX */
pub const XK_7: KeySym = 0x0037; /* U+0037 DIGIT SEVEN */
pub const XK_8: KeySym = 0x0038; /* U+0038 DIGIT EIGHT */
pub const XK_9: KeySym = 0x0039; /* U+0039 DIGIT NINE */
pub const XK_colon: KeySym = 0x003a; /* U+003A COLON */
pub const XK_semicolon: KeySym = 0x003b; /* U+003B SEMICOLON */
pub const XK_less: KeySym = 0x003c; /* U+003C LESS-THAN SIGN */
pub const XK_equal: KeySym = 0x003d; /* U+003D EQUALS SIGN */
pub const XK_greater: KeySym = 0x003e; /* U+003E GREATER-THAN SIGN */
pub const XK_question: KeySym = 0x003f; /* U+003F QUESTION MARK */
pub const XK_at: KeySym = 0x0040; /* U+0040 COMMERCIAL AT */
pub const XK_A: KeySym = 0x0041; /* U+0041 LATIN CAPITAL LETTER A */
pub const XK_B: KeySym = 0x0042; /* U+0042 LATIN CAPITAL LETTER B */
pub const XK_C: KeySym = 0x0043; /* U+0043 LATIN CAPITAL LETTER C */
pub const XK_D: KeySym = 0x0044; /* U+0044 LATIN CAPITAL LETTER D */
pub const XK_E: KeySym = 0x0045; /* U+0045 LATIN CAPITAL LETTER E */
pub const XK_F: KeySym = 0x0046; /* U+0046 LATIN CAPITAL LETTER F */
pub const XK_G: KeySym = 0x0047; /* U+0047 LATIN CAPITAL LETTER G */
pub const XK_H: KeySym = 0x0048; /* U+0048 LATIN CAPITAL LETTER H */
pub const XK_I: KeySym = 0x0049; /* U+0049 LATIN CAPITAL LETTER I */
pub const XK_J: KeySym = 0x004a; /* U+004A LATIN CAPITAL LETTER J */
pub const XK_K: KeySym = 0x004b; /* U+004B LATIN CAPITAL LETTER K */
pub const XK_L: KeySym = 0x004c; /* U+004C LATIN CAPITAL LETTER L */
pub const XK_M: KeySym = 0x004d; /* U+004D LATIN CAPITAL LETTER M */
pub const XK_N: KeySym = 0x004e; /* U+004E LATIN CAPITAL LETTER N */
pub const XK_O: KeySym = 0x004f; /* U+004F LATIN CAPITAL LETTER O */
pub const XK_P: KeySym = 0x0050; /* U+0050 LATIN CAPITAL LETTER P */
pub const XK_Q: KeySym = 0x0051; /* U+0051 LATIN CAPITAL LETTER Q */
pub const XK_R: KeySym = 0x0052; /* U+0052 LATIN CAPITAL LETTER R */
pub const XK_S: KeySym = 0x0053; /* U+0053 LATIN CAPITAL LETTER S */
pub const XK_T: KeySym = 0x0054; /* U+0054 LATIN CAPITAL LETTER T */
pub const XK_U: KeySym = 0x0055; /* U+0055 LATIN CAPITAL LETTER U */
pub const XK_V: KeySym = 0x0056; /* U+0056 LATIN CAPITAL LETTER V */
pub const XK_W: KeySym = 0x0057; /* U+0057 LATIN CAPITAL LETTER W */
pub const XK_X: KeySym = 0x0058; /* U+0058 LATIN CAPITAL LETTER X */
pub const XK_Y: KeySym = 0x0059; /* U+0059 LATIN CAPITAL LETTER Y */
pub const XK_Z: KeySym = 0x005a; /* U+005A LATIN CAPITAL LETTER Z */
pub const XK_bracketleft: KeySym = 0x005b; /* U+005B LEFT SQUARE BRACKET */
pub const XK_backslash: KeySym = 0x005c; /* U+005C REVERSE SOLIDUS */
pub const XK_bracketright: KeySym = 0x005d; /* U+005D RIGHT SQUARE BRACKET */
pub const XK_asciicircum: KeySym = 0x005e; /* U+005E CIRCUMFLEX ACCENT */
pub const XK_underscore: KeySym = 0x005f; /* U+005F LOW LINE */
pub const XK_grave: KeySym = 0x0060; /* U+0060 GRAVE ACCENT */
pub const XK_quoteleft: KeySym = 0x0060; /* deprecated */
pub const XK_a: KeySym = 0x0061; /* U+0061 LATIN SMALL LETTER A */
pub const XK_b: KeySym = 0x0062; /* U+0062 LATIN SMALL LETTER B */
pub const XK_c: KeySym = 0x0063; /* U+0063 LATIN SMALL LETTER C */
pub const XK_d: KeySym = 0x0064; /* U+0064 LATIN SMALL LETTER D */
pub const XK_e: KeySym = 0x0065; /* U+0065 LATIN SMALL LETTER E */
pub const XK_f: KeySym = 0x0066; /* U+0066 LATIN SMALL LETTER F */
pub const XK_g: KeySym = 0x0067; /* U+0067 LATIN SMALL LETTER G */
pub const XK_h: KeySym = 0x0068; /* U+0068 LATIN SMALL LETTER H */
pub const XK_i: KeySym = 0x0069; /* U+0069 LATIN SMALL LETTER I */
pub const XK_j: KeySym = 0x006a; /* U+006A LATIN SMALL LETTER J */
pub const XK_k: KeySym = 0x006b; /* U+006B LATIN SMALL LETTER K */
pub const XK_l: KeySym = 0x006c; /* U+006C LATIN SMALL LETTER L */
pub const XK_m: KeySym = 0x006d; /* U+006D LATIN SMALL LETTER M */
pub const XK_n: KeySym = 0x006e; /* U+006E LATIN SMALL LETTER N */
pub const XK_o: KeySym = 0x006f; /* U+006F LATIN SMALL LETTER O */
pub const XK_p: KeySym = 0x0070; /* U+0070 LATIN SMALL LETTER P */
pub const XK_q: KeySym = 0x0071; /* U+0071 LATIN SMALL LETTER Q */
pub const XK_r: KeySym = 0x0072; /* U+0072 LATIN SMALL LETTER R */
pub const XK_s: KeySym = 0x0073; /* U+0073 LATIN SMALL LETTER S */
pub const XK_t: KeySym = 0x0074; /* U+0074 LATIN SMALL LETTER T */
pub const XK_u: KeySym = 0x0075; /* U+0075 LATIN SMALL LETTER U */
pub const XK_v: KeySym = 0x0076; /* U+0076 LATIN SMALL LETTER V */
pub const XK_w: KeySym = 0x0077; /* U+0077 LATIN SMALL LETTER W */
pub const XK_x: KeySym = 0x0078; /* U+0078 LATIN SMALL LETTER X */
pub const XK_y: KeySym = 0x0079; /* U+0079 LATIN SMALL LETTER Y */
pub const XK_z: KeySym = 0x007a; /* U+007A LATIN SMALL LETTER Z */
pub const XK_braceleft: KeySym = 0x007b; /* U+007B LEFT CURLY BRACKET */
pub const XK_bar: KeySym = 0x007c; /* U+007C VERTICAL LINE */
pub const XK_braceright: KeySym = 0x007d; /* U+007D RIGHT CURLY BRACKET */
pub const XK_asciitilde: KeySym = 0x007e; /* U+007E TILDE */
