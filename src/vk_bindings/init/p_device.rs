use ash::{
    vk,
};
use crate::{
    State,
    constants,
    errors::Error as AAError,
};

use std::{
    ops::Deref,
    collections::HashSet,
    ffi::CStr,
};

use super::{
    instance::Instance,
    surface::Surface,
};


pub struct PhysicalDevice {
    device: vk::PhysicalDevice,
    pub queues: QueueFamilyIndices,
    pub features: vk::PhysicalDeviceFeatures,
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


impl PhysicalDevice {
    pub fn chose(state:&State, instance:&Instance, surface:&Surface) -> Result<Self, AAError> {
        if state.v_exp() {
            println!("\nCHOSSING:\tDEBUG_MESSENGER\nvalidation layers activated");
        }
        let p_devices = unsafe{instance.enumerate_physical_devices().unwrap()};
        if p_devices.len() == 0 {
            return Err(AAError::NoGPU);
        }
        let mut best = vk::PhysicalDevice::null();
        let mut best_queue = QueueFamilyOptionalIndices::default();
        let mut best_score = 0;
        
        
        for p_device in p_devices.into_iter() {
            
            let (current_score, current_queue) = Self::rate(state, instance, surface, p_device);
            if current_score > best_score {
                best_score = current_score;
                best_queue = current_queue;
                best = p_device;
            }
        }
        
        if best != vk::PhysicalDevice::null() {
            Ok(Self{
                device: best,
                queues: best_queue.into(),
                features: vk::PhysicalDeviceFeatures::default(),
            })
        } else {
            Err(AAError::NoGPU)
        }
    }
    
    fn rate(state:&State, instance:&Instance, surface:&Surface, p_device:vk::PhysicalDevice) -> (i64, QueueFamilyOptionalIndices) {
        let queues = Self::find_queue_families(state, instance, surface, p_device);
        let device_support = Self::check_device_support(state, instance, p_device);
        let mut score:i64 = 0;
        let properties = unsafe{instance.get_physical_device_properties(p_device)};
        let features = unsafe{instance.get_physical_device_features(p_device)};
        
        if state.v_dmp() {
            println!("{:#?}", properties);
            println!("{:#?}", features);
        }
        
        if properties.device_type == vk::PhysicalDeviceType::DISCRETE_GPU {
            score += 100;
        }
        
        score += i64::from(properties.limits.max_image_dimension2_d);
        
        if features.geometry_shader<=0 || !queues.complete() {
            return (0, QueueFamilyOptionalIndices::default());
        }
        
        return (score, queues);
    }
    

    fn find_queue_families(state:&State, instance:&Instance, surface:&Surface, p_device:vk::PhysicalDevice) -> QueueFamilyOptionalIndices {
        let mut holder = QueueFamilyOptionalIndices::default();
        let properties = unsafe{instance.get_physical_device_queue_family_properties(p_device)};
        
        if state.v_dmp() {
            println!("{:?}", &properties);
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
    
    fn check_device_support(state:&State, instance:&Instance, p_device:vk::PhysicalDevice) -> bool {
        let device_extensions = unsafe{instance.enumerate_device_extension_properties(p_device)}.unwrap();
        
        let mut set = HashSet::from(constants::DEVICE_EXTENSIONS.clone());
        
        for extension in device_extensions {
            let name_holder = unsafe{CStr::from_ptr(extension.extension_name.as_ptr()).to_string_lossy()};
            set.remove(&name_holder as &str);
        }
        
        set.is_empty()
    }
}

impl Deref for PhysicalDevice {
    type Target = vk::PhysicalDevice;
    
    fn deref(&self) -> &Self::Target {
        &self.device
    }

}

impl QueueFamilyOptionalIndices {
    fn complete(&self) -> bool {
        self.graphics_family.is_some() && self.present_family.is_some()
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


