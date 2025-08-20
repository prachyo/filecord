use anyhow::*;

pub fn compress_zstd(data: &[u8], level: i32) -> Result<Vec<u8>> {
    let mut enc = zstd::stream::Encoder::new(Vec::new(), level)?;
    enc.include_checksum(true)?;
    enc.auto_finish();
    enc.write_all(data)?;
    Ok(enc.finish()?)
}

pub fn decompress_zstd(data: &[u8]) -> Result<Vec<u8>> {
    let mut dec = zstd::stream::Decoder::new(data)?;
    let mut out = Vec::new();
    std::io::copy(&mut dec, &mut out)?;
    Ok(out)
}
