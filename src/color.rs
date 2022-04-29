use crate::math::Vec4;

#[macro_export]
macro_rules! color(
    (hex($x:expr)) => { // 0xAARRGGBB
        Color::new(
            (($x >> 16) & 0xFF) as u8,
            (($x >> 8) & 0xFF) as u8,
            (($x >> 0) & 0xFF) as u8,
            (($x >> 24) & 0xFF) as u8,
        )
    };

    // 0-255 components
    (rgb8($r:expr, $g:expr, $b:expr)) => {
        Color::new(
            $r,
            $g,
            $b,
            255,
        )
    };
    (rgba8($r:expr, $g:expr, $b:expr, $a:expr)) => {
        Color::new(
            $r,
            $g,
            $b,
            $a,
        )
    };

    // normalized 0-1 components
    (rgb($r:expr, $g:expr, $b:expr)) => {Color::new(($r * 255.0) as u8, ($g * 255.0) as u8, ($b * 255.0) as u8, 255)};
    (rgba($r:expr, $g:expr, $b:expr, $a:expr)) => {Color::new(($r * 255.0) as u8, ($g * 255.0) as u8, ($b * 255.0) as u8, ($a * 255.0) as u8)};

    // srgb
    // hsv
);

#[repr(C)]
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

pub const BLACK: Color = color!(rgb(0.0, 0.0, 0.0));
pub const WHITE: Color = color!(rgb(1.0, 1.0, 1.0));
pub const RED: Color = color!(rgb(1.0, 0.0, 0.0));
pub const GREEN: Color = color!(rgb(0.0, 1.0, 0.0));
pub const BLUE: Color = color!(rgb(0.0, 0.0, 1.0));

pub const YELLOW: Color = color!(rgb(1.0, 1.0, 0.0));

pub const DARK_GREEN: Color = color!(rgb(0.0, 0.5, 0.0));
pub const DARK_BLUE: Color = color!(rgb(0.0, 0.0, 0.5));
pub const DARK_GREY: Color = color!(rgb(0.2, 0.2, 0.2));
pub const LIGHT_GREY: Color = color!(rgb(0.6, 0.6, 0.6));
pub const BROWN: Color = color!(rgb(0.5, 0.0, 0.0));
pub const CYAN: Color = color!(rgb(0.0, 0.5, 0.5));
pub const GREY: Color = color!(rgb(0.5, 0.5, 0.5));
pub const PURPLE: Color = color!(rgb(0.5, 0.0, 0.5));
pub const ORANGE: Color = color!(rgb(1.0, 0.5, 0.0));
//pub const ORANGE: Color = color!(rgb8(255, 255, 0));

impl Color {
    pub const fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self {
            r,
            g,
            b,
            a,
        }
    }

    pub fn from_hex(linear: u32) -> Self {
        Self::from_hex_linear(linear)
    }

    pub fn from_hex_linear(linear: u32) -> Self {
        color!(hex(linear))
    }

    pub fn from_hex_srgb(_srgb: u32) -> Self {
        todo!()
    }

    pub fn invert(&self) -> Self {
        Self::new(255 - self.r, 255 - self.g, 255 - self.b, self.a)
    }

    pub fn as_f32(&self) -> [f32; 4] {
        srgb_to_linear(self.as_u32())
        //[self.r as f32 / 255.0, self.g as f32 / 255.0, self.b as f32 / 255.0, self.a as f32 / 255.0]
    }

    pub fn as_u32(&self) -> u32 {
        //color_to_u32(self.as_f32())
        // 0xAARRGGBB
        (self.a as u32) << 24 | (self.r as u32) << 16 | (self.g as u32) << 8 | (self.b as u32)
    }
}

impl From<Vec4> for Color {
    fn from(color: Vec4) -> Self {
        color!(rgba(color.x, color.y, color.z, color.w))
    }
}
impl From<[f32; 4]> for Color {
    fn from(color: [f32; 4]) -> Self {
        color!(rgba(color[0], color[1], color[2], color[3]))
    }
}

#[allow(clippy::identity_op)]
pub fn color_to_f32(color: u32) -> [f32; 4] {
    let a = ((color >> 24) & 0xFF) as f32 / 255.0;
    let r = ((color >> 16) & 0xFF) as f32 / 255.0;
    let g = ((color >> 8) & 0xFF) as f32 / 255.0;
    let b = ((color >> 0) & 0xFF) as f32 / 255.0;
    [r, g, b, a]
}

pub fn color_to_u32(color: [f32; 4]) -> u32 {
    let r = (color[0] * 255.0).clamp(0.0, 255.0) as u32;
    let g = (color[1] * 255.0).clamp(0.0, 255.0) as u32;
    let b = (color[2] * 255.0).clamp(0.0, 255.0) as u32;
    let a = (color[3] * 255.0).clamp(0.0, 255.0) as u32;

    (a << 24) | (r << 16) | (g << 8) | b
}

pub fn srgb_to_linear(color: u32) -> [f32; 4] {
    // 0 ≤ S ≤ 0.04045	L = S/12.92
    // 0.04045 < S ≤ 1	L = ((S+0.055)/1.055)^2.4
    let mut color = color_to_f32(color);
    //println!("sRGB: {:?}", color);
    for c in color.iter_mut() {
        if *c <= 0.04045 {
            *c /= 12.92;
        } else {
            *c = ((*c + 0.055) / 1.055).powf(2.4);
        }
    }
    //println!("Linear: {:?}", color);
    color
}

pub fn linear_to_srgb(color: u32) -> [f32; 4] {
    // 0 ≤ L ≤ 0.0031308	S = L×12.92
    // 0.0031308 < L ≤ 1	S = 1.055×L^1/2.4 − 0.055

    let mut color = color_to_f32(color);
    //println!("Linear: {:?}", color);
    for c in color.iter_mut() {
        if *c <= 0.0031308 {
            *c *= 12.92;
        } else {
            *c = 1.055 * (*c).powf(1.0 / 2.4) - 0.055;
        }
    }
    //println!("sRGB: {:?}", color);
    color
}
