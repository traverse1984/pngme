mod chunk;
mod chunk_type;
mod png;

use crate::err::*;

pub use chunk::*;
pub use chunk_type::ChunkType;
pub use png::Png;

use crate::INT_MAX;

pub fn to_usize(val: u32) -> PngRes<usize> {
    Ok(val.try_into().map_err(|_| PngErr::IntOverflow)?)
}

pub fn expect_usize(val: u32) -> usize {
    val.try_into().expect("Could not convert u32 to usize.")
}

pub fn to_u32(val: usize) -> PngRes<u32> {
    Ok(val.try_into().map_err(|_| PngErr::IntOverflow)?)
}

pub fn area(width: u32, height: u32) -> PngRes<usize> {
    let (width, height) = (to_usize(width)?, to_usize(height)?);
    usize::checked_mul(width, height).ok_or(PngErr::IntOverflow)
}

pub fn clamp_u32(val: u32) -> u32 {
    u32::clamp(val, 1, INT_MAX)
}
