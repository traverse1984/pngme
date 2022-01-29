use crate::chunk::{self, Chunk};
use crate::chunk_type::ChunkType;
use crate::png::{Png, PngError};
use crate::px::Px;
use rand::Rng;
use std::fs;
use std::io::{ErrorKind, Read, Write};
use std::str::FromStr;

type CodingResult<T = ()> = Result<T, PngError>;

use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::GzBuilder;
use flate2::{Decompress, FlushDecompress};

fn read_png(filename: &str) -> Result<Png, PngError> {
    match fs::read(filename) {
        Ok(buf) => Png::try_from(buf.as_slice()),
        Err(e) => Err(match e.kind() {
            ErrorKind::NotFound => PngError::FileNotFound,
            _ => PngError::FileNotRead,
        }),
    }
}

fn write_png(filename: &str, png: Png) -> CodingResult {
    fs::write(filename, png.as_bytes().as_slice()).map_err(|_| PngError::FileNotWritten)
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

    write_png(filename, png)
}

fn write_remove(filename: &str, chunk_type: &str, checked: bool) -> CodingResult {
    if checked {
        ChunkType::from_str(chunk_type)?.checked_me_type()?;
    }

    let mut png = read_png(filename)?;
    png.remove_chunk(chunk_type)?;
    write_png(filename, png)
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

pub fn scrub(filename: &str) -> CodingResult {
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

    // let f = std::fs::File::create("test.gz").unwrap();
    // GzBuilder::new()
    //     .filename("test.gz")
    //     .write(f, flate2::Compression::default())
    //     .finish()
    //     .unwrap();

    // let f = fs::File::create("test.gz").unwrap();
    // let mut enc = GzBuilder::new()
    //     .filename("test.gz")
    //     .write(f, flate2::Compression::default());

    // enc.write(b"Hello there world, how are you doing?").unwrap();
    // enc.finish().unwrap();

    // let r = std::fs::read("test.gz").unwrap();

    let mut vect = Vec::with_capacity(buf.len() * 10);
    let mut decoder = Decompress::new(true);

    decoder
        .decompress_vec(&buf, &mut vect, FlushDecompress::Finish)
        .unwrap();

    println!("Decoded!");

    Ok(())
}

pub fn test() -> CodingResult {
    let mut redbox = Px::rect(50, 50, Px::hex(0xFFFFFF));
    let red = Px::hex(0xFF0000);
    let mut rng = rand::thread_rng();

    for _ in 0..10000 {
        let y = rng.gen_range(0..50);
        let x = rng.gen_range(0..50);
        let col = rng.gen::<u32>();
        redbox[y][x] = Px::hexa(col);
    }

    let head = Chunk::ihdr(50, 50)?;
    let data = Chunk::idat(&redbox)?;
    let end = Chunk::iend()?;

    let png = Png::from_chunks(vec![head, data, end]);

    write_png("test.png", png)?;

    Ok(())
}
