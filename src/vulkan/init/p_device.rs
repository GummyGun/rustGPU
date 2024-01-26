use ash::{
    vk,
};

use crate::{
    State,
    constants,
    errors::Error as AAError,
};

use super::{
    instance::Instance,
    surface::Surface,
    swapchain::SwapchainSupportDetails,
};

use std::{
    collections::HashSet,
    ffi::CStr,
};

pub struct PDevice {
    p_device: vk::PhysicalDevice,
    pub queues: QueueFamilyIndices,
    pub features: vk::PhysicalDeviceFeatures,
    pub swapchain_details: SwapchainSupportDetails,
    pub memory_properties: vk::PhysicalDeviceMemoryProperties,
    pub properties: vk::PhysicalDeviceProperties,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct QueueFamilyOptionalIndices {
    pub graphics_family: Option<u32>,
    pub present_family: Option<u32>,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct QueueFamilyIndices {
    pub graphics_family: u32,
    pub present_family: u32,
}

impl PDevice {
    
    pub fn chose(state:&State, instance:&Instance, surface:&Surface) -> Result<Self, AAError> {
        if state.v_exp() {
            println!("\nCHOSSING:\tPHYSICAL DEVICE");
        }
        let p_devices = unsafe{instance.enumerate_physical_devices().unwrap()};
        if p_devices.len() == 0 {
            return Err(AAError::NoGPU);
        }
        let mut best = vk::PhysicalDevice::null();
        let mut best_queue = QueueFamilyOptionalIndices::default();
        let mut best_sc_details = SwapchainSupportDetails::default();
        let mut best_properties = vk::PhysicalDeviceProperties::default();
        let mut best_features = vk::PhysicalDeviceFeatures::default();
        let mut best_score = 0;
        
        
        for p_device in p_devices.into_iter() {
            
            if let Ok((current_score, current_queue, sc_support_details, properties, features)) = Self::rate(state, instance, surface, p_device) {
                if current_score > best_score {
                    best_score = current_score;
                    best_queue = current_queue;
                    best_sc_details = sc_support_details;
                    best_properties = properties;
                    best_features = features;
                    best = p_device;
                }
            }
        }
        
        
        if best != vk::PhysicalDevice::null() {
            if state.v_exp() {
                println!("physical device succesfully selected");
            }
            
            let queue = QueueFamilyIndices::from(best_queue);
            assert!(!queue.different_families(), "queues should be the same");
            
            let memory_properties = unsafe{instance.get_physical_device_memory_properties(best)};
            
            if state.v_exp() {
                println!("getting memory properties");
            }
            
            if state.v_dmp() {
                println!("{:?}", &memory_properties);
            }
            
            Ok(Self{
                p_device: best,
                queues: queue,
                features: best_features,
                swapchain_details: best_sc_details,
                memory_properties: memory_properties,
                properties: best_properties,
            })
        } else {
            Err(AAError::NoGPU)
        }
    }
    
    fn rate(state:&State, 
        instance:&Instance, 
        surface:&Surface, 
        p_device:vk::PhysicalDevice
    ) -> Result<(i64, QueueFamilyOptionalIndices, SwapchainSupportDetails, vk::PhysicalDeviceProperties, vk::PhysicalDeviceFeatures), ()> {
        
        let queues = Self::find_queue_families(state, instance, surface, p_device);
        if !queues.complete() && !Self::check_device_support(instance, p_device) {
            return Err(());
        }
        let swapchain_support = SwapchainSupportDetails::query_swapchain_support(surface, p_device);
        if !swapchain_support.min_requirements() {
            return Err(());
        }
        
        
        let mut vulkan11_features = vk::PhysicalDeviceVulkan11Features::default();
        let mut vulkan12_features = vk::PhysicalDeviceVulkan12Features::default();
        let mut vulkan13_features = vk::PhysicalDeviceVulkan13Features::default();
        
        let mut available_features = vk::PhysicalDeviceFeatures2::builder()
            .push_next(&mut vulkan11_features)
            .push_next(&mut vulkan12_features)
            .push_next(&mut vulkan13_features);
        
        
        unsafe{instance.get_physical_device_features2(p_device, &mut available_features)};
        
        let available_features = available_features.features;
        let enabled_features = Self::check_features(&available_features, &vulkan11_features, &vulkan12_features, &vulkan13_features)?;
        
        /*
        panic!("{:#?}", &vulkan11_features);
        panic!("{:#?}", &vulkan12_features);
        panic!("{:#?}", &vulkan13_features);
        */
        
        let mut score:i64 = 0;
        let properties = unsafe{instance.get_physical_device_properties(p_device)};
        
        if state.v_dmp() {
            println!("{:#?}", properties);
            println!("{:#?}", available_features);
            println!("{:#?}", enabled_features);
        }
        
        if properties.device_type == vk::PhysicalDeviceType::DISCRETE_GPU {
            score += 100;
        }
        score += i64::from(properties.limits.max_image_dimension2_d);
        
        Ok((score, queues, swapchain_support, properties, enabled_features))
    }
    

    fn find_queue_families(
        state:&State, 
        instance:&Instance, 
        surface:&Surface, 
        p_device:vk::PhysicalDevice
    ) -> QueueFamilyOptionalIndices {
        let mut holder = QueueFamilyOptionalIndices::default();
        let properties = unsafe{instance.get_physical_device_queue_family_properties(p_device)};
        
        if state.v_dmp() {
            println!("{:#?}", &properties);
        }
        
        for (index, queue) in properties.iter().enumerate() {
            let index_u32 = u32::try_from(index).expect("no gpu has that much queues");
            
            let present_suport = unsafe{surface.get_physical_device_surface_support(p_device, index_u32, surface.surface).unwrap()};
            
            match (present_suport, holder.present_family) {
                (true, None) => {
                    holder.present_family = Some(index_u32);
                }
                (_, _) => {}
            }
            
            if queue.queue_flags.contains(vk::QueueFlags::GRAPHICS) {
                holder.graphics_family = Some(index_u32);
            }
        }
        
        return holder;
    }
    
    
    
    fn check_device_support(instance:&Instance, p_device:vk::PhysicalDevice) -> bool {
        let device_extensions = unsafe{instance.enumerate_device_extension_properties(p_device)}.unwrap();
        
        let mut set = HashSet::from(constants::DEVICE_EXTENSIONS.clone());
        
        for extension in device_extensions {
            let name_holder = unsafe{CStr::from_ptr(extension.extension_name.as_ptr()).to_string_lossy()};
            set.remove(&name_holder as &str);
        }
        
        set.is_empty()
    }
    
    
    #[allow(unused_variables)]
    pub fn check_features(
        features:&vk::PhysicalDeviceFeatures,
        vk_features11:&vk::PhysicalDeviceVulkan11Features,
        vk_features12:&vk::PhysicalDeviceVulkan12Features,
        vk_features13:&vk::PhysicalDeviceVulkan13Features,
    ) -> Result<vk::PhysicalDeviceFeatures, ()> {
        if features.geometry_shader == vk::TRUE && 
            features.fill_mode_non_solid == vk::TRUE &&
            vk_features12.buffer_device_address == vk::TRUE && 
            vk_features12.descriptor_indexing == vk::TRUE &&
            vk_features13.dynamic_rendering == vk::TRUE && 
            vk_features13.synchronization2 == vk::TRUE {
            
            let holder = vk::PhysicalDeviceFeatures::builder()
                .sampler_anisotropy(true)
                .fill_mode_non_solid(true)
                .build();
            Ok(holder)
        } else {
            Err(())
        }
    }
    
    pub fn underlying(&self) -> vk::PhysicalDevice {
        self.p_device
    }
}

impl QueueFamilyOptionalIndices {
    fn complete(&self) -> bool {
        self.graphics_family.is_some() && self.present_family.is_some()
    }
    
}

impl QueueFamilyIndices {
    pub fn different_families(&self) -> bool {
        self.graphics_family != self.present_family
    }
    
    pub fn queue_indices(&self) -> [u32;2] {
        [self.graphics_family, self.present_family]
    }
    
}

impl From<QueueFamilyOptionalIndices> for QueueFamilyIndices {
    fn from(base:QueueFamilyOptionalIndices) -> Self {
        Self{
            graphics_family:base.graphics_family.unwrap(),
            present_family:base.present_family.unwrap(),
        }
    }
}
