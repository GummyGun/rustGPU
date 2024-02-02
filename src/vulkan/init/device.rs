use crate::State;
use crate::macros;
use crate::AAError;
use crate::constants;


use super::logger::device as logger;
use super::VkDestructor;
use super::VkDestructorArguments;
use super::instance::Instance;
use super::p_device::PDevice;
use super::p_device::QueueFamilyIndices;


use std::collections::HashSet;
use std::ffi::CStr;
use std::ffi::c_char;

use ash::vk;

pub struct Device {
    device: ash::Device,
    pub queue_handles: QueueHandles,
}

macros::impl_deref!(Device, ash::Device, device);
macros::impl_underlying!(Device, ash::Device, device);

pub struct QueueHandles {
    pub graphics: vk::Queue,
    pub presentation: vk::Queue,
}


impl Device {
    pub fn create(state:&State, instance:&Instance, p_device:&PDevice) -> Result<Self, AAError> {
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
        
        let av_extensions = Extensions::get(instance, p_device);
        av_extensions.debug_print(state);
        let extensions = av_extensions.handle_logic(state);
        
        
        let mut dynamic_rendering = vk::PhysicalDeviceDynamicRenderingFeatures::builder()
            .dynamic_rendering(true);
        
        let mut synchronization2 = vk::PhysicalDeviceSynchronization2Features::builder()
            .synchronization2(true);
        
        let mut buffer_device_address = vk::PhysicalDeviceBufferDeviceAddressFeatures::builder()
            .buffer_device_address(true)
            .buffer_device_address_capture_replay(true)
            .buffer_device_address_multi_device(true);
        
        
        let mut descriptor_indexing = vk::PhysicalDeviceDescriptorIndexingFeatures::builder();
        
        
        let device_create_info = vk::DeviceCreateInfo::builder()
            .queue_create_infos(&queue_create_info[..])
            .enabled_features(&p_device.features)
            .enabled_extension_names(&extensions)
            .push_next(&mut dynamic_rendering)
            .push_next(&mut synchronization2)
            .push_next(&mut buffer_device_address)
            .push_next(&mut descriptor_indexing);
        
        
        /*
        TODO: Add device layers
        */
        
        let device = unsafe{instance.create_device(p_device.underlying(), &device_create_info, None)}?;
        let queue_handles = Self::get_queue_handles(&device, &p_device.queues);
        
        if state.v_dmp() {
            println!("queue handles fetched");
        }
        
        
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

struct Extensions(Vec<vk::ExtensionProperties>);

impl Extensions {
    fn get(instance:&Instance, p_device:&PDevice) -> Self {
        let holder = unsafe{instance.enumerate_device_extension_properties(p_device.underlying())}.expect("simple vulkan functions should not fail");
        Self(holder)
    }
    
    fn debug_print(&self, state:&State) {
        if state.v_exp() {
            println!("Device Extensions:");
            for layer in &self.0 {
                let name_holder = unsafe{CStr::from_ptr(layer.extension_name.as_ptr())};
                println!("\t{:?}", name_holder);
            }
        }
    }
    
    
    fn validate(&self) -> Result<Vec<*const c_char>, AAError> {
        
        let mut set:HashSet<&'static str> = HashSet::from(constants::DEVICE_EXTENSIONS);//(extensions);
        
        let mut holder = Vec::<*const c_char>::with_capacity(set.len());
        
        for extension in &self.0 {
            let name_holder = unsafe{CStr::from_ptr(extension.extension_name.as_ptr())}.to_string_lossy();
            if set.remove(&name_holder as &str) {
                holder.push(extension.extension_name.as_ptr() as *const c_char);
                println!("{}", name_holder);
            }
        }
        
        
        if set.is_empty() {
            Ok(holder)
        } else {
            Err(AAError::MissingExtensions(set))
        }
        
    }
    
    fn handle_logic(&self, state:&State) -> Vec<*const c_char> {
        match (state.v_exp(), self.validate()) {
            (true, Ok(holder)) => {
                println!("all device extensions found");
                holder
            }
            (false, Ok(holder)) => {holder}
            (_, Err(err)) => {panic!("Extensions required were not available: {:?}", err);}
        }
    }
}


impl VkDestructor for Device {
    fn destruct(self, mut args:VkDestructorArguments) {
        logger::destruct();
        args.unwrap_none();
        unsafe{self.device.destroy_device(None)};
    }
}
