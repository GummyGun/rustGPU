use ash::{
    vk,
    prelude::VkResult,
};
use crate::{
    State,
    //errors::Error as AAError,
};

use std::{
    ops::Deref,
    collections::HashSet,
};

use super::{
    instance::Instance,
    p_device::PhysicalDevice,
    p_device::QueueFamilyIndices,
};



pub struct Device {
    device: ash::Device,
    queue_handles: QueueHandles,
}

pub struct QueueHandles {
    pub graphics: vk::Queue,
    pub presentation: vk::Queue,
}


impl Device {
    pub fn create(state:&State, instance:&Instance, p_device:&PhysicalDevice) -> VkResult<Self> {
        if  state.v_exp() {
            println!("\nCREATING:\tDEVICE");
        }
        
        let mut queue_create_info:Vec<vk::DeviceQueueCreateInfo> = Vec::new();
        let priority_arr = [1.0];
        
        let mut queue_set:HashSet<u32> = HashSet::new();
        queue_set.insert(p_device.queues.graphics_family);
        queue_set.insert(p_device.queues.present_family);
        
        for elem in queue_set {
            let holder = vk::DeviceQueueCreateInfo::builder()
                .queue_family_index(elem)
                .queue_priorities(&priority_arr);
            queue_create_info.push(*holder);
            
        }
        
        /*
        holder.queue_family_index = p_device.queues.graphics_family.unwrap();
        holder.queue_count = 1;
        holder.p_queue_priorities = priority_ptr;
        */
        
        let device_create_info = vk::DeviceCreateInfo::builder()
            .queue_create_infos(&queue_create_info[..])
            .enabled_features(&p_device.features);
        
        /*
        TODO: Add device layers
        */
        
        let device = unsafe{instance.create_device(**p_device, &device_create_info, None)?};
        let queue_handles = Self::get_queue_handles(&device, &p_device.queues);
        
        Ok(Self{
            device: device,
            queue_handles: queue_handles
        })
    }
    
    
    fn get_queue_handles(device:&ash::Device, queue_indices:&QueueFamilyIndices) -> QueueHandles {
        let graphics = unsafe{device.get_device_queue(0, queue_indices.graphics_family)};
        let presentation = unsafe{device.get_device_queue(0, queue_indices.present_family)};
        QueueHandles{
            graphics: graphics,
            presentation: presentation,
        }
    }
    
}

impl Deref for Device {
    type Target = ash::Device;
    
    fn deref(&self) -> &Self::Target {
        &self.device
    }
}

impl Drop for Device {
    fn drop(&mut self) {
        unsafe{self.device.destroy_device(None)};
    }
}

