use nalgebra as na;
use na::{
    Vector2,
    Vector3,
    Matrix4,
};

use crate::{
    State,
    errors::Error as AAError,
};


use std::{
    mem::size_of,
    collections::HashMap,
};

#[allow(dead_code)]
pub const VERTEX_ARR:[Vertex; 8] = [
    Vertex{position:Vector3::new(-0.5f32, -0.5f32, 0.0f32), color:Vector3::new(1f32, 0f32, 0f32), texcoord:Vector2::new(0.0f32, 0.0f32)},
    Vertex{position:Vector3::new(0.5f32, -0.5f32, 0.0f32), color:Vector3::new(0f32, 1f32, 0f32), texcoord:Vector2::new(1.0f32, 0.0f32)},
    Vertex{position:Vector3::new(0.5f32, 0.5f32, 0.0f32), color:Vector3::new(0f32, 0f32, 1f32), texcoord:Vector2::new(1.0f32, 1.0f32)},
    Vertex{position:Vector3::new(-0.5f32, 0.5f32, 0.0f32), color:Vector3::new(1f32, 1f32, 1f32), texcoord:Vector2::new(0.0f32, 1.0f32)},
    
    Vertex{position:Vector3::new(-0.5f32, -0.5f32, -0.5f32), color:Vector3::new(1f32, 0f32, 0f32), texcoord:Vector2::new(0.0f32, 0.0f32)},
    Vertex{position:Vector3::new(0.5f32, -0.5f32, -0.5f32), color:Vector3::new(0f32, 1f32, 0f32), texcoord:Vector2::new(1.0f32, 0.0f32)},
    Vertex{position:Vector3::new(0.5f32, 0.5f32, -0.5f32), color:Vector3::new(0f32, 0f32, 1f32), texcoord:Vector2::new(1.0f32, 1.0f32)},
    Vertex{position:Vector3::new(-0.5f32, 0.5f32, -0.5f32), color:Vector3::new(1f32, 1f32, 1f32), texcoord:Vector2::new(0.0f32, 1.0f32)},
];

#[allow(dead_code)]
pub const VERTEX_INDEX:[u32; 12] = [
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

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
struct DedupHelper {
    pub position: Vector3<i32>,
    pub color: Vector3<i32>,
    pub texcoord: Vector2<i32>,
}

#[repr(C)]
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct Vertex {
    pub position: Vector3<f32>,
    pub color: Vector3<f32>,
    pub texcoord: Vector2<f32>,
}

#[repr(C)]
#[derive(Debug, Default, Clone)]
pub struct Model {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
    pub image: image::RgbaImage,
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


#[allow(dead_code)]
impl Vertex {
    pub const fn size_usize() -> usize {
        size_of::<Self>()
    }
    pub const fn size_u64() -> u64 {
        size_of::<Self>() as u64
    }
}

impl Model {
    pub fn load(state:&State, obj_file:&'static str, texture_file:&'static str) -> Result<Self, AAError> {
        use image::io::Reader as ImageReader;
        
        if state.v_exp() {
            println!("loading model {} with texture {}", obj_file, texture_file);
        }
        
        let (mut model_vec, _materials) = tobj::load_obj(
            obj_file,
            &tobj::GPU_LOAD_OPTIONS
        )?;
        
        if model_vec.len() != 1 {
            return Err(AAError::ComplexObj);
        }
        
        let model = model_vec.pop().unwrap();
        let mesh = model.mesh;
        
        
        let mut deduping_vertex_helper:HashMap<DedupHelper, u32> = HashMap::new();
        let mut index_vec:Vec<u32> = Vec::new();
        let mut vertex_vec:Vec<Vertex> = Vec::new();
        
        for current_u32 in mesh.indices.iter() {
            let next_len = u32::try_from(vertex_vec.len()).unwrap();
            
            let mut vertex = Vertex::default();
            let current = usize::try_from(*current_u32).unwrap();
            
            vertex.position = Self::vector3_from_index(&mesh.positions, current);
            vertex.texcoord = Self::vector2_from_index(&mesh.texcoords, current);
            vertex.color = Vector3::new(1f32, 1f32, 1f32);
            
            let dedup = DedupHelper::from(&vertex);
            match deduping_vertex_helper.get(&dedup) {
                Some(vertex_index) => {
                    index_vec.push(*vertex_index);
                }
                None => {
                    deduping_vertex_helper.insert(dedup, next_len);
                    index_vec.push(next_len);
                    vertex_vec.push(vertex);
                }
            }
            
        }
        
        
        /*
        for (index, (guide, real)) in guide.into_iter().zip(index_vec.clone().into_iter().map(|number|{vertex_vec[usize::try_from(number).unwrap()]})).enumerate() {
            if guide != real {
                println!("=================================== {}", index);
            }
        }
        */
        
        let image_holder = ImageReader::open(texture_file).unwrap().decode().map_err(|_| AAError::DecodeError).unwrap().into_rgba8();
        
        if state.v_exp() {
            println!("amount of indices: {}", index_vec.len());
            println!("amount of vertices:  {}", vertex_vec.len());
        }
        
        
        Ok(Self{
            vertices: vertex_vec,
            indices: index_vec,
            image: image_holder
        })
    }
    
    #[inline(always)]
    fn vector3_from_index(vec:&Vec<f32>, index:usize) -> Vector3<f32> {
        Vector3::new(
            vec[index*3+0],
            vec[index*3+1],
            vec[index*3+2],
        )
    }
    
    #[inline(always)]
    fn vector2_from_index(vec:&Vec<f32>, index:usize) -> Vector2<f32> {
        
        Vector2::new(
            vec[index*2+0],
            1f32 - vec[index*2+1],
        )
    }
    
    
}

impl From<&Vertex> for DedupHelper {
    fn from(base:&Vertex) -> Self {
        let holder = base.position*10_000_000f32;
        let position = Vector3::new(
            holder[0] as i32,
            holder[1] as i32,
            holder[2] as i32,
        );
        let holder = base.color*10_000_000f32;
        let color = Vector3::new(
            holder[0] as i32,
            holder[1] as i32,
            holder[2] as i32,
        );
        let holder = base.texcoord*10_000_000f32;
        let texcoord = Vector2::new(
            holder[0] as i32,
            holder[1] as i32,
        );
        Self{
            texcoord:texcoord,
            color:color,
            position:position,
        }
    }
}
