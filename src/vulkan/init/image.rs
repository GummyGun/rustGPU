use crate::AAError;
use crate::macros;
use crate::logger;
use crate::errors::messages::GPU_FREE;

use super::VkDestructor;
use super::VkDestructorArguments;
use super::Device;
use super::Allocator;
use super::memory;

use std::slice::from_ref;

use ash::vk;
use gpu_allocator::vulkan as gpu_vk;

pub struct Image {
    pub image: vk::Image,
    pub view: vk::ImageView,
    pub allocation: gpu_vk::Allocation,
    pub extent: vk::Extent3D,
    pub format: vk::Format,
}

macros::impl_underlying!(Image, vk::Image, image);

#[derive(Debug)]
pub struct ImageMetadata {
    d_name: Option<&'static str>,
    pub format: vk::Format,
    usage: ash::vk::ImageUsageFlags,
    aspect_flags: ash::vk::ImageAspectFlags,
}

pub const RENDER:ImageMetadata = {
    use vk::ImageUsageFlags as IUF;
    use vk::ImageAspectFlags as IAF;
    ImageMetadata{
        d_name: Some("RENDER IMAGE"),
        format: vk::Format::R16G16B16A16_SFLOAT,
        usage: IUF::from_raw(0x1b),
        //IUF::TRANSFER_SRC | IUF::TRANSFER_DST
        //IUF::STORAGE      | IUF::COLOR_ATTACHMENT
        aspect_flags: IAF::COLOR,
    }
};

pub const DEPTH:ImageMetadata = {
    use vk::ImageUsageFlags as IUF;
    use vk::ImageAspectFlags as IAF;
    ImageMetadata{
        d_name: Some("DEPTH IMAGE"),
        format: vk::Format::D32_SFLOAT,
        usage: IUF::from_raw(0x20),
        //IUF::DEPTH_STENCIL_ATTACHMENT
        aspect_flags: IAF::DEPTH,
    }
};

impl Image {
    
//----
    pub fn get_extent2d(&self) -> vk::Extent2D {
        vk::Extent2D{
            width: self.extent.width,
            height: self.extent.height,
        }
    }
    
//----
    pub fn create(
        device: &mut Device,
        allocator: &mut Allocator,
        extent: vk::Extent3D,
        metadata: ImageMetadata,
    ) -> Result<Self, AAError> {
        logger::create!("image");
        
        match metadata.d_name {
            Some(d_name) => {
                logger::various_log!("image",
                    (logger::Trace, "Image name {:?}", d_name)
                );
            }
            None => {}
        }
        
        let format = metadata.format;
        let extent = extent;
        let create_info = Self::create_info(format, metadata.usage, extent);
        
        let image = unsafe{device.create_image(&create_info, None)}?;
        let memory_requirements = unsafe{device.get_image_memory_requirements(image)};
        
        
        let allocation = allocator.allocate(metadata.d_name.unwrap_or_default(), memory_requirements, memory::GpuOnly);
        
        unsafe{device.bind_image_memory(image, allocation.memory(), allocation.offset())}?;
        
        let view = Self::create_view(device, image, format, metadata.aspect_flags)?;
        
        Ok(Self{image, view, allocation, extent, format})
    }
    
//----
    pub fn create_view(
        device: &Device,
        image: vk::Image,
        format: vk::Format,
        aspect: vk::ImageAspectFlags,
    ) -> Result<vk::ImageView, AAError> {
        let view_create_info = Self::view_create_info(image, format, aspect);
        Ok(unsafe{device.create_image_view(&view_create_info, None)}?)
    }
    
    
//----
    fn create_info(
        format: vk::Format, 
        usage_flags: vk::ImageUsageFlags,
        extent: vk::Extent3D,
    ) -> vk::ImageCreateInfo {
        let mut holder = vk::ImageCreateInfo::default();
        holder.image_type = vk::ImageType::TYPE_2D;
        holder.mip_levels = 1;
        holder.array_layers = 1;
        holder.samples = vk::SampleCountFlags::TYPE_1;
        holder.tiling = vk::ImageTiling::OPTIMAL;
        holder.usage = usage_flags;
        holder.format = format;
        holder.extent = extent;
        holder
    }
    
    
    
//----
    fn view_create_info(
        image: vk::Image,
        format: vk::Format, 
        aspect_flags: vk::ImageAspectFlags,
    ) -> vk::ImageViewCreateInfo {
        let mut holder = vk::ImageViewCreateInfo::default();
        holder.view_type = vk::ImageViewType::TYPE_2D;
        holder.format = format;
        holder.image = image;
        holder.subresource_range.base_mip_level = 0;
        holder.subresource_range.level_count = 1;
        holder.subresource_range.base_array_layer = 0;
        holder.subresource_range.layer_count = 1;
        holder.subresource_range.aspect_mask = aspect_flags;
        holder
    }
    
    

//----
    pub fn subresource_range(aspect:vk::ImageAspectFlags) -> vk::ImageSubresourceRange {
        let mut holder = vk::ImageSubresourceRange::default();
        holder.aspect_mask = aspect;
        holder.level_count = vk::REMAINING_MIP_LEVELS;
        holder.layer_count = vk::REMAINING_ARRAY_LAYERS;
        holder
    }
    

    

//----
    pub fn copy_from_image(&mut self, device:&Device, cmd:vk::CommandBuffer, src:Image) {
        Self::raw_copy_image_to_image(device, cmd, src.image, src.extent, self.image, self.extent);
    }
    

//----
    pub fn raw_copy_image_to_image(device:&Device, cmd:vk::CommandBuffer, src:vk::Image, src_extent:vk::Extent3D, dst:vk::Image, dst_extent:vk::Extent3D) {
        
        let mut blit_region = vk::ImageBlit2::default();
        
        blit_region.src_offsets[1].x = src_extent.width as i32;
        blit_region.src_offsets[1].y = src_extent.height as i32;
        blit_region.src_offsets[1].z = src_extent.depth as i32;
        
        blit_region.dst_offsets[1].x = dst_extent.width as i32;
        blit_region.dst_offsets[1].y = dst_extent.height as i32;
        blit_region.dst_offsets[1].z = dst_extent.depth as i32;
        
        blit_region.src_subresource.aspect_mask = vk::ImageAspectFlags::COLOR;
        blit_region.src_subresource.base_array_layer = 0;
        blit_region.src_subresource.layer_count = 1;
        blit_region.src_subresource.mip_level = 0;
        
        blit_region.dst_subresource.aspect_mask = vk::ImageAspectFlags::COLOR;
        blit_region.dst_subresource.base_array_layer = 0;
        blit_region.dst_subresource.layer_count = 1;
        blit_region.dst_subresource.mip_level = 0;
        
        let cmd_info = vk::BlitImageInfo2::builder()
            .src_image(src)
            .src_image_layout(vk::ImageLayout::TRANSFER_SRC_OPTIMAL)
            .dst_image(dst)
            .dst_image_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
            .filter(vk::Filter::LINEAR)
            .regions(from_ref(&blit_region));
        
        unsafe{device.cmd_blit_image2(cmd, &cmd_info)}
    }
    

    
//----
    pub fn transition_image(
        device: &Device,
        command_buffer: vk::CommandBuffer,
        image: vk::Image,
        old_layout: vk::ImageLayout,
        new_layout: vk::ImageLayout,
    ) {
        //logger::transitioning_image(old_layout, new_layout);
        
        let image_aspect = match new_layout {
            vk::ImageLayout::DEPTH_ATTACHMENT_OPTIMAL => {
                vk::ImageAspectFlags::DEPTH
            }
            _ => {
                vk::ImageAspectFlags::COLOR
            }
        };
        
        let subresource = Image::subresource_range(image_aspect);
        
        let image_barrier = vk::ImageMemoryBarrier2::builder()
            .image(image)
            .old_layout(old_layout)
            .new_layout(new_layout)
            .src_stage_mask(vk::PipelineStageFlags2::ALL_COMMANDS)
            .src_access_mask(vk::AccessFlags2::MEMORY_WRITE)
            .dst_stage_mask(vk::PipelineStageFlags2::ALL_COMMANDS)
            .dst_access_mask(vk::AccessFlags2::MEMORY_WRITE|vk::AccessFlags2::MEMORY_READ)
            .subresource_range(subresource);
        
        let dependency = ash::vk::DependencyInfo::builder()
            .image_memory_barriers(from_ref(&image_barrier));
        
        let _ = unsafe{device.cmd_pipeline_barrier2(command_buffer, &dependency)};
        
    }
    
}



impl VkDestructor for Image {
    fn destruct(self, mut args:VkDestructorArguments) {
        logger::destruct!("image");
        let (device, allocator) = args.unwrap_dev_all();
        unsafe{device.destroy_image_view(self.view, None)};
        unsafe{device.destroy_image(self.image, None)};
        allocator.free(self.allocation).expect(GPU_FREE);
    }
}


