use ash::{
    vk,
    prelude::VkResult,
};

use super::{
    DeviceDestroy,
    device::Device,
    buffers::UniformBuffers,
    image::Image,
    sampler::Sampler,
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
    pub layout: vk::DescriptorSetLayout,
}

pub struct DescriptorControl {
    pub layout: vk::DescriptorSetLayout,
    pub pool: vk::DescriptorPool,
    pub sets: [vk::DescriptorSet; constants::fif::USIZE],
}

impl DescriptorControl {
    pub fn create(state:&State, device:&Device) -> VkResult<DescriptorControlLayout> {
        if state.v_exp() {
            println!("\nCREATING:\tDESCRIPTOR SET LAYOUT");
        }
        
        let d_s_layout_binding = [
            vk::DescriptorSetLayoutBinding::builder()
                .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                .descriptor_count(1)
                .stage_flags(vk::ShaderStageFlags::VERTEX)
                .build(),
            vk::DescriptorSetLayoutBinding::builder()
                .binding(1)
                .descriptor_count(1)
                .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .stage_flags(vk::ShaderStageFlags::FRAGMENT)
                .build(),
        ];
        
        let create_info = vk::DescriptorSetLayoutCreateInfo::builder()
            .bindings(&d_s_layout_binding[..]);
        
        let layout = unsafe{device.create_descriptor_set_layout(&create_info, None)}?;
        Ok(DescriptorControlLayout{
            layout:layout,
        })
    }

    pub fn complete(
        state:&State, 
        device:&Device, 
        layout:DescriptorControlLayout, 
        sampler: &Sampler,
        texture: &Image,
        uniform_buffers:&UniformBuffers,
    ) -> VkResult<Self> {
        use constants::fif;
        
        let layout = layout.layout;
        
        if state.v_exp() {
            println!("\nCREATING:\tDESCRIPTOR POOL AND SETS");
        }
        
        let pool_size = [
            vk::DescriptorPoolSize::builder()
                .ty(vk::DescriptorType::UNIFORM_BUFFER)
                .descriptor_count(fif::U32)
                .build(),
            vk::DescriptorPoolSize::builder()
                .ty(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .descriptor_count(fif::U32)
                .build(),
        ];
        
        let create_info = vk::DescriptorPoolCreateInfo::builder()
            .pool_sizes(&pool_size[..])
            .max_sets(fif::U32);
        
        
        //panic!("{:#?}\n\n\n{:#?}\n\n\n\n", &pool_size, &create_info);
        
        let pool = unsafe{device.create_descriptor_pool(&create_info, None)}?;
        
        let layouts = vec![layout; fif::USIZE];
        
        let allocate_info = vk::DescriptorSetAllocateInfo::builder()
            .descriptor_pool(pool)
            .set_layouts(&layouts[..]);
        
        let sets_vec = unsafe{device.allocate_descriptor_sets(&allocate_info)}?;
        let mut sets_arr = [vk::DescriptorSet::null(); fif::USIZE];
        
        for (index, set) in sets_vec.into_iter().enumerate() {
            sets_arr[index] = set;
            
            let uniform_descriptor = vk::DescriptorBufferInfo::builder()
                .buffer(uniform_buffers[index].buffer)
                .range(graphics::UniformBufferObject::size_u64());
            
            let image_descriptor = vk::DescriptorImageInfo::builder()
                .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                .image_view(texture.view)
                .sampler(sampler.sampler);
            
            let write_descriptor = [
                vk::WriteDescriptorSet::builder()
                    .dst_set(set)
                    .dst_binding(0)
                    .dst_array_element(0)
                    .descriptor_type(ash::vk::DescriptorType::UNIFORM_BUFFER)
                    .buffer_info(from_ref(&uniform_descriptor))
                    .build(),
                    
                vk::WriteDescriptorSet::builder()
                    .dst_set(set)
                    .dst_binding(1)
                    .dst_array_element(0)
                    .descriptor_type(ash::vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                    .image_info(from_ref(&image_descriptor))
                    .build(),
            ];
            
            unsafe{device.update_descriptor_sets(&write_descriptor[..], &[])}
        }
        
        Ok(Self{
            sets: sets_arr,
            pool: pool,
            layout: layout,
        })
    }
}

impl DeviceDestroy for DescriptorControl {
    fn device_drop(&mut self, state:&State, device:&Device) {
        if state.v_nor() {
            println!("[0]:deleting uniform buffers");
        }
        unsafe{device.destroy_descriptor_pool(self.pool, None)};
        if state.v_nor() {
            println!("[0]:deleting uniform buffers");
        }
        unsafe{device.destroy_descriptor_set_layout(self.layout, None)};
        
    }
}

