use ash::{
    vk, 
    prelude::VkResult,
};

use super::{
    DeviceDrop,
    device::Device,
    p_device::PhysicalDevice,
    command::CommandControl,
};

use crate::{
    State,
    graphics::Vertex,
    graphics::VERTEX_ARR,
    graphics::VERTEX_INDEX,
    errors::Error as AAError,
};

use std::{
    mem::{
        size_of,
        align_of,
    },
    ptr::addr_of,
    slice::from_raw_parts,
};

pub struct Buffer {
    pub buffer: vk::Buffer,
    pub memory: vk::DeviceMemory,
}

impl Buffer {
    pub fn create_vertex(
        state:&State, 
        p_device:&PhysicalDevice, 
        device:&Device, 
        command:&CommandControl,
    ) -> VkResult<Self> {
        use vk::BufferUsageFlags as BUF;
        use vk::MemoryPropertyFlags as MPF;
        
        if  state.v_exp() {
            println!("\nCREATING:\tVERTEX BUFFER");
        }
        let raw_size = u64::try_from(size_of::<Vertex>()*VERTEX_ARR.len()).expect("vertex buffer size should fit in u64");
        
        let staging_memory_flags = MPF::HOST_VISIBLE | MPF::HOST_COHERENT;
        
        let (mut staging, staging_size) = Self::create_buffer(state, p_device, device, raw_size, BUF::TRANSFER_SRC, staging_memory_flags)?;
        
        let memory_ptr = unsafe{device.map_memory(staging.memory, 0, staging_size, vk::MemoryMapFlags::empty())}?;
        let mut vert_align = unsafe{ash::util::Align::new(memory_ptr, align_of::<Vertex>() as u64, staging_size)};
        vert_align.copy_from_slice(&VERTEX_ARR);
        
        unsafe{device.unmap_memory(staging.memory)};
        
        let vertex_buffer_usage = BUF::VERTEX_BUFFER | BUF::TRANSFER_DST;
        let vertex_memory_flags = MPF::HOST_VISIBLE | MPF::HOST_COHERENT;
        
        let (holder, _) = Self::create_buffer(state, p_device, device, raw_size, vertex_buffer_usage, vertex_memory_flags)?;
        
        Self::copy_buffer(state, device, command.staging_buffer, &staging, &holder, staging_size);
        
        if state.v_exp() {
            println!("deleting stage buffer");
        }
        staging.silent_drop(device);
        Ok(holder)
    }
    
    pub fn create_index(
        state:&State, 
        p_device:&PhysicalDevice, 
        device:&Device, 
        command:&CommandControl,
    ) -> VkResult<Self> {
        use vk::BufferUsageFlags as BUF;
        use vk::MemoryPropertyFlags as MPF;
        
        if  state.v_exp() {
            println!("\nCREATING:\tINDEX BUFFER");
        }
        let raw_size = u64::try_from(size_of::<u16>()*VERTEX_INDEX .len()).expect("index buffer size should fit in u64");
        
        let staging_memory_flags = MPF::HOST_VISIBLE | MPF::HOST_COHERENT;
        
        let (mut staging, staging_size) = Self::create_buffer(state, p_device, device, raw_size, BUF::TRANSFER_SRC, staging_memory_flags)?;
        
        let memory_ptr = unsafe{device.map_memory(staging.memory, 0, staging_size, vk::MemoryMapFlags::empty())}?;
        
        let mut vert_align = unsafe{ash::util::Align::new(memory_ptr, align_of::<u16>() as u64, staging_size)};
        
        vert_align.copy_from_slice(&VERTEX_INDEX);
        /*
        let tmp = unsafe{from_raw_parts(memory_ptr as *const u16, VERTEX_INDEX .len())};
        println!("hola {:#?}", tmp);
        */
        unsafe{device.unmap_memory(staging.memory)};
        
        let vertex_buffer_usage = BUF::INDEX_BUFFER | BUF::TRANSFER_DST;
        let vertex_memory_flags = MPF::HOST_VISIBLE | MPF::HOST_COHERENT;
        
        
        let (holder, _) = Self::create_buffer(state, p_device, device, raw_size, vertex_buffer_usage, vertex_memory_flags)?;
        
        Self::copy_buffer(state, device, command.staging_buffer, &staging, &holder, staging_size);
        
        if state.v_exp() {
            println!("deleting stage buffer");
        }
        staging.silent_drop(device);
        Ok(holder)
        
    }
    
    fn create_buffer(
        state: &State, 
        p_device: &PhysicalDevice, 
        device: &Device, 
        size: u64, 
        usage: vk::BufferUsageFlags, 
        memory_flags: vk::MemoryPropertyFlags,
    ) -> VkResult<(Self, u64)> {
        
        let create_info = vk::BufferCreateInfo::builder()
            .size(size)
            .usage(usage)
            .sharing_mode(vk::SharingMode::EXCLUSIVE);
        
        let buffer = unsafe{device.create_buffer(&create_info, None)}?;
        
        let memory_requirements = unsafe{device.get_buffer_memory_requirements(buffer)};
        
        let index_holder = Self::find_memory_type_index(state, p_device, &memory_requirements, memory_flags).expect("required memory type is not present");
        
        let allocate_info = ash::vk::MemoryAllocateInfo::builder()
            .allocation_size(memory_requirements.size)
            .memory_type_index(index_holder);
        
        let memory = unsafe{device.allocate_memory(&allocate_info, None)}?;
        
        unsafe{device.bind_buffer_memory(buffer, memory, 0)}?;
        
        Ok((Self{buffer:buffer, memory:memory}, memory_requirements.size))
    }
    
    pub fn copy_buffer(
        state:&State, 
        device:&Device, 
        buffer:vk::CommandBuffer, 
        src_buff:&Buffer, 
        dst_buff:&Buffer, 
        size:vk::DeviceSize,
    ) {
        if state.v_exp() {
            println!("copying buffer");
        }
        
        unsafe{device.reset_command_buffer(buffer, vk::CommandBufferResetFlags::empty())}.expect("reseting buffer should not fail");
        
        let begin_info = ash::vk::CommandBufferBeginInfo::builder()
            .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);
        
        unsafe{device.begin_command_buffer(buffer, &begin_info)}.expect("should not fail");
        
        let buffer_copy = [
            vk::BufferCopy::builder()
                .size(size)
                .build()
        ];
        
        unsafe{device.cmd_copy_buffer(buffer, src_buff.buffer, dst_buff.buffer, &buffer_copy)};
        unsafe{device.end_command_buffer(buffer)}.expect("should not fail");
        
        let buffer_slice = unsafe{from_raw_parts(addr_of!(buffer), 1)};
        
        let submit_info = [
            vk::SubmitInfo::builder()
                .command_buffers(buffer_slice)
                .build(),
        ];
        
        unsafe{device.queue_submit(device.queue_handles.graphics, &submit_info[..], vk::Fence::null())}.expect("should not fail");
        unsafe{device.device_wait_idle()}.expect("waiting for iddle should not fail");
    }
    
    pub fn find_memory_type_index(
        state:&State, 
        p_device:&PhysicalDevice, 
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
    
    fn silent_drop(&mut self, device:&Device) {
        unsafe{device.destroy_buffer(self.buffer, None)}
        unsafe{device.free_memory(self.memory, None)}
    }
}

impl DeviceDrop for Buffer {
    fn device_drop(&mut self, state:&State, device:&Device) {
        if state.v_nor() {
            println!("[0]:deleting buffer");
        }
        self.silent_drop(device);
    }
}

