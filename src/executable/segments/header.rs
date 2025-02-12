use crate::{Serializable, SerializationError};

#[derive(Debug, Clone)]
pub enum SegmentType {
    Text,
    SymbolTable,
}

impl TryFrom<u8> for SegmentType {
    type Error = SerializationError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(SegmentType::Text),
            255 => Ok(SegmentType::SymbolTable),
            v => Err(SerializationError::InvalidSegmentType(v)),
        }
    }
}

impl From<SegmentType> for u8 {
    fn from(value: SegmentType) -> Self {
        match value {
            SegmentType::Text => 0,
            SegmentType::SymbolTable => 255,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TextSegmentHeader {
    pub location: usize,
    pub bit_length: u64,
}

#[derive(Debug, Clone)]
pub struct SymbolTableHeader {
    pub entry_count: u32,
    pub names_length: u32,
}

#[derive(Debug, Clone)]
pub enum SegmentHeader {
    Text(TextSegmentHeader),
    SymbolTable(SymbolTableHeader),
}

impl Serializable for SegmentHeader {
    fn serialize(&self) -> Vec<u8> {
        let mut data = Vec::with_capacity(16);
        match self {
            SegmentHeader::Text(header) => {
                data.push(SegmentType::Text.into()); // len=1
                data.extend(vec![0u8; 7]); // len=8
                data.extend(header.location.to_le_bytes()); // len=16
                data.extend(header.bit_length.to_le_bytes()); // len=24
            }
            SegmentHeader::SymbolTable(header) => {
                data.push(SegmentType::SymbolTable.into());
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
                let location = usize::from_le_bytes([
                    data[8], data[9], data[10], data[11], data[12], data[13], data[14], data[15],
                ]);
                let bit_length = u64::from_le_bytes([
                    data[16], data[17], data[18], data[19], data[20], data[21], data[22], data[23],
                ]);
                Ok((
                    24,
                    SegmentHeader::Text(TextSegmentHeader {
                        location,
                        bit_length,
                    }),
                ))
            }
            255 => {
                let entry_count = u32::from_le_bytes([data[4], data[5], data[6], data[7]]);
                let names_length = u32::from_le_bytes([data[8], data[9], data[10], data[11]]);
                Ok((
                    16,
                    SegmentHeader::SymbolTable(SymbolTableHeader {
                        entry_count,
                        names_length,
                    }),
                ))
            }
            v => Err(SerializationError::InvalidSegmentType(v)),
        }
    }
}

impl SegmentHeader {
    pub fn segment_size(&self) -> usize {
        match self {
            SegmentHeader::Text(header) => (header.bit_length as usize + 7) / 8,
            SegmentHeader::SymbolTable(header) => {
                (header.entry_count as usize * 12) + header.names_length as usize
            }
        }
    }
}
