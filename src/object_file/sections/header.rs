use crate::object_file::serializable::{Serializable, SerializationError};

#[derive(Debug, Clone)]
pub enum SectionType {
    Text,
    SymbolTable,
}

impl TryFrom<u8> for SectionType {
    type Error = SerializationError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(SectionType::Text),
            255 => Ok(SectionType::SymbolTable),
            v => Err(SerializationError::InvalidSectionType(v)),
        }
    }
}

impl From<SectionType> for u8 {
    fn from(value: SectionType) -> Self {
        match value {
            SectionType::Text => 0,
            SectionType::SymbolTable => 255,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TextSectionHeader {
    pub bit_length: u64,
}

#[derive(Debug, Clone)]
pub struct SymbolTableHeader {
    pub entry_count: u32,
    pub names_length: u32,
}

#[derive(Debug, Clone)]
pub enum SectionHeader {
    Text(TextSectionHeader),
    SymbolTable(SymbolTableHeader),
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
            }
        }
        data
    }

    fn deserialize(data: &[u8]) -> Result<(usize, Self), SerializationError> {
        if data.len() < 16 {
            return Err(SerializationError::DataTooShort);
        }

        let section_type = SectionType::try_from(data[0])?;
        match section_type {
            SectionType::Text => {
                let bit_length = u64::from_le_bytes([
                    data[8], data[9], data[10], data[11],
                    data[12], data[13], data[14], data[15],
                ]);
                Ok((16, SectionHeader::Text(TextSectionHeader { bit_length })))
            }
            SectionType::SymbolTable => {
                let entry_count = u32::from_le_bytes([
                    data[4], data[5], data[6], data[7],
                ]);
                let names_length = u32::from_le_bytes([
                    data[8], data[9], data[10], data[11],
                ]);
                Ok((16, SectionHeader::SymbolTable(SymbolTableHeader { 
                    entry_count, 
                    names_length 
                })))
            }
        }
    }
} 