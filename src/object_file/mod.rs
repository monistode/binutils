pub mod header;
pub mod placed;
pub mod relocations;
pub mod sections;

pub use header::ObjectHeader;
pub use relocations::{Relocation, RelocationTable};
pub use sections::*;

use crate::{Architecture, Serializable, SerializationError, SymbolTable};

#[derive(Debug, Clone)]
pub struct ObjectFile {
    architecture: Architecture,
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
                symbol_table.add_symbol(section_id as u32, symbol);
            }
            for relocation in section.relocations() {
                relocation_table.add_relocation(section_id as u32, relocation);
            }
        }

        // Serialize header with all sections (including symbol and relocation tables)
        let header = ObjectHeader {
            architecture: self.architecture,
            section_count: self.sections.len() as u64 + 2, // +2 for symbol and relocation tables
        };
        data.extend(header.serialize());

        // Create and serialize all section headers and data
        let mut section_data = Vec::new();
        let mut headers = Vec::new();

        // Add regular section headers first
        for section in &self.sections {
            let (header, bytes) = section.serialize();
            headers.push(header);
            section_data.extend(bytes);
        }

        // Add symbol and relocation table headers last
        let (symbol_header, symbol_data) = symbol_table.serialize_as_section();
        let (relocation_header, relocation_data) = relocation_table.serialize();
        headers.push(symbol_header);
        headers.push(relocation_header);
        section_data.extend(symbol_data);
        section_data.extend(relocation_data);

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
            if data.len() < offset + 16 {
                // Minimum section header size
                return Err(SerializationError::DataTooShort);
            }
            let (size, section_header) = SectionHeader::deserialize(&data[offset..])?;
            headers.push(section_header);
            offset += size;
        }

        // Last two sections must be symbol table and relocation table
        let section_count = headers.len();
        if section_count < 2 {
            return Err(SerializationError::InvalidData);
        }
        if !matches!(headers[section_count - 2], SectionHeader::SymbolTable(_)) {
            return Err(SerializationError::InvalidData);
        }
        if !matches!(
            headers[section_count - 1],
            SectionHeader::RelocationTable(_)
        ) {
            return Err(SerializationError::InvalidData);
        }

        // Ensure no other symbol/relocation table sections exist
        if headers[..section_count - 2].iter().any(|h| {
            matches!(
                h,
                SectionHeader::SymbolTable(_) | SectionHeader::RelocationTable(_)
            )
        }) {
            return Err(SerializationError::InvalidData);
        }

        // Calculate offsets to symbol and relocation tables
        let mut section_data_offset = offset;
        for header in &headers[..section_count - 2] {
            section_data_offset += header.section_size() as usize;
        }

        // Load symbol and relocation tables first
        let symbol_offset = section_data_offset;
        let (_, symbol_table) = SymbolTable::deserialize_section(
            match &headers[section_count - 2] {
                SectionHeader::SymbolTable(h) => h,
                _ => unreachable!(),
            },
            &data[symbol_offset..],
        )?;

        let relocation_offset = symbol_offset + headers[section_count - 2].section_size() as usize;
        let (_, relocation_table) = RelocationTable::deserialize(
            match &headers[section_count - 1] {
                SectionHeader::RelocationTable(h) => h,
                _ => unreachable!(),
            },
            &data[relocation_offset..],
        )?;

        // Process regular sections
        let mut sections = Vec::new();
        let mut current_offset = offset;

        for (idx, section_header) in headers[..section_count - 2].iter().enumerate() {
            match section_header {
                SectionHeader::Text(_) => {
                    let symbols = symbol_table.get_symbols(idx as u32);
                    let relocations = relocation_table.get_relocations(idx as u32);
                    let (size, section) = Section::deserialize(
                        section_header,
                        &data[current_offset..],
                        symbols,
                        relocations,
                    )?;
                    sections.push(section);
                    current_offset += size;
                }
                _ => return Err(SerializationError::InvalidData),
            }
        }

        Ok((
            relocation_offset + headers[section_count - 1].section_size() as usize,
            ObjectFile {
                architecture: header.architecture,
                sections,
            },
        ))
    }
}

impl ObjectFile {
    pub fn new(architecture: Architecture) -> Self {
        ObjectFile {
            architecture,
            sections: Vec::new(),
        }
    }

    pub fn with_sections(architecture: Architecture, sections: Vec<Section>) -> Self {
        ObjectFile {
            architecture,
            sections,
        }
    }

    pub fn add_section(&mut self, section: Section) {
        self.sections.push(section);
    }

    pub fn sections(self) -> Vec<Section> {
        self.sections
    }

    pub fn architecture(&self) -> Architecture {
        self.architecture
    }

    pub fn merge(&mut self, other: ObjectFile) {
        if self.architecture != other.architecture {
            panic!("Cannot merge object files with different architectures");
        }
        self.sections.extend(other.sections);
    }
}
