use crate::math::Vec4;

#[macro_export]
macro_rules! color(
    (hex($x:expr)) => {
        Color(Vec4::new(
            (($x >> 16) & 0xFF) as f32 / 255.0,
            (($x >> 8) & 0xFF) as f32 / 255.0,
            (($x >> 0) & 0xFF) as f32 / 255.0,
            (($x >> 24) & 0xFF) as f32 / 255.0,
        ))
    };

    // 0-255 components
    (rgb8($r:expr, $g:expr, $b:expr)) => {
        Color(Vec4::new(
            ($r as f32) / 255.0,
            ($g as f32) / 255.0,
            ($b as f32) / 255.0,
            1.0,
        ))
    };
    (rgba8($r:expr, $g:expr, $b:expr, $a:expr)) => {
        Color(Vec4::new(
            ($r as f32) / 255.0,
            ($g as f32) / 255.0,
            ($b as f32) / 255.0,
            ($a as f32) / 255.0,
        ))
    };

    // normalized 0-1 components
    (rgb($r:expr, $g:expr, $b:expr)) => {Color(Vec4::new($r, $g, $b, 1.0))};
    (rgba($r:expr, $g:expr, $b:expr, $a:expr)) => {Color(Vec4::new($r, $g, $b, $a))};

    // srgb
    // hsv
);

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct Color(pub Vec4);

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

impl Color {
    pub fn from_hex(linear: u32) -> Self {
        Self::from_hex_linear(linear)
    }

    pub fn from_hex_linear(linear: u32) -> Self {
        Self(color_to_f32(linear).into())
    }

    pub fn from_hex_srgb(_srgb: u32) -> Self {
        todo!()
    }

    pub fn invert(&self) -> Self {
        Self(Vec4::new(1.0 - self.0.x, 1.0 - self.0.y, 1.0 - self.0.z, self.0.w))
    }

    pub fn as_f32(&self) -> [f32; 4] {
        [self.0.x, self.0.y, self.0.z, self.0.w]
    }

    pub fn as_u32(&self) -> u32 {
        color_to_u32([self.0.x, self.0.y, self.0.z, self.0.w])
    }
}

impl From<Vec4> for Color {
    fn from(color: Vec4) -> Self {
        Self(color)
    }
}
impl From<[f32; 4]> for Color {
    fn from(color: [f32; 4]) -> Self {
        Self(Vec4::new(color[0], color[1], color[2], color[3]))
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
