#![allow(non_upper_case_globals)]
#![allow(non_snake_case)]
#![allow(unreachable_code)]
use icarus::*;

fn main() {}

#[allow(dead_code)]
fn generate_glyphs<P: AsRef<str>>(path: P) {
    unsafe {
        let mut width = 0;
        let mut height = 0;
        let mut channels = 0;
        let mut path = path.as_ref().to_string();
        path.push(0 as char);
        let pixels = stbi_load(path.as_ptr() as *const i8, &mut width, &mut height, &mut channels, 1);

        // 7x9 quads with 1 pixel of padding
        let mut glyphs = vec![];
        for row in 0..6 {
            for col in 0..18 {
                let quad = (7 * col, 9 * row, 7, 9);
                let mut glyph = vec![];
                for y in quad.1 + 1..quad.1 + 9 - 1 {
                    for x in quad.0 + 1..quad.0 + 7 - 1 {
                        glyph.push(if *pixels.offset((y * width + x) as isize) == 0 {
                            0
                        } else {
                            1
                        });
                    }
                }
                glyphs.push(glyph);
            }
        }
        println!("{:?}", glyphs);
    }
}
