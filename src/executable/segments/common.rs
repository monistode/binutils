use super::header::{SegmentHeader, TextSegmentHeader};
use super::text::TextSegment;
use crate::{Architecture, SerializationError, Symbol};

#[derive(Debug, Clone)]
pub enum Segment {
    Text(TextSegment),
}

impl Segment {
    pub fn serialize(&self) -> (SegmentHeader, Vec<u8>) {
        match self {
            Segment::Text(text) => {
                let bytes = text.serialize();
                let section_header = SegmentHeader::Text(TextSegmentHeader {
                    location: text.offset,
                    bit_length: text.data.len() as u64,
                });
                (section_header, bytes)
            }
        }
    }

    pub fn deserialize(
        header: &SegmentHeader,
        data: &[u8],
        architecture: Architecture,
        symbols: Vec<Symbol>,
    ) -> Result<(usize, Self), SerializationError> {
        match header {
            SegmentHeader::Text(header) => {
                let (size, section) =
                    TextSegment::deserialize(header, data, architecture, symbols)?;
                Ok((size, Segment::Text(section)))
            }
            _ => Err(SerializationError::InvalidSectionType(0)),
        }
    }

    pub fn symbols(&self) -> Vec<Symbol> {
        match self {
            Segment::Text(text) => text.symbols.clone(),
        }
    }
}
