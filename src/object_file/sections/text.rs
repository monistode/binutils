use bitvec::prelude::*;
use crate::object_file::serializable::{Architecture, Serializable};
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

    pub fn deserialize(header: &SectionHeader, data: &[u8], architecture: Architecture) -> (usize, Self) {
        match header {
            SectionHeader::Text(header) => {
                match architecture {
                    Architecture::Stack => {
                        let mut bits = BitVec::new();
                        for i in 0..header.bit_length as usize {
                            let bit = data[i / 8] & (1 << (i % 8)) != 0;
                            bits.push(bit);
                        }
                        let bytes_read = (header.bit_length + 7) as usize / 8;
                        (bytes_read, TextSection { data: bits, symbols: Vec::new() })
                    },
                    _ => {
                        let mut bits = BitVec::new();
                        for i in 0..header.bit_length as usize * 8 {
                            let bit = data[i / 8] & (1 << (i % 8)) != 0;
                            bits.push(bit);
                        }
                        (header.bit_length as usize, TextSection { data: bits, symbols: Vec::new() })
                    }
                }
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
        }
        data
    }

    fn deserialize(data: &[u8]) -> (usize, Self) {
        match data[0] {
            0 => {
                let bit_length = u64::from_le_bytes([
                    data[8], data[9], data[10], data[11],
                    data[12], data[13], data[14], data[15],
                ]);
                (16, SectionHeader::Text(TextSectionHeader { bit_length }))
            }
            _ => panic!("Invalid section type"),
        }
    }
}
