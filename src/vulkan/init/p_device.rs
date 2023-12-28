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
    device::Device,
};

use std::{
    collections::HashSet,
    ffi::CStr,
};

pub struct PDevice {
    pub p_device: vk::PhysicalDevice,
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
        let mut best_score = 0;
        
        
        for p_device in p_devices.into_iter() {
            
            if let Ok((current_score, current_queue, sc_support_details, properties)) = Self::rate(state, instance, surface, p_device) {
                if current_score > best_score {
                    best_score = current_score;
                    best_queue = current_queue;
                    best_sc_details = sc_support_details;
                    best_properties = properties;
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
            
            let mut features = vk::PhysicalDeviceFeatures::default();
            Device::populate_features(state, &mut features);
            
            Ok(Self{
                p_device: best,
                queues: queue,
                features: features,
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
    ) -> Result<(i64, QueueFamilyOptionalIndices, SwapchainSupportDetails, vk::PhysicalDeviceProperties), ()> {
        
        let queues = Self::find_queue_families(state, instance, surface, p_device);
        if !queues.complete() && !Self::check_device_support(instance, p_device) {
            return Err(());
        }
        let swapchain_support = SwapchainSupportDetails::query_swapchain_support(surface, p_device);
        if !swapchain_support.min_requirements() {
            return Err(());
        }
        let features = unsafe{instance.get_physical_device_features(p_device)};
        if features.sampler_anisotropy != vk::TRUE {
            panic!("si");
        }
        
        
        let mut score:i64 = 0;
        let properties = unsafe{instance.get_physical_device_properties(p_device)};
        
        if state.v_dmp() {
            println!("{:#?}", properties);
            println!("{:#?}", features);
        }
        
        if properties.device_type == vk::PhysicalDeviceType::DISCRETE_GPU {
            score += 100;
        }
        
        score += i64::from(properties.limits.max_image_dimension2_d);
        
        if features.geometry_shader<=0 {
            Err(()) 
        } else {
            Ok((score, queues, swapchain_support, properties))
        }
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


