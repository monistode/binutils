use bitvec::prelude::*;
use crate::object_file::serializable::{Architecture, Serializable, SerializationError};
use crate::object_file::symbols::Symbol;

#[derive(Debug, Clone)]
pub struct TextSection {
    pub data: BitVec,
    pub symbols: Vec<Symbol>,
}

#[derive(Debug, Clone)]
pub struct TextSectionHeader {
    pub bit_length: u64,
}

#[derive(Debug, Clone)]
pub enum SectionHeader {
    Text(TextSectionHeader),
    SymbolTable {
        entry_count: u32,
        names_length: u32,
    },
}

impl TextSection {
    pub fn new(data: BitVec, symbols: Vec<Symbol>) -> Self {
        TextSection { data, symbols }
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
        architecture: Architecture
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
                Ok((bytes_read, TextSection { data: bits, symbols: Vec::new() }))
            }
        }
    }
}

impl Serializable for SectionHeader {
    fn serialize(&self) -> Vec<u8> {
        let mut data = Vec::with_capacity(16);
        match self {
            SectionHeader::Text(header) => {
                data.push(0); // Section type
                data.extend([0; 7]); // Padding to 8 bytes
                data.extend(header.bit_length.to_le_bytes());
            }
            SectionHeader::SymbolTable { entry_count, names_length } => {
                data.push(255); // Section type for symbol table
                data.extend([0; 3]); // Padding to 4 bytes
                data.extend(entry_count.to_le_bytes());
                data.extend(names_length.to_le_bytes());
            }
        }
        data
    }

    fn deserialize(data: &[u8]) -> Result<(usize, Self), SerializationError> {
        if data.len() < 16 {
            return Err(SerializationError::DataTooShort);
        }

        match data[0] {
            0 => {
                let bit_length = u64::from_le_bytes([
                    data[8], data[9], data[10], data[11],
                    data[12], data[13], data[14], data[15],
                ]);
                Ok((16, SectionHeader::Text(TextSectionHeader { bit_length })))
            }
            255 => {
                let entry_count = u32::from_le_bytes([
                    data[4], data[5], data[6], data[7],
                ]);
                let names_length = u32::from_le_bytes([
                    data[8], data[9], data[10], data[11],
                ]);
                Ok((16, SectionHeader::SymbolTable { entry_count, names_length }))
            }
            v => Err(SerializationError::InvalidSectionType(v)),
        }
    }
}
