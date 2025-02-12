use crate::Serializable;

#[derive(Debug, Clone, Copy)]
pub struct SegmentFlags {
    pub executable: bool,
    pub writable: bool,
    pub readable: bool,
    pub special: bool,
}

impl Serializable for SegmentFlags {
    fn serialize(&self) -> Vec<u8> {
        let mut byte = 0u8;
        if self.executable {
            byte |= 0b00000001;
        }
        if self.writable {
            byte |= 0b00000010;
        }
        if self.readable {
            byte |= 0b00000100;
        }
        if self.special {
            byte |= 0b00001000;
        }
        vec![byte]
    }
    fn deserialize(data: &[u8]) -> Result<(usize, Self), crate::SerializationError> {
        let byte = data.get(0).ok_or(crate::SerializationError::DataTooShort)?;
        Ok((
            1,
            SegmentFlags {
                executable: byte & 0b00000001 != 0,
                writable: byte & 0b00000010 != 0,
                readable: byte & 0b00000100 != 0,
                special: byte & 0b00001000 != 0,
            },
        ))
    }
}
