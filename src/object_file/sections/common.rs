use super::header::{SectionHeader, TextSectionHeader};
use super::text::TextSection;
use crate::address::AddressIndexable;
use crate::executable::segments::{Segment, TextSegment};
use crate::object_file::placed::{LinkerError, Placement};
use crate::object_file::relocations::Relocation;
use crate::serializable::{Architecture, SerializationError};
use crate::symbols::Symbol;

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
        relocations: Vec<Relocation>,
    ) -> Result<(usize, Self), SerializationError> {
        match header {
            SectionHeader::Text(header) => {
                let (size, section) =
                    TextSection::deserialize(header, data, architecture, symbols, relocations)?;
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

    pub fn to_segment(&self, placement: &Placement, offset: usize) -> Result<Segment, LinkerError> {
        let text_byte_width = match placement.architecture() {
            Architecture::Stack => 6,
        };
        match self {
            Section::Text(text) => {
                let mut data = text.data.clone();
                for relocation in text.relocations.iter() {
                    let symbol = placement.find_symbol(relocation.symbol.as_str());
                    let symbol = match symbol {
                        None => return Err(LinkerError::SymbolNotFound(relocation.symbol.clone())),
                        Some(symbol) => symbol,
                    };
                    let offset = if relocation.relative {
                        symbol - relocation.address
                    } else {
                        symbol.0 as i64
                    } / text_byte_width;
                    // Check bounds - +-2^16
                    if offset > 2_i64.pow(16) as i64 || offset < -(2_i64.pow(16) as i64) {
                        return Err(LinkerError::RelocationOutOfRange(relocation.symbol.clone()));
                    }
                    data.write(
                        relocation.address,
                        data.index(relocation.address).wrapping_add(offset as u16),
                    );
                }
                Ok(Segment::Text(TextSegment::new(
                    offset,
                    data,
                    text.symbols.clone(),
                )))
            }
        }
    }
}
