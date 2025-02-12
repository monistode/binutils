use crate::{Serializable, SerializationError};

use super::flags::SegmentFlags;

#[derive(Debug, Clone)]
pub struct SegmentHeader {
    pub address_space_start: u64, // These are the addresses - in bytes
    pub address_space_size: u64,
    pub disk_bit_count: usize,
    pub flags: SegmentFlags,
}

impl Serializable for SegmentHeader {
    fn serialize(&self) -> Vec<u8> {
        let mut data = Vec::new();
        data.extend(self.address_space_start.to_le_bytes());
        data.extend(self.address_space_size.to_le_bytes());
        data.extend(self.disk_bit_count.to_le_bytes());
        data.extend(self.flags.serialize());
        data
    }

    fn deserialize(data: &[u8]) -> Result<(usize, Self), SerializationError> {
        if data.len() < 24 {
            return Err(SerializationError::DataTooShort);
        }
        let address_space_start = u64::from_le_bytes([
            data[0], data[1], data[2], data[3], data[4], data[5], data[6], data[7],
        ]);
        let address_space_size = u64::from_le_bytes([
            data[8], data[9], data[10], data[11], data[12], data[13], data[14], data[15],
        ]);
        let disk_bit_count = u64::from_le_bytes([
            data[16], data[17], data[18], data[19], data[20], data[21], data[22], data[23],
        ]) as usize;
        let (flags_size, flags) = SegmentFlags::deserialize(&data[24..])?;
        return Ok((
            24 + flags_size,
            SegmentHeader {
                address_space_start,
                address_space_size,
                disk_bit_count,
                flags,
            },
        ));
    }
}

impl SegmentHeader {
    pub fn segment_size(&self) -> usize {
        (self.disk_bit_count + 7) / 8
    }
}
