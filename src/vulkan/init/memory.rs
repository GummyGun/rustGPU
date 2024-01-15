use crate::AAError;
use crate::errors::messanges::GPU_ALLOCATION;

use super::logger::memory as logger;
use super::VkDestructor;
use super::DestructorArguments;
use super::instance::Instance;
use super::p_device::PDevice;
use super::device::Device;

use std::ops::Deref;
use std::ops::DerefMut;

use ash::vk;
use gpu_allocator::vulkan as vkmem;
use gpu_allocator as gpuall;

pub struct Allocator {
    allocator:std::mem::ManuallyDrop<vkmem::Allocator>,
}

impl Allocator {
    pub fn create(
        instance: &Instance,
        p_device: &PDevice,
        device: &Device,
    ) -> Result<Self, AAError> {
        logger::alloc::create();
        
        let create_info = vkmem::AllocatorCreateDesc {
            instance: instance.underlying(),
            device: device.underlying(),
            physical_device: p_device.underlying(),
            debug_settings: Default::default(),
            buffer_device_address: true,  // Ideally, check the BufferDeviceAddressFeatures struct.
            allocation_sizes: Default::default(),
        };
        
        let allocator = vkmem::Allocator::new(&create_info)?;
        
        Ok(Self{
            allocator: std::mem::ManuallyDrop::new(allocator),
        })
    }
    
    pub fn allocate_gpu_only(
        &mut self,
        name: &str,
        requirements: vk::MemoryRequirements,
    ) -> vkmem::Allocation {
        
        logger::alloc::gpu_allocation(name);
        let alloc_info = vkmem::AllocationCreateDesc{
            name: name,
            requirements: requirements,
            location: gpuall::MemoryLocation::GpuOnly,
            linear: true,
            allocation_scheme: vkmem::AllocationScheme::GpuAllocatorManaged,
        };
        
        self.allocate(&alloc_info).expect(GPU_ALLOCATION)
    }
}

impl Deref for Allocator {
    type Target = vkmem::Allocator;
    fn deref(&self) -> &Self::Target {
        &self.allocator
    }
}

impl DerefMut for Allocator {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.allocator
    }
}


impl VkDestructor for Allocator {
    fn destruct(mut self, mut args:DestructorArguments) {
        let _ = args.unwrap_dev();
        logger::alloc::destruct();
        unsafe{std::mem::ManuallyDrop::drop(&mut self.allocator)};
    }
}


/*
pub fn find_memory_type_index(
    state:&State, 
    p_device:&PDevice, 
    memory_requirements:&vk::MemoryRequirements, 
    flags:vk::MemoryPropertyFlags,
) -> Result<u32, AAError> { 
    
    if state.v_exp() {
        println!("finding memory");
    }
    let memory_prop = &p_device.memory_properties;
    let memory_type_count = usize::try_from(memory_prop.memory_type_count).expect("GPUs doesn't have that much memory types");
    
    if state.v_dmp() {
        println!("{:#?}", memory_prop);
        println!("{:#?}", memory_requirements);
    }
    
    if state.v_dmp() {
        println!("{:#?}", memory_prop);
    }
    memory_prop.memory_types[..memory_type_count]
    .iter() .enumerate()
    .find(|(index, memory_type)| {
        (1 << index) & memory_requirements.memory_type_bits != 0 && memory_type.property_flags & flags == flags
    }).map(|(index, _memory_type)| {
        index as _
    }).ok_or(AAError::NoSuitableMemory)
}

pub fn copy_buffer_2_buffer(
    state: &State, 
    device: &Device, 
    command: &CommandControl,
    src_buf: &Buffer, 
    dst_buf: &mut Buffer, 
    size: vk::DeviceSize,
) {
    if state.v_exp() {
        println!("copying buffer");
    }
    let buffer = command.setup_su_buffer(device);
    
    let buffer_copy = vk::BufferCopy::builder()
        .size(size);
    
    unsafe{device.cmd_copy_buffer(buffer, src_buf.buffer, dst_buf.buffer, from_ref(&buffer_copy))};
    
    command.submit_su_buffer(device);
}

pub fn copy_buffer_2_image(
    state: &State, 
    device: &Device, 
    command: &CommandControl,
    src_buf: &Buffer, 
    dst_img: &mut vk::Image, 
    extent: &vk::Extent3D,
) {
    if state.v_exp() {
        println!("copying buffer");
    }
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

