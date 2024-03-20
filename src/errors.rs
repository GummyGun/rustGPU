use thiserror::Error;
use std::collections::HashSet;
use std::io;
use ash::vk;
use sdl2::video;
use gpu_allocator as gpu;


#[derive(Error, Debug)]
pub enum Error {
    #[error("debug error")]
    TODOError,
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
    
    #[error("swapchain support is strange")]
    SwapchainImageSize,
    
    #[error("layout not set on pipeline creation")]
    LayoutNotSet,
    
    #[error("cant create mesh from empty arrays")]
    EmptyMesh,
    
    
    #[error("invalid load transform")]
    InvalidLoadTransform,
    #[error("lobj error")]
    LobjError(#[from] tobj::LoadError),
    #[error("VK error")]
    VkError(#[from] vk::Result),
    #[error("gpu_allocator error")]
    GPUAlocError(#[from] gpu::AllocationError),
    #[error("IO error")]
    IoError(#[from] io::Error),
    #[error("string error")]
    StringError(String),
    #[error("WindowBuild error")]
    SDL2Error(#[from] video::WindowBuildError),
    
    #[error("gltf error")]
    GLTFError(#[from] gltf::Error),
    
}

/*
use crate::errors::messages:: ;
*/
pub mod messages {
    pub const U32_TO_USIZE:&'static str = "conversion should be granted on regular architecture";
    pub const VK_CAST:&'static str = "vk castings that are granted not give problems";
    pub const SIMPLE_VK_FN:&'static str = "simple vk functions should not fail";
    pub const SIMPLE_SDL_FN:&'static str = "simple sdl2 functions should not fail";
    pub const BAD_DESTRUCTOR:&'static str = "destruct did not receive the right information";
    pub const GPU_ALLOCATION:&'static str = "gpu allocation should not fail";
    pub const GPU_FREE:&'static str = "gpu free should not fail";
    pub const STANDARD_CONV:&'static str = "conversion is granted by de standard";
    pub const GRANTED:&'static str = "things the programer knows but the compiled does not";
    pub const RESOURCE_FILE:&'static str = "the resource file should be found in a particular path";
    
    pub const VK_UNRECOVERABLE:&'static str = "things the programer knows but the compiled does not";
    
    pub const COMPILETIME_ASSERT:&'static str = "things the programer knows but the compiled does not";
    pub const MODEL_DENSITY:&'static str = "model vertex indices should fit in a u32";
    
    pub const ALREADY_DESTROYED:&'static str = "can't run methods on destroyed objects";
    pub const NON_DESTROYED:&'static str = "dropping non_destroyed object use destruct";
    
    pub const NON_EMPTY_WRAPPER:&'static str = "trying to fill a wrapper that is not empty";
    pub const REDUNDANT_DEREFED_DESTRUCTOR:&'static str = "the defered destructor was already built";
    pub const REDUNDANT_DESTRUCTOR:&'static str = "object was allready set to be defered destructed";
    pub const LEAKING_OBJECTS:&'static str = "destruction stack is being dropped before dispatching";
    
    pub const SU_COMMAND_FAIL:&'static str = "single use instant command should not fail";
    pub const CPU_ACCESIBLE:&'static str = "memory should be granted to be cpu accesible";
    
    pub const RESOURCE_REFERENCED:&'static str = "resource is still reference somewhere";
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
