use ash::{
    prelude::VkResult,
    vk,
};
use crate::{
    State,
    Verbosity,
    errors::Error as AAError,
};

use std::{
    ops::Deref,
};

use super::{
    instance::Instance,
};


pub struct PhysicalDevice {
    device: vk::PhysicalDevice,
}

#[derive(Debug, Default)]
pub struct QueueFamilyIndices {
    graphics_family: Option<u32>,
}


impl PhysicalDevice {
    pub fn chose(state:&State, instance:&Instance) -> Result<Self, AAError> {
        if let Verbosity::Expresive = state.verbosity {
            println!("\nCHOSSING:\tDEBUG_MESSENGER\nvalidation layers activated");
        }
        let p_devices = unsafe{instance.enumerate_physical_devices().unwrap()};
        if p_devices.len() == 0 {
            return Err(AAError::NoGPU);
        }
        let mut best = vk::PhysicalDevice::null();
        let mut best_score = 0;
        
        for p_device in p_devices.into_iter() {
            let current_score = Self::rate(state, instance, p_device);
            if current_score >= best_score {
                best_score = current_score;
                best = p_device;
            }
        }
        
        if best != vk::PhysicalDevice::null() {
            Ok(Self{
                device: best,
            })
        } else {
            Err(AAError::NoGPU)
        }
    }
    
    fn rate(state:&State, instance:&Instance, p_device:vk::PhysicalDevice) -> i64 {
        let queues = Self::find_queue_families(state, instance, p_device);
        let mut score:i64 = 0;
        let properties = unsafe{instance.get_physical_device_properties(p_device)};
        let features = unsafe{instance.get_physical_device_features(p_device)};
        
        if let Verbosity::Expresive = state.verbosity {
            println!("{:#?}", properties);
            println!("{:#?}", features);
        }
        
        if properties.device_type == vk::PhysicalDeviceType::DISCRETE_GPU {
            score += 100;
        }
        
        score += i64::from(properties.limits.max_image_dimension2_d);
        
        if features.geometry_shader<=0 || !queues.is_complete() {
            return 0;
        }
        
        return score;
    }
    

    fn find_queue_families(state:&State, instance:&Instance, p_device:vk::PhysicalDevice) -> QueueFamilyIndices {
        let mut holder = QueueFamilyIndices::default();
        let properties = unsafe{instance.get_physical_device_queue_family_properties(p_device)};
        dbg!(&properties);
        
        for (index, queue) in properties.iter().enumerate() {
            if queue.queue_flags.contains(vk::QueueFlags::GRAPHICS) {
                holder.graphics_family = Some(u32::try_from(index).expect("no gpu has that much queues"));
            }
        }
        
        return holder;
    }
}

impl Deref for PhysicalDevice {
    type Target = vk::PhysicalDevice;
    
    fn deref(&self) -> &Self::Target {
        &self.device
    }

}

impl QueueFamilyIndices {
    
    fn is_complete(&self) -> bool {
        self.graphics_family.is_some()
    }
    
}
