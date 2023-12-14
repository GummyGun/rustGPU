use ash::{
    vk,
};

use super::{
    device::Device,
    p_device::PDevice,
    command::CommandControl,
    buffers::Buffer,
    image::Image,
    
};

use crate::{
    State,
    errors::Error as AAError,
};

use std::{
    slice::from_ref,
};

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
    dst_img: &mut Image, 
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
        dst_img.image,
        vk::ImageLayout::TRANSFER_DST_OPTIMAL,
        from_ref(&image_copy)
    )};
    
    command.submit_su_buffer(device);
}


