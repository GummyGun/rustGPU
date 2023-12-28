use ash::vk;
use memoffset::offset_of;

use crate::{
    graphics::{
        Vertex,
    },
};

use std::mem::size_of;


impl Vertex {
    pub const fn binding_description() -> &'static[vk::VertexInputBindingDescription] {
        if size_of::<Vertex>() > u32::MAX as usize {
            panic!("Vertex size is too big");
        }
        const HOLDER:[vk::VertexInputBindingDescription; 1] = [
            vk::VertexInputBindingDescription{
                binding: 0,
                stride: size_of::<Vertex>() as u32,
                input_rate: vk::VertexInputRate::VERTEX,
            },
        ];
        &HOLDER
    }
    
    pub const fn attribute_description() -> &'static[vk::VertexInputAttributeDescription] {
        if offset_of!(Vertex, position) > u32::MAX as usize {
            panic!("Vertex size is too big");
        }
        if offset_of!(Vertex, color) > u32::MAX as usize {
            panic!("Vertex size is too big");
        }
        
        const HOLDER:[vk::VertexInputAttributeDescription; 3] = [
            vk::VertexInputAttributeDescription{
                binding: 0,
                location: 0,
                format: vk::Format::R32G32B32_SFLOAT,
                offset: offset_of!(Vertex, position) as u32
            },
            vk::VertexInputAttributeDescription{
                binding: 0,
                location: 1,
                format: vk::Format::R32G32B32_SFLOAT,
                offset: offset_of!(Vertex, color) as u32
            },
            vk::VertexInputAttributeDescription{
                binding: 0,
                location: 2,
                format: vk::Format::R32G32_SFLOAT,
                offset: offset_of!(Vertex, texcoord) as u32
            }
            
        ];
        &HOLDER
    }
    
}

