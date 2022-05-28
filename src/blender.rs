use crate::parsing::*;

use std::io::Cursor;
use std::io::Read;

#[derive(Debug, Default)]
pub struct DnaInstance {}

#[derive(Debug, Default)]
pub struct Dna {
    pub names: Vec<String>,
    pub types: Vec<String>,
    pub types_len: Vec<usize>,
    pub structs: Vec<DnaStruct>,
}
#[derive(Debug, Default)]
pub struct DnaStruct {
    pub name: String,
    pub fields: Vec<DnaField>,
}
#[derive(Default)]
pub struct DnaField {
    pub ttype: String,
    pub name: String,
}
impl std::fmt::Debug for DnaField {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.ttype, self.name)
    }
}

#[derive(Debug, Default)]
pub struct Blend {
    pub blocks: Vec<BlendBlock>,
    pub dna: Dna,
}
#[derive(Debug, Default)]
pub struct BlendBlock {
    pub tag: String,
    pub size: usize,
    pub addr: usize,
    pub sdna_idx: usize,
    pub count: usize,
    pub data: Vec<u8>,
}

pub fn parse_dna(bytes: &[u8]) -> std::io::Result<Dna> {
    let mut reader = Cursor::new(bytes);
    let sdna_tag = read_tag(&mut reader)?;
    assert_eq!(sdna_tag, "SDNA");
    let name_tag = read_tag(&mut reader)?;
    assert_eq!(name_tag, "NAME");
    let names_count = read_u32_le(&mut reader)?;
    let mut names = Vec::with_capacity(names_count as usize);
    for _ in 0..names_count {
        let name = read_str(&mut reader)?;
        names.push(name);
    }
    align_to(&mut reader, 4)?;
    let type_tag = read_tag(&mut reader)?;
    assert_eq!(type_tag, "TYPE");
    let types_count = read_u32_le(&mut reader)?;
    let mut types = Vec::with_capacity(types_count as usize);
    for _ in 0..types_count {
        let ttype = read_str(&mut reader)?;
        types.push(ttype);
    }
    align_to(&mut reader, 4)?;
    let tlen_tag = read_tag(&mut reader)?;
    assert_eq!(tlen_tag, "TLEN");
    let mut types_len = Vec::with_capacity(types_count as usize);
    for _ in 0..types_count {
        let tlen = read_u16_le(&mut reader)?;
        types_len.push(tlen as usize);
    }
    align_to(&mut reader, 4)?;
    let struct_tag = read_tag(&mut reader)?;
    assert_eq!(struct_tag, "STRC");
    let structs_count = read_u32_le(&mut reader)?;
    let mut structs = Vec::with_capacity(structs_count as usize);
    for _ in 0..structs_count {
        let struct_type = read_u16_le(&mut reader)? as usize;
        let field_count = read_u16_le(&mut reader)? as usize;
        let mut fields = vec![];
        for _ in 0..field_count {
            let field_type = read_u16_le(&mut reader)? as usize;
            let field_name = read_u16_le(&mut reader)? as usize;
            fields.push(DnaField {
                ttype: types[field_type].clone(),
                name: names[field_name].clone(),
            });
        }
        structs.push(DnaStruct {
            name: types[struct_type].clone(),
            fields,
        });
    }

    Ok(Dna {
        names,
        types,
        types_len,
        structs,
    })
}
fn align_to(mut r: &mut Cursor<&[u8]>, n: usize) -> std::io::Result<()> {
    assert!(n == 2 || n == 4 || n == 8 || n == 16);
    let n = n as u64;
    while r.position() & (n - 1) != 0 {
        _ = read_u8(&mut r)?;
    }
    Ok(())
}

impl Dna {
    pub fn instantiate(&self, struct_name: &str, data: &[u8]) -> DnaInstance {
        let size = self.get_struct_size(struct_name);
        assert_eq!(size, data.len());
        todo!()
    }

    pub fn get_struct_size(&self, name: &str) -> usize {
        let s = self.structs.iter().find(|s| s.name == name).unwrap();
        //println!("{:#?}", s);
        let mut size = 0;
        for field in &s.fields {
            size += match (field.name.as_str(), field.ttype.as_str()) {
                (name, _t) if name.starts_with("*") => 8,
                (name, t) if name.ends_with("]") => {
                    let mut arr_size = 1;
                    for (start, _) in field.name.match_indices("[") {
                        let end = start + field.name[start..].find("]").unwrap();
                        arr_size *= &field.name[start + 1..end].parse::<usize>().unwrap();
                    }
                    let found = self.types.iter().enumerate().find(|(_, tt)| tt == &t);
                    let type_size = if let Some(tt) = found {
                        self.types_len[tt.0]
                    } else {
                        todo!()
                    };
                    //println!("{} {}*{}", field.name, arr_size, type_size);
                    arr_size * type_size
                }
                (_name, t) => {
                    let found = self.types.iter().enumerate().find(|(_, tt)| tt == &t);
                    if let Some(tt) = found {
                        //println!("{} {}", field.name, self.types_len[tt.0]);
                        self.types_len[tt.0]
                    } else {
                        todo!()
                    }
                }
            }
        }
        size
    }
}

pub fn parse_blend(bytes: &[u8]) -> std::io::Result<Blend> {
    let mut blend = Blend::default();

    let mut reader = Cursor::new(bytes);
    const BLEND_HEADER_SIZE: usize = 12;
    let mut b = [0; BLEND_HEADER_SIZE];
    reader.read_exact(&mut b)?;
    assert_eq!(&b[..7], b"BLENDER");
    assert_eq!(b[7], '-' as u8); // TODO: We only support 8 byte pointers
    assert_eq!(b[8], 'v' as u8); // TODO: We only support little endian
    let version = std::str::from_utf8(&b[9..12]).unwrap().parse::<usize>().unwrap();
    assert!(version > 280);

    let mut dna = None;
    loop {
        let tag = read_tag(&mut reader)?;
        let block_size = read_u32_le(&mut reader)? as usize;
        let addr = read_u64_le(&mut reader)? as usize;
        let sdna_idx = read_u32_le(&mut reader)? as usize;
        let count = read_u32_le(&mut reader)? as usize;

        let mut data = vec![0; block_size];
        reader.read_exact(&mut data)?;

        if tag == "DNA1" {
            dna = Some(parse_dna(&data)?);
        }

        blend.blocks.push(BlendBlock {
            tag: tag.clone(),
            size: block_size,
            addr,
            sdna_idx,
            count,
            data,
        });

        if tag == "ENDB" {
            break;
        }
    }
    blend.dna = dna.expect("Could not find required DNA1 block in .blend file");

    let mut data_structs_set = std::collections::BTreeSet::new();
    let mut verts = vec![];
    let mut edges = vec![];
    let mut loops = vec![];
    let mut polys = vec![];
    for block in &blend.blocks {
        match block.tag.as_str() {
            "REND" => {
                //println!("[{}] sdna: {} {:#?}", block.tag.as_str(), block.sdna_idx, blend.dna.structs[block.sdna_idx]);
            }
            "DATA" => {
                // ARegion, ArmatureModifierData, Base, Bone, BrushGpencilSettings, Collection,
                // CollectionChild, CollectionObject, ConsoleLine, CurveMapPoint, CurveMapping,
                // CurveProfile, CurveProfilePoint, CustomDataLayer, Editing, FileSelectParams,
                // FreestyleLineSet, GpPaint, GpSculptPaint, GpVertexPaint, GpWeightPaint,
                // IDProperty, ImageTile, LayerCollection, LineStyleGeometryModifier_Sampling,
                // Link, MDeformVert, MDeformWeight, MEdge, MLoop, MLoopUV, MPoly, MVert,
                // MaterialGPencilStyle, MirrorModifierData, PaintToolSlot, PaletteColor, Panel,
                // PanelCategoryStack, PartDeflect, PreviewImage, RegionView3D, RenderSlot,
                // SceneRenderView, ScrArea, ScrEdge, ScrGlobalAreaData,
                // ScrVert, Sculpt, SequencerToolSettings, SpaceAction, SpaceBUts, SpaceConsole,
                // SpaceFile, SpaceImage, SpaceInfo, SpaceNode, SpaceOops, SpaceSpreadsheet, SpaceStatusBar,
                // SpaceText, SpaceTopBar, SpreadsheetColumn, SpreadsheetColumnID, SpreadsheetContextObject,
                // Stereo3dFormat, ToolSettings, TreeStore, TreeStoreElem,
                // VPaint, View3D, ViewLayer, WorkSpaceDataRelation, WorkSpaceInstanceHook,
                // bConstraint, bDeformGroup, bKinematicConstraint, bNode, bNodeLink, bNodeSocket,
                // bNodeSocketValueFloat, bNodeSocketValueRGBA, bNodeSocketValueVector, bNodeTree, bNodeTreePath,
                // bPose, bPoseChannel, bRotateLikeConstraint, bToolRef, uiList, wmWindow
                let sname = blend.dna.structs[block.sdna_idx].name.as_str();
                data_structs_set.insert(sname.to_string());
                //println!("[{}] sdna: {} {}", block.tag.as_str(), block.sdna_idx, sname);

                let mut r = Cursor::new(&block.data);
                let block_size = blend.dna.get_struct_size(sname) as u64;
                if sname == "MVert" || sname == "MEdge" || sname == "MLoop" || sname == "MPoly" {
                    println!("[{}]", sname);
                }
                for _ in 0..block.count {
                    let pos = r.position();
                    match sname {
                        "MVert" => {
                            let x = read_f32_le(&mut r)?;
                            let y = read_f32_le(&mut r)?;
                            let z = read_f32_le(&mut r)?;
                            println!("  {}, {}, {}", x, y, z);
                            verts.push((x, y, z));
                        }
                        "MEdge" => {
                            let v1 = read_u32_le(&mut r)? as usize;
                            let v2 = read_u32_le(&mut r)? as usize;
                            println!("  v1: {:?}, v2: {:?}", verts[v1], verts[v2]);
                            edges.push((v1, v2));
                        }
                        "MLoop" => {
                            let v = read_u32_le(&mut r)? as usize;
                            let e = read_u32_le(&mut r)? as usize;
                            let _v1 = verts[edges[e].0];
                            let _v2 = verts[edges[e].1];
                            println!("  v: {:?}, e: {:?} -> {:?}", verts[v], _v1, _v2);
                            loops.push((v, e));
                        }
                        "MPoly" => {
                            let loopstart = read_u32_le(&mut r)? as usize;
                            let totloop = read_u32_le(&mut r)?;
                            println!("  loopstart: {}, totloop: {}", loopstart, totloop);
                            polys.push((loopstart, totloop));
                        }
                        _ => {}
                    }
                    let total_bytes_read = r.position() - pos;
                    r.set_position(r.position() + (block_size - total_bytes_read));
                }
            }
            "DNA1" => {}
            "GLOB" => {
                // FileGlobal
                //println!("[{}] sdna: {} {:#?}", block.tag.as_str(), block.sdna_idx, blend.dna.structs[block.sdna_idx]);
            }
            "TEST" => {
                // Thumbnail
                let mut r = Cursor::new(&block.data);
                let _width = read_u32_le(&mut r)?;
                let _height = read_u32_le(&mut r)?;
                let _pixels = &block.data[8..];
                //println!("Thumbnail: {}x{}, pixels: {}\n", width, height, pixels.len());

                // Write as .ppm
                //let pixels = data[8..].chunks(4).map(|c| &c[..3]).flatten().map(|b| *b).collect::<Vec<_>>();
                //let mut file = fs::File::create("/home/alex/blendthumb.ppm")?;
                //file.write_all(format!("P6\n{} {}\n255\n", width, height).as_bytes())?;
                //file.write_all(&pixels)?;
            }
            "USER" => {
                // UserDef
                //println!("[{}] sdna: {} {:#?}", block.tag.as_str(), block.sdna_idx, blend.dna.structs[block.sdna_idx]);
            }
            "ENDB" => {}
            "AR" => {
                // bArmature
                //println!("[{}] sdna: {} {:#?}", block.tag.as_str(), block.sdna_idx, blend.dna.structs[block.sdna_idx]);
            }
            "BR" => {
                // Brush
                //println!("[{}] sdna: {} {:#?}", block.tag.as_str(), block.sdna_idx, blend.dna.structs[block.sdna_idx]);
            }
            "CA" => {
                // Camera
                //println!("[{}] sdna: {} {:#?}", block.tag.as_str(), block.sdna_idx, blend.dna.structs[block.sdna_idx]);
            }
            "GD" => {
                // bGPdata?
                //println!("[{}] sdna: {} {:#?}", block.tag.as_str(), block.sdna_idx, blend.dna.structs[block.sdna_idx]);
            }
            "GR" => {
                // Collection
                //println!("[{}] sdna: {} {:#?}", block.tag.as_str(), block.sdna_idx, blend.dna.structs[block.sdna_idx]);
            }
            "IM" => {
                // Image
                //println!("[{}] sdna: {} {:#?}", block.tag.as_str(), block.sdna_idx, blend.dna.structs[block.sdna_idx]);
            }
            "LA" => {
                // Lamp
                //println!("[{}] sdna: {} {:#?}", block.tag.as_str(), block.sdna_idx, blend.dna.structs[block.sdna_idx]);
            }
            "LS" => {
                // FreestyleLineStyle
                //println!("[{}] sdna: {} {:#?}", block.tag.as_str(), block.sdna_idx, blend.dna.structs[block.sdna_idx]);
            }
            "MA" => {
                // Material
                //println!("[{}] sdna: {} {:#?}", block.tag.as_str(), block.sdna_idx, blend.dna.structs[block.sdna_idx]);
            }
            "ME" => {
                // Mesh
                //println!("[{}] sdna: {} {:#?}", block.tag.as_str(), block.sdna_idx, blend.dna.structs[block.sdna_idx]);
            }
            "OB" => {
                // Object
                //println!("[{}] sdna: {} {:#?}", block.tag.as_str(), block.sdna_idx, blend.dna.structs[block.sdna_idx]);
            }
            "PL" => {
                // Palette
                //println!("[{}] sdna: {} {:#?}", block.tag.as_str(), block.sdna_idx, blend.dna.structs[block.sdna_idx]);
            }
            "SC" => {
                // Scene
                //println!("[{}] sdna: {} {:#?}", block.tag.as_str(), block.sdna_idx, blend.dna.structs[block.sdna_idx]);
            }
            "SN" => {
                // bScreen
                //println!("[{}] sdna: {} {:#?}", block.tag.as_str(), block.sdna_idx, blend.dna.structs[block.sdna_idx]);
            }
            "TX" => {
                // Text
                //println!("[{}] sdna: {} {:#?}", block.tag.as_str(), block.sdna_idx, blend.dna.structs[block.sdna_idx]);
            }
            "WM" => {
                // wmWindowManager
                //println!("[{}] sdna: {} {:#?}", block.tag.as_str(), block.sdna_idx, blend.dna.structs[block.sdna_idx]);
            }
            "WO" => {
                // World
                //println!("[{}] sdna: {} {:#?}", block.tag.as_str(), block.sdna_idx, blend.dna.structs[block.sdna_idx]);
            }
            "WS" => {
                // WorkSpace
                //println!("[{}] sdna: {} {:#?}", block.tag.as_str(), block.sdna_idx, blend.dna.structs[block.sdna_idx]);
            }
            t => panic!("Unknown block tag: {}", t),
        }
    }
    //println!("{} {:#?}", data_structs_set.len(), data_structs_set);

    Ok(blend)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    #[allow(unused_imports)]
    use std::path::{Path, PathBuf};

    #[test]
    fn dna() -> std::io::Result<()> {
        let bytes = fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join("test/dna.bin")).unwrap();
        assert!(bytes.len() != 0);
        let dna = parse_dna(&bytes)?;
        println!("#names:  {}", dna.names.len());
        println!("#types:   {}", dna.types.len());
        println!("#structs: {}", dna.structs.len());

        Ok(())
    }

    #[test]
    fn blend() -> std::io::Result<()> {
        let test_files = [
            Path::new(env!("CARGO_MANIFEST_DIR")).join("test/startup.blend"),
            //PathBuf::from("/home/alex/tmp/base_model.blend"),
            //PathBuf::from("/home/alex/tmp/base_model.blend1"),
        ];
        for path in test_files {
            println!("{:?}", path);
            let bytes = fs::read(path).unwrap();
            assert!(bytes.len() != 0);
            let _ = parse_blend(&bytes)?;
        }
        Ok(())
    }
}
