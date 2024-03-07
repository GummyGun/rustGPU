use crate::AAError;
use crate::macros;
use crate::logger;
use crate::errors::messages::GPU_FREE;

use super::VkDestructor;
use super::VkDestructorType;
use super::VkDeferedDestructor;
use super::VkDynamicDestructor;
use super::VkDestructorArguments;
use super::Device;
use super::Allocator;

use std::mem::align_of;
use std::mem::ManuallyDrop;

use ash::vk;
use gpu_allocator::vulkan as gpu_vk;
use gpu_allocator as gpu_all;

#[derive(Debug)]
pub struct Buffer{
    pub buffer: vk::Buffer,
    pub allocation: ManuallyDrop<gpu_vk::Allocation>,
    test: i32,
}
macros::impl_underlying!(Buffer, vk::Buffer, buffer);

impl Buffer{
    pub fn create(
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
            allocation:ManuallyDrop::new(allocation),
            test:1
        })
    }
    
    /*
    pub fn get_slice_mut(&mut self) -> Option<&mut [u8]> {
        self.allocation.mapped_slice_mut()
    }
    */
    
    pub fn get_align<T>(&mut self, offset:usize, size:u64) -> Option<ash::util::Align<T>> {
        let ptr = self.allocation.mapped_ptr()?;
        let real_ptr = unsafe{ptr.as_ptr().byte_add(offset)};
        Some(unsafe{ash::util::Align::new(real_ptr, align_of::<T>() as u64, size)})
        //let mut index_align:ash::util::Align<T> = unsafe{ash::util::Align::new(real_ptr, align_of::<T>() as u64, size)};
        //Some(index_align)
    }
    
    pub fn get_device_address(&self, device:&Device) -> vk::DeviceAddress {
        let device_address_info = vk::BufferDeviceAddressInfo::builder()
            .buffer(self.buffer);
        unsafe{device.get_buffer_device_address(&device_address_info)}
    }
    
    
    pub unsafe fn unsafe_clone(&mut self) -> Self {
        let mut holder = std::ptr::read(self);
        self.test=3;
        holder.test=4;
        println!("{:#?}", holder);
        println!("{:#?}", self);
        holder
    }
    
    
    fn internal_destroy(mut self, device:&mut Device, allocator:&mut Allocator) {
        logger::destruct!("buffer");
        unsafe{device.destroy_buffer(self.buffer, None)};
        unsafe{allocator.free(ManuallyDrop::into_inner(self.allocation)).expect(GPU_FREE)};
    }
}


impl VkDestructor for Buffer {
    fn destruct(self, mut args:VkDestructorArguments) {
        let (device, allocator) = args.unwrap_dev_all();
        self.internal_destroy(device, allocator);
    }
}


impl VkDeferedDestructor for Buffer {
    fn defered_destruct(&mut self) -> VkDynamicDestructor {
        let target = unsafe{self.unsafe_clone()};
        let callback = Box::new(move |mut args:VkDestructorArguments|{
            let target:Self = target;
            let (device, allocator) = args.unwrap_dev_all();
            target.internal_destroy(device, allocator);
        });
        (callback, VkDestructorType::DevAll)
    }
}

