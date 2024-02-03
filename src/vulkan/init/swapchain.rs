use crate::AAError;
use crate::macros;
use crate::logger;

use crate::window::Window;
use crate::errors::messages::U32_TO_USIZE;
use crate::errors::messages::SIMPLE_VK_FN;
use crate::constants::sc_max_images;

use super::VkDestructor;
use super::VkDestructorArguments;
use super::device::Device;
use super::instance::Instance;
use super::surface::Surface;
use super::p_device::PDevice;
use super::image::Image;

use std::cmp::min;

use ash::vk;
use arrayvec::ArrayVec;

#[derive(Debug, Default)]
pub struct SwapchainSupportDetails {
    pub surface_capabilities: vk::SurfaceCapabilitiesKHR,
    pub surface_formats: Vec<vk::SurfaceFormatKHR>,
    pub present_modes: Vec<vk::PresentModeKHR>,
}


#[derive(Clone)]
pub struct Swapchain {
    pub image_count: usize,
    pub image_views: ArrayVec<vk::ImageView, {sc_max_images::USIZE}>,
    pub images: ArrayVec<vk::Image, {sc_max_images::USIZE}>,
    pub extent: vk::Extent2D,
    pub surface_format: vk::SurfaceFormatKHR,
    pub swapchain: vk::SwapchainKHR,
    swapchain_loader: ash::extensions::khr::Swapchain,
}

macros::impl_deref!(Swapchain, ash::extensions::khr::Swapchain, swapchain_loader);


impl Swapchain {
    
    pub fn create(window:&Window, instance:&mut Instance, surface:&Surface, p_device:&PDevice, device:&mut Device) -> Result<Self, AAError> {
        logger::create!("swapchain");
        
        let surface_format = p_device.swapchain_details.choose_surface_format();
        let present_mode = p_device.swapchain_details.choose_present_mode();
        let swap_extent = p_device.swapchain_details.choose_swap_extent(window);
        let queue_indices = p_device.queues.queue_indices();
        
        let min_img_cnt = p_device.swapchain_details.surface_capabilities.min_image_count;
        let max_img_cnt = p_device.swapchain_details.surface_capabilities.max_image_count;
        
        let max_limit = min(max_img_cnt, sc_max_images::U32);
        
        let image_count = match min_img_cnt {
            0 => {
                max_limit
            }
            limit => {
                if limit+1 < max_limit {
                    limit+1
                } else {
                    max_limit
                }
            }
        };
        
        let image_count_usize = usize::try_from(image_count).expect(U32_TO_USIZE);
        
        
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
        
        
        let images_holder = unsafe{swapchain_loader.get_swapchain_images(swapchain)?};
        let mut images:ArrayVec<vk::Image, {sc_max_images::USIZE}> = ArrayVec::default();
        
        for image in images_holder.into_iter() {
            images.push(image);
        }
        
        let image_views = Self::create_image_views(&device, &images[..], surface_format.format)?;
        
        Ok(Self{
            image_count: image_count_usize,
            image_views:image_views,
            images:images,
            swapchain:swapchain,
            swapchain_loader:swapchain_loader,
            extent:swap_extent,
            surface_format:surface_format,
        })
    }
    
    
    fn create_image_views(device:&Device, images:&[vk::Image], format:vk::Format) -> Result<ArrayVec<vk::ImageView, {sc_max_images::USIZE}>, AAError> {
        let mut image_views_holder:ArrayVec<vk::ImageView, {sc_max_images::USIZE}> = ArrayVec::new();//[vk::ImageView::null(); sc_max_images::USIZE];
        
        for (index, image) in images.iter().enumerate() {
            logger::various_log!("swapchain",
                (logger::Trace, "Creating swapchain image {index}"),
            );
            let holder = Image::create_view(
                device, 
                *image, 
                format, 
                vk::ImageAspectFlags::COLOR
            )?;
            image_views_holder.push(holder);
        }
        
        Ok(image_views_holder)
    }
    
    
    pub fn get_next_image(&mut self, semaphore:vk::Semaphore) -> (vk::Image, vk::ImageView, u32) {
        
        let (image_index, _invalid_surface) = unsafe{
            self.acquire_next_image(
                self.swapchain, 
                u64::MAX, 
                semaphore, 
                vk::Fence::null()
            )
        }.expect("next image should not fail");
        
        (self.images[image_index as usize], self.image_views[image_index as usize], image_index)
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
    
    fn internal_destroy(inself:&mut Self, device:&Device) {//inself
        logger::destruct!("swapchain");
        for view in inself.image_views.iter() {
            unsafe{device.destroy_image_view(*view, None)};
        }
        unsafe{inself.destroy_swapchain(inself.swapchain, None)};
    }
    
}

impl VkDestructor for Swapchain {
    fn destruct(mut self, mut args:VkDestructorArguments) {
        let device = args.unwrap_dev();
        Self::internal_destroy(&mut self, device);
    }
}


/*
impl Swapchain {
    pub fn destroy_callback(&mut self) -> (Box<dyn FnOnce(VkDestructorArguments)>, VkDestructorType) {
        let target = self.clone();
        let callback = Box::new(move |arguments:VkDestructorArguments|{
            let mut target = target;
            if let VkDestructorArguments::Dev(device) = arguments {
                Self::internal_destroy(&mut target, device);
            } else {
                panic!("{}", BAD_DESTRUCTOR);
            }
        });
        (callback, VkDestructorType::Dev)
    }
}
*/



impl SwapchainSupportDetails {
    
    pub fn query_swapchain_support(surface:&Surface, p_device:vk::PhysicalDevice) -> SwapchainSupportDetails {
        
        let surface_capabilities = unsafe{surface.get_physical_device_surface_capabilities(p_device, surface.surface).expect(SIMPLE_VK_FN)};
        let surface_formats = unsafe{surface.get_physical_device_surface_formats(p_device, surface.surface).expect(SIMPLE_VK_FN)};
        let present_modes = unsafe{surface.get_physical_device_surface_present_modes(p_device, surface.surface).expect(SIMPLE_VK_FN)};
        SwapchainSupportDetails{
            surface_capabilities,
            surface_formats,
            present_modes
        }
    }
    
    pub fn min_requirements(&self) -> bool {
        !self.surface_formats.is_empty() && !self.present_modes.is_empty()
    }
    
    fn choose_surface_format(&self) -> vk::SurfaceFormatKHR {
        logger::various_log!("swapchain",
            (logger::Trace, "Surface formats \n{:#?}", &self.surface_formats),
        );
        
        for format in &self.surface_formats {
            match (format.format, format.color_space) {
                (vk::Format::R8G8B8A8_SRGB, vk::ColorSpaceKHR::SRGB_NONLINEAR) => {}
                (_,_) => { 
                    continue;
                }
            }
            let format = *format;
            logger::various_log!("swapchain",
                (logger::Debug, "Found desired format \n{:#?}", format),
            );
            return format;
        }
        
        logger::various_log!("swapchain",
            (logger::Debug, "Desired format not found setting for \n{:#?}", self.surface_formats[0])
        );
        
        self.surface_formats[0]
    }
    
    fn choose_present_mode(&self) -> vk::PresentModeKHR {
        
        logger::various_log!("swapchain",
            (logger::Trace, "Present modes \n{:#?}", &self.present_modes),
        );
        //logger::present_chossing(&self.present_modes);
        
        for mode in &self.present_modes {
            if mode == &vk::PresentModeKHR::MAILBOX {
                logger::various_log!("swapchain",
                    (logger::Debug, "Found desired present mode \n{:#?}", vk::PresentModeKHR::MAILBOX)
                );
                return vk::PresentModeKHR::MAILBOX;
            }
            
        }
        logger::various_log!("swapchain",
            (logger::Debug, "Desired present mode not found setting for \n{:#?}", vk::PresentModeKHR::FIFO)
        );
        vk::PresentModeKHR::FIFO
    }
    
    fn choose_swap_extent(&self, window:&Window) -> vk::Extent2D {
        if self.surface_capabilities.current_extent.width != u32::MAX {
            logger::various_log!("swapchain",
                (logger::Debug, "Normal display width:{} height:{}", self.surface_capabilities.current_extent.width, self.surface_capabilities.current_extent.height)
            );
            
            self.surface_capabilities.current_extent
        } else {
            let (_width, _height) = window.get_pixel_dimensions();
            todo!("high DPI displays not supported yet!");
        }
    }
}

