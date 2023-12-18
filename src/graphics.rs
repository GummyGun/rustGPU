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


use std::mem::size_of;

#[allow(dead_code)]
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

#[repr(C)]
#[derive(Debug, Default, Clone, Copy)]
pub struct Vertex {
    pub position: Vector3<f32>,
    pub color: Vector3<f32>,
    pub coordenates: Vector2<f32>,
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
        println!("{:?}", model.name);
        println!("{:?}", model.mesh.positions.len());
        println!("{:?}", model.mesh.positions.len());
        
        let mesh = model.mesh;
        
        
        
        let mut index_vec:Vec<u32> = Vec::new();
        let mut vertex_vec:Vec<Vertex> = Vec::new();
        
        for (index, current_u32) in mesh.indices.iter().enumerate() {
            let mut vertex = Vertex::default();
            
            let current = usize::try_from(*current_u32).unwrap();
            
            vertex.position = Self::vector3_from_index(&mesh.positions, current);
            vertex.coordenates = Self::vector2_from_index(&mesh.texcoords, current);
            
            vertex.color = Vector3::new(1f32, 1f32, 1f32);
            
            let index_u32 = u32::try_from(index).expect("models cant have more than u^32 vertices");
            
            //println!("{:?}", vertex);
            vertex_vec.push(vertex);
            index_vec.push(index_u32);
        }
        
        let image_holder = ImageReader::open(texture_file).unwrap().decode().map_err(|_| AAError::DecodeError).unwrap().into_rgba8();
        
        /*
        println!("{:?}", index_vec);
        println!("{:?}", vertex_vec);
        panic!("{:#?} {:#?}", models, materials);
        */
        
        Ok(Self{
            vertices: vertex_vec,
            indices: index_vec,
            image: image_holder
        })
    }
    
    #[inline(always)]
    fn vector3_from_index<T:Copy>(vec:&Vec<T>, index:usize) -> Vector3<T> {
        
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

