use ash::{
    vk,
    prelude::VkResult,
};

use super::{
    ActiveDestroy,
    instance::Instance,
    p_device::PDevice,
    p_device::QueueFamilyIndices,
};

use crate::{
    State,
    constants,
};

use std::{
    ops::Deref,
    collections::HashSet,
    ffi::CStr,
};

pub struct Device {
    device: ash::Device,
    pub queue_handles: QueueHandles,
}

pub struct QueueHandles {
    pub graphics: vk::Queue,
    pub presentation: vk::Queue,
}


impl Device {
    pub fn create(state:&State, instance:&Instance, p_device:&PDevice) -> VkResult<Self> {
        if state.v_exp() {
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
        
        let extensions = Extensions::get(instance, p_device);
        extensions.debug_print(state);
        
        let extensions:Vec<_> = constants::DEVICE_EXTENSIONS_CSTR[..].iter().map(|extension|{
            extension.as_ptr()
        }).collect();
        
        if state.v_exp() {
            println!("device extensions available");
        }
        
        //println!("{:#?}", &p_device.features);
        //todo!("{:#?}", &p_device.features);
        
        let mut features = p_device.features.clone();
        features.fill_mode_non_solid = vk::TRUE;
        
        let device_create_info = vk::DeviceCreateInfo::builder()
            .queue_create_infos(&queue_create_info[..])
            .enabled_features(&features)
            .enabled_extension_names(&extensions);
        
        
        /*
        TODO: Add device layers
        */
        
        let device = unsafe{instance.create_device(p_device.p_device, &device_create_info, None)}?;
        let queue_handles = Self::get_queue_handles(&device, &p_device.queues);
        
        if state.v_dmp() {
            println!("queue handles fetched");
        }
        
        
        Ok(Self{
            device: device,
            queue_handles: queue_handles
        })
    }
    
    pub fn populate_features(state:&State, features:&mut vk::PhysicalDeviceFeatures) {        
        
        if state.v_exp() {
            println!("\t[X]:enabling sampleranisotropy");
        }
        features.sampler_anisotropy = vk::TRUE;
        
        if state.v_dmp() {
            println!("{:#?}", features);
        }
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

struct Extensions(Vec<vk::ExtensionProperties>);

impl Extensions {
    fn get(instance:&Instance, p_device:&PDevice) -> Self {
        let holder = unsafe{instance.enumerate_device_extension_properties(p_device.p_device)}.expect("simple vulkan functions should not fail");
        Self(holder)
    }
    
    fn debug_print(&self, state:&State) {
        if state.v_exp() {
            println!("Device Layers:");
            for layer in &self.0 {
                let name_holder = unsafe{CStr::from_ptr(layer.extension_name.as_ptr())};
                println!("\t{:?}", name_holder);
            }
        }
    }
    
    
}


impl Deref for Device {
    type Target = ash::Device;
    
    fn deref(&self) -> &Self::Target {
        &self.device
    }
}

impl ActiveDestroy for Device {
    fn active_drop(&mut self, state:&State) {
        if state.v_nor() {
            println!("[0]:deleting device");
        }
        unsafe{self.device.destroy_device(None)};
    }
}
