pub use header::ExecutableHeader;
pub use segments::{Segment, SegmentHeader};

use crate::{Architecture, Serializable, SerializationError, SymbolTable};

pub mod header;
pub mod segments;

#[derive(Debug, Clone)]
pub struct Executable {
    header: ExecutableHeader,
    segments: Vec<Segment>,
}

impl Serializable for Executable {
    fn serialize(&self) -> Vec<u8> {
        let mut data = Vec::new();

        // Optionally create symbol table from segment data - using the same section based table
        // because why not
        let mut symbol_table = SymbolTable::new();

        for (segment_id, segment) in self.segments.iter().enumerate() {
            for symbol in segment.symbols() {
                symbol_table.add_symbol(segment_id as u32, symbol);
            }
        }

        // Serialize header with all segments (including symbol table)
        let header = ExecutableHeader {
            architecture: self.header.architecture,
            segment_count: self.segments.len() as u64 + 1, // +1 for symbol table
            entry_point: self.header.entry_point,
        };
        data.extend(header.serialize());

        // Create and serialize all segment headers and data
        let mut segment_data = Vec::new();
        let mut headers = Vec::new();

        // Add regular segment headers first
        for segment in &self.segments {
            let (header, bytes) = segment.serialize();
            headers.push(header);
            segment_data.extend(bytes);
        }

        // Add symbol table headers last
        let (symbol_header, symbol_data) = symbol_table.serialize_as_segment();
        headers.push(symbol_header);
        segment_data.extend(symbol_data);

        // Add all headers followed by all segment data
        for header in headers {
            data.extend(header.serialize());
        }
        data.extend(segment_data);

        data
    }

    fn deserialize(data: &[u8]) -> Result<(usize, Self), SerializationError> {
        if data.len() < 9 {
            return Err(SerializationError::DataTooShort);
        }

        // Parse header
        let (header_size, header) = ExecutableHeader::deserialize(data)?;
        let mut offset = header_size;

        // Read all segment headers
        let mut headers = Vec::new();
        for _ in 0..header.segment_count {
            if data.len() < offset + 16 {
                // Minimum segment header size
                return Err(SerializationError::DataTooShort);
            }
            let (size, segment_header) = SegmentHeader::deserialize(&data[offset..])?;
            headers.push(segment_header);
            offset += size;
        }

        // Last segment must be symbol table - TODO optional
        let segment_count = headers.len();
        if segment_count < 1 {
            return Err(SerializationError::InvalidData);
        }
        if !headers[segment_count - 1].flags.special
            || !headers[segment_count - 1].address_space_start == 0
        {
            return Err(SerializationError::InvalidData);
        }

        // Ensure no other symbol table segments exist
        if headers[..segment_count - 1]
            .iter()
            .any(|h| h.flags.special && h.address_space_start == 0)
        {
            return Err(SerializationError::InvalidData);
        }

        // Calculate offsets to symbol and relocation tables
        let mut segment_data_offset = offset;
        for header in &headers[..segment_count - 1] {
            segment_data_offset += header.segment_size();
        }

        // Load symbol table first
        let symbol_offset = segment_data_offset;
        let (_, symbol_table) =
            SymbolTable::deserialize_segment(&headers[segment_count - 1], &data[symbol_offset..])?;

        // Process regular segments
        let mut segments = Vec::new();
        let mut current_offset = offset;

        for (idx, segment_header) in headers[..segment_count - 1].iter().enumerate() {
            let symbols = symbol_table.get_symbols(idx as u32);
            let (size, segment) =
                Segment::deserialize(segment_header, &data[current_offset..], symbols)?;
            segments.push(segment);
            current_offset += size;
        }

        Ok((
            symbol_offset + headers[segment_count - 1].segment_size(), // TODO sure?
            Executable { header, segments },
        ))
    }
}

impl Executable {
    pub fn new(architecture: Architecture, segments: Vec<Segment>) -> Self {
        Executable {
            header: ExecutableHeader::new(architecture, 0),
            segments,
        }
    }

    pub fn segments(&self) -> &[Segment] {
        &self.segments
    }

    pub fn segments_mut(&mut self) -> &mut Vec<Segment> {
        &mut self.segments
    }

    pub fn architecture(&self) -> Architecture {
        self.header.architecture
    }

    pub fn entry_point(&self) -> u64 {
        self.header.entry_point
    }
}
