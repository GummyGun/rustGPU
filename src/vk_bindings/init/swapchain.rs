use ash::{
    vk,
    prelude::VkResult,
};

use super::{
    DeviceDrop,
    device::Device,
    instance::Instance,
    surface::Surface,
    p_device::PhysicalDevice,
    render_pass::RenderPass,
};

use crate::{
    State,
    window::{
        Window
    },
};

use std::{
    ops::Deref,
};



pub struct SwapchainBasic {
    pub image_views: Vec<vk::ImageView>,
    pub images: Vec<vk::Image>,
    pub extent: vk::Extent2D,
    pub surface_format: vk::SurfaceFormatKHR,
    
    pub swapchain: vk::SwapchainKHR,
    swapchain_loader: ash::extensions::khr::Swapchain,
}

#[derive(Debug, Default)]
pub struct SwapchainSupportDetails {
    pub surface_capabilities: vk::SurfaceCapabilitiesKHR,
    pub surface_formats: Vec<vk::SurfaceFormatKHR>,
    pub present_modes: Vec<vk::PresentModeKHR>,
    
}

pub struct Swapchain {
    pub swapchain_basic: SwapchainBasic,
    pub framebuffers: Vec<vk::Framebuffer>
}

impl Deref for Swapchain {
    type Target = SwapchainBasic;
    fn deref(&self) -> &Self::Target {
        &self.swapchain_basic
    }
    
}

impl Swapchain {
    pub fn complete(state:&State, device:&Device, swapchain:SwapchainBasic, render_pass:&RenderPass) -> VkResult<Self> {
        if  state.v_exp() {
            println!("\nCREATING:\tFRAME BUFFER");
        }
        
        let mut framebuffers_holder:Vec<vk::Framebuffer> = Vec::with_capacity(swapchain.image_views.len());
        
        for image_view in &swapchain.image_views {
            
            let attachments = [*image_view];
            let create_info = vk::FramebufferCreateInfo::builder()
                .render_pass(render_pass.as_inner())
                .attachments(&attachments[..])
                .width(swapchain.extent.width)
                .height(swapchain.extent.height)
                .layers(1);
            let holder = unsafe{device.create_framebuffer(&create_info, None)}?;
            framebuffers_holder.push(holder);
        }
        
        Ok(Self{
            swapchain_basic: swapchain,
            framebuffers: framebuffers_holder,
        })
    }
    
    pub fn direct_create(state:&State, window:&Window, instance:&Instance, surface:&Surface, p_device:&PhysicalDevice, device:&Device, render_pass:&RenderPass) -> VkResult<Self> {
        let holder = SwapchainBasic::create(state, window, instance, surface, p_device, device)?;
        Self::complete(state, device, holder, render_pass)
    }
    
    
}


impl SwapchainBasic {
    
    pub fn create(state:&State, window:&Window, instance:&Instance, surface:&Surface, p_device:&PhysicalDevice, device:&Device) -> VkResult<Self> {
        if  state.v_exp() {
            println!("\nCREATING:\tSWAPCHAIN");
        }
        
        let surface_format = p_device.swapchain_details.choose_surface_format(&state);
        let present_mode = p_device.swapchain_details.choose_present_mode(&state);
        let swap_extent = p_device.swapchain_details.choose_swap_extent(&state, window);
        let queue_indices = p_device.queues.queue_indices();
        
        let mut image_count = p_device.swapchain_details.surface_capabilities.min_image_count+1;
        
        if p_device.swapchain_details.surface_capabilities.max_image_count>0 && image_count>p_device.swapchain_details.surface_capabilities.max_image_count {
            image_count = p_device.swapchain_details.surface_capabilities.max_image_count;
        }
        
        
        let mut create_info = ash::vk::SwapchainCreateInfoKHR::builder()
            .surface(surface.surface)
            .min_image_count(image_count)
            .image_format(surface_format.format)
            .image_color_space(surface_format.color_space)
            .image_extent(swap_extent)
            .image_array_layers(1)
            .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
            .pre_transform(p_device.swapchain_details.surface_capabilities.current_transform)
            .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
            .present_mode(present_mode)
            .clipped(true)
            .old_swapchain(vk::SwapchainKHR::null());
        
        
        create_info = if p_device.queues.different_families() {
            create_info.image_sharing_mode(vk::SharingMode::CONCURRENT)
                .queue_family_indices(&queue_indices)
        } else {
            create_info.image_sharing_mode(vk::SharingMode::EXCLUSIVE)
        };
        
        let swapchain_loader = ash::extensions::khr::Swapchain::new(instance, device);
        
        let swapchain = unsafe{swapchain_loader.create_swapchain(&create_info, None)?};
        
        let images = unsafe{swapchain_loader.get_swapchain_images(swapchain)?};
        
        let image_views = Self::create_image_views(&device, &images, &surface_format.format)?;
        
        Ok(Self{
            image_views:image_views,
            images:images,
            swapchain:swapchain,
            swapchain_loader:swapchain_loader,
            extent:swap_extent,
            surface_format:surface_format,
        })
    }
    
    fn create_image_views(device:&Device, images:&Vec<vk::Image>, format:&vk::Format) -> VkResult<Vec<vk::ImageView>> {
        let mut image_views_holder:Vec<vk::ImageView> = Vec::with_capacity(images.len());
        
        let component_create_info = vk::ComponentMapping::builder()
            .r(ash::vk::ComponentSwizzle::IDENTITY)
            .g(ash::vk::ComponentSwizzle::IDENTITY)
            .b(ash::vk::ComponentSwizzle::IDENTITY)
            .a(ash::vk::ComponentSwizzle::IDENTITY);
        
        let subresource_range_create_info = vk::ImageSubresourceRange::builder()
            .aspect_mask(vk::ImageAspectFlags::COLOR)
            .base_mip_level(0)
            .level_count(1)
            .base_array_layer(0)
            .layer_count(1);
        
        let mut create_info = vk::ImageViewCreateInfo::builder()
            .format(*format)
            .view_type(vk::ImageViewType::TYPE_2D)
            .components(*component_create_info)
            .subresource_range(*subresource_range_create_info);
        
        for image in images {
            create_info = create_info.image(*image);
            let holder = unsafe{device.create_image_view(&create_info, None)?};
            image_views_holder.push(holder);
        }
        Ok(image_views_holder)
    }
    
}


impl DeviceDrop for Swapchain {
    fn device_drop(&mut self, state:&State, device:&Device) {
        if state.v_nor() {
            println!("[0]:deleting Swapchain framebuffers");
        }
        for framebuffer in self.framebuffers.iter() {
            unsafe{device.destroy_framebuffer(*framebuffer, None)};
        }
        if state.v_nor() {
            println!("[0]:deleting images");
        }
        for view in self.image_views.iter() {
            unsafe{device.destroy_image_view(*view, None)};
        }
        if state.v_nor() {
            println!("[0]:deleting swapchain");
        }
        unsafe{self.destroy_swapchain(self.swapchain, None)};
        /*
        */
    }
}


impl Deref for SwapchainBasic {
    type Target = ash::extensions::khr::Swapchain;
    fn deref(&self) -> &Self::Target {
        &self.swapchain_loader
    }
}

impl SwapchainSupportDetails {
    
    pub fn query_swapchain_support(surface:&Surface, p_device:vk::PhysicalDevice) -> SwapchainSupportDetails {
        let surface_capabilities = unsafe{surface.get_physical_device_surface_capabilities(p_device, surface.surface).unwrap()};
        let surface_formats = unsafe{surface.get_physical_device_surface_formats(p_device, surface.surface).unwrap()};
        let present_modes = unsafe{surface.get_physical_device_surface_present_modes(p_device, surface.surface).unwrap()};
        SwapchainSupportDetails{
            surface_capabilities,
            surface_formats,
            present_modes
        }
    }
    
    pub fn min_requirements(&self) -> bool {
        !self.surface_formats.is_empty() && !self.present_modes.is_empty()
    }
    
    fn choose_surface_format(&self, state:&State) -> vk::SurfaceFormatKHR {
        if state.v_dmp() {
            println!("{:#?}", &self.surface_formats);
        }
        
        for format in &self.surface_formats {
            if format.format == vk::Format::R8G8B8A8_SRGB && format.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR {
                if state.v_exp() {
                    println!("found target {:#?}", &format);
                }
                return *format;
            }
        }
        if state.v_exp() {
            println!("didn't found target settling for {:#?}", &self.surface_formats[0]);
        }
        self.surface_formats[0]
    }
    
    fn choose_present_mode(&self, state:&State) -> vk::PresentModeKHR {
        if state.v_dmp() {
            println!("{:#?}", &self.present_modes);
        }
        
        for mode in &self.present_modes {
            if mode == &vk::PresentModeKHR::MAILBOX {
                if state.v_exp() {
                    println!("found target Mailbox", );
                }
                return vk::PresentModeKHR::MAILBOX;
            }
            
        }
        if state.v_exp() {
            println!("MAILBOX not available settling for FIFO");
        }
        vk::PresentModeKHR::FIFO
    }
    
    fn choose_swap_extent(&self, state:&State, window:&Window) -> vk::Extent2D {
        if self.surface_capabilities.current_extent.width != u32::MAX {
            if state.v_exp() {
                println!("normal display width:{} height:{}", self.surface_capabilities.current_extent.width, self.surface_capabilities.current_extent.height);
            }
            self.surface_capabilities.current_extent
        } else {
            let (_width, _height) = window.get_pixel_dimensions();
            todo!("high DPI displays not supported!");
        }
    }
}

