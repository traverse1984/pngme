use crate::chunk::{self, Chunk};
use crate::chunk_type::ChunkType;
use crate::png::{Png, PngError};
use std::fs;
use std::io::ErrorKind;
use std::str::FromStr;

type CodingResult<T = ()> = Result<T, PngError>;

fn read_png(filename: &str) -> Result<Png, PngError> {
    match fs::read(filename) {
        Ok(buf) => Png::try_from(buf.as_slice()),
        Err(e) => Err(match e.kind() {
            ErrorKind::NotFound => PngError::FileNotFound,
            _ => PngError::FileNotRead,
        }),
    }
}

fn write_png(filename: &str, buf: &[u8]) -> CodingResult {
    fs::write(filename, buf).map_err(|_| PngError::FileNotWritten)
}

fn write_encode(filename: &str, chunk_type: &str, message: &str, checked: bool) -> CodingResult {
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

    write_png(filename, png.as_bytes().as_slice())
}

fn write_remove(filename: &str, chunk_type: &str, checked: bool) -> CodingResult {
    if checked {
        ChunkType::from_str(chunk_type)?.checked_me_type()?;
    }

    let mut png = read_png(filename)?;
    png.remove_chunk(chunk_type)?;
    write_png(filename, png.as_bytes().as_slice())
}

pub fn encode(filename: &str, chunk_type: &str, message: &str) -> CodingResult {
    write_encode(filename, chunk_type, message, true)
}

pub fn encode_unchecked(filename: &str, chunk_type: &str, message: &str) -> CodingResult {
    write_encode(filename, chunk_type, message, false)
}

pub fn decode(filename: &str, chunk_type: &str) -> CodingResult<String> {
    let png = read_png(filename)?;
    png.chunk_by_type(chunk_type).map_or_else(
        || Err(PngError::ChunkNotFound),
        |chunk| chunk.data_as_string(),
    )
}

pub fn remove(filename: &str, chunk_type: &str) -> CodingResult {
    write_remove(filename, chunk_type, true)
}

pub fn remove_unchecked(filename: &str, chunk_type: &str) -> CodingResult {
    write_remove(filename, chunk_type, false)
}

pub fn print(filename: &str) -> CodingResult<String> {
    Ok(read_png(filename)?.to_string())
}
