use super::serializable::*;

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
pub struct SymbolTableHeader {
    entry_count: u32,
    names_length: u32,
}

impl Serializable for SymbolTableHeader {
    fn serialize(&self) -> Vec<u8> {
        let mut data = Vec::with_capacity(9);
        data.push(255); // Special section type for symbol table
        data.extend(self.entry_count.to_le_bytes());
        data.extend(self.names_length.to_le_bytes());
        data
    }

    fn deserialize(data: &[u8]) -> (usize, Self) {
        if data[0] != 255 {
            panic!("Invalid symbol table header");
        }
        let entry_count = u32::from_le_bytes([data[1], data[2], data[3], data[4]]);
        let names_length = u32::from_le_bytes([data[5], data[6], data[7], data[8]]);
        (9, SymbolTableHeader { entry_count, names_length })
    }
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

    pub fn serialize(&self) -> (SymbolTableHeader, Vec<u8>) {
        let mut data = Vec::new();
        
        // Entries
        for entry in &self.entries {
            data.extend((entry.section_id as u32).to_le_bytes());
            data.extend((entry.offset.0 as u32).to_le_bytes());
            data.extend((entry.name_offset as u32).to_le_bytes());
        }

        // Names
        data.extend(&self.names);

        let header = SymbolTableHeader {
            entry_count: self.entries.len() as u32,
            names_length: self.names.len() as u32,
        };

        (header, data)
    }

    pub fn deserialize(header: &SymbolTableHeader, data: &[u8]) -> (usize, Self) {
        let mut offset = 0;
        let mut entries = Vec::new();

        // Read entries
        for _ in 0..header.entry_count {
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

            entries.push(SymbolEntry {
                section_id,
                offset: Address(addr),
                name_offset,
            });
        }

        // Read names
        let names = data[offset..offset + header.names_length as usize].to_vec();
        
        (offset + header.names_length as usize, SymbolTable { entries, names })
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
