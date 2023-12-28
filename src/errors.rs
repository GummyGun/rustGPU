use thiserror::Error;
use std::collections::HashSet;
use std::io;
use ash::vk;


#[derive(Error, Debug)]
pub enum Error {
    /*
    #[error("debug error")]
    TODOError,
    */
    #[error("No suitable GPU")]
    NoGPU,
    #[error("Missing Extensions: {0:?}")]
    MissingExtensions(HashSet<&'static str>),
    #[error("Missing Layers: {0:?}")]
    MissingLayers(HashSet<&'static str>),
    #[error("No suitable memory")]
    NoSuitableMemory,
    #[error("Decoding error")]
    DecodeError,
    #[error("Image transition")]
    ImageLayoutUnsuported,
    #[error("format not supported")]
    UnsuportedFormat,
    #[error("only simple obj are supported")]
    ComplexObj,
    #[error("only simple gltf are supported")]
    ComplexGltf,
    #[error("Lobj error")]
    LobjError(#[from] tobj::LoadError),
    #[error("VK error")]
    VkError(#[from] vk::Result),
    #[error("IO error")]
    IoError(#[from] io::Error),
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
