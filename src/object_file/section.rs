use bitvec::prelude::*;
use super::serializable::*;
use super::symbols::Symbol;

#[derive(Debug, Clone)]
pub struct TextSection {
    pub data: BitVec,
    pub symbols: Vec<Symbol>,
}

#[derive(Debug, Clone)]
pub struct TextSectionHeader {
    bit_length: u64,
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

    pub fn deserialize(header: &SectionHeader, data: &[u8], architecture: Architecture) 
        -> Result<(usize, Self), SerializationError> 
    {
        match header {
            SectionHeader::Text(header) => {
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
            v => Err(SerializationError::InvalidSectionType(v)),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Section {
    Text(TextSection),
}

impl Section {
    pub fn serialize(&self) -> (SectionHeader, Vec<u8>) {
        match self {
            Section::Text(text) => {
                let bytes = text.serialize();
                let section_header = SectionHeader::Text(TextSectionHeader {
                    bit_length: text.data.len() as u64,
                });
                (section_header, bytes)
            }
        }
    }

    pub fn deserialize(header: &SectionHeader, data: &[u8], architecture: Architecture) 
        -> Result<(usize, Self), SerializationError> 
    {
        match header {
            SectionHeader::Text { .. } => {
                let (size, section) = TextSection::deserialize(header, data, architecture)?;
                Ok((size, Section::Text(section)))
            }
        }
    }

    pub fn symbols(&self) -> Vec<Symbol> {
        match self {
            Section::Text(text) => text.symbols.clone(),
        }
    }
}
