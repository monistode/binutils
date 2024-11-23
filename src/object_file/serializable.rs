#[derive(Debug)]
pub enum SerializationError {
    InvalidArchitecture(u8),
    InvalidSectionType(u8),
    InvalidSymbolTableHeader,
    InvalidData,
    DataTooShort,
}

pub trait Serializable: Sized {
    fn serialize(&self) -> Vec<u8>;
    fn deserialize(data: &[u8]) -> Result<(usize, Self), SerializationError>;
}

#[derive(Debug, Clone, Copy)]
pub enum Architecture {
    Stack = 0,
}

impl TryFrom<u8> for Architecture {
    type Error = SerializationError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Architecture::Stack),
            v => Err(SerializationError::InvalidArchitecture(v)),
        }
    }
}