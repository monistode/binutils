use crate::object_file::serializable::{Architecture, Serializable, SerializationError};
use crate::object_file::symbols::Symbol;

#[derive(Debug, Clone)]
pub enum Section {
    Text(super::text::TextSection),
}

impl Section {
    pub fn serialize(&self) -> (super::text::SectionHeader, Vec<u8>) {
        match self {
            Section::Text(text) => {
                let bytes = text.serialize();
                let section_header = super::text::SectionHeader::Text(super::text::TextSectionHeader {
                    bit_length: text.data.len() as u64,
                });
                (section_header, bytes)
            }
        }
    }

    pub fn deserialize(
        header: &super::text::SectionHeader,
        data: &[u8],
        architecture: Architecture
    ) -> Result<(usize, Self), SerializationError> {
        match header {
            super::text::SectionHeader::Text(header) => {
                let (size, section) = super::text::TextSection::deserialize(header, data, architecture)?;
                Ok((size, Section::Text(section)))
            }
            super::text::SectionHeader::SymbolTable { .. } => {
                Err(SerializationError::InvalidSectionType(0))
            }
        }
    }

    pub fn symbols(&self) -> Vec<Symbol> {
        match self {
            Section::Text(text) => text.symbols.clone(),
        }
    }
}
