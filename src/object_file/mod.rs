mod serializable;
mod header;
pub mod sections;
pub mod symbols;

pub use serializable::{Architecture, Serializable, SerializationError};
pub use header::ObjectHeader;
pub use sections::*;
pub use symbols::{Address, Symbol, SymbolTable};

#[derive(Debug, Clone)]
pub struct ObjectFile {
    header: ObjectHeader,
    symbol_table: Option<SymbolTable>,
    sections: Vec<Section>,
}

impl Serializable for ObjectFile {
    fn serialize(&self) -> Vec<u8> {
        let mut data = Vec::new();
        
        // Create symbol table section - ensure we only use one
        let mut symbol_table = self.symbol_table.clone().unwrap_or_else(SymbolTable::new);
        for (section_id, section) in self.sections.iter().enumerate() {
            for symbol in section.symbols() {
                symbol_table.add_symbol(section_id, symbol);
            }
        }
        let (symbol_header, symbol_data) = symbol_table.serialize();

        // Serialize header with all sections (including symbol table)
        let header = ObjectHeader {
            architecture: self.header.architecture,
            section_count: self.sections.len() as u64 + 1, // +1 for symbol table
        };
        data.extend(header.serialize());

        // Create and serialize all section headers
        let mut section_data = Vec::new();
        let mut headers = Vec::new();

        // Add symbol table header first
        headers.push(symbol_header);
        section_data.extend(symbol_data);

        // Add regular section headers and data
        for section in &self.sections {
            let (header, bytes) = section.serialize();
            headers.push(header);
            section_data.extend(bytes);
        }

        // Add all section headers
        for header in headers {
            data.extend(header.serialize());
        }

        // Add all section data
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

        // First section must be symbol table
        if headers.is_empty() || headers[0].section_type != SectionType::SymbolTable {
            return Err(SerializationError::InvalidData);
        }

        // Ensure no other symbol table sections exist
        if headers[1..].iter().any(|h| h.section_type == SectionType::SymbolTable) {
            return Err(SerializationError::InvalidData);
        }

        // Read sections
        let mut sections = Vec::new();
        let mut symbol_table = None;
        let mut section_data_offset = offset;
        
        // Process all sections
        for (idx, section_header) in headers.iter().enumerate() {
            if data.len() < section_data_offset {
                return Err(SerializationError::DataTooShort);
            }

            if idx == 0 {
                // First section is symbol table
                let (size, table) = SymbolTable::deserialize(
                    section_header,
                    &data[section_data_offset..],
                )?;
                symbol_table = Some(table);
                section_data_offset += size;
            } else {
                // Regular sections
                let (size, section) = Section::deserialize(
                    section_header,
                    &data[section_data_offset..],
                    header.architecture,
                )?;
                sections.push(section);
                section_data_offset += size;
            }
        }

        Ok((section_data_offset, ObjectFile { 
            header, 
            symbol_table,
            sections 
        }))
    }
}
