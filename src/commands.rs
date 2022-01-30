use crate::chunk::{self, Chunk};
use crate::chunk_type::ChunkType;
use crate::img::{self, ImageData};
use crate::png::{Png, PngError};
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
    let mut rect = ImageData::new_bg(500, 500, img::hex(0xDEDEDE)).unwrap();
    let mut rng = rand::thread_rng();

    rect.slice(5..19, 10..39).fill(img::hex(0x0000C4));

    for y in 0usize..10 {
        for x in 0usize..20 {
            if x == 0 && y == 0 {
                continue;
            }

            rect.copy_filter((..25, ..50), (25 * x, 50 * y), |&px| {
                let count = (y * 10 + x) as u32;
                let [r, g, b, _] = (px + (count * 16) << count % 32).to_be_bytes();
                img::rgba(r, g, b, 200 - count as u8)
            });

            //
        }
    }

    // for _ in 0..1000 {
    //     let x = rng.gen_range(0..rect.width());
    //     let y = rng.gen_range(0..rect.height());
    //     let col = rng.gen::<u32>();
    //     rect.slice(x..x + 15, y..y + 15).iter_mut().for_each(|px| {
    //         *px = *px ^ col << 1;
    //     });
    // }

    let head = Chunk::ihdr(500, 500)?;
    let data = Chunk::idat(rect.to_bytes().as_slice())?;
    let end = Chunk::iend()?;

    let png = Png::from_chunks(vec![head, data, end]);

    write_png("test.png", png)?;

    Ok(())
}
