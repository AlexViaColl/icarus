pub fn read_str<R: std::io::BufRead>(mut r: R) -> std::io::Result<String> {
    let mut data = vec![];
    r.read_until(0, &mut data)?;
    _ = data.pop();
    Ok(String::from_utf8(data).unwrap())
}
pub fn read_tag<R: std::io::BufRead>(mut r: R) -> std::io::Result<String> {
    let mut b = [0; 4];
    r.read_exact(&mut b)?;
    let mut b = b.to_vec();
    while b[b.len() - 1] == 0 {
        b.pop();
    }
    Ok(String::from_utf8(b.to_vec()).unwrap())
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
pub fn read_u64_le<R: std::io::Read>(mut r: R) -> std::io::Result<u64> {
    let mut b = [0; 8];
    r.read_exact(&mut b)?;
    Ok((b[7] as u64) << 56
        | (b[6] as u64) << 48
        | (b[5] as u64) << 40
        | (b[4] as u64) << 32
        | (b[3] as u64) << 24
        | (b[2] as u64) << 16
        | (b[1] as u64) << 8
        | b[0] as u64)
}
pub fn read_f32_le<R: std::io::Read>(mut r: R) -> std::io::Result<f32> {
    let mut b = [0; 4];
    r.read_exact(&mut b)?;
    Ok(f32::from_le_bytes(b))
}
