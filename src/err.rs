use std::fmt;

pub type PngRes<T = ()> = Result<T, PngErr>;

#[derive(Debug)]
pub enum PngErr {
    InvalidHeader,
    InvalidByte,
    ShortSegment,
    ShortChunk,
    ExpectNonCritical,
    ExpectReservedBit,
    ExpectPrivate,
    LengthMismatch,
    CRCMismatch,
    NotUTF8,
    ChunkNotFound,
    FileNotFound,
    FileNotRead,
    FileNotWritten,
    IHDRWidthOverflow,
    IHDRHeightOverflow,
    ZeroWidth,
    ZeroHeight,
    WidthOverflow,
    HeightOverflow,
    WidthMismatch,
    DataLengthMismatch,
    FilterLengthMismatch,
    OutOfBoundsX,
    OutOfBoundsY,
    IntOverflow,
    XOverflow,
    YOverflow,
}

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
        write!(f, "{:?}", self)
    }
}
