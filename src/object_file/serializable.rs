pub trait Serializable {
    fn serialize(&self) -> Vec<u8>;
    fn deserialize(data: &[u8]) -> (usize, Self);
}

#[derive(Debug, Clone, Copy)]
pub enum Architecture {
    Stack = 0,
}