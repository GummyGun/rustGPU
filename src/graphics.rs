use nalgebra as na;
use na::{
    Vector2,
    Vector3,
    Matrix4,
};


use std::mem::size_of;

pub const VERTEX_ARR:[Vertex; 8] = [
    Vertex{position:Vector3::new(-0.5f32, -0.5f32, 0.0f32), color:Vector3::new(1f32, 0f32, 0f32), coordenates:Vector2::new(0.0f32, 0.0f32)},
    Vertex{position:Vector3::new(0.5f32, -0.5f32, 0.0f32), color:Vector3::new(0f32, 1f32, 0f32), coordenates:Vector2::new(1.0f32, 0.0f32)},
    Vertex{position:Vector3::new(0.5f32, 0.5f32, 0.0f32), color:Vector3::new(0f32, 0f32, 1f32), coordenates:Vector2::new(1.0f32, 1.0f32)},
    Vertex{position:Vector3::new(-0.5f32, 0.5f32, 0.0f32), color:Vector3::new(1f32, 1f32, 1f32), coordenates:Vector2::new(0.0f32, 1.0f32)},
    
    Vertex{position:Vector3::new(-0.5f32, -0.5f32, -0.5f32), color:Vector3::new(1f32, 0f32, 0f32), coordenates:Vector2::new(0.0f32, 0.0f32)},
    Vertex{position:Vector3::new(0.5f32, -0.5f32, -0.5f32), color:Vector3::new(0f32, 1f32, 0f32), coordenates:Vector2::new(1.0f32, 0.0f32)},
    Vertex{position:Vector3::new(0.5f32, 0.5f32, -0.5f32), color:Vector3::new(0f32, 0f32, 1f32), coordenates:Vector2::new(1.0f32, 1.0f32)},
    Vertex{position:Vector3::new(-0.5f32, 0.5f32, -0.5f32), color:Vector3::new(1f32, 1f32, 1f32), coordenates:Vector2::new(0.0f32, 1.0f32)},
    
];

pub const VERTEX_INDEX:[u16; 12] = [
    //1, 0, 2, 3, 2, 0, //3, 1, 4
    0, 1, 2, 2, 3, 0, 
    4, 5, 6, 6, 7, 4, 
];

#[repr(C)]
#[derive(Default, Debug, Clone, Copy)]
pub struct UniformBufferObject {
    pub model: Matrix4<f32>,
    pub view: Matrix4<f32>,
    pub proj: Matrix4<f32>,
}

#[allow(dead_code)]
impl UniformBufferObject {
    pub const fn size_usize() -> usize {
        size_of::<Self>()
    }
    pub const fn size_u64() -> u64 {
        size_of::<Self>() as u64
    }
}


#[repr(C)]
#[derive(Debug, Default, Clone, Copy)]
pub struct Vertex{
    pub position: Vector3<f32>,
    pub color: Vector3<f32>,
    pub coordenates: Vector2<f32>,
}
#[allow(dead_code)]
impl Vertex {
    pub const fn size_usize() -> usize {
        size_of::<Self>()
    }
    pub const fn size_u64() -> u64 {
        size_of::<Self>() as u64
    }
}
