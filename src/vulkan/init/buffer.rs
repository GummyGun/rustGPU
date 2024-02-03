use crate::AAError;
use crate::logger;
use crate::errors::messages::GPU_FREE;

use super::VkDestructor;
use super::VkDestructorArguments;
use super::PDevice;
use super::Device;
use super::memory;
use super::Allocator;


use ash::vk;
use gpu_allocator::vulkan as gpu_vk;
use gpu_allocator as gpu_all;

pub struct Buffer{
    buffer: vk::Buffer,
    allocation: gpu_vk::Allocation,
}

impl Buffer{
    pub fn create(
        p_device: &PDevice, 
        device:&mut Device, 
        allocator: &mut Allocator,
        name_arg: Option<&str>,
        size: u64, 
        usage_flags: vk::BufferUsageFlags, 
        location: gpu_all::MemoryLocation,
    ) -> Result<Self, AAError> {
        logger::create!("buffer");
        let create_info = vk::BufferCreateInfo::builder()
            .size(size)
            .usage(usage_flags)
            .sharing_mode(vk::SharingMode::EXCLUSIVE);
        
        let buffer = unsafe{device.create_buffer(&create_info, None)}?;
        
        let memory_requirements = unsafe{device.get_buffer_memory_requirements(buffer)};
        
        let name = match name_arg {
            Some(name) => name,
            None => "",
        };
        
        let allocation = allocator.allocate(name, memory_requirements, location);
        
        unsafe{device.bind_buffer_memory(buffer, allocation.memory(), allocation.offset())}?;
        
        Ok(Self{
            buffer,
            allocation,
        })
    }
    
    pub fn get_slice_mut(&mut self) -> Option<&mut [u8]> {
        self.allocation.mapped_slice_mut()
    }
}


impl VkDestructor for Buffer {
    fn destruct(self, mut args:VkDestructorArguments) {
        logger::destruct!("buffer");
        let (device, allocator) = args.unwrap_dev_all();
        unsafe{device.destroy_buffer(self.buffer, None)};
        allocator.free(self.allocation).expect(GPU_FREE);
    }
}

