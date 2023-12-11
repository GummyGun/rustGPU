use ash::{
    vk,
};

/*
use super::{
    VInit,
};
*/


use crate::{
    graphics::Vertex,
};

//use nalgebra as na;

use memoffset::offset_of;

use std::{
    mem::size_of,
};

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
        
        const HOLDER:[vk::VertexInputAttributeDescription; 2] = [
            vk::VertexInputAttributeDescription{
                binding: 0,
                location: 0,
                format: vk::Format::R32G32_SFLOAT,
                offset: offset_of!(Vertex, position) as u32
            },
            vk::VertexInputAttributeDescription{
                binding: 0,
                location: 1,
                format: vk::Format::R32G32B32_SFLOAT,
                offset: offset_of!(Vertex, color) as u32
            }
        ];
        &HOLDER
    }
}

/*
impl VInit {
    
    pub fn test(&self) {
        //let vector = Vector3::new(1, 2, 3);
        println!("{:?}", size_of::<na::Vector3<f32>>());
        println!("{:?}", size_of::<na::Vector2<f32>>());
        println!("{:?}", size_of::<Vertex>());
        println!("{:?}", size_of::<[Vertex; 3]>());
        println!("{:?}", Vertex::attribute_description());
    }
    
}
*/
