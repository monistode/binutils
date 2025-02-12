use crate::serializable::*;

#[derive(Debug, Clone)]
pub struct ExecutableHeader {
    pub(crate) architecture: Architecture,
    pub(crate) segment_count: u64,
    pub(crate) entry_point: u64,
}

impl Serializable for ExecutableHeader {
    fn serialize(&self) -> Vec<u8> {
        let mut data = Vec::new();
        data.push(self.architecture as u8);
        data.extend(self.segment_count.to_le_bytes());
        data.extend(self.entry_point.to_le_bytes());
        data
    }

    fn deserialize(data: &[u8]) -> Result<(usize, Self), SerializationError> {
        if data.len() < 17 {
            return Err(SerializationError::DataTooShort);
        }

        let architecture = Architecture::try_from(data[0])?;
        let segment_count = u64::from_le_bytes([
            data[1], data[2], data[3], data[4], data[5], data[6], data[7], data[8],
        ]);
        let entry_point = u64::from_le_bytes([
            data[9], data[10], data[11], data[12], data[13], data[14], data[15], data[16],
        ]);

        Ok((
            17,
            ExecutableHeader {
                architecture,
                segment_count,
                entry_point,
            },
        ))
    }
}

impl ExecutableHeader {
    pub fn new(architecture: Architecture, segment_count: u64) -> Self {
        ExecutableHeader {
            architecture,
            segment_count,
            entry_point: 0, // TODO search for start symbol
        }
    }
}
