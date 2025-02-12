use super::header::TextSectionHeader;
use crate::object_file::relocations::Relocation;
use crate::serializable::{Architecture, SerializationError};
use crate::symbols::Symbol;
use bitvec::prelude::*;

#[derive(Debug, Clone)]
pub struct TextSection {
    pub data: BitVec,
    pub symbols: Vec<Symbol>,
    pub relocations: Vec<Relocation>,
}

impl TextSection {
    pub fn new(data: BitVec, symbols: Vec<Symbol>, relocations: Vec<Relocation>) -> Self {
        TextSection {
            data,
            symbols,
            relocations,
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
        header: &TextSectionHeader,
        data: &[u8],
        symbols: Vec<Symbol>,
        relocations: Vec<Relocation>,
    ) -> Result<(usize, Self), SerializationError> {
        let required_bytes = (header.bit_length as usize + 7) / 8;
        if data.len() < required_bytes {
            return Err(SerializationError::DataTooShort);
        }

        let mut bits = BitVec::new();
        for i in 0..header.bit_length as usize {
            let bit = data[i / 8] & (1 << (i % 8)) != 0;
            bits.push(bit);
        }
        let bytes_read = (header.bit_length + 7) as usize / 8;
        Ok((
            bytes_read,
            TextSection {
                data: bits,
                symbols,
                relocations,
            },
        ))
    }
}
