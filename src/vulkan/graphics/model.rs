use ash::vk;

use super::super::{
    DeviceDestroy,
    Device,
    PDevice,
    CommandControl,
    Image,
    Buffer,
};

use crate::{
    State,
    graphics,
    AAError,
    
};


pub struct Model {
    pub texture: Image,
    pub vertex_buffer: Buffer,
    pub index_buffer: Buffer,
    pub index_count: u32,
    pub texture_descriptor: vk::DescriptorSet,
}



impl Model {
    pub fn vk_load(
        state:&State, 
        p_device:&PDevice,
        device:&Device,
        command_control:&CommandControl,
        metadata:(&'static str, &'static str, graphics::FileType),
        load_transformations:(Option<((f32,f32,f32), f32)>, Option<(f32,f32,f32)>, Option<(graphics::SizeTransformation, f32)>),
    ) -> Result<Self, AAError> { 
        
        if state.v_exp() {
            println!("\nCREATING:\tMODEL with obj file {} and texture {}", metadata.0, metadata.1);
        }
        
        let transformation = graphics::LoadTransformation::default();
        
        let transformation = if let Some((axis, rotation)) = load_transformations.0 {
            transformation.load_rotation((axis.0, axis.1, axis.2), rotation)
        } else {
            transformation
        };
        
        let transformation = if let Some((operation, factor)) = load_transformations.2 {
            transformation.load_size_transformation(operation, factor).expect("tranformation should be bigger than 1")
        } else {
            transformation
        };
        
        let transformation = if let Some((x,y,z)) = load_transformations.1 {
            transformation.load_translation((x,y,z))   
        } else {
            transformation
        };
        
        
        if state.v_exp() {
            println!("load modification {:?}", transformation);
        }
        
        let model_raw = graphics::Model::load(state, metadata, transformation).unwrap();
        
        let texture = Image::create(state, &p_device, &device, &command_control, &model_raw.image)?;
        let vertex_buffer = Buffer::create_vertex(state, &p_device, &device, &command_control, &model_raw.vertices)?;
        let index_buffer = Buffer::create_index(state, &p_device, &device, &command_control, &model_raw.indices)?;
        
        Ok(Self{
            texture: texture,
            vertex_buffer: vertex_buffer,
            index_buffer: index_buffer,
            index_count: u32::try_from(model_raw.indices.len()).unwrap(),
            texture_descriptor: vk::DescriptorSet::null(),
        })
    }
    
    pub fn render(
        &self,
        state:&State,
    ) -> (vk::Buffer, vk::Buffer, vk::DescriptorSet, u32) {
        if state.v_dmp() {
            println!("fetching model info for rendering");
        }
        (self.vertex_buffer.buffer, self.index_buffer.buffer, self.texture_descriptor, self.index_count) 
    }
}

impl DeviceDestroy for Model {
    fn device_drop(&mut self, state:&State, device:&Device) {
        if state.v_nor() {
            println!("[0]:deleting depth buffer");
        }
        self.texture.device_drop(state, device);
        self.vertex_buffer.device_drop(state, device);
        self.index_buffer.device_drop(state, device);
    }
}

impl DeviceDestroy for Vec<Model> {
    
    fn device_drop(&mut self, state:&State, device:&Device) {
        
        for elem in self.iter_mut() {
            elem.device_drop(state, device);
        }
    }
}
