use crate::chunk_type::ChunkType;
use crate::png::PngError;
use crc::crc32;
use std::fmt;

pub fn segment4(bytes: &[u8]) -> Result<[u8; 4], PngError> {
    bytes.try_into().map_err(|_| PngError::ShortSegment)
}

#[derive(Debug)]
pub struct Chunk {
    length: u32,
    chunk_type: ChunkType,
    data: Vec<u8>,
    crc: u32,
}

impl Chunk {
    const INT_MAX: usize = 2147483648;

    pub fn new(chunk_type: ChunkType, data: Vec<u8>) -> Self {
        let length = data.len();
        if length > Self::INT_MAX {
            panic!("Data length exceeds specified maximum of 2^31 bytes.");
        }

        let crc = crc::crc32::checksum_ieee(
            chunk_type
                .bytes()
                .iter()
                .chain(data.iter())
                .copied()
                .collect::<Vec<u8>>()
                .as_slice(),
        );

        Self {
            length: length as u32,
            chunk_type,
            data,
            crc,
        }
    }

    pub fn length(&self) -> u32 {
        self.length
    }

    pub fn chunk_type(&self) -> &ChunkType {
        &self.chunk_type
    }

    pub fn data(&self) -> &[u8] {
        &self.data
    }

    pub fn crc(&self) -> u32 {
        self.crc
    }

    pub fn data_as_string(&self) -> Result<String, PngError> {
        String::from_utf8(self.data.to_vec()).map_err(|_| PngError::NotUTF8)
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        self.length
            .to_be_bytes()
            .iter()
            .chain(self.chunk_type.bytes().iter())
            .chain(self.data.iter())
            .chain(self.crc.to_be_bytes().iter())
            .copied()
            .collect()
    }
}

impl TryFrom<&[u8]> for Chunk {
    type Error = PngError;
    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        if bytes.len() < 12 {
            return Err(PngError::ShortChunk);
        }

        let crc_offset = bytes.len() - 4;

        let length = u32::from_be_bytes(segment4(&bytes[0..4])?);
        let chunk_type = ChunkType::from_bytes(&bytes[4..8])?;
        let crc = u32::from_be_bytes(segment4(&bytes[crc_offset..])?);
        let data = bytes[8..crc_offset].to_vec();

        if data.len() != length as usize {
            return Err(PngError::LengthMismatch);
        }

        if crc::crc32::checksum_ieee(&bytes[4..crc_offset]) != crc {
            return Err(PngError::CRCMismatch);
        }

        Ok(Chunk {
            length,
            chunk_type,
            data,
            crc,
        })
    }
}

impl fmt::Display for Chunk {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let Self {
            length,
            chunk_type,
            crc,
            ..
        } = self;

        let message = self.data_as_string().unwrap_or(String::from("???"));

        write!(
            f,
            "Chunk [{}] {} (0x{:x}): {}",
            chunk_type, length, crc, message
        )
    }
}

#[derive(Debug)]
pub struct ChunkIter<'a> {
    cur: &'a [u8],
    tainted: bool,
}

impl<'a> ChunkIter<'a> {
    pub fn new(cur: &'a [u8]) -> Self {
        Self {
            cur,
            tainted: false,
        }
    }
}

impl<'a> Iterator for ChunkIter<'a> {
    type Item = Result<Chunk, PngError>;
    fn next(&mut self) -> Option<Result<Chunk, PngError>> {
        if self.tainted || self.cur.len() == 0 {
            return None;
        } else if self.cur.len() < 12 {
            self.tainted = true;
            return Some(Err(PngError::ShortChunk));
        }

        let len = segment4(&self.cur[0..4]).unwrap();
        let len = u32::from_be_bytes(len) as usize + 12;

        if self.cur.len() < len {
            self.tainted = true;
            return Some(Err(PngError::ShortChunk));
        }

        let chunk = &self.cur[0..len];
        self.cur = &self.cur[len..];

        Chunk::try_from(chunk).map_or_else(
            |e| {
                self.tainted = true;
                Some(Err(e))
            },
            |chunk| Some(Ok(chunk)),
        )
    }
}

#[cfg(test)]
mod iter_tests {
    use super::*;

    fn valid_chunk() -> Vec<u8> {
        let data_length: u32 = 11;
        let chunk_type = "ItEr".as_bytes();
        let message = "Hello World".as_bytes();
        let crc: u32 = 3520753346;

        data_length
            .to_be_bytes()
            .iter()
            .chain(chunk_type.iter())
            .chain(message.iter())
            .chain(crc.to_be_bytes().iter())
            .copied()
            .collect()
    }

    fn invalid_chunk() -> Vec<u8> {
        let data_length: u32 = 11; // Valid
        let chunk_type = "iter".as_bytes(); // Invalid
        let message = "Hello World".as_bytes();
        let crc: u32 = 1234; // Invalid

        data_length
            .to_be_bytes()
            .iter()
            .chain(chunk_type.iter())
            .chain(message.iter())
            .chain(crc.to_be_bytes().iter())
            .copied()
            .collect()
    }

    #[test]
    fn test_valid_chunks() {
        let bytes: Vec<u8> = vec![valid_chunk(), valid_chunk(), valid_chunk()]
            .into_iter()
            .flatten()
            .collect();

        let ref_chunk = valid_chunk();
        for chunk in ChunkIter::new(bytes.as_slice()) {
            assert!(chunk.is_ok());
            assert_eq!(chunk.unwrap().as_bytes().as_slice(), ref_chunk.as_slice());
        }
    }

    #[test]
    fn test_invalid_chunks() {
        let bytes: Vec<u8> = vec![valid_chunk(), invalid_chunk(), valid_chunk()]
            .into_iter()
            .flatten()
            .collect();

        let ref_chunk = valid_chunk();
        let mut iter = ChunkIter::new(bytes.as_slice());

        let first = iter.next().unwrap();
        assert!(first.is_ok());
        assert_eq!(first.unwrap().as_bytes().as_slice(), ref_chunk.as_slice());

        let second = iter.next().unwrap();
        assert!(second.is_err());

        let third = iter.next();
        assert!(third.is_none());
    }

    #[test]
    fn test_invalid_length() {
        let bytes = valid_chunk();
        let big_chunk: Vec<u8> = 12345678u32
            .to_be_bytes()
            .into_iter()
            .chain((&bytes[4..]).iter().copied())
            .chain(valid_chunk().into_iter())
            .collect();

        let mut iter = ChunkIter::new(big_chunk.as_slice());

        assert!(iter.next().unwrap().is_err());
        assert!(iter.next().is_none());
    }

    #[test]
    fn test_empty_buffer() {
        assert!(ChunkIter::new(&[]).next().is_none());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chunk_type::ChunkType;
    use std::str::FromStr;

    fn testing_chunk() -> Chunk {
        let data_length: u32 = 42;
        let chunk_type = "RuSt".as_bytes();
        let message_bytes = "This is where your secret message will be!".as_bytes();
        let crc: u32 = 2882656334;

        let chunk_data: Vec<u8> = data_length
            .to_be_bytes()
            .iter()
            .chain(chunk_type.iter())
            .chain(message_bytes.iter())
            .chain(crc.to_be_bytes().iter())
            .copied()
            .collect();

        Chunk::try_from(chunk_data.as_ref()).unwrap()
    }

    #[test]
    fn test_chunk_length() {
        let chunk = testing_chunk();
        assert_eq!(chunk.length(), 42);
    }

    #[test]
    fn test_chunk_type() {
        let chunk = testing_chunk();
        assert_eq!(chunk.chunk_type().to_string(), String::from("RuSt"));
    }

    #[test]
    fn test_chunk_string() {
        let chunk = testing_chunk();
        let chunk_string = chunk.data_as_string().unwrap();
        let expected_chunk_string = String::from("This is where your secret message will be!");
        assert_eq!(chunk_string, expected_chunk_string);
    }

    #[test]
    fn test_chunk_crc() {
        let chunk = testing_chunk();
        assert_eq!(chunk.crc(), 2882656334);
    }

    #[test]
    fn test_valid_chunk_from_bytes() {
        let data_length: u32 = 42;
        let chunk_type = "RuSt".as_bytes();
        let message_bytes = "This is where your secret message will be!".as_bytes();
        let crc: u32 = 2882656334;

        let chunk_data: Vec<u8> = data_length
            .to_be_bytes()
            .iter()
            .chain(chunk_type.iter())
            .chain(message_bytes.iter())
            .chain(crc.to_be_bytes().iter())
            .copied()
            .collect();

        let chunk = Chunk::try_from(chunk_data.as_ref()).unwrap();

        let chunk_string = chunk.data_as_string().unwrap();
        let expected_chunk_string = String::from("This is where your secret message will be!");

        assert_eq!(chunk.length(), 42);
        assert_eq!(chunk.chunk_type().to_string(), String::from("RuSt"));
        assert_eq!(chunk_string, expected_chunk_string);
        assert_eq!(chunk.crc(), 2882656334);
    }

    #[test]
    fn test_invalid_chunk_from_bytes() {
        let data_length: u32 = 42;
        let chunk_type = "RuSt".as_bytes();
        let message_bytes = "This is where your secret message will be!".as_bytes();
        let crc: u32 = 2882656333;

        let chunk_data: Vec<u8> = data_length
            .to_be_bytes()
            .iter()
            .chain(chunk_type.iter())
            .chain(message_bytes.iter())
            .chain(crc.to_be_bytes().iter())
            .copied()
            .collect();

        let chunk = Chunk::try_from(chunk_data.as_ref());

        assert!(chunk.is_err());
    }

    #[test]
    pub fn test_chunk_trait_impls() {
        let data_length: u32 = 42;
        let chunk_type = "RuSt".as_bytes();
        let message_bytes = "This is where your secret message will be!".as_bytes();
        let crc: u32 = 2882656334;

        let chunk_data: Vec<u8> = data_length
            .to_be_bytes()
            .iter()
            .chain(chunk_type.iter())
            .chain(message_bytes.iter())
            .chain(crc.to_be_bytes().iter())
            .copied()
            .collect();

        let chunk: Chunk = TryFrom::try_from(chunk_data.as_ref()).unwrap();

        let _chunk_string = format!("{}", chunk);
    }
}
