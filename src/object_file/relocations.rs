use super::sections::header::{RelocationTableHeader, SectionHeader};
use crate::serializable::*;
use crate::Address;

#[derive(Debug, Clone)]
pub struct Relocation {
    pub symbol: String,
    pub address: Address,
    pub relative: bool,
}

#[derive(Debug, Clone)]
struct RelocationEntry {
    section_id: usize,
    symbol_offset: usize,
    address: Address,
    relative: bool,
}

#[derive(Debug, Clone)]
pub struct RelocationTable {
    entries: Vec<RelocationEntry>,
    names: Vec<u8>,
}

impl RelocationTable {
    pub fn new() -> Self {
        RelocationTable {
            entries: Vec::new(),
            names: Vec::new(),
        }
    }

    pub fn add_relocation(&mut self, section_id: usize, relocation: Relocation) {
        let symbol_offset = self.names.len();
        self.names.extend(relocation.symbol.as_bytes());
        self.names.push(0); // null terminator

        self.entries.push(RelocationEntry {
            section_id,
            symbol_offset,
            address: relocation.address,
            relative: relocation.relative,
        });
    }

    pub fn serialize(&self) -> (SectionHeader, Vec<u8>) {
        let mut data = Vec::new();

        // Entries
        for entry in &self.entries {
            data.extend((entry.section_id as u32).to_le_bytes());
            data.extend((entry.symbol_offset as u32).to_le_bytes());
            data.extend((entry.address.0 as u32).to_le_bytes());
            data.push(entry.relative as u8);
            data.push(0); // padding for alignment
            data.push(0);
            data.push(0);
        }

        // Names
        data.extend(&self.names);

        let header = SectionHeader::RelocationTable(RelocationTableHeader {
            entry_count: self.entries.len() as u32,
            names_length: self.names.len() as u32,
        });

        (header, data)
    }

    pub fn deserialize(
        header: &RelocationTableHeader,
        data: &[u8],
    ) -> Result<(usize, Self), SerializationError> {
        let required_size = (header.entry_count as usize * 16) + header.names_length as usize;
        if data.len() < required_size {
            return Err(SerializationError::DataTooShort);
        }

        let mut offset = 0;
        let mut entries = Vec::new();

        // Read entries
        for _ in 0..header.entry_count {
            if offset + 16 > data.len() {
                return Err(SerializationError::DataTooShort);
            }

            let section_id = u32::from_le_bytes([
                data[offset],
                data[offset + 1],
                data[offset + 2],
                data[offset + 3],
            ]) as usize;
            offset += 4;

            let symbol_offset = u32::from_le_bytes([
                data[offset],
                data[offset + 1],
                data[offset + 2],
                data[offset + 3],
            ]) as usize;
            offset += 4;

            let addr = u32::from_le_bytes([
                data[offset],
                data[offset + 1],
                data[offset + 2],
                data[offset + 3],
            ]) as usize;
            offset += 4;

            let relative = data[offset] != 0;
            offset += 4; // Skip padding bytes too

            if symbol_offset >= header.names_length as usize {
                return Err(SerializationError::InvalidData);
            }

            entries.push(RelocationEntry {
                section_id,
                symbol_offset,
                address: Address(addr),
                relative,
            });
        }

        // Read names
        if offset + header.names_length as usize > data.len() {
            return Err(SerializationError::DataTooShort);
        }
        let names = data[offset..offset + header.names_length as usize].to_vec();

        // Validate that all names are properly null-terminated
        if !names.iter().any(|&b| b == 0) {
            return Err(SerializationError::InvalidData);
        }

        Ok((
            offset + header.names_length as usize,
            RelocationTable { entries, names },
        ))
    }

    pub fn get_relocations(&self, section_id: usize) -> Vec<Relocation> {
        self.entries
            .iter()
            .filter(|entry| entry.section_id == section_id)
            .map(|entry| {
                let mut symbol = String::new();
                let mut i = entry.symbol_offset;
                while i < self.names.len() && self.names[i] != 0 {
                    symbol.push(self.names[i] as char);
                    i += 1;
                }
                Relocation {
                    symbol,
                    address: Address(entry.address.0),
                    relative: entry.relative,
                }
            })
            .collect()
    }
}
