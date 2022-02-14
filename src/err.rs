use std::fmt;

pub type PngRes<T = ()> = Result<T, PngErr>;

#[derive(Debug)]
pub enum PngErr {
    DivisionByZero,
    IntOverflow,
    InvalidHeader,
    InvalidByte,
    InvalidIHDR,
    IHDRWidthOverflow,
    IHDRHeightOverflow,
    InvalidSegment,
    ShortChunk,
    LengthMismatch,
    WidthMismatch,
    ZeroWidth,
    CRCMismatch,
    NotUTF8,
    ExpectNonCritical,
    ExpectReservedBit,
    ExpectPrivate,
    ChunkNotFound,
    FileNotFound,
    FileNotRead,
    FileNotWritten,
    DataLengthMismatch,
    CompressError,
    DecompressError,
}

use PngErr::*;

impl PngErr {
    pub fn not_or(cond: bool, err: PngErr) -> PngRes {
        cond.then(|| Err(err)).unwrap_or(Ok(()))
    }

    pub fn is_or(cond: bool, err: PngErr) -> PngRes {
        (!cond).then(|| Err(err)).unwrap_or(Ok(()))
    }
}

impl std::error::Error for PngErr {}

impl fmt::Display for PngErr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let message = match self {
            DivisionByZero => "Attempt was made to divide by zero.",
            IntOverflow => "Calculation overflowed.",
            InvalidHeader => "Data did not contain a valid PNG header.",
            InvalidByte => "Chunk type contained invalid bytes.",
            InvalidIHDR => "The IHDR chunk was invalid or not found.",
            IHDRWidthOverflow => "The IHDR chunk width exceeds the PNG max.",
            IHDRHeightOverflow => "The IHDR chunk height exceeds the PNG max.",
            InvalidSegment => "Expected exactly 4 bytes.",
            ShortChunk => "Data is too short to be a valid chunk.",
            LengthMismatch => "Chunk data did not match it's expected length.",
            WidthMismatch => "Scanlines had variable width when converting from 2d.",
            ZeroWidth => "Image would have zero width when converting from 2d.",
            CRCMismatch => "The computed CRC did not match, data may be corrupt.",
            NotUTF8 => "Chunk data is not UT8.",
            ExpectNonCritical => "Expected a lowercase character at position 1 (non-critical).",
            ExpectReservedBit => "Expected an uppercase character at position 3 (reserved bit).",
            ExpectPrivate => "Expected a lowercase character at position 2 (private).",
            ChunkNotFound => "That chunk was not found.",
            FileNotFound => "That file was not found.",
            FileNotRead => "Could not read that file.",
            FileNotWritten => "Could not write that file.",
            DataLengthMismatch => "Data does not align to the image dimensions.",
            CompressError => "An error occurred compressing image data.",
            DecompressError => "An error occurred decompressing image data.",
        };

        write!(f, "{}", message)
    }
}
