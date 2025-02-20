use crate::address::Address;
use crate::SerializationError;

#[derive(Debug, Clone)]
pub struct Symbol {
    pub name: String,
    pub address: Address,
}

#[derive(Debug, Clone)]
pub struct SymbolTable {
    entries: Vec<(String, Address)>, // (name, absolute_address) pairs
}

impl SymbolTable {
    pub fn new() -> Self {
        SymbolTable {
            entries: Vec::new(),
        }
    }

    pub fn add_symbol(&mut self, name: String, address: Address) {
        self.entries.push((name, address));
    }

    pub fn serialize(&self) -> (Vec<u8>, u64, u64) {
        let mut data = Vec::new();
        let mut name_data = Vec::new();

        // Write entries
        for (name, address) in &self.entries {
            data.extend((address.0 as u64).to_le_bytes());
            data.extend((name_data.len() as u32).to_le_bytes());
            name_data.extend(name.as_bytes());
            name_data.push(0); // null terminator
        }

        // Append name data
        data.extend(&name_data);

        (data, self.entries.len() as u64, name_data.len() as u64)
    }

    pub fn deserialize(data: &[u8], symbol_count: u64, name_table_size: u64) -> Result<(usize, Self), SerializationError> {
        let mut entries = Vec::new();
        let mut offset = 0;
        let name_table_start = symbol_count as usize * 12; // 8 bytes for address + 4 for name offset

        // Validate total size
        let required_size = name_table_start + name_table_size as usize;
        if data.len() < required_size {
            return Err(SerializationError::DataTooShort);
        }

        // Read entries
        for _ in 0..symbol_count {
            if offset + 12 > data.len() {
                return Err(SerializationError::DataTooShort);
            }

            let address = u64::from_le_bytes([
                data[offset],
                data[offset + 1],
                data[offset + 2],
                data[offset + 3],
                data[offset + 4],
                data[offset + 5],
                data[offset + 6],
                data[offset + 7],
            ]);
            offset += 8;

            let name_offset = u32::from_le_bytes([
                data[offset],
                data[offset + 1],
                data[offset + 2],
                data[offset + 3],
            ]) as usize;
            offset += 4;

            if name_offset >= name_table_size as usize {
                return Err(SerializationError::InvalidData);
            }

            // Read name
            let mut name = String::new();
            let mut i = name_table_start + name_offset;
            while i < data.len() && data[i] != 0 {
                name.push(data[i] as char);
                i += 1;
            }
            if i >= data.len() || data[i] != 0 {
                return Err(SerializationError::InvalidData);
            }

            entries.push((name, Address(address as usize)));
        }

        Ok((required_size, SymbolTable { entries }))
    }

    pub fn symbols(&self) -> Vec<Symbol> {
        self.entries
            .iter()
            .map(|(name, address)| Symbol {
                name: name.clone(),
                address: *address,
            })
            .collect()
    }

    pub fn find_symbol(&self, name: &str) -> Option<Address> {
        self.entries
            .iter()
            .find(|(symbol_name, _)| symbol_name == name)
            .map(|(_, address)| *address)
    }
} 