use crate::chunk::Chunk;
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

pub fn encode(filename: &str, chunk_type: &str, message: &str) -> CodingResult {
    let mut png = read_png(filename)?;

    let chunk = Chunk::new(
        ChunkType::from_str(chunk_type)?,
        message.as_bytes().to_vec(),
    );

    png.remove_chunk(chunk_type).ok();
    png.append_chunk(chunk);

    write_png(filename, png.as_bytes().as_slice())
}

pub fn decode(filename: &str, chunk_type: &str) -> CodingResult<String> {
    let png = read_png(filename)?;
    png.chunk_by_type(chunk_type).map_or_else(
        || Err(PngError::ChunkNotFound),
        |chunk| Ok(chunk.to_string()),
    )
}

pub fn remove(filename: &str, chunk_type: &str) -> CodingResult {
    let mut png = read_png(filename)?;
    png.remove_chunk(chunk_type)?;
    write_png(filename, png.as_bytes().as_slice())
}

pub fn print(filename: &str) -> CodingResult<String> {
    Ok(read_png(filename)?.to_string())
}
