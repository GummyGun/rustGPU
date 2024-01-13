use ash::{
    vk,
    prelude::VkResult,
};

use super::{
    DeviceDestroy,
    device::Device,
    instance::Instance,
    p_device::PDevice,
    image::Image,
    swapchain::Swapchain,
};

use crate::{
    State,
    errors::Error as AAError,
};

pub struct DepthBuffer{
    pub image: Image,
    pub format: vk::Format,
}


impl DepthBuffer {
    pub fn create(
        state: &State,
        instance: &Instance, 
        p_device: &PDevice,
        device: &Device,
        swapchain: &Swapchain,
        
    ) -> VkResult<Self> {
        if state.v_exp() {
            println!("\nCREATING:\tDEPTH BUFFER");
        }
        
        let target_tiling = vk::ImageTiling::OPTIMAL;
        let depth_format = Self::find_depth_format(state, instance, p_device, target_tiling);
        if state.v_exp() {
            println!("depth buffer format is: {:?}", depth_format);
        }
        let extent = vk::Extent3D::from(swapchain.extent);
        let (image, _image_size) = Image::create_image(
            state, 
            p_device, 
            device, 
            &extent, 
            depth_format, 
            vk::ImageTiling::OPTIMAL,
            vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
        )?;
        let image_view = Image::create_image_view(state, device, &image.0, depth_format, vk::ImageAspectFlags::DEPTH)?;
        Ok(Self{
            image: Image::from((image, image_view, 0)),
            format: depth_format,
        })
    }
    
    
    fn find_depth_format(
        state: &State,
        instance: &Instance, 
        p_device: &PDevice,
        target_tiling: vk::ImageTiling,
    ) -> vk::Format {
        use ash::vk::Format as F;
        Self::find_supported_format(
            state, 
            instance, 
            p_device, 
            &[F::D32_SFLOAT, F::D32_SFLOAT_S8_UINT, F::D24_UNORM_S8_UINT],
            target_tiling,
            vk::FormatFeatureFlags::DEPTH_STENCIL_ATTACHMENT,
        ).expect("format should be present")
    }
    
    fn find_supported_format(
        state: &State,
        instance: &Instance, 
        p_device: &PDevice,
        formats: &[vk::Format],
        tiling: vk::ImageTiling,
        features: vk::FormatFeatureFlags,
    ) -> Result<vk::Format, AAError> {
        
        for format in formats.into_iter() {
            let format_properties = unsafe{instance.get_physical_device_format_properties(p_device.underlying(), *format)};
            //println!("\n\n\n{:#?}", format_properties);
            if tiling == ash::vk::ImageTiling::LINEAR && ((format_properties.linear_tiling_features & features) == features) {
                if state.v_exp() {
                    println!("correct linear image tiling");
                }
                return Ok(*format);
            } else if tiling == ash::vk::ImageTiling::OPTIMAL && ((format_properties.optimal_tiling_features & features) == features) {
                if state.v_exp() {
                    println!("correct optimal image tiling");
                }
                return Ok(*format);
            }
            
        }
        Err(AAError::UnsuportedFormat)
    }
}

impl DeviceDestroy for DepthBuffer {
    fn device_destroy(&mut self, state:&State, device:&Device) {
        if state.v_nor() {
            println!("[0]:deleting depth buffer");
        }
        self.image.silent_drop(device);
    }
}
