use crate::object_file::serializable::{Architecture, Serializable, SerializationError};
use crate::object_file::symbols::Symbol;
use crate::object_file::relocations::Relocation;
use super::header::{SectionHeader, TextSectionHeader};
use super::text::TextSection;

#[derive(Debug, Clone)]
pub enum Section {
    Text(TextSection),
}

impl Section {
    pub fn serialize(&self) -> (SectionHeader, Vec<u8>) {
        match self {
            Section::Text(text) => {
                let bytes = text.serialize();
                let section_header = SectionHeader::Text(TextSectionHeader {
                    bit_length: text.data.len() as u64,
                });
                (section_header, bytes)
            }
        }
    }

    pub fn deserialize(
        header: &SectionHeader,
        data: &[u8],
        architecture: Architecture,
        symbols: Vec<Symbol>,
        relocations: Vec<Relocation>
    ) -> Result<(usize, Self), SerializationError> {
        match header {
            SectionHeader::Text(header) => {
                let (size, section) = TextSection::deserialize(
                    header, 
                    data, 
                    architecture,
                    symbols,
                    relocations
                )?;
                Ok((size, Section::Text(section)))
            }
            _ => Err(SerializationError::InvalidSectionType(0)),
        }
    }

    pub fn symbols(&self) -> Vec<Symbol> {
        match self {
            Section::Text(text) => text.symbols.clone(),
        }
    }

    pub fn relocations(&self) -> Vec<Relocation> {
        match self {
            Section::Text(text) => text.relocations.clone(),
        }
    }
}
