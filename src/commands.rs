use crate::img::{self, Img};
use crate::{Image, Quad};
use rand::Rng;
use std::error::Error;
use std::fs;
use std::io::{ErrorKind, Read, Write};
use std::str::FromStr;

use crate::{col, Color};

use crate::err::*;
use crate::png::{Chunk, ChunkType, Png};
use flate2::read::{ZlibDecoder, ZlibEncoder};
use flate2::{Decompress, FlushDecompress, Status};

fn read_png(filename: &str) -> PngRes<Png> {
    match fs::read(filename) {
        Ok(buf) => Png::try_from(buf.as_slice()),
        Err(e) => Err(match e.kind() {
            ErrorKind::NotFound => PngErr::FileNotFound,
            _ => PngErr::FileNotRead,
        }),
    }
}

fn write_png(filename: &str, png: Png) -> PngRes {
    fs::write(filename, png.as_bytes().as_slice()).map_err(|_| PngErr::FileNotWritten)
}

fn write_encode(filename: &str, chunk_type: &str, message: &str, checked: bool) -> PngRes {
    let chunk = Chunk::new(
        ChunkType::from_str(chunk_type)?,
        message.as_bytes().to_vec(),
    );

    if checked {
        chunk.checked_me_chunk()?;
    }

    let mut png = read_png(filename)?;

    png.remove_chunk(chunk_type).ok();
    png.append_chunk(chunk);

    write_png(filename, png)
}

fn write_remove(filename: &str, chunk_type: &str, checked: bool) -> PngRes {
    if checked {
        ChunkType::from_str(chunk_type)?.checked_me_type()?;
    }

    let mut png = read_png(filename)?;
    png.remove_chunk(chunk_type)?;
    write_png(filename, png)
}

pub fn encode(filename: &str, chunk_type: &str, message: &str) -> PngRes {
    write_encode(filename, chunk_type, message, true)
}

pub fn encode_unchecked(filename: &str, chunk_type: &str, message: &str) -> PngRes {
    write_encode(filename, chunk_type, message, false)
}

pub fn decode(filename: &str, chunk_type: &str) -> PngRes<String> {
    let png = read_png(filename)?;
    png.chunk_by_type(chunk_type).map_or_else(
        || Err(PngErr::ChunkNotFound),
        |chunk| chunk.data_as_string(),
    )
}

pub fn remove(filename: &str, chunk_type: &str) -> PngRes {
    write_remove(filename, chunk_type, true)
}

pub fn remove_unchecked(filename: &str, chunk_type: &str) -> PngRes {
    write_remove(filename, chunk_type, false)
}

pub fn print(filename: &str) -> PngRes<String> {
    Ok(read_png(filename)?.to_string())
}

pub fn scrub(filename: &str) -> PngRes {
    // let mut png = read_png(filename)?;
    // png.scrub();
    // write_png(filename, png)?;
    // Ok(())

    let png = read_png(filename)?;
    let mut buf = Vec::new();

    for chunk in png.chunks() {
        if chunk.chunk_type().bytes() == "IDAT".as_bytes() {
            buf.extend_from_slice(chunk.data());
        }
    }

    let mut output = Vec::new();
    let mut decoder = ZlibDecoder::new(buf.as_slice());
    let output = decoder.read_to_end(&mut output);

    println!("Len: {}", output.unwrap());

    println!("Decoded!");

    Ok(())
}

pub fn test() -> PngRes {
    let mut rect = Img::new_bg(1250, 1250, col!(0x000000));
    let mut slice = rect.slice(0..=5, 0..=5);

    let mut colr = 0x000000u32;

    for w in 0..5 {
        for x in 0..5 {
            for y in 0..50 {
                for z in 0..50 {
                    let colr = col!(((1 + x) * (1 + w)) * 10, (y + 1) * 5, (z + 1) * 5);
                    slice
                        .pos(x as u32 * 250 + z as u32 * 5, w as u32 * 250 + y as u32 * 5)
                        .fill(colr);
                }
            }
        }
    }

    let head = Chunk::ihdr(1250, 1250)?;
    let data = Chunk::idat(rect.to_bytes().as_slice())?;
    let end = Chunk::iend()?;

    let png = Png::from_chunks(vec![head, data, end]);

    write_png("test.png", png)?;

    Ok(())
}
