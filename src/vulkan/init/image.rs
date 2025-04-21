use crate::AAError;
use crate::macros;
use crate::logger;
use crate::errors::messages::GPU_FREE;
use crate::errors::messages::CPU_ACCESIBLE;

use super::VkDestructor;
use super::VkDestructorType;
use super::VkDeferedDestructor;
use super::VkDynamicDestructor;
use super::VkDestructorArguments;
use super::Device;
use super::Allocator;
use super::CommandControl;
use super::memory;
use super::Buffer;

use std::slice::from_ref;
use std::mem::ManuallyDrop;

use ash::vk;
use gpu_allocator::vulkan as gpu_vk;

pub struct Image {
    pub image: vk::Image,
    pub view: vk::ImageView,
    pub allocation: ManuallyDrop<gpu_vk::Allocation>,
    pub extent: vk::Extent3D,
    pub extent_2d: vk::Extent2D,
    pub format: vk::Format,
}

macros::impl_underlying!(Image, vk::Image, image);

#[derive(Debug, Clone)]
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

pub const TEXTURE:ImageMetadata = {
    use vk::ImageUsageFlags as IUF;
    use vk::ImageAspectFlags as IAF;
    ImageMetadata{
        d_name: Some("TEXTURE IMAGE"),
        format: vk::Format::R8G8B8A8_UNORM,
        usage: IUF::from_raw(0x06),
        //IUF::TRANSFER_DST | IUF::SAMPLED
        aspect_flags: IAF::COLOR,
    }
};

impl ImageMetadata {
    pub fn texture(name:&'static str) -> Self {
        let mut holder = TEXTURE.clone();
        holder.d_name = Some(name);
        holder
    }
}

impl Image {
    
//----
    pub fn get_extent2d(&self) -> vk::Extent2D {
        self.extent_2d
    }
    
//----
    pub fn create(
        device: &mut Device,
        allocator: &mut Allocator,
        extent: vk::Extent3D,
        metadata: ImageMetadata,
        overwrite_name: Option<&str>,
    ) -> Result<Self, AAError> {
        logger::create!("image");
        
        let name = match (overwrite_name, metadata.d_name) {
            (Some(overwrite_name), _) => {overwrite_name}
            (None, Some(default_name)) => {default_name}
            (None, None) => {""}
        };
        
        logger::various_log!("image",
            (logger::Trace, "Image name {:?}", name)
        );
        
        let format = metadata.format;
        let extent = extent;
        let extent_2d = Self::extent_3d_to_extent_2d(extent);
        let create_info = Self::create_info(format, metadata.usage, extent);
        
        let image = unsafe{device.create_image(&create_info, None)}?;
        let memory_requirements = unsafe{device.get_image_memory_requirements(image)};
        
        
        let allocation = allocator.allocate(name, memory_requirements, memory::GpuOnly);
        
        unsafe{device.bind_image_memory(image, allocation.memory(), allocation.offset())}?;
        
        let view = Self::create_view(device, image, format, metadata.aspect_flags)?;
        
        Ok(Self{
            image, 
            view, 
            allocation: ManuallyDrop::new(allocation), 
            extent, 
            extent_2d, 
            format
        })
    }
    
//----
    pub fn create_texture(
        device: &mut Device,
        allocator: &mut Allocator,
        cmd_ctrl: &mut CommandControl,
        extent: vk::Extent3D,
        overwrite_name: Option<&str>,
        data: &[u32],
    ) -> Result<Self, AAError> {
        
        let upload_buffer_size = u64::from(extent.depth * extent.width * extent.height * 4);
        
        let mut upload_buffer = Buffer::create(device, allocator, Some("upload buffer"), upload_buffer_size, vk::BufferUsageFlags::TRANSFER_SRC, memory::CpuToGpu)?;
        {
            let mut align = upload_buffer.get_align::<u32>(0, upload_buffer_size).expect(CPU_ACCESIBLE);
            align.copy_from_slice(data);
        }
        
        let holder = Self::create(device, allocator, extent, TEXTURE, overwrite_name)?;
        
        let _copy_state = cmd_ctrl.run_su_buffer(device, &mut |device, cmd|{
            let image_handle = holder.underlying();
            Self::transition_image(device, cmd, image_handle, vk::ImageLayout::UNDEFINED, vk::ImageLayout::TRANSFER_DST_OPTIMAL);
            
            let subresource = vk::ImageSubresourceLayers::builder()
                .aspect_mask(vk::ImageAspectFlags::COLOR)
                .mip_level(0)
                .base_array_layer(0)
                .layer_count(1);
            
            let image_copy = vk::BufferImageCopy::builder()
                .buffer_offset(0)
                .buffer_row_length(0)
                .buffer_image_height(0)
                .image_subresource(*subresource)
                .image_offset(vk::Offset3D::default())
                .image_extent(extent);
            
            unsafe{device.cmd_copy_buffer_to_image(
                cmd,
                upload_buffer.underlying(),
                image_handle,
                vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                from_ref(&image_copy)
                
            )};
            Self::transition_image(device, cmd, image_handle, vk::ImageLayout::TRANSFER_DST_OPTIMAL, vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL);
            Ok(())
        })?;
        
        upload_buffer.destruct(VkDestructorArguments::DevAll(device, allocator));
        Ok(holder)
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
    
//----
    fn extent_3d_to_extent_2d(base:vk::Extent3D) -> vk::Extent2D {
        vk::Extent2D{width: base.width, height: base.height}
    }
    
//----
    unsafe fn unsafe_clone(&self) -> Self {
        std::ptr::read(self)
    }
    
//----
    fn internal_destroy(self, device:&mut Device, allocator:&mut Allocator) {
        logger::destruct!("image");
        unsafe{device.destroy_image_view(self.view, None)};
        unsafe{device.destroy_image(self.image, None)};
        allocator.free(ManuallyDrop::into_inner(self.allocation)).expect(GPU_FREE);
    }

}



impl VkDestructor for Image {
    fn destruct(self, mut args:VkDestructorArguments) {
        let (device, allocator) = args.unwrap_dev_all();
        self.internal_destroy(device, allocator);
    }
}


impl VkDeferedDestructor for Image {
    fn defered_destruct(&mut self) -> VkDynamicDestructor {
        let target = unsafe{self.unsafe_clone()};
        let callback = Box::new(move |mut args:VkDestructorArguments|{
            let target = target;
            let (device, allocator) = args.unwrap_dev_all();
            target.internal_destroy(device, allocator);
        });
        (callback, VkDestructorType::DevAll)
    }
}

pub fn init_textures(device:&mut Device, allocator:&mut Allocator, cmd_ctrl:&mut CommandControl) -> (Image, Image, Image, Image){
    
    let texture_extent = vk::Extent3D{width:1, height:1, depth:1};
    
    let white_pixel:u32 = 0x00_ffffff;
    let white_texture = Image::create_texture(device, allocator, cmd_ctrl, texture_extent, Some("white texture"), from_ref(&white_pixel)).unwrap();
    
    let grey_pixel:u32 = 0x00_aaaaaa;
    let grey_texture = Image::create_texture(device, allocator, cmd_ctrl, texture_extent, Some("grey texture"), from_ref(&grey_pixel)).unwrap();
    
    let black_pixel:u32 = 0x11_00_00_00;
    let black_texture = Image::create_texture(device, allocator, cmd_ctrl, texture_extent, Some("black texture"), from_ref(&black_pixel)).unwrap();
    
    let magenta_pixel:u32 = 0x11_FF_00_FF;
    
    let texture_extent = vk::Extent3D{width:64, height:64, depth:1};
    let error_data:[u32; 64*64] = std::array::from_fn(|index|{
        let pixel_row = index/16;
        let pixel_col = index%16;
        let pattern_row = pixel_row/4;
        let pattern_col = pixel_col/4;
        if pattern_row&1 == pattern_col&1 {
            black_pixel
        } else {
            magenta_pixel
        }
    });
    
    let error_texture = Image::create_texture(device, allocator, cmd_ctrl, texture_extent, Some("error texture"), &error_data).unwrap();
    
    (white_texture, grey_texture, black_texture, error_texture)
}

