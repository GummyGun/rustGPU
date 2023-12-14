use ash::{
    vk,
    prelude::VkResult,
};
use image::io::Reader as ImageReader;
use image::EncodableLayout;

use super::{
    memory,
    DeviceDrop,
    device::Device,
    p_device::PDevice,
    command::CommandControl,
    buffers::Buffer,
};

use crate::{
    State,
    errors::Error as AAError,
};

use std::{
    mem::align_of,
    slice::from_ref,
};


pub struct Image {
    pub image: vk::Image,
    pub memory: vk::DeviceMemory,
}

impl Image {
    pub fn create(
        state:&State, 
        p_device:&PDevice,
        device:&Device, 
        command:&CommandControl, 
        file_name:&str
    ) -> Result<Self, AAError> {
        use vk::BufferUsageFlags as BUF;
        use vk::MemoryPropertyFlags as MPF;
        use vk::ImageUsageFlags as IUF;
        
        if state.v_exp() {
            println!("\nCREATING:\tIMAGE");
        }
        
        let img = ImageReader::open(file_name)?.decode().map_err(|_| AAError::DecodeError)?.into_rgba8();
        
        //println!("{:?}", img.as_bytes().len()/(img.width()*img.height()) as usize);
        let raw_size = u64::try_from(img.as_bytes().len()).expect("image size should fit in a u64");
        
        let staging_memory_flags = MPF::HOST_VISIBLE | MPF::HOST_COHERENT;
        let (staging, staging_size) = Buffer::create_buffer(state, p_device, device, raw_size, BUF::TRANSFER_SRC, staging_memory_flags)?;
        
        let memory_ptr = unsafe{device.map_memory(staging.memory, 0, raw_size, vk::MemoryMapFlags::empty())}?;
        let mut vert_align = unsafe{ash::util::Align::new(memory_ptr, align_of::<u8>() as u64, staging_size)};
        vert_align.copy_from_slice(&img.as_bytes());
        
        unsafe{device.unmap_memory(staging.memory)};
        
        let usage_flags = IUF::TRANSFER_DST | IUF::SAMPLED;
        let memory_flags = MPF::DEVICE_LOCAL;
        let extent = vk::Extent3D::builder()
            .width(img.width())
            .height(img.height())
            .depth(1);
        
        let (mut image, _image_size) = Self::create_image(
            state,
            p_device,
            device,
            &extent,
            usage_flags,
            memory_flags,
        )?;
        
        image.transition_image_layout(
            state,
            device, 
            command,
            //VK_FORMAT_R8G8B8A8_SRGB, 
            vk::ImageLayout::UNDEFINED, 
            vk::ImageLayout::TRANSFER_DST_OPTIMAL,
        )?;
        
        if state.v_exp() {
            println!("copying from buffer to image");
        }
        
        memory::copy_buffer_2_image(
            state,
            device,
            command,
            &staging,
            &mut image,
            &extent,
        );
        
        image.transition_image_layout(
            state,
            device, 
            command,
            //VK_FORMAT_R8G8B8A8_SRGB, 
            vk::ImageLayout::TRANSFER_DST_OPTIMAL,
            vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL, 
        )?;
        
        staging.staging_drop(state, device);
        
        Ok(image)
    }
    
    pub fn create_image (
        state: &State, 
        p_device: &PDevice, 
        device: &Device, 
        extent: &vk::Extent3D,
        usage_flags: vk::ImageUsageFlags, 
        memory_flags: vk::MemoryPropertyFlags,
    ) -> VkResult<(Self, u64)> {
        
        let create_info = vk::ImageCreateInfo::builder()
            .image_type(vk::ImageType::TYPE_2D)
            .extent(*extent)
            .mip_levels(1)
            .array_layers(1)
            .format(vk::Format::R8G8B8A8_SRGB)
            .tiling(vk::ImageTiling::OPTIMAL)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .usage(usage_flags)
            .sharing_mode(vk::SharingMode::EXCLUSIVE)
            .samples(vk::SampleCountFlags::TYPE_1);
        
        let image = unsafe{device.create_image(&create_info, None)}?;
        
        let memory_requirements = unsafe{device.get_image_memory_requirements(image)};
        
        let index_holder = memory::find_memory_type_index(state, p_device, &memory_requirements, memory_flags).expect("required memory type is not present");
        
        let allocate_info = ash::vk::MemoryAllocateInfo::builder()
            .allocation_size(memory_requirements.size)
            .memory_type_index(index_holder);
        
        let memory = unsafe{device.allocate_memory(&allocate_info, None)}?;
        
        unsafe{device.bind_image_memory(image, memory, 0)}?;
        
        Ok((Self{image:image, memory:memory}, memory_requirements.size))
    }
    
    fn transition_image_layout(
        &self,
        state: &State,
        device: &Device,
        command: &CommandControl,
        //format: vk::Format,
        old_layout: vk::ImageLayout,
        new_layout: vk::ImageLayout,
    ) -> Result<(), AAError> {
        use vk::ImageLayout as IL;
        
        if state.v_exp() {
            println!("moving image from {:?} {:?}", old_layout, new_layout);
        }
        
        let (src_access_mask, dst_access_maks, src_stage, dst_stage) = match (old_layout, new_layout) {
            (IL::UNDEFINED, IL::TRANSFER_DST_OPTIMAL) => {
                (vk::AccessFlags::empty(), vk::AccessFlags::TRANSFER_WRITE, vk::PipelineStageFlags::TOP_OF_PIPE, vk::PipelineStageFlags::TRANSFER)
            }
            (IL::TRANSFER_DST_OPTIMAL, IL::SHADER_READ_ONLY_OPTIMAL) => {
                (vk::AccessFlags::TRANSFER_WRITE, vk::AccessFlags::SHADER_READ, vk::PipelineStageFlags::TRANSFER, vk::PipelineStageFlags::FRAGMENT_SHADER)
            }
            (_, _) => {
                return Err(AAError::ImageLayoutUnsuported);
            }
        };
        
        let buffer = command.setup_su_buffer(device);
        
        let subresource = vk::ImageSubresourceRange::builder()
            .aspect_mask(vk::ImageAspectFlags::COLOR)
            .base_mip_level(0)
            .level_count(1)
            .base_array_layer(0)
            .layer_count(1);
        
        let memory_barrier = vk::ImageMemoryBarrier::builder()
            .old_layout(old_layout)
            .new_layout(new_layout)
            .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
            .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
            .image(self.image)
            .subresource_range(*subresource)
            .src_access_mask(src_access_mask)
            .dst_access_mask(dst_access_maks);
        
        unsafe{device.cmd_pipeline_barrier(
            buffer,
            src_stage,
            dst_stage,
            vk::DependencyFlags::empty(),
            &[],
            &[],
            from_ref(&memory_barrier),
        )};
        
        command.submit_su_buffer(device);
        Ok(())
    }
}

impl DeviceDrop for Image {
    fn device_drop(&mut self, state:&State, device:&Device) {
        if state.v_nor() {
            println!("[0]:deleting texture image");
        }
        unsafe{device.destroy_image(self.image, None)}
        unsafe{device.free_memory(self.memory, None)}
    }
}



