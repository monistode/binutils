pub mod address;
pub mod definition;
pub mod executable;
pub mod object_file;
pub mod serializable;
pub mod symbols;

pub use address::Address;
pub use definition::{Definition, RawDefinition};
pub use executable::Executable;
pub use object_file::ObjectFile;
pub use serializable::{Architecture, Serializable, SerializationError};
pub use symbols::{Symbol, SymbolTable};

use object_file::placed::{LinkerError, PlacedSection, Placement};

impl TryFrom<ObjectFile> for Executable {
    type Error = LinkerError;

    fn try_from(object: ObjectFile) -> Result<Self, Self::Error> {
        let architecture = object.architecture();
        let mut placed = Placement::new(
            object
                .sections()
                .into_iter()
                .map(|section| PlacedSection::new(section))
                .collect(),
            architecture,
        );
        placed.place();
        return Ok(Executable::new(architecture, placed.as_segments()?));
    }
}
