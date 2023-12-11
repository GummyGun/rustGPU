use nalgebra as na;
use na::{
    Vector2,
    Vector3,
};


pub const VERTEX_ARR:[Vertex; 4] = [
    Vertex{position:Vector2::<f32>::new(-0.5f32, -0.5f32), color:Vector3::<f32>::new(1f32, 1f32, 1f32)},
    Vertex{position:Vector2::<f32>::new(0.5f32, -0.5f32), color:Vector3::<f32>::new(1f32, 0f32, 0f32)},
    Vertex{position:Vector2::<f32>::new(0.5f32, 0.5f32), color:Vector3::<f32>::new(0f32, 1f32, 0f32)},
    Vertex{position:Vector2::<f32>::new(-0.5f32, 0.5f32), color:Vector3::<f32>::new(0f32, 0f32, 1f32)},
];

pub const VERTEX_INDEX:[u16; 6] = [
    0, 1, 2, 2, 3, 0
];

#[repr(C)]
#[derive(Debug, Default, Clone, Copy)]
pub struct Vertex{
    pub position: Vector2<f32>,
    pub color: Vector3<f32>,
}
