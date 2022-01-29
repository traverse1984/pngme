use crate::chunk;
use crate::png::PngError;
use std::fmt;
use std::str::FromStr;

#[derive(Eq, Debug)]
pub struct ChunkType {
    bytes: [u8; 4],
}

impl ChunkType {
    pub fn checked_me_type(&self) -> Result<&Self, PngError> {
        if self.is_critical() {
            Err(PngError::ExpectNonCritical)
        } else if self.is_public() {
            Err(PngError::ExpectPrivate)
        } else if !self.is_reserved_bit_valid() {
            Err(PngError::ExpectReservedBit)
        } else {
            Ok(self)
        }
    }

    pub fn bytes(&self) -> [u8; 4] {
        self.bytes.clone()
    }

    pub fn is_critical(&self) -> bool {
        (b'A'..=b'Z').contains(&self.bytes[0])
    }

    pub fn is_public(&self) -> bool {
        (b'A'..=b'Z').contains(&self.bytes[1])
    }

    pub fn is_safe_to_copy(&self) -> bool {
        (b'a'..=b'z').contains(&self.bytes[3])
    }

    pub fn is_reserved_bit_valid(&self) -> bool {
        (b'A'..=b'Z').contains(&self.bytes[2])
    }

    pub fn is_valid(&self) -> bool {
        for byte in &self.bytes {
            match byte {
                b'a'..=b'z' | b'A'..=b'Z' => continue,
                _ => return false,
            }
        }
        self.is_reserved_bit_valid()
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, PngError> {
        let bytes = chunk::segment4(&bytes)?;

        for byte in &bytes {
            match byte {
                b'a'..=b'z' | b'A'..=b'Z' => continue,
                _ => return Err(PngError::InvalidByte),
            }
        }

        Ok(ChunkType { bytes })
    }
}

impl FromStr for ChunkType {
    type Err = PngError;
    fn from_str(chunk_str: &str) -> Result<Self, Self::Err> {
        Self::from_bytes(chunk_str.as_bytes())
    }
}

impl TryFrom<[u8; 4]> for ChunkType {
    type Error = PngError;
    fn try_from(bytes: [u8; 4]) -> Result<Self, Self::Error> {
        Self::from_bytes(&bytes)
    }
}

impl PartialEq for ChunkType {
    fn eq(&self, rhs: &Self) -> bool {
        self.bytes == rhs.bytes
    }
}

impl fmt::Display for ChunkType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", String::from_utf8_lossy(&self.bytes))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::convert::TryFrom;
    use std::str::FromStr;

    #[test]
    pub fn test_bytes_from_bytes() {
        let expected = [82, 117, 83, 116];
        let actual = ChunkType::try_from([82, 117, 83, 116]).unwrap();

        assert_eq!(expected, actual.bytes());
    }

    #[test]
    pub fn test_bytes_from_str() {
        let expected = ChunkType::try_from([82, 117, 83, 116]).unwrap();
        let actual = ChunkType::from_str("RuSt").unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    pub fn test_bytes_is_critical() {
        let chunk = ChunkType::from_str("RuSt").unwrap();
        assert!(chunk.is_critical());
    }

    #[test]
    pub fn test_bytes_is_not_critical() {
        let chunk = ChunkType::from_str("ruSt").unwrap();
        assert!(!chunk.is_critical());
    }

    #[test]
    pub fn test_bytes_is_public() {
        let chunk = ChunkType::from_str("RUSt").unwrap();
        assert!(chunk.is_public());
    }

    #[test]
    pub fn test_bytes_is_not_public() {
        let chunk = ChunkType::from_str("RuSt").unwrap();
        assert!(!chunk.is_public());
    }

    #[test]
    pub fn test_bytes_is_reserved_bit_valid() {
        let chunk = ChunkType::from_str("RuSt").unwrap();
        assert!(chunk.is_reserved_bit_valid());
    }

    #[test]
    pub fn test_bytes_is_reserved_bit_invalid() {
        let chunk = ChunkType::from_str("Rust").unwrap();
        assert!(!chunk.is_reserved_bit_valid());
    }

    #[test]
    pub fn test_bytes_is_safe_to_copy() {
        let chunk = ChunkType::from_str("RuSt").unwrap();
        assert!(chunk.is_safe_to_copy());
    }

    #[test]
    pub fn test_bytes_is_unsafe_to_copy() {
        let chunk = ChunkType::from_str("RuST").unwrap();
        assert!(!chunk.is_safe_to_copy());
    }

    #[test]
    pub fn test_valid_chunk_is_valid() {
        let chunk = ChunkType::from_str("RuSt").unwrap();
        assert!(chunk.is_valid());
    }

    #[test]
    pub fn test_invalid_chunk_is_valid() {
        let chunk = ChunkType::from_str("Rust").unwrap();
        assert!(!chunk.is_valid());

        let chunk = ChunkType::from_str("Ru1t");
        assert!(chunk.is_err());
    }

    #[test]
    pub fn test_bytes_string() {
        let chunk = ChunkType::from_str("RuSt").unwrap();
        assert_eq!(&chunk.to_string(), "RuSt");
    }

    #[test]
    pub fn test_bytes_trait_impls() {
        let bytes_1: ChunkType = TryFrom::try_from([82, 117, 83, 116]).unwrap();
        let bytes_2: ChunkType = FromStr::from_str("RuSt").unwrap();
        let _chunk_string = format!("{}", bytes_1);
        let _are_chunks_equal = bytes_1 == bytes_2;
    }
}
