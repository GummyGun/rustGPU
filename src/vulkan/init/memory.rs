use crate::AAError;
use crate::macros;
use crate::logger;
use crate::errors::messages::GPU_ALLOCATION;

use super::VkDestructor;
use super::VkDestructorArguments;
use super::Instance;
use super::PDevice;
use super::Device;
use super::CommandControl;
use super::Buffer;

use std::mem::ManuallyDrop;
use std::slice::from_ref;

use ash::vk;
use gpu_allocator::vulkan as gpu_vk;
use gpu_allocator as gpu_all;

pub use gpu_all::MemoryLocation::*;
pub struct Allocator {
    allocator:ManuallyDrop<gpu_vk::Allocator>,
}

macros::impl_deref_mut!(Allocator, gpu_vk::Allocator, allocator);



impl Allocator {
    pub fn create(
        instance: &mut Instance,
        p_device: &PDevice,
        device: &mut Device,
    ) -> Result<Self, AAError> {
        logger::create!("allocator");
        
        let create_info = gpu_vk::AllocatorCreateDesc {
            instance: instance.underlying(),
            device: device.underlying(),
            physical_device: p_device.underlying(),
            debug_settings: Default::default(),
            buffer_device_address: true,  // Ideally, check the BufferDeviceAddressFeatures struct.
            allocation_sizes: Default::default(),
        };
        
        let allocator = gpu_vk::Allocator::new(&create_info)?;
        
        Ok(Self{
            allocator: ManuallyDrop::new(allocator),
        })
    }
    
    pub fn allocate(
        &mut self,
        name: &str,
        requirements: vk::MemoryRequirements,
        location: gpu_all::MemoryLocation,
    ) -> gpu_vk::Allocation {
        
        logger::various_log!("allocator", 
            (logger::Trace, "Allocation name: {:?}", name),
        );
        
        let alloc_info = gpu_vk::AllocationCreateDesc{
            name: name,
            requirements: requirements,
            location: location,
            linear: true,
            allocation_scheme: gpu_vk::AllocationScheme::GpuAllocatorManaged,
        };
        
        self.allocator.allocate(&alloc_info).expect(GPU_ALLOCATION)
    }
    
    pub fn into_inner(self) -> gpu_vk::Allocator {
        ManuallyDrop::into_inner(self.allocator)
    }
}


impl VkDestructor for Allocator {
    fn destruct(mut self, mut args:VkDestructorArguments) {
        args.unwrap_dev();
        logger::destruct!("allocator");
        unsafe{ManuallyDrop::drop(&mut self.allocator)};
    }
}


pub fn copy_buffer_2_buffer(
    device: &Device, 
    cmd: &CommandControl,
    src_buf: &Buffer, 
    src_offset: u64,
    dst_buf: &mut Buffer, 
    dst_offset: u64,
    size: vk::DeviceSize,
) {
    
    logger::various_log!("memory",
        (logger::Trace, "copying from 1 buffer to another")
    );
    let buffer = cmd.setup_su_buffer(device);
    
    let buffer_copy = vk::BufferCopy::builder()
        .size(size)
        .src_offset(src_offset)
        .dst_offset(dst_offset);
    
    unsafe{device.cmd_copy_buffer(buffer, src_buf.buffer, dst_buf.buffer, from_ref(&buffer_copy))};
    cmd.submit_su_buffer(device);
}

/*
pub fn find_memory_type_index(
    p_device:&PDevice, 
    memory_requirements:&vk::MemoryRequirements, 
    flags:vk::MemoryPropertyFlags,
) -> Result<u32, AAError> { 
    
    /*
    if state.v_exp() {
        println!("finding memory");
    }
    */
    let memory_prop = &p_device.memory_properties;
    let memory_type_count = usize::try_from(memory_prop.memory_type_count).expect("GPUs doesn't have that much memory types");
    
    /*
    if state.v_dmp() {
        println!("{:#?}", memory_prop);
        println!("{:#?}", memory_requirements);
    }
    
    if state.v_dmp() {
        println!("{:#?}", memory_prop);
    }
    */
    memory_prop.memory_types[..memory_type_count]
    .iter() .enumerate()
    .find(|(index, memory_type)| {
        (1 << index) & memory_requirements.memory_type_bits != 0 && memory_type.property_flags & flags == flags
    }).map(|(index, _memory_type)| {
        index as _
    }).ok_or(AAError::NoSuitableMemory)
}
*/

/*

pub fn copy_buffer_2_image(
    device: &Device, 
    command: &CommandControl,
    src_buf: &Buffer, 
    dst_img: &mut vk::Image, 
    extent: &vk::Extent3D,
) {
    /*
    if state.v_exp() {
        println!("copying buffer");
    }
    */
    let buffer = command.setup_su_buffer(device);
    
    let subresource = vk::ImageSubresourceLayers::builder()
        .aspect_mask(vk::ImageAspectFlags::COLOR)
        .mip_level(0)
        .base_array_layer(0)
        .layer_count(1);
    
    let image_copy = vk::BufferImageCopy::builder()
        .buffer_offset(0)
        .buffer_row_length(0)
        .buffer_image_height(0)
        .image_subresource(*subresource)
        .image_offset(vk::Offset3D::default())
        .image_extent(*extent);
    
    unsafe{device.cmd_copy_buffer_to_image(
        buffer,
        src_buf.buffer,
        *dst_img,
        vk::ImageLayout::TRANSFER_DST_OPTIMAL,
        from_ref(&image_copy)
    )};
    
    command.submit_su_buffer(device);
}

*/
