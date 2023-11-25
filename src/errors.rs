use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("No suitable GPU")]
    NoGPU,
    #[error("Missing Extensions")]
    MissingExtensions,
    #[error("Missing Layers: {0:?}")]
    MissingLayers(Vec<&'static str>),
}

/*
#[derive(Error, Debug)]
pub enum Error {
    #[error("data store disconnected")]
    Disconnect(#[from] io::Error),
    #[error("the data for key `{0}` is not available")]
    Redaction(String),
    #[error("invalid header (expected {expected:?}, found {found:?})")]
    InvalidHeader {
        expected: String,
        found: String,
    },
    #[error("unknown data store error")]
    Unknown,
}

*/
