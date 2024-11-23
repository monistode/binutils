mod serializable;
mod header;
pub mod sections;
pub mod symbols;
pub mod relocations;

pub use serializable::{Architecture, Serializable, SerializationError};
pub use header::ObjectHeader;
pub use sections::*;
pub use symbols::{SymbolTable, Symbol, Address};
pub use relocations::{RelocationTable, Relocation};

#[derive(Debug, Clone)]
pub struct ObjectFile {
    header: ObjectHeader,
    sections: Vec<Section>,
}

impl Serializable for ObjectFile {
    fn serialize(&self) -> Vec<u8> {
        let mut data = Vec::new();
        
        // Create symbol and relocation tables from section data
        let mut symbol_table = SymbolTable::new();
        let mut relocation_table = RelocationTable::new();
        
        for (section_id, section) in self.sections.iter().enumerate() {
            for symbol in section.symbols() {
                symbol_table.add_symbol(section_id, symbol);
            }
            for relocation in section.relocations() {
                relocation_table.add_relocation(section_id, relocation);
            }
        }
        
        let (symbol_header, symbol_data) = symbol_table.serialize();
        let (relocation_header, relocation_data) = relocation_table.serialize();

        // Serialize header with all sections (including symbol and relocation tables)
        let header = ObjectHeader {
            architecture: self.header.architecture,
            section_count: self.sections.len() as u64 + 2, // +2 for symbol and relocation tables
        };
        data.extend(header.serialize());

        // Create and serialize all section headers and data
        let mut section_data = Vec::new();
        let mut headers = Vec::new();

        // Add symbol and relocation table headers first
        headers.push(symbol_header);
        section_data.extend(symbol_data);
        headers.push(relocation_header);
        section_data.extend(relocation_data);

        // Add regular section headers and data next
        for section in &self.sections {
            let (header, bytes) = section.serialize();
            headers.push(header);
            section_data.extend(bytes);
        }

        // Add all headers followed by all section data
        for header in headers {
            data.extend(header.serialize());
        }
        data.extend(section_data);
        
        data
    }

    fn deserialize(data: &[u8]) -> Result<(usize, Self), SerializationError> {
        if data.len() < 9 {
            return Err(SerializationError::DataTooShort);
        }

        // Parse header
        let (header_size, header) = ObjectHeader::deserialize(data)?;
        let mut offset = header_size;

        // Read all section headers
        let mut headers = Vec::new();
        for _ in 0..header.section_count {
            if data.len() < offset + 16 { // Minimum section header size
                return Err(SerializationError::DataTooShort);
            }
            let (size, section_header) = SectionHeader::deserialize(&data[offset..])?;
            headers.push(section_header);
            offset += size;
        }

        // First two sections must be symbol table and relocation table
        let section_count = headers.len();
        if section_count < 2 {
            return Err(SerializationError::InvalidData);
        }
        if !matches!(headers[0], SectionHeader::SymbolTable(_)) {
            return Err(SerializationError::InvalidData);
        }
        if !matches!(headers[1], SectionHeader::RelocationTable(_)) {
            return Err(SerializationError::InvalidData);
        }

        // Ensure no other symbol/relocation table sections exist
        if headers[2..].iter().any(|h| matches!(h, 
            SectionHeader::SymbolTable(_) | SectionHeader::RelocationTable(_))) {
            return Err(SerializationError::InvalidData);
        }

        // Read sections
        let mut sections = Vec::new();
        let mut symbol_table = None;
        let mut relocation_table = None;
        let mut section_data_offset = offset;
        
        // Process symbol and relocation tables first
        for (idx, section_header) in headers.iter().enumerate().take(2) {
            match (idx, section_header) {
                (0, SectionHeader::SymbolTable(symbol_header)) => {
                    let (size, table) = SymbolTable::deserialize(
                        symbol_header,
                        &data[section_data_offset..],
                    )?;
                    symbol_table = Some(table);
                    section_data_offset += size;
                }
                (1, SectionHeader::RelocationTable(relocation_header)) => {
                    let (size, table) = RelocationTable::deserialize(
                        relocation_header,
                        &data[section_data_offset..],
                    )?;
                    relocation_table = Some(table);
                    section_data_offset += size;
                }
                _ => return Err(SerializationError::InvalidData),
            }
        }

        // Process remaining sections with the loaded tables
        let symbol_table = symbol_table.ok_or(SerializationError::InvalidData)?;
        let relocation_table = relocation_table.ok_or(SerializationError::InvalidData)?;

        for (idx, section_header) in headers.iter().enumerate().skip(2) {
            match section_header {
                SectionHeader::Text(_) => {
                    let symbols = symbol_table.get_symbols(idx - 2);
                    let relocations = relocation_table.get_relocations(idx - 2);
                    let (size, section) = Section::deserialize(
                        section_header,
                        &data[section_data_offset..],
                        header.architecture,
                        symbols,
                        relocations
                    )?;
                    sections.push(section);
                    section_data_offset += size;
                }
                _ => return Err(SerializationError::InvalidData),
            }
        }

        Ok((section_data_offset, ObjectFile { 
            header,
            sections 
        }))
    }
}
