use crate::AAError;
use crate::State;

use super::logger::image as logger;
use super::DeviceDestroy;
use super::Device;
use super::Allocator;

use ash::vk;
use gpu_allocator::vulkan as vkmem;
//use gpu_allocator as gpuall;

pub struct Image2 {
    pub image: vk::Image,
    pub view: vk::ImageView,
    pub allocation: vkmem::Allocation,
    pub extent: vk::Extent3D,
    pub format: vk::Format,
}

#[derive(Debug)]
pub struct Image2Metadata {
    d_name: Option<&'static str>,
    format: vk::Format,
    usage: ash::vk::ImageUsageFlags,
    aspect_flags: ash::vk::ImageAspectFlags,
}

pub const RENDER:Image2Metadata = {
    use vk::ImageUsageFlags as IUF;
    use vk::ImageAspectFlags as IAF;
    Image2Metadata{
        d_name: Some("RENDER"),
        format: vk::Format::R16G16B16A16_SFLOAT,
        usage: IUF::from_raw(0x1b),
        //IUF::TRANSFER_SRC | IUF::TRANSFER_DST
        //IUF::STORAGE      | IUF::COLOR_ATTACHMENT
        aspect_flags: IAF::COLOR,
    }
};

impl Image2 {
    
    
//----
    pub fn create(
        device: &Device,
        allocator: &mut Allocator,
        extent: vk::Extent3D,
        metadata: Image2Metadata,
    ) -> Result<Self, AAError> {
        logger::creation(metadata.d_name);
        
        let format = metadata.format;
        let extent = extent;
        let create_info = Self::create_info(format, metadata.usage, extent);
        
        let image = unsafe{device.create_image(&create_info, None)}?;
        let memory_requirements = unsafe{device.get_image_memory_requirements(image)};
        
        
        let allocation = allocator.allocate_gpu_only(metadata.d_name.unwrap_or_default(), memory_requirements);
        
        unsafe{device.bind_image_memory(image, allocation.memory(), allocation.offset())}?;
        
        let view = Self::create_view(device, image, format, metadata.aspect_flags)?;//unsafe{device.create_image_view(&view_create_info, None)}?;
        
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
    

    


}


impl DeviceDestroy for Image2 {
    fn device_destroy(&mut self, _:&State, device:&Device) {
        
        panic!("decide how to hande destruction");
    }
}


