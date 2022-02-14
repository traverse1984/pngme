use crate::err::*;
use flate2::{read::ZlibDecoder, write::ZlibEncoder, Compression};
use std::{
    fs,
    io::{ErrorKind, Read, Write},
};

pub fn compress(data: &[u8]) -> PngRes<Vec<u8>> {
    let mut compress = ZlibEncoder::new(Vec::new(), Compression::default());

    compress.write_all(data).or(Err(PngErr::CompressError))?;
    compress.finish().or(Err(PngErr::CompressError))
}

pub fn decompress(data: &[u8]) -> PngRes<Vec<u8>> {
    let mut decompress = ZlibDecoder::new(data);
    let mut output = Vec::with_capacity(data.len());
    decompress
        .read_to_end(&mut output)
        .map_or(Err(PngErr::DecompressError), |_| Ok(output))
}

pub fn read(filename: &str) -> PngRes<Vec<u8>> {
    fs::read(filename).map_err(|err| match err.kind() {
        ErrorKind::NotFound => PngErr::FileNotFound,
        _ => PngErr::FileNotRead,
    })
}

pub fn write(filename: &str, data: &[u8]) -> PngRes {
    fs::write(filename, data).map_err(|_| PngErr::FileNotWritten)
}
