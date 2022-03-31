use crate::math::Vec4;

pub struct Color(Vec4);

impl Color {
    pub fn from_hex_linear(_linear: u32) -> Self {
        todo!()
    }

    pub fn from_hex_srgb(_srgb: u32) -> Self {
        todo!()
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

pub fn color_to_u32(_color: [f32; 4]) -> u32 {
    todo!()
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
