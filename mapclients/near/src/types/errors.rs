// use anomaly::{BoxError, Context};
// use thiserror::Error;

/// The main error type verification methods will return.
/// See [`Kind`] for the different kind of errors.
// pub type Error = anomaly::Error<Kind>;

pub struct Error;

/// All error kinds related to the light client.
#[derive(Clone, Debug)]
pub enum Kind {
    // #[error("invalid data length while converting slice to fixed-size array type ({current} != {expected}")]
    InvalidDataLength { current: usize, expected: usize },

    // #[error("rlp decode error")]
    RlpDecodeError,

    // #[error("invalid validator set diff: {msg}")]
    InvalidValidatorSetDiff { msg: &'static str },

    // #[error("attempted to insert invalid data to chain")]
    InvalidChainInsertion,

    // #[error("aggregated seal does not aggregate enough seals, num_seals: {current}, minimum quorum size: {expected}")]
    MissingSeals { current: usize, expected: usize },

    // #[error("BLS verify error")]
    BlsVerifyError,

    // #[error("BLS invalid signature")]
    BlsInvalidSignature,

    // #[error("BLS invalid public key")]
    BlsInvalidPublicKey,

    // #[error("header verification failed: {msg}")]
    HeaderVerificationError { msg: &'static str },

    // #[error("unkown error occurred")]
    Unknown,
}