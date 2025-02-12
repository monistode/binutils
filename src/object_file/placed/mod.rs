use crate::{executable::segments::Segment, Address, Architecture};

use super::Section;

#[derive(Debug)]
pub enum LinkerError {
    SymbolNotFound(String),
    RelocationOutOfRange(String),
}

pub struct PlacedSection {
    section: Section,
    offset: usize, // in bytes
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum SectionType {
    TextSpace,
    DataSpace,
    Unified,
}

impl PlacedSection {
    pub fn new(section: Section) -> Self {
        PlacedSection { section, offset: 0 }
    }

    pub fn section(&self) -> &Section {
        &self.section
    }

    pub fn section_type(&self) -> SectionType {
        SectionType::Unified // TODO
    }

    pub fn offset(&self) -> usize {
        self.offset
    }

    pub fn size(&self, architecture: Architecture) -> usize {
        let text_byte_width = match architecture {
            Architecture::Stack => 6,
        };
        match &self.section {
            Section::Text(text) => (text.data.len() + text_byte_width - 1) / text_byte_width,
        }
    }

    pub fn find_symbol(&self, name: &str) -> Option<Address> {
        for symbol in self.section.symbols().iter() {
            if symbol.name == name {
                return Some(symbol.address + self.offset);
            }
        }
        return None;
    }

    pub fn to(&mut self, offset: usize) {
        self.offset = offset;
    }
}

pub struct Placement {
    sections: Vec<PlacedSection>,
    architecture: Architecture,
}

impl Placement {
    pub fn new(sections: Vec<PlacedSection>, architecture: Architecture) -> Self {
        Placement {
            sections,
            architecture,
        }
    }

    pub fn architecture(&self) -> Architecture {
        self.architecture
    }

    pub fn find_symbol(&self, name: &str) -> Option<Address> {
        for section in self.sections.iter() {
            if let Some(address) = section.find_symbol(name) {
                return Some(address);
            }
        }
        return None;
    }

    pub fn place(&mut self) {
        // We need to make sure no segments intersect
        for address_space in [
            SectionType::TextSpace,
            SectionType::DataSpace,
            SectionType::Unified,
        ]
        .iter()
        {
            let mut last_end = 0;
            for section in self.sections.iter_mut() {
                if section.section_type() != *address_space {
                    continue;
                }
                section.to(last_end);
                last_end = section.offset() + section.size(self.architecture);
            }
        }
    }

    pub fn as_segments(&self) -> Result<Vec<Segment>, LinkerError> {
        self.sections
            .iter()
            .map(|section| section.section().to_segment(self, section.offset()))
            .collect()
    }
}
