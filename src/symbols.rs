use crate::executable::segments::flags::SegmentFlags;
use crate::executable::segments::SegmentHeader;
use crate::object_file::{SectionHeader, SymbolTableHeader};

use super::address::Address;
use super::serializable::*;

#[derive(Debug, Clone)]
pub struct Symbol {
    pub name: String,
    pub address: Address,
}

#[derive(Debug, Clone)]
struct SymbolEntry {
    section_id: u32,
    offset: Address,
    name_offset: u32,
}

#[derive(Debug, Clone)]
pub struct SymbolTable {
    entries: Vec<SymbolEntry>,
    names: Vec<u8>,
}

impl SymbolTable {
    pub fn new() -> Self {
        SymbolTable {
            entries: Vec::new(),
            names: Vec::new(),
        }
    }

    pub fn add_symbol(&mut self, section_id: u32, symbol: Symbol) {
        let name_offset = self.names.len() as u32;
        self.names.extend(symbol.name.as_bytes());
        self.names.push(0); // null terminator

        self.entries.push(SymbolEntry {
            section_id,
            offset: symbol.address,
            name_offset,
        });
    }

    pub fn serialize_as_section(&self) -> (SectionHeader, Vec<u8>) {
        let mut data = Vec::new();

        // Entries
        for entry in &self.entries {
            data.extend(entry.section_id.to_le_bytes());
            data.extend((entry.offset.0 as u32).to_le_bytes());
            data.extend(entry.name_offset.to_le_bytes());
        }

        // Names
        data.extend(&self.names);

        let header = SectionHeader::SymbolTable(SymbolTableHeader {
            entry_count: self.entries.len() as u32,
            names_length: self.names.len() as u32,
        });

        (header, data)
    }

    pub fn serialize_as_segment(&self) -> (SegmentHeader, Vec<u8>) {
        let mut data = Vec::new();

        // Entries
        for entry in &self.entries {
            data.extend(entry.section_id.to_le_bytes());
            data.extend((entry.offset.0 as u32).to_le_bytes());
            data.extend(entry.name_offset.to_le_bytes());
        }

        // Names
        data.extend(&self.names);

        let header = SegmentHeader {
            // TODO do what rust does best
            address_space_start: 0, // Special -> special section type
            address_space_size: self.entries.len() as u64, // Since it's special, we can use this field
            // for whatever
            // Special -> disk_bit_count doesn't have to follow the rules either
            disk_bit_count: data.len(),
            flags: SegmentFlags {
                executable: false,
                writable: false,
                readable: false,
                special: true,
            },
        };

        (header, data)
    }

    pub fn deserialize_section(
        header: &SymbolTableHeader,
        data: &[u8],
    ) -> Result<(usize, Self), SerializationError> {
        let required_size = (header.entry_count as usize * 12) + header.names_length as usize;
        if data.len() < required_size {
            return Err(SerializationError::DataTooShort);
        }

        let mut offset = 0;
        let mut entries = Vec::new();

        // Read entries
        for _ in 0..header.entry_count {
            if offset + 12 > data.len() {
                return Err(SerializationError::DataTooShort);
            }

            let section_id = u32::from_le_bytes([
                data[offset],
                data[offset + 1],
                data[offset + 2],
                data[offset + 3],
            ]);
            offset += 4;

            let addr = u32::from_le_bytes([
                data[offset],
                data[offset + 1],
                data[offset + 2],
                data[offset + 3],
            ]) as usize;
            offset += 4;

            let name_offset = u32::from_le_bytes([
                data[offset],
                data[offset + 1],
                data[offset + 2],
                data[offset + 3],
            ]);
            offset += 4;

            if name_offset >= header.names_length {
                return Err(SerializationError::InvalidData);
            }

            entries.push(SymbolEntry {
                section_id,
                offset: Address(addr),
                name_offset,
            });
        }

        // Read names
        if offset + header.names_length as usize > data.len() {
            return Err(SerializationError::DataTooShort);
        }
        let names = data[offset..offset + header.names_length as usize].to_vec();

        // Validate that all names are properly null-terminated
        if names.len() > 0 {
            if !names.iter().any(|&b| b == 0) {
                return Err(SerializationError::InvalidData);
            }
        }

        Ok((
            offset + header.names_length as usize,
            SymbolTable { entries, names },
        ))
    }

    pub fn deserialize_segment(
        header: &SegmentHeader,
        data: &[u8],
    ) -> Result<(usize, Self), SerializationError> {
        let required_size = header.disk_bit_count as usize;
        if data.len() < required_size {
            return Err(SerializationError::DataTooShort);
        }

        let mut offset = 0;
        let mut entries = Vec::new();

        // Read entries
        for _ in 0..header.address_space_size {
            if offset + 12 > data.len() {
                return Err(SerializationError::DataTooShort);
            }

            let section_id = u32::from_le_bytes([
                data[offset],
                data[offset + 1],
                data[offset + 2],
                data[offset + 3],
            ]);
            offset += 4;

            let addr = u32::from_le_bytes([
                data[offset],
                data[offset + 1],
                data[offset + 2],
                data[offset + 3],
            ]) as usize;
            offset += 4;

            let name_offset = u32::from_le_bytes([
                data[offset],
                data[offset + 1],
                data[offset + 2],
                data[offset + 3],
            ]);
            offset += 4;

            if name_offset >= header.disk_bit_count as u32 - header.address_space_size as u32 * 12 {
                return Err(SerializationError::InvalidData);
            }

            entries.push(SymbolEntry {
                section_id,
                offset: Address(addr),
                name_offset,
            });
        }

        // Read names
        let names = data[offset..header.disk_bit_count as usize].to_vec();

        // Validate that all names are properly null-terminated
        if !names.iter().any(|&b| b == 0) {
            return Err(SerializationError::InvalidData);
        }

        Ok((
            header.disk_bit_count as usize,
            SymbolTable { entries, names },
        ))
    }

    pub fn get_symbols(&self, section_id: u32) -> Vec<Symbol> {
        self.entries
            .iter()
            .filter(|entry| entry.section_id == section_id)
            .map(|entry| {
                let mut name = String::new();
                let mut i = entry.name_offset as usize;
                while i < self.names.len() && self.names[i] != 0 {
                    name.push(self.names[i] as char);
                    i += 1;
                }
                Symbol {
                    name,
                    address: Address(entry.offset.0),
                }
            })
            .collect()
    }
}
