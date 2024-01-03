mod model;

use nalgebra as na;

use na::{
    Vector2,
    Vector3,
    Matrix4,
};

use crate::{
    errors::Error as AAError,
};

#[repr(C)]
#[derive(Debug, Default, Clone)]
pub struct Model {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
    pub image: image::RgbaImage,
}

use std::{
    mem::size_of,
};


#[derive(Debug)]
pub enum FileType {
    Obj,
    Gltf,
}

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

#[repr(C)]
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct Vertex {
    pub position: Vector3<f32>,
    pub color: Vector3<f32>,
    pub texcoord: Vector2<f32>,
}

#[derive(Debug, Default, Clone)]
pub struct LoadTransformation {
    rotation_translation: nalgebra::UnitDualQuaternion<f32>,
    size: Option<LoadSizeTransformation>,
}

#[derive(Debug, Clone)]
enum LoadSizeTransformation {
    Enlarge(f32),
    Shrink(f32),
}

#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub enum SizeTransformation {
    Enlarge,
    Shrink,
}

impl From<(SizeTransformation, f32)> for LoadSizeTransformation {
    fn from(base:(SizeTransformation, f32)) -> Self {
        match base.0 {
            SizeTransformation::Enlarge => LoadSizeTransformation::Enlarge(base.1),
            SizeTransformation::Shrink => LoadSizeTransformation::Shrink(base.1),
        }
    }
}

impl LoadTransformation {
    pub fn load_rotation(
        mut self, 
        axis:(f32, f32, f32), 
        rotation:f32, 
    ) -> Self {
        
        let axis = na::Vector3::<f32>::new(axis.0, axis.1, axis.2);
        let norm_axis = na::Unit::new_normalize(axis);
        let rotation = na::Unit::from_axis_angle(&norm_axis, rotation);
        
        let dual_quat = na::UnitDualQuaternion::from_parts(self.rotation_translation.translation(), rotation);
        self.rotation_translation = dual_quat;
        
        self
    }
    
    pub fn load_translation(
        mut self,
        translation:(f32, f32, f32), 
    ) -> Self {
        
        let translation = na::Translation3::from(na::Vector3::new(translation.0, translation.1, translation.2));
        let dual_quat = na::UnitDualQuaternion::from_parts(translation, self.rotation_translation.rotation());
        self.rotation_translation = dual_quat;
        
        
        self
    }
    
    pub fn load_size_transformation(
        mut self,
        action: SizeTransformation,
        factor: f32,
    ) -> Result<Self, AAError> {
        if factor < 1.0 {
            return Err(AAError::InvalidLoadTransform);
        }
        self.size = Some(LoadSizeTransformation::from((action, factor)));
        Ok(self)
    }
    
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

