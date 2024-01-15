use ash::{
    vk,
    prelude::VkResult,
};

use zerocopy::AsBytes;

use super::{
    memory,
    DeviceDestroy,
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
    pub view: vk::ImageView,
    pub memory: vk::DeviceMemory,
    pub mip_level: i32,
}

impl Image {
    pub fn create(
        state:&State, 
        p_device:&PDevice,
        device:&Device, 
        command:&CommandControl, 
        image_data:&image::RgbaImage,
    ) -> Result<Self, AAError> {
        
        use vk::BufferUsageFlags as BUF;
        use vk::MemoryPropertyFlags as MPF;
        use vk::ImageUsageFlags as IUF;
        
        if state.v_exp() {
            println!("\nCREATING:\tIMAGE");
        }
        
        //let image:image::RgbaImage = image::io::Reader::open(crate::constants::path::TEST_TEXTURE)?.decode().map_err(|_| AAError::DecodeError)?.into_rgba8();
        //let image_data = &image;
        
        let raw_size = u64::try_from(image_data.as_bytes().len()).expect("image size should fit in a u64");
        let staging_memory_properties = MPF::HOST_VISIBLE | MPF::HOST_COHERENT;
        let (staging, staging_size) = Buffer::create_buffer(state, p_device, device, raw_size, BUF::TRANSFER_SRC, staging_memory_properties)?;
        
        let memory_ptr = unsafe{device.map_memory(staging.memory, 0, raw_size, vk::MemoryMapFlags::empty())}?;
        let mut vert_align = unsafe{ash::util::Align::new(memory_ptr, align_of::<u8>() as u64, staging_size)};
        vert_align.copy_from_slice(&image_data.as_bytes());
        
        unsafe{device.unmap_memory(staging.memory)};
        
        let usage_flags = IUF::TRANSFER_DST | IUF::SAMPLED;
        let memory_properties = MPF::DEVICE_LOCAL;
        let extent = vk::Extent3D::builder()
            .width(image_data.width())
            .height(image_data.height())
            .depth(1);
        
        let (mut image, _image_size) = Self::create_image(
            state,
            p_device,
            device,
            &extent,
            vk::Format::R8G8B8A8_SRGB,
            vk::ImageTiling::OPTIMAL,
            usage_flags,
            memory_properties,
        )?;
        
        Self::transition_image_layout(
            state,
            device, 
            command,
            &image.0,
            vk::Format::R8G8B8A8_SRGB, 
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
            &mut image.0,
            &extent,
        );
        
        Self::transition_image_layout(
            state,
            device, 
            command,
            &image.0,
            vk::Format::R8G8B8A8_SRGB, 
            vk::ImageLayout::TRANSFER_DST_OPTIMAL,
            vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL, 
        )?;
        
        staging.staging_drop(state, device);
        
        let image_view = Self::create_image_view(
            state, 
            device, 
            &image.0, 
            vk::Format::R8G8B8A8_SRGB, 
            vk::ImageAspectFlags::COLOR
        )?;
        
        Ok(Self::from((image, image_view, 0)))
    }
    
    pub fn create_image (
        state: &State, 
        p_device: &PDevice, 
        device: &Device, 
        extent: &vk::Extent3D,
        format: vk::Format,
        tiling: vk::ImageTiling,
        usage_flags: vk::ImageUsageFlags, 
        memory_properties: vk::MemoryPropertyFlags,
    ) -> VkResult<((vk::Image, vk::DeviceMemory), u64)> {
        
        let create_info = vk::ImageCreateInfo::builder()
            .image_type(vk::ImageType::TYPE_2D)
            .extent(*extent)
            .mip_levels(1)
            .array_layers(1)
            .format(format)
            .tiling(tiling)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .usage(usage_flags)
            .sharing_mode(vk::SharingMode::EXCLUSIVE)
            .samples(vk::SampleCountFlags::TYPE_1);
        
        let image = unsafe{device.create_image(&create_info, None)}?;
        
        let memory_requirements = unsafe{device.get_image_memory_requirements(image)};
        
        let index_holder = memory::find_memory_type_index(state, p_device, &memory_requirements, memory_properties).expect("required memory type is not present");
        
        let allocate_info = ash::vk::MemoryAllocateInfo::builder()
            .allocation_size(memory_requirements.size)
            .memory_type_index(index_holder);
        
        let memory = unsafe{device.allocate_memory(&allocate_info, None)}?;
        
        unsafe{device.bind_image_memory(image, memory, 0)}?;
        
        Ok(((image, memory), memory_requirements.size))
    }
    
    fn transition_image_layout(
        state: &State,
        device: &Device,
        command: &CommandControl,
        image: &vk::Image,
        format: vk::Format,
        old_layout: vk::ImageLayout,
        new_layout: vk::ImageLayout,
    ) -> Result<(), AAError> {
        use vk::ImageLayout as IL;
        
        if state.v_exp() {
            println!("moving image from {:?} {:?}", old_layout, new_layout);
        }
        
        let aspect_mask = if new_layout == IL::DEPTH_STENCIL_ATTACHMENT_OPTIMAL {
            if Self::has_stencil_component(format) {
                vk::ImageAspectFlags::DEPTH | vk::ImageAspectFlags::STENCIL
            } else {
                vk::ImageAspectFlags::DEPTH
            }
        } else {
            vk::ImageAspectFlags::COLOR
        };
        
        let (src_access_mask, dst_access_maks, src_stage, dst_stage) = match (old_layout, new_layout) {
            (IL::UNDEFINED, IL::TRANSFER_DST_OPTIMAL) => {
                (vk::AccessFlags::empty(), vk::AccessFlags::TRANSFER_WRITE, vk::PipelineStageFlags::TOP_OF_PIPE, vk::PipelineStageFlags::TRANSFER)
            }
            (IL::TRANSFER_DST_OPTIMAL, IL::SHADER_READ_ONLY_OPTIMAL) => {
                (vk::AccessFlags::TRANSFER_WRITE, vk::AccessFlags::SHADER_READ, vk::PipelineStageFlags::TRANSFER, vk::PipelineStageFlags::FRAGMENT_SHADER)
            }
            (IL::UNDEFINED, IL::DEPTH_STENCIL_ATTACHMENT_OPTIMAL) => {
                (vk::AccessFlags::empty(), vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_READ |vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE, vk::PipelineStageFlags::TOP_OF_PIPE, vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS)
            }
            (_, _) => {
                return Err(AAError::ImageLayoutUnsuported);
            }
        };
        
        let buffer = command.setup_su_buffer(device);
        
        let subresource = vk::ImageSubresourceRange::builder()
            .aspect_mask(aspect_mask)
            .base_mip_level(0)
            .level_count(1)
            .base_array_layer(0)
            .layer_count(1);
        
        let memory_barrier = vk::ImageMemoryBarrier::builder()
            .old_layout(old_layout)
            .new_layout(new_layout)
            .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
            .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
            .image(*image)
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
    
    
    pub fn create_image_view(
        state: &State,
        device: &Device,
        image: &vk::Image,
        format: vk::Format,
        aspect: vk::ImageAspectFlags,
    ) -> VkResult<vk::ImageView> {
        
        if state.v_exp() {
            println!("creating image views");
        }
        
        let component_create_info = vk::ComponentMapping::builder()
            .r(vk::ComponentSwizzle::IDENTITY)
            .g(vk::ComponentSwizzle::IDENTITY)
            .b(vk::ComponentSwizzle::IDENTITY)
            .a(vk::ComponentSwizzle::IDENTITY);
        
        let subresource_range_create_info = vk::ImageSubresourceRange::builder()
            .aspect_mask(vk::ImageAspectFlags::COLOR)
            .base_mip_level(0)
            .level_count(1)
            .base_array_layer(0)
            .aspect_mask(aspect)
            .layer_count(1);
        
        let create_info = vk::ImageViewCreateInfo::builder()
            .format(format)
            .view_type(vk::ImageViewType::TYPE_2D)
            .components(*component_create_info)
            .subresource_range(*subresource_range_create_info)
            .image(*image);
        
        let holder = unsafe{device.create_image_view(&create_info, None)?};
        
        Ok(holder)
    }
    
    
    fn has_stencil_component(format: vk::Format) -> bool {
        format == vk::Format::D32_SFLOAT_S8_UINT || format == vk::Format::D24_UNORM_S8_UINT
    }
    
    pub fn silent_drop(&mut self, device:&Device) {
        unsafe{device.destroy_image_view(self.view, None)};
        unsafe{device.destroy_image(self.image, None)};
        unsafe{device.free_memory(self.memory, None)};
    }
    
}

impl From<((vk::Image, vk::DeviceMemory), vk::ImageView, i32)> for Image {
    fn from(base:((vk::Image, vk::DeviceMemory), vk::ImageView, i32)) -> Self {
        Self{image:base.0.0, memory:base.0.1, view: base.1, mip_level:base.2}
    }
}

impl DeviceDestroy for Image {
    fn device_destroy(&mut self, state:&State, device:&Device) {
        if state.v_nor() {
            println!("[0]:deleting texture image");
        }
        self.silent_drop(device);
    }
}



