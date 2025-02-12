use crate::serializable::{Serializable, SerializationError};

#[derive(Debug, Clone)]
pub enum SectionType {
    Text,
    SymbolTable,
    RelocationTable,
}

impl TryFrom<u8> for SectionType {
    type Error = SerializationError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(SectionType::Text),
            255 => Ok(SectionType::SymbolTable),
            254 => Ok(SectionType::RelocationTable),
            v => Err(SerializationError::InvalidSectionType(v)),
        }
    }
}

impl From<SectionType> for u8 {
    fn from(value: SectionType) -> Self {
        match value {
            SectionType::Text => 0,
            SectionType::SymbolTable => 255,
            SectionType::RelocationTable => 254,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TextSectionHeader {
    pub bit_length: usize,
}

#[derive(Debug, Clone)]
pub struct SymbolTableHeader {
    pub entry_count: u32,
    pub names_length: u32,
}

#[derive(Debug, Clone)]
pub struct RelocationTableHeader {
    pub entry_count: u32,
    pub names_length: u32,
}

#[derive(Debug, Clone)]
pub enum SectionHeader {
    Text(TextSectionHeader),
    SymbolTable(SymbolTableHeader),
    RelocationTable(RelocationTableHeader),
}

impl Serializable for SectionHeader {
    fn serialize(&self) -> Vec<u8> {
        let mut data = Vec::with_capacity(16);
        match self {
            SectionHeader::Text(header) => {
                data.push(SectionType::Text.into());
                data.extend([0; 7]); // Padding to 8 bytes
                data.extend(header.bit_length.to_le_bytes());
            }
            SectionHeader::SymbolTable(header) => {
                data.push(SectionType::SymbolTable.into());
                data.extend([0; 3]); // Padding to 4 bytes
                data.extend(header.entry_count.to_le_bytes());
                data.extend(header.names_length.to_le_bytes());
                data.extend([0; 4]); // Padding to 16 bytes
            }
            SectionHeader::RelocationTable(header) => {
                data.push(SectionType::RelocationTable.into());
                data.extend([0; 3]); // Padding to 4 bytes
                data.extend(header.entry_count.to_le_bytes());
                data.extend(header.names_length.to_le_bytes());
                data.extend([0; 4]); // Padding to 16 bytes
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
                    data[8], data[9], data[10], data[11], data[12], data[13], data[14], data[15],
                ]) as usize;
                Ok((16, SectionHeader::Text(TextSectionHeader { bit_length })))
            }
            255 | 254 => {
                let entry_count = u32::from_le_bytes([data[4], data[5], data[6], data[7]]);
                let names_length = u32::from_le_bytes([data[8], data[9], data[10], data[11]]);
                let header = if data[0] == 255 {
                    SectionHeader::SymbolTable(SymbolTableHeader {
                        entry_count,
                        names_length,
                    })
                } else {
                    SectionHeader::RelocationTable(RelocationTableHeader {
                        entry_count,
                        names_length,
                    })
                };
                Ok((16, header))
            }
            v => Err(SerializationError::InvalidSectionType(v)),
        }
    }
}

impl SectionHeader {
    pub fn section_size(&self) -> u64 {
        match self {
            SectionHeader::Text(header) => (header.bit_length as u64 + 7) / 8,
            SectionHeader::SymbolTable(header) => {
                (header.entry_count as u64 * 12) + header.names_length as u64
            }
            SectionHeader::RelocationTable(header) => {
                (header.entry_count as u64 * 16) + header.names_length as u64
            }
        }
    }
}
