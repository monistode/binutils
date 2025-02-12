use crate::serializable::*;

#[derive(Debug, Clone)]
pub struct ExecutableHeader {
    pub(crate) architecture: Architecture,
    pub(crate) segment_count: u64,
}

impl Serializable for ExecutableHeader {
    fn serialize(&self) -> Vec<u8> {
        let mut data = Vec::new();
        data.push(self.architecture as u8);
        data.extend(self.segment_count.to_le_bytes());
        data
    }

    fn deserialize(data: &[u8]) -> Result<(usize, Self), SerializationError> {
        if data.len() < 9 {
            return Err(SerializationError::DataTooShort);
        }

        let architecture = Architecture::try_from(data[0])?;
        let segment_count = u64::from_le_bytes([
            data[1], data[2], data[3], data[4], data[5], data[6], data[7], data[8],
        ]);

        Ok((
            9,
            ExecutableHeader {
                architecture,
                segment_count,
            },
        ))
    }
}

impl ExecutableHeader {
    pub fn new(architecture: Architecture, segment_count: u64) -> Self {
        ExecutableHeader {
            architecture,
            segment_count,
        }
    }
}
