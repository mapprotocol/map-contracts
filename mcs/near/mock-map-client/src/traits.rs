
pub trait FromBytes {
    fn from_bytes(data: &[u8]) -> Result<&Self, ()>;
}