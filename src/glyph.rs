#![allow(non_upper_case_globals)]
#![allow(dead_code)]
#![allow(unused_variables)]

use crate::stb_image::*;

// https://developer.apple.com/fonts/TrueType-Reference-Manual/RM01/Chap1.html
// Bezier Quadratic Curve: from point p0 to p2 with p1 an off-curve point.
// p(t) = (1-t)^2 p0 + 2t(1-t)p1 + t^2 p2
// A Glyph can have 0 or more contours.

// TTF Format:
// Font directory
//   Offset subtable
//     u32 scaler_type      00 01 00 00
//     u16 num_tables             00 15 (21)
//     u16 search_range           01 00 (256)
//     u16 entry_selector         00 04
//     u16 range_shift            00 50 (80)
//   Table directory:
//     GDEF, GPOS, GSUB, LTSH, OS/2, VDMX, cmap, cvt, fpgm, gasp, glyf, hdmx, head, hhea, hmtx,
//     kern, loca, maxp, name, post, prep
//     u32 tag
//     u32 checksum
//     u32 offset
//     u32 length

#[repr(C)]
#[derive(Debug)]
struct OffsetSubtable {
    scaler_type: u32,
    num_tables: u16,
    search_range: u16,
    entry_selector: u16,
    range_shift: u16,
}
#[repr(C)]
#[derive(Debug)]
struct TableDirectory {
    tag: Tag,
    checksum: u32,
    offset: u32,
    length: u32,
}

const TAG_cmap: Tag = Tag::new(b"cmap"); // character to glyph mapping
const TAG_glyf: Tag = Tag::new(b"glyf"); // glyph data
const TAG_head: Tag = Tag::new(b"head"); // font header
const TAG_hhea: Tag = Tag::new(b"hhea"); // horizontal header
const TAG_hmtx: Tag = Tag::new(b"hmtx"); // horizontal metrics
const TAG_loca: Tag = Tag::new(b"loca"); // index to location
const TAG_maxp: Tag = Tag::new(b"maxp"); // maximum profile
const TAG_name: Tag = Tag::new(b"name"); // naming
const TAG_post: Tag = Tag::new(b"post"); // PostScript

#[derive(PartialEq, Eq, Copy, Clone)]
struct Tag(u32);
impl Tag {
    const fn new(s: &[u8; 4]) -> Self {
        Self((s[0] as u32) << 24 | (s[1] as u32) << 16 | (s[2] as u32) << 8 | (s[3] as u32))
    }
}
impl std::fmt::Debug for Tag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}{}{}{} ({:08x})",
            ((self.0 >> 24) & 0xFF) as u8 as char,
            ((self.0 >> 16) & 0xFF) as u8 as char,
            ((self.0 >> 8) & 0xFF) as u8 as char,
            (self.0 & 0xFF) as u8 as char,
            self.0,
        )
    }
}

fn calc_table_checksum(table: &[u32]) -> u32 {
    todo!()
}

pub fn read_u8<R: std::io::Read>(mut r: R) -> std::io::Result<u8> {
    let mut b = [0; 1];
    r.read_exact(&mut b)?;
    Ok(b[0])
}
pub fn read_u16_be<R: std::io::Read>(mut r: R) -> std::io::Result<u16> {
    let mut b = [0; 2];
    r.read_exact(&mut b)?;
    Ok((b[0] as u16) << 8 | b[1] as u16)
}
pub fn read_i16_be<R: std::io::Read>(r: R) -> std::io::Result<i16> {
    Ok(read_u16_be(r)? as i16)
}
pub fn read_u32_be<R: std::io::Read>(mut r: R) -> std::io::Result<u32> {
    let mut b = [0; 4];
    r.read_exact(&mut b)?;
    Ok((b[0] as u32) << 24 | (b[1] as u32) << 16 | (b[2] as u32) << 8 | b[3] as u32)
}
pub fn read_u64_be<R: std::io::Read>(mut r: R) -> std::io::Result<u64> {
    let mut b = [0; 8];
    r.read_exact(&mut b)?;
    Ok((b[0] as u64) << 56
        | (b[1] as u64) << 48
        | (b[2] as u64) << 40
        | (b[3] as u64) << 32
        | (b[4] as u64) << 24
        | (b[5] as u64) << 16
        | (b[6] as u64) << 8
        | b[7] as u64)
}
pub fn read_u16_le<R: std::io::Read>(mut r: R) -> std::io::Result<u16> {
    let mut b = [0; 2];
    r.read_exact(&mut b)?;
    Ok((b[1] as u16) << 8 | b[0] as u16)
}
pub fn read_u32_le<R: std::io::Read>(mut r: R) -> std::io::Result<u32> {
    let mut b = [0; 4];
    r.read_exact(&mut b)?;
    Ok((b[3] as u32) << 24 | (b[2] as u32) << 16 | (b[1] as u32) << 8 | b[0] as u32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tag_new() {
        assert_eq!(Tag::new(b"cmap"), Tag(0x636d6170));
    }

    #[test]
    #[ignore]
    fn parse_ttf() -> std::io::Result<()> {
        let bytes = std::fs::read("/usr/share/fonts/TTF/knewave.ttf").unwrap();
        let mut reader = std::io::Cursor::new(bytes);
        let offset_subtable = OffsetSubtable {
            scaler_type: read_u32_be(&mut reader)?,
            num_tables: read_u16_be(&mut reader)?,
            search_range: read_u16_be(&mut reader)?,
            entry_selector: read_u16_be(&mut reader)?,
            range_shift: read_u16_be(&mut reader)?,
        };
        println!("{:#?}", offset_subtable);
        println!();
        for _ in 0..offset_subtable.num_tables {
            let table_dir = TableDirectory {
                tag: Tag(read_u32_be(&mut reader)?),
                checksum: read_u32_be(&mut reader)?,
                offset: read_u32_be(&mut reader)?,
                length: read_u32_be(&mut reader)?,
            };
            println!("{:#?}", table_dir);

            let table_bytes =
                &reader.get_ref()[(table_dir.offset as usize)..(table_dir.offset + table_dir.length) as usize];
            let mut reader = std::io::Cursor::new(table_bytes);
            match table_dir.tag {
                TAG_head => {
                    let version = read_u32_be(&mut reader)?;
                    let font_revision = read_u32_be(&mut reader)?;
                    let checksum_adjustment = read_u32_be(&mut reader)?;
                    let magic_number = read_u32_be(&mut reader)?;
                    let flags = read_u16_be(&mut reader)?;
                    let units_per_em = read_u16_be(&mut reader)?;
                    let created = read_u64_be(&mut reader)?;
                    let modified = read_u64_be(&mut reader)?;
                    let xmin = read_i16_be(&mut reader)?;
                    let ymin = read_i16_be(&mut reader)?;
                    let xmax = read_i16_be(&mut reader)?;
                    let ymax = read_i16_be(&mut reader)?;
                    let mac_style = read_u16_be(&mut reader)?;
                    let lowest_rec_ppem = read_u16_be(&mut reader)?;
                    let font_direction_hint = read_i16_be(&mut reader)?;
                    let index_to_loc_format = read_i16_be(&mut reader)?;
                    let glyph_data_format = read_i16_be(&mut reader)?;
                    println!("version: 0x{:08x}, revision: {}, checksum: {}, magic: 0x{:08x}, flags: 0b{:032b}, units per em: {}, created: {}, modified: {}",
                        version, font_revision, checksum_adjustment, magic_number, flags, units_per_em, created, modified);
                    println!(
                        "xmin: {}, ymin: {}, xmax: {}, ymax: {}, mac style: {}",
                        xmin, ymin, xmax, ymax, mac_style
                    );
                    println!(
                        "lowest rec ppem: {}, direction hint: {}, index to loc format: {}, glyph data format: {}",
                        lowest_rec_ppem, font_direction_hint, index_to_loc_format, glyph_data_format
                    );
                    //break;
                }
                TAG_maxp => {
                    let version = read_u32_be(&mut reader)?;
                    let num_glyphs = read_u16_be(&mut reader)?;
                    println!("version: 0x{:08x}, num glyphs: {}", version, num_glyphs);
                }
                TAG_cmap => {
                    let version = read_u16_be(&mut reader)?;
                    let num_subtables = read_u16_be(&mut reader)?;
                    //println!("Version: {}", version);
                    //println!("Num Subtables: {}", num_subtables);

                    for _ in 0..num_subtables {
                        let platform_id = read_u16_be(&mut reader)?;
                        let platform_specific_id = read_u16_be(&mut reader)?;
                        let offset = read_u32_be(&mut reader)?;
                        //println!(
                        //    "platform id: {}, platform specific id: {}, offset: 0x{:08x}",
                        //    platform_id, platform_specific_id, offset
                        //);
                    }

                    //break;
                }
                TAG_glyf => {
                    for glyph_idx in 0..211 {
                        println!("GLYPH {}", glyph_idx);
                        let num_contours = read_i16_be(&mut reader)?;
                        let xmin = read_i16_be(&mut reader)?;
                        let ymin = read_i16_be(&mut reader)?;
                        let xmax = read_i16_be(&mut reader)?;
                        let ymax = read_i16_be(&mut reader)?;
                        println!(
                            "num contours: {}, xmin: {}, ymin: {}, xmax: {}, ymax: {}",
                            num_contours, xmin, ymin, xmax, ymax
                        );
                        if num_contours >= 0 {
                            // Simple Glyph
                            for _ in 0..num_contours {
                                let end_pt_of_contour = read_u16_be(&mut reader)?;
                                println!("end pt of contour: {}", end_pt_of_contour);
                            }
                            let instruction_len = read_u16_be(&mut reader)?;
                            println!("instruction len: {}", instruction_len);
                            for _ in 0..instruction_len {
                                let instruction = read_u8(&mut reader)?;
                            }
                            let flag = read_u8(&mut reader)?;
                            println!("0b{:08b}", flag);
                            if flag & 0x01 != 0 {
                                println!("ON_CURVE_POINT");
                            }
                            if flag & 0x02 != 0 {
                                println!("X_SHORT_VECTOR");
                                // x coordinate is 1 byte long
                            }
                            if flag & 0x04 != 0 {
                                println!("Y_SHORT_VECTOR");
                                // y coordinate is 1 byte long
                            }
                            if flag & 0x08 != 0 {
                                println!("REPEAT_FLAG");
                            }
                            if flag & 0x10 != 0 {
                                println!("X_IS_SAME_OR_POSITIVE_X_SHORT_VECTOR");
                            }
                            if flag & 0x20 != 0 {
                                println!("Y_IS_SAME_OR_POSITIVE_Y_SHORT_VECTOR");
                            }
                            if flag & 0x40 != 0 {
                                println!("OVERLAP_SIMPLE");
                            }
                            panic!("TODO: flags...");
                        } else {
                            // Compound Glyph
                            panic!("Compount Glyphs not supported yet!");
                        }
                    }
                    //break;
                }
                _ => {}
            }
        }

        //assert_eq!(bytes.len(), 0);
        Ok(())
    }
}

pub const GLYPH_WIDTH: usize = 5;
pub const GLYPH_HEIGHT: usize = 7;
pub const GLYPH_PIXEL_SIZE: f32 = 10.0;
pub type Glyph = [u8; GLYPH_WIDTH * GLYPH_HEIGHT];
pub const GLYPHS: [Glyph; 95] = [
    [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0], // <Space>
    [0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0], // !
    [0, 1, 0, 1, 0, 0, 1, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0], // "
    [0, 1, 0, 1, 0, 0, 1, 0, 1, 0, 1, 1, 1, 1, 1, 0, 1, 0, 1, 0, 1, 1, 1, 1, 1, 0, 1, 0, 1, 0, 0, 1, 0, 1, 0], // #
    [0, 0, 1, 0, 0, 0, 1, 1, 1, 1, 1, 0, 1, 0, 0, 0, 1, 1, 1, 0, 0, 0, 1, 0, 1, 1, 1, 1, 1, 0, 0, 0, 1, 0, 0], // $
    [1, 1, 0, 0, 0, 1, 1, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 1, 1, 0, 0, 0, 1, 1], // %
    [0, 1, 1, 1, 0, 1, 0, 0, 1, 0, 1, 0, 1, 0, 0, 0, 1, 0, 0, 0, 1, 0, 1, 0, 1, 1, 0, 0, 1, 0, 0, 1, 1, 0, 1], // &
    [0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0], // '
    [0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 1, 0], // (
    [0, 1, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0], // )
    [0, 0, 0, 0, 0, 1, 0, 1, 0, 1, 0, 1, 1, 1, 0, 1, 1, 1, 1, 1, 0, 1, 1, 1, 0, 1, 0, 1, 0, 1, 0, 0, 0, 0, 0], // *
    [0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 1, 1, 1, 1, 1, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0], // +
    [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0], // ,
    [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0], // -
    [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0], // .
    [0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0], // /
    [0, 1, 1, 1, 0, 1, 0, 0, 0, 1, 1, 0, 0, 1, 1, 1, 0, 1, 0, 1, 1, 1, 0, 0, 1, 1, 0, 0, 0, 1, 0, 1, 1, 1, 0], // 0
    [0, 0, 1, 0, 0, 0, 1, 1, 0, 0, 1, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 1, 1, 1, 1, 1], // 1
    [0, 1, 1, 1, 0, 1, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 1, 1, 1, 1, 1], // 2
    [0, 1, 1, 1, 0, 1, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 1, 1, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0, 1, 0, 1, 1, 1, 0], // 3
    [0, 0, 1, 1, 0, 0, 1, 0, 1, 0, 1, 0, 0, 1, 0, 1, 1, 1, 1, 1, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0], // 4
    [1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 1, 1, 1, 1, 0], // 5
    [0, 1, 1, 1, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 1, 1, 1, 0, 1, 0, 0, 0, 1, 1, 0, 0, 0, 1, 0, 1, 1, 1, 0], // 6
    [1, 1, 1, 1, 1, 0, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0], // 7
    [0, 1, 1, 1, 0, 1, 0, 0, 0, 1, 1, 0, 0, 0, 1, 0, 1, 1, 1, 0, 1, 0, 0, 0, 1, 1, 0, 0, 0, 1, 0, 1, 1, 1, 0], // 8
    [0, 1, 1, 1, 0, 1, 0, 0, 0, 1, 1, 0, 0, 0, 1, 0, 1, 1, 1, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 1, 1, 1, 0], // 9
    [0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0], // :
    [0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0], // ;
    [0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0], // <
    [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0], // =
    [0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0], // >
    [0, 1, 1, 1, 0, 1, 0, 0, 0, 1, 1, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0], // ?
    [0, 1, 1, 1, 0, 1, 0, 0, 0, 1, 1, 0, 1, 1, 1, 1, 0, 1, 0, 1, 1, 0, 1, 1, 1, 1, 0, 0, 0, 0, 0, 1, 1, 1, 0], // @
    [0, 1, 1, 1, 0, 1, 0, 0, 0, 1, 1, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 1, 1, 0, 0, 0, 1, 1, 0, 0, 0, 1], // A
    [1, 1, 1, 1, 0, 1, 0, 0, 0, 1, 1, 0, 0, 0, 1, 1, 1, 1, 1, 0, 1, 0, 0, 0, 1, 1, 0, 0, 0, 1, 1, 1, 1, 1, 0], // B
    [0, 1, 1, 1, 0, 1, 0, 0, 0, 1, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 1, 0, 1, 1, 1, 0], // C
    [1, 1, 1, 1, 0, 1, 0, 0, 0, 1, 1, 0, 0, 0, 1, 1, 0, 0, 0, 1, 1, 0, 0, 0, 1, 1, 0, 0, 0, 1, 1, 1, 1, 1, 0], // D
    [1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 1, 1, 1, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 1, 1, 1, 1], // E
    [1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 1, 1, 1, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0], // F
    [0, 1, 1, 1, 0, 1, 0, 0, 0, 1, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 1, 1, 1, 0, 0, 0, 1, 0, 1, 1, 1, 0], // G
    [1, 0, 0, 0, 1, 1, 0, 0, 0, 1, 1, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 1, 1, 0, 0, 0, 1, 1, 0, 0, 0, 1], // H
    [1, 1, 1, 1, 1, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 1, 1, 1, 1, 1], // I
    [1, 1, 1, 1, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 1, 0, 0, 0, 1, 0, 1, 1, 1, 0], // J
    [1, 0, 0, 0, 1, 1, 0, 0, 0, 1, 1, 0, 0, 1, 0, 1, 1, 1, 0, 0, 1, 0, 0, 1, 0, 1, 0, 0, 0, 1, 1, 0, 0, 0, 1], // K
    [1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 1, 1, 1, 1], // L
    [1, 0, 0, 0, 1, 1, 1, 0, 1, 1, 1, 0, 1, 0, 1, 1, 0, 0, 0, 1, 1, 0, 0, 0, 1, 1, 0, 0, 0, 1, 1, 0, 0, 0, 1], // M
    [1, 0, 0, 0, 1, 1, 0, 0, 0, 1, 1, 1, 0, 0, 1, 1, 0, 1, 0, 1, 1, 0, 0, 1, 1, 1, 0, 0, 0, 1, 1, 0, 0, 0, 1], // N
    [0, 1, 1, 1, 0, 1, 0, 0, 0, 1, 1, 0, 0, 0, 1, 1, 0, 0, 0, 1, 1, 0, 0, 0, 1, 1, 0, 0, 0, 1, 0, 1, 1, 1, 0], // O
    [1, 1, 1, 1, 0, 1, 0, 0, 0, 1, 1, 0, 0, 0, 1, 1, 1, 1, 1, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0], // P
    [0, 1, 1, 1, 0, 1, 0, 0, 0, 1, 1, 0, 0, 0, 1, 1, 0, 0, 0, 1, 1, 0, 1, 0, 1, 1, 0, 0, 1, 0, 0, 1, 1, 0, 1], // Q
    [1, 1, 1, 1, 0, 1, 0, 0, 0, 1, 1, 0, 0, 0, 1, 1, 1, 1, 1, 0, 1, 0, 0, 0, 1, 1, 0, 0, 0, 1, 1, 0, 0, 0, 1], // R
    [0, 1, 1, 1, 1, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 1, 1, 1, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 1, 1, 1, 1, 0], // S
    [1, 1, 1, 1, 1, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0], // T
    [1, 0, 0, 0, 1, 1, 0, 0, 0, 1, 1, 0, 0, 0, 1, 1, 0, 0, 0, 1, 1, 0, 0, 0, 1, 1, 0, 0, 0, 1, 0, 1, 1, 1, 0], // U
    [1, 0, 0, 0, 1, 1, 0, 0, 0, 1, 1, 0, 0, 0, 1, 1, 0, 0, 0, 1, 1, 0, 0, 0, 1, 0, 1, 0, 1, 0, 0, 0, 1, 0, 0], // V
    [1, 0, 0, 0, 1, 1, 0, 0, 0, 1, 1, 0, 0, 0, 1, 1, 0, 0, 0, 1, 1, 0, 1, 0, 1, 1, 1, 0, 1, 1, 1, 0, 0, 0, 1], // W
    [1, 0, 0, 0, 1, 1, 0, 0, 0, 1, 0, 1, 0, 1, 0, 0, 0, 1, 0, 0, 0, 1, 0, 1, 0, 1, 0, 0, 0, 1, 1, 0, 0, 0, 1], // X
    [1, 0, 0, 0, 1, 1, 0, 0, 0, 1, 0, 1, 0, 1, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0], // Y
    [1, 1, 1, 1, 1, 0, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 0, 1, 1, 1, 1, 1], // Z
    [0, 0, 1, 1, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 1, 0], // [
    [0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0], // \
    [0, 1, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 1, 1, 0, 0], // ]
    [0, 0, 1, 0, 0, 0, 1, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0], // ^
    [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1], // _
    [0, 1, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0], // `
    [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 0, 0, 0, 0, 0, 1, 0, 1, 1, 1, 1, 1, 0, 0, 0, 1, 0, 1, 1, 1, 1], // a
    [1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 1, 1, 1, 0, 1, 0, 0, 0, 1, 1, 0, 0, 0, 1, 1, 0, 0, 0, 1, 1, 1, 1, 1, 0], // b
    [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 0, 1, 0, 0, 0, 1, 1, 0, 0, 0, 0, 1, 0, 0, 0, 1, 0, 1, 1, 1, 0], // c
    [0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 1, 1, 1, 1, 1, 0, 0, 0, 1, 1, 0, 0, 0, 1, 1, 0, 0, 0, 1, 0, 1, 1, 1, 1], // d
    [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 0, 1, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 1, 1, 1, 1], // e
    [0, 0, 1, 1, 0, 0, 1, 0, 0, 0, 1, 1, 1, 1, 1, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0], // f
    [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 0, 0, 0, 1, 0, 1, 1, 1, 1, 0, 0, 0, 0, 1, 0, 1, 1, 1, 0], // g
    [1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 1, 1, 1, 0, 1, 0, 0, 0, 1, 1, 0, 0, 0, 1, 1, 0, 0, 0, 1, 1, 0, 0, 0, 1], // h
    [0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 1, 1, 1, 1, 1], // i
    [0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 1, 0, 0, 0, 1, 0, 1, 1, 1, 0], // j
    [1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 1, 0, 1, 1, 1, 0, 0, 1, 0, 0, 1, 0, 1, 0, 0, 0, 1, 1, 0, 0, 0, 1], // k
    [1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 1, 1, 1, 0], // l
    [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 0, 1, 0, 1, 0, 1, 0, 1, 1, 0, 1, 0, 1, 1, 0, 0, 0, 1, 1, 0, 0, 0, 1], // m
    [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 0, 1, 0, 0, 0, 1, 1, 0, 0, 0, 1, 1, 0, 0, 0, 1, 1, 0, 0, 0, 1], // n
    [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 0, 1, 0, 0, 0, 1, 1, 0, 0, 0, 1, 1, 0, 0, 0, 1, 0, 1, 1, 1, 0], // o
    [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 0, 1, 0, 0, 0, 1, 1, 1, 1, 1, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0], // p
    [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 0, 0, 0, 1, 0, 1, 1, 1, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1], // q
    [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 1, 1, 0, 1, 1, 0, 0, 1, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0], // r
    [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 1, 1, 1, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 0], // s
    [0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 1, 1, 1, 1, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 1, 0, 0, 1, 1, 0], // t
    [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 1, 1, 0, 0, 0, 1, 1, 0, 0, 0, 1, 1, 0, 0, 0, 1, 0, 1, 1, 1, 1], // u
    [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 1, 1, 0, 0, 0, 1, 1, 0, 0, 0, 1, 0, 1, 0, 1, 0, 0, 0, 1, 0, 0], // v
    [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 1, 1, 0, 0, 0, 1, 1, 0, 0, 0, 1, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0], // w
    [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 1, 0, 1, 0, 1, 0, 0, 0, 1, 0, 0, 0, 1, 0, 1, 0, 1, 0, 0, 0, 1], // x
    [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 1, 1, 0, 0, 0, 1, 0, 1, 1, 1, 1, 0, 0, 0, 0, 1, 0, 1, 1, 1, 0], // y
    [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 1, 1, 1, 1, 1], // z
    [0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 1, 0], // {
    [0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0], // |
    [0, 1, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0], // }
    [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 1, 1, 0, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0], // ~
];

#[allow(dead_code)]
pub fn generate_glyphs<P: AsRef<str>>(path: P) {
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
