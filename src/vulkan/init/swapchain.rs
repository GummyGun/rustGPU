use ash::{
    vk,
    prelude::VkResult,
};

use super::{
    DeviceDestroy,
    device::Device,
    instance::Instance,
    surface::Surface,
    p_device::PDevice,
    render_pass::RenderPass,
    image::Image,
    depth_buffer::DepthBuffer,
};

use crate::{
    State,
    window::{
        Window
    },
};

use std::{
    ops::Deref,
    slice::from_ref,
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
    pub image_views: Vec<vk::ImageView>,
    pub images: Vec<vk::Image>,
    pub extent: vk::Extent2D,
    pub surface_format: vk::SurfaceFormatKHR,
    
    pub swapchain: vk::SwapchainKHR,
    swapchain_loader: ash::extensions::khr::Swapchain,
    pub framebuffers: Vec<vk::Framebuffer>
}

impl Deref for Swapchain {
    type Target = ash::extensions::khr::Swapchain;
    fn deref(&self) -> &Self::Target {
        &self.swapchain_loader
    }
    
}

impl Swapchain {
    
    pub fn create(state:&State, window:&Window, instance:&Instance, surface:&Surface, p_device:&PDevice, device:&Device) -> VkResult<SwapchainBasic> {
        if state.v_exp() {
            println!("\nCREATING:\tSWAPCHAIN");
        }
        
        //println!("{:?}", p_device.swapchain_details.surface_formats);
        let surface_format = p_device.swapchain_details.choose_surface_format(&state);
        /*
        println!("{:?}", surface_format);
        */
        let present_mode = p_device.swapchain_details.choose_present_mode(&state);
        let swap_extent = p_device.swapchain_details.choose_swap_extent(&state, window);
        let queue_indices = p_device.queues.queue_indices();
        
        let mut image_count = p_device.swapchain_details.surface_capabilities.min_image_count+1;
        
        if p_device.swapchain_details.surface_capabilities.max_image_count>0 && image_count>p_device.swapchain_details.surface_capabilities.max_image_count {
            image_count = p_device.swapchain_details.surface_capabilities.max_image_count;
        }
        
        
        let mut create_info = vk::SwapchainCreateInfoKHR::builder()
            .surface(surface.surface)
            .min_image_count(image_count)
            .image_format(surface_format.format)
            .image_color_space(surface_format.color_space)
            .image_extent(swap_extent)
            .image_array_layers(1)
            .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT | vk::ImageUsageFlags::TRANSFER_DST)
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
        
        let image_views = SwapchainBasic::create_image_views(state, &device, &images, surface_format.format)?;
        
        Ok(SwapchainBasic{
            image_views:image_views,
            images:images,
            swapchain:swapchain,
            swapchain_loader:swapchain_loader,
            extent:swap_extent,
            surface_format:surface_format,
            
        })
    }
    
    pub fn complete(
        state:&State, 
        device:&Device, 
        swapchain:SwapchainBasic, 
        depth:&DepthBuffer,
        render_pass:&RenderPass, 
    ) -> VkResult<Self> {
        if state.v_exp() {
            println!("\nCREATING:\tFRAME BUFFER");
        }
        
        let mut framebuffers_holder:Vec<vk::Framebuffer> = Vec::with_capacity(swapchain.image_views.len());
        
        for image_view in &swapchain.image_views {
            
            let attachments = [
                *image_view,
                depth.image.view,
            ];
            
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
            image_views: swapchain.image_views,
            images: swapchain.images,
            extent: swapchain.extent,
            surface_format: swapchain.surface_format,
            swapchain: swapchain.swapchain,
            swapchain_loader: swapchain.swapchain_loader,
            framebuffers: framebuffers_holder,
        })
        
    }
    
    pub fn transition_sc_image(
        &self,
        state: &State,
        device: &Device,
        image: vk::Image,
        command_buffer: vk::CommandBuffer,
        old_layout: vk::ImageLayout,
        new_layout: vk::ImageLayout,
    ) {
        if state.v_dmp() {
            println!("transitioning swapchain image");
        }
        
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
    
    
    /*
    #[allow(dead_code)]
    pub fn direct_create( //TODO: this function shouldn't be linted as unused
        state:&State, 
        window:&Window, 
        instance:&Instance, 
        surface:&Surface,
        p_device:&PDevice, 
        device:&Device, 
        render_pass:&RenderPass,
        depth:&DepthBuffer,
    ) -> VkResult<Self> {
        let holder = Self::create(state, window, instance, surface, p_device, device)?;
        Self::complete(state, device, holder, depth, render_pass)
    }
    */
    
    
}


impl SwapchainBasic {
    
    fn create_image_views(state:&State, device:&Device, images:&Vec<vk::Image>, format:vk::Format) -> VkResult<Vec<vk::ImageView>> {
        
        let mut image_views_holder:Vec<vk::ImageView> = Vec::with_capacity(images.len());
        
        for (index, image) in images.iter().enumerate() {
            if state.v_exp() {
                println!("creating swapchain image {index}");
            }
            let holder = Image::create_image_view(
                state, 
                device, 
                image, 
                format, 
                vk::ImageAspectFlags::COLOR
            )?;
            image_views_holder.push(holder);
        }
        
        Ok(image_views_holder)
    }
    
}


impl DeviceDestroy for Swapchain {
    fn device_drop(&mut self, state:&State, device:&Device) {
        if state.v_nor() {
            println!("[0]:deleting swapchain framebuffers");
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
            match (format.format, format.color_space) {
                (vk::Format::R8G8B8A8_SRGB, vk::ColorSpaceKHR::SRGB_NONLINEAR) => {}
                (vk::Format::B8G8R8A8_SRGB, vk::ColorSpaceKHR::SRGB_NONLINEAR) => {}
                (_,_) => { 
                    continue;
                }
            }
            if state.v_exp() {
                println!("found target {:#?}", &format);
            }
            return *format;
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

