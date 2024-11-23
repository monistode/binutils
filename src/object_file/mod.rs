mod serializable;
mod header;
pub mod sections;

pub use serializable::{Architecture, Serializable};
pub use header::ObjectHeader;
pub use sections::*;

#[derive(Debug, Clone)]
pub struct ObjectFile {
    header: ObjectHeader,
    sections: Vec<Section>,
}

impl Serializable for ObjectFile {
    fn serialize(&self) -> Vec<u8> {
        let mut data = Vec::new();
        
        // Serialize header
        let header = ObjectHeader {
            architecture: self.header.architecture,
            section_count: self.sections.len() as u64,
        };
        data.extend(header.serialize());

        // Create and serialize section headers and data
        let mut section_data = Vec::new();
        let mut headers = Vec::new();

        for section in &self.sections {
            let (header, bytes) = section.serialize();
            headers.push(header);
            section_data.extend(bytes);
        }

        // Serialize section headers
        for header in headers {
            data.extend(header.serialize());
        }

        // Add section data
        data.extend(section_data);
        data
    }

    fn deserialize(data: &[u8]) -> (usize, Self) {
        let (header_size, header) = ObjectHeader::deserialize(data);
        let mut offset = header_size;
        
        // Read section headers
        let mut headers = Vec::new();
        for _ in 0..header.section_count {
            let (size, section_header) = SectionHeader::deserialize(&data[offset..]);
            headers.push(section_header);
            offset += size;
        }

        // Read sections
        let mut sections = Vec::new();
        let mut section_data_offset = offset;
        
        for section_header in &headers {
            let (size, section) = Section::deserialize(
                section_header,
                &data[section_data_offset..],
                header.architecture,
            );
            sections.push(section);
            section_data_offset += size;
        }

        (section_data_offset, ObjectFile { header, sections })
    }
}
