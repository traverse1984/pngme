use crate::{err::*, png::Png};
use flate2::{read::ZlibDecoder, write::ZlibEncoder, Compression};
use std::{
    fs,
    io::{ErrorKind, Read, Write},
};

fn compress(data: &[u8]) -> PngRes<Vec<u8>> {
    let mut compress = ZlibEncoder::new(Vec::new(), Compression::default());

    compress.write_all(data).or(Err(PngErr::EncodeError))?;
    compress.finish().or(Err(PngErr::EncodeError))
}

// let png = read_png(filename)?;
//     let mut buf = Vec::new();

//     for chunk in png.chunks() {
//         if chunk.chunk_type().bytes() == "IDAT".as_bytes() {
//             buf.extend_from_slice(chunk.data());
//         }
//     }

fn decompress(data: &[u8]) -> PngRes<Vec<u8>> {
    let mut decompress = ZlibDecoder::new(data);
    let mut output = Vec::with_capacity(data.len());
    decompress
        .read_to_end(&mut output)
        .map_or(Err(PngErr::DecodeError), |_| Ok(output))
}

pub fn read_png(filename: &str) -> PngRes<Png> {
    Png::try_from(
        fs::read(filename)
            .map_err(|err| match err.kind() {
                ErrorKind::NotFound => PngErr::FileNotFound,
                _ => PngErr::FileNotRead,
            })?
            .as_slice(),
    )
}

pub fn write_png(filename: &str, png: &Png) -> PngRes {
    fs::write(filename, png.as_bytes().as_slice()).map_err(|_| PngErr::FileNotWritten)
}
