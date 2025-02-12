use crate::{Architecture, SerializationError, Symbol};

use super::header::TextSegmentHeader;
use bitvec::prelude::*;

#[derive(Debug, Clone)]
pub struct TextSegment {
    pub offset: usize,
    pub data: BitVec,
    pub symbols: Vec<Symbol>,
}

impl TextSegment {
    pub fn new(offset: usize, data: BitVec, symbols: Vec<Symbol>) -> Self {
        TextSegment {
            offset,
            data,
            symbols,
        }
    }

    pub fn serialize(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        for i in 0..((self.data.len() + 7) / 8) {
            let mut byte = 0u8;
            for j in 0..8 {
                if i * 8 + j < self.data.len() && self.data[i * 8 + j] {
                    byte |= 1 << j;
                }
            }
            bytes.push(byte);
        }
        bytes
    }

    pub fn deserialize(
        header: &TextSegmentHeader,
        data: &[u8],
        architecture: Architecture,
        symbols: Vec<Symbol>,
    ) -> Result<(usize, Self), SerializationError> {
        let required_bytes = (header.bit_length as usize + 7) / 8;
        if data.len() < required_bytes {
            return Err(SerializationError::DataTooShort);
        }

        match architecture {
            Architecture::Stack => {
                let mut bits = BitVec::new();
                for i in 0..header.bit_length as usize {
                    let bit = data[i / 8] & (1 << (i % 8)) != 0;
                    bits.push(bit);
                }
                let bytes_read = (header.bit_length + 7) as usize / 8;
                Ok((
                    bytes_read,
                    TextSegment {
                        offset: header.location,
                        data: bits,
                        symbols,
                    },
                ))
            }
        }
    }
}
