use crate::object_file::serializable::{Architecture, Serializable};

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

    pub fn deserialize(header: &super::text::SectionHeader, data: &[u8], architecture: Architecture) -> (usize, Self) {
        match header {
            super::text::SectionHeader::Text { .. } => {
                let (size, section) = super::text::TextSection::deserialize(header, data, architecture);
                (size, Section::Text(section))
            }
        }
    }
}
