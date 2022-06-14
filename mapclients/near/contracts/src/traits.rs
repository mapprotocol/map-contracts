use crate::types::errors::Kind;

// "Deafult" trait is implemented for a few selected fixed-array types. Taken we can't implement
// the trait outside of a crate, we created a new one that mimics the stdlib.
pub trait DefaultFrom {
    fn default() -> Self;
}

pub trait FromBytes {
    fn from_bytes(data: &[u8]) -> Result<&Self, Kind>;
}

pub trait ToRlp {
    fn to_rlp(&self) -> Vec<u8>;
}

pub trait FromRlp {
    fn from_rlp(bytes: &[u8]) -> Result<Self, Kind>
    where
        Self: std::marker::Sized;
}