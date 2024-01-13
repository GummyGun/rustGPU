use ash::{
    vk,
    prelude::VkResult,
};

use super::{
    DeviceDestroy,
    device::Device,
    buffers::UniformBuffers,
    sampler::Sampler,
};

use super::super::{
    Model,
};

use crate::{
    State,
    constants,
    graphics,
};


use std::{
    slice::from_ref,
};


pub struct DescriptorControlLayout {
    pub layouts: Vec<vk::DescriptorSetLayout>,
}

pub struct DescriptorControl {
    pub layouts: Vec<vk::DescriptorSetLayout>,
    pub pool: vk::DescriptorPool,
    pub sets: [vk::DescriptorSet; constants::fif::USIZE],
}

impl DescriptorControl {
    #[allow(dead_code)]
    pub fn create(state:&State, device:&Device) -> VkResult<DescriptorControlLayout> {
        if state.v_exp() {
            println!("\nCREATING:\tDESCRIPTOR SET LAYOUT");
        }
        
        let uniform_layout_binding = [
            vk::DescriptorSetLayoutBinding::builder()
                .binding(0)
                .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                .descriptor_count(1)
                .stage_flags(vk::ShaderStageFlags::VERTEX)
                .build(),
        ];
        
        let texture_layout_binding = [
            vk::DescriptorSetLayoutBinding::builder()
                .binding(0)
                .descriptor_count(1)
                .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .stage_flags(vk::ShaderStageFlags::FRAGMENT)
                .build(),
        ];
        
        let create_info = [
            vk::DescriptorSetLayoutCreateInfo::builder()
                .bindings(&uniform_layout_binding[..])
                .build(),
            vk::DescriptorSetLayoutCreateInfo::builder()
                .bindings(&texture_layout_binding[..])
                .build(),
        ];
        
        let layouts = create_info.iter().map(|create_info|{
            unsafe{device.create_descriptor_set_layout(create_info, None)}.unwrap()
        }).collect();
        
        Ok(DescriptorControlLayout{
            layouts:layouts,
        })
    }

    #[allow(dead_code)]
    pub fn complete(
        state: &State, 
        device: &Device, 
        layouts: DescriptorControlLayout, 
        sampler: &Sampler,
        models: &mut [Model],
        uniform_buffers: &UniformBuffers,
    ) -> VkResult<Self> {
        use constants::fif;
        
        if state.v_exp() {
            println!("\nCREATING:\tDESCRIPTOR POOL AND SETS");
        }
        
        let models_len_u32 = u32::try_from(models.len()).unwrap();
        
        let pool_size = [
            vk::DescriptorPoolSize::builder()
                .ty(vk::DescriptorType::UNIFORM_BUFFER)
                .descriptor_count(fif::U32)
                .build(),
            vk::DescriptorPoolSize::builder()
                .ty(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .descriptor_count(models_len_u32)
                .build(),
        ];
        
        let create_info = vk::DescriptorPoolCreateInfo::builder()
            .pool_sizes(&pool_size[..])
            .max_sets(fif::U32 + models_len_u32);
        
        
        let pool = unsafe{device.create_descriptor_pool(&create_info, None)}?;
        
        let mut uniform_layouts = vec![layouts.layouts[0]; fif::USIZE];
        uniform_layouts.extend(vec![layouts.layouts[1]; models.len()]);
        
        let allocate_info = vk::DescriptorSetAllocateInfo::builder()
            .descriptor_pool(pool)
            .set_layouts(&uniform_layouts[..]);
        
        let sets_vec = unsafe{device.allocate_descriptor_sets(&allocate_info)}?;
        
        let mut sets_arr = [vk::DescriptorSet::null(); fif::USIZE];
        
        
        for (index, set) in sets_vec[0..fif::USIZE].into_iter().enumerate() {
            let set = *set;
            sets_arr[index] = set;
            
            let uniform_descriptor = vk::DescriptorBufferInfo::builder()
                .buffer(uniform_buffers[index].buffer)
                .range(graphics::UniformBufferObject::size_u64());
            
            let write_descriptor = [
                vk::WriteDescriptorSet::builder()
                    .dst_set(set)
                    .dst_binding(0)
                    .dst_array_element(0)
                    .descriptor_type(ash::vk::DescriptorType::UNIFORM_BUFFER)
                    .buffer_info(from_ref(&uniform_descriptor))
                    .build(),
            ];
            
            unsafe{device.update_descriptor_sets(&write_descriptor[..], &[])}
        }
        
        for (set, model) in sets_vec[fif::USIZE..].into_iter().zip(models.iter_mut()) {
            let set = *set;
            
            let image_descriptor = vk::DescriptorImageInfo::builder()
                .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                .image_view(model.texture.view)
                .sampler(sampler.sampler);
            
            let write_descriptor = [
                
                vk::WriteDescriptorSet::builder()
                    .dst_set(set)
                    .dst_binding(0)
                    .dst_array_element(0)
                    .descriptor_type(ash::vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                    .image_info(from_ref(&image_descriptor))
                    .build(),
            ];
            
            unsafe{device.update_descriptor_sets(&write_descriptor[..], &[])}
            
            model.texture_descriptor = set;
        }
        
        
        Ok(Self{
            sets: sets_arr,
            pool: pool,
            layouts: layouts.layouts,
        })
    }
    
}

impl DeviceDestroy for DescriptorControl {
    fn device_destroy(&mut self, state:&State, device:&Device) {
        if state.v_nor() {
            println!("[0]:deleting uniform buffers");
        }
        unsafe{device.destroy_descriptor_pool(self.pool, None)};
        if state.v_nor() {
            println!("[0]:deleting uniform buffers");
        }
        
        for layouts in &self.layouts {
            unsafe{device.destroy_descriptor_set_layout(*layouts, None)};
        }
        
    }
}


