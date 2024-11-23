use super::serializable::*;
use super::sections::header::{SectionHeader, SymbolTableHeader};

#[derive(Debug, Clone)]
pub struct Address(pub usize);

impl std::ops::Add<usize> for Address {
    type Output = Address;
    fn add(self, rhs: usize) -> Self::Output {
        Address(self.0 + rhs)
    }
}

#[derive(Debug, Clone)]
pub struct Symbol {
    pub name: String,
    pub address: Address,
}

#[derive(Debug, Clone)]
struct SymbolEntry {
    section_id: usize,
    offset: Address,
    name_offset: usize,
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

    pub fn add_symbol(&mut self, section_id: usize, symbol: Symbol) {
        let name_offset = self.names.len();
        self.names.extend(symbol.name.as_bytes());
        self.names.push(0); // null terminator

        self.entries.push(SymbolEntry {
            section_id,
            offset: symbol.address,
            name_offset,
        });
    }

    pub fn serialize(&self) -> (SectionHeader, Vec<u8>) {
        let mut data = Vec::new();
        
        // Entries
        for entry in &self.entries {
            data.extend((entry.section_id as u32).to_le_bytes());
            data.extend((entry.offset.0 as u32).to_le_bytes());
            data.extend((entry.name_offset as u32).to_le_bytes());
        }

        // Names
        data.extend(&self.names);

        let header = SectionHeader::SymbolTable(SymbolTableHeader {
            entry_count: self.entries.len() as u32,
            names_length: self.names.len() as u32,
        });

        (header, data)
    }

    pub fn deserialize(header: &SymbolTableHeader, data: &[u8]) -> Result<(usize, Self), SerializationError> {
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
                data[offset], data[offset + 1],
                data[offset + 2], data[offset + 3],
            ]) as usize;
            offset += 4;

            let addr = u32::from_le_bytes([
                data[offset], data[offset + 1],
                data[offset + 2], data[offset + 3],
            ]) as usize;
            offset += 4;

            let name_offset = u32::from_le_bytes([
                data[offset], data[offset + 1],
                data[offset + 2], data[offset + 3],
            ]) as usize;
            offset += 4;

            if name_offset >= header.names_length as usize {
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
        if !names.iter().any(|&b| b == 0) {
            return Err(SerializationError::InvalidData);
        }

        Ok((offset + header.names_length as usize, SymbolTable { entries, names }))
    }

    pub fn get_symbols(&self) -> Vec<Symbol> {
        self.entries.iter().map(|entry| {
            let mut name = String::new();
            let mut i = entry.name_offset;
            while i < self.names.len() && self.names[i] != 0 {
                name.push(self.names[i] as char);
                i += 1;
            }
            Symbol {
                name,
                address: Address(entry.offset.0),
            }
        }).collect()
    }
}
