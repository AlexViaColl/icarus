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
