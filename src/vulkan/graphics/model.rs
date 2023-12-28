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
        metadata:(&'static str, &'static str, graphics::FileType)
    ) -> Result<Self, AAError> { 
        
        if state.v_exp() {
            println!("\nCREATING:\tMODEL with obj file {} and texture {}", metadata.0, metadata.1);
        }
        let model_raw = graphics::Model::load(state, metadata).unwrap();
        
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
