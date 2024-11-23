use bitvec::prelude::*;

pub trait Serializable {
    fn serialize(&self) -> Vec<u8>;
    fn deserialize(data: &[u8]) -> (usize, Self);
}

#[derive(Debug, Clone, Copy)]
pub enum Architecture {
    Stack = 0,
}

#[derive(Debug, Clone)]
pub struct ObjectHeader {
    architecture: Architecture,
}

impl Serializable for ObjectHeader {
    fn serialize(&self) -> Vec<u8> {
        let mut data = Vec::new();
        data.push(self.architecture as u8);
        data
    }

    fn deserialize(data: &[u8]) -> (usize, Self) {
        (
            1,
            ObjectHeader {
                architecture: match data[0] {
                    0 => Architecture::Stack,
                    _ => panic!("Invalid architecture"),
                },
            },
        )
    }
}

#[derive(Debug, Clone)]
pub struct TextSection {
    pub data: BitVec,
}

impl Serializable for TextSection {
    fn serialize(&self) -> Vec<u8> {
        let mut data = Vec::new();
        data.extend(self.data.len().to_le_bytes().iter());

        let words = self.data.as_raw_slice();
        for word in words {
            data.extend(word.to_le_bytes().iter());
        }
        data
    }

    fn deserialize(data: &[u8]) -> (usize, Self) {
        let size = u64::from_le_bytes([
            data[0], data[1], data[2], data[3], data[4], data[5], data[6], data[7],
        ]) as usize;
        let mut bits = BitVec::new();
        for i in 0..size {
            let bit = data[8 + i / 8] & (1 << (i % 8)) != 0;
            bits.push(bit);
        }
        (size + 8, TextSection { data: bits })
    }
}

#[derive(Debug, Clone)]
pub enum Section {
    Text(TextSection),
}

impl Serializable for Section {
    fn serialize(&self) -> Vec<u8> {
        match self {
            Section::Text(section) => {
                let mut data = Vec::new();
                data.push(0);
                data.extend(section.serialize());
                data
            }
        }
    }

    fn deserialize(data: &[u8]) -> (usize, Self) {
        match data[0] {
            0 => {
                let (size, section) = TextSection::deserialize(&data[1..]);
                (size + 1, Section::Text(section))
            }
            _ => panic!("Invalid section type"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ObjectFile {
    header: ObjectHeader,
    sections: Vec<Section>,
}

impl Serializable for ObjectFile {
    fn serialize(&self) -> Vec<u8> {
        let mut data = Vec::new();
        data.extend(self.header.serialize());
        for section in &self.sections {
            data.extend(section.serialize());
        }
        data
    }

    fn deserialize(data: &[u8]) -> (usize, Self) {
        let (header_size, header) = ObjectHeader::deserialize(data);
        let mut sections = Vec::new();
        let mut offset = header_size;
        while offset < data.len() {
            let (size, section) = Section::deserialize(&data[offset..]);
            sections.push(section);
            offset += size;
        }
        (offset, ObjectFile { header, sections })
    }
}
