use bitvec::vec::BitVec;

use super::flags::SegmentFlags;
use super::header::SegmentHeader;
use crate::{SerializationError, Symbol};

#[derive(Debug, Clone)]
pub struct Segment {
    pub address_space_start: u64, // These are the addresses - in bytes
    pub address_space_size: u64,
    pub disk_bit_count: usize,
    pub flags: SegmentFlags,
    pub data: BitVec,
    symbols: Vec<Symbol>,
}

impl Segment {
    pub fn new(
        address_space_start: u64,
        address_space_size: u64,
        disk_bit_count: usize,
        flags: SegmentFlags,
        data: BitVec,
        symbols: Vec<Symbol>,
    ) -> Self {
        Segment {
            address_space_start,
            address_space_size,
            disk_bit_count,
            flags,
            data,
            symbols,
        }
    }

    pub fn serialize(&self) -> (SegmentHeader, Vec<u8>) {
        let mut bytes = Vec::new();
        for i in 0..((self.data.len() + 7) / 8) {
            let mut byte = 0u8;
            for j in 0..8 {
                if i * 8 + j < self.data.len() && self.data[i * 8 + j] {
                    byte |= 1 << j;
                }
            }
            bytes.push(byte);
        }
        (
            SegmentHeader {
                address_space_start: self.address_space_start,
                address_space_size: self.address_space_size,
                disk_bit_count: self.disk_bit_count,
                flags: self.flags,
            },
            bytes,
        )
    }

    pub fn deserialize(
        header: &SegmentHeader,
        data: &[u8],
        symbols: Vec<Symbol>,
    ) -> Result<(usize, Self), SerializationError> {
        let required_bytes = (header.disk_bit_count + 7) / 8;
        if data.len() < required_bytes {
            return Err(SerializationError::DataTooShort);
        }

        let mut bits = BitVec::new();
        for i in 0..header.disk_bit_count {
            let bit = data[i / 8] & (1 << (i % 8)) != 0;
            bits.push(bit);
        }
        let bytes_read = (header.disk_bit_count + 7) / 8;
        Ok((
            bytes_read,
            Segment {
                address_space_start: header.address_space_start,
                address_space_size: header.address_space_size,
                disk_bit_count: header.disk_bit_count,
                flags: header.flags,
                data: bits,
                symbols,
            },
        ))
    }

    pub fn symbols(&self) -> Vec<Symbol> {
        self.symbols.clone()
    }
}
