use crate::parsing::*;

use std::io::Cursor;

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
#[derive(Debug, Default)]
pub struct DnaField {
    pub ttype: String,
    pub name: String,
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn dna() -> std::io::Result<()> {
        let bytes = fs::read("/home/alex/thirdparty/blender-git/blender/dna.bin").unwrap();
        assert!(bytes.len() != 0);
        let dna = parse_dna(&bytes)?;
        println!("#names:  {}", dna.names.len());
        println!("#types:   {}", dna.types.len());
        println!("#structs: {}", dna.structs.len());

        Ok(())
    }
}
