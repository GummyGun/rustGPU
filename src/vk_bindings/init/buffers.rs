use ash::{
    vk, 
    prelude::VkResult,
};

use super::{
    DeviceDrop,
    device::Device,
    p_device::PDevice,
    command::CommandControl,
};

use crate::{
    State,
    graphics::{
        Vertex,
        VERTEX_ARR,
        VERTEX_INDEX,
        UniformBufferObject,
    },
    constants,
    errors::Error as AAError,
};

use std::{
    mem::{
        size_of,
        align_of,
        //MaybeUninit,
    },
    ptr::addr_of,
    slice::from_raw_parts,
    ffi::{
        c_void,
    },
    ops::{
        Deref,
        DerefMut,
    },
};

#[derive(Default)]
pub struct Buffer {
    pub buffer: vk::Buffer,
    pub memory: vk::DeviceMemory,
}

pub struct UniformBuffer {
    pub buffer: Buffer,
    pub map: *mut c_void,
    pub align: ash::util::Align<UniformBufferObject>,
}

pub struct UniformBuffers {
    pub buffers: [UniformBuffer; constants::fif::USIZE],
}

impl Buffer {
    pub fn create_vertex(
        state:&State, 
        p_device:&PDevice, 
        device:&Device, 
        command:&CommandControl,
    ) -> VkResult<Self> {
        use vk::BufferUsageFlags as BUF;
        use vk::MemoryPropertyFlags as MPF;
        
        let raw_size = u64::try_from(size_of::<Vertex>()*VERTEX_ARR.len()).expect("vertex buffer size should fit in u64");
        if state.v_exp() {
            println!("\nCREATING:\tVERTEX BUFFER");
            println!("vertex_buffer size in bytes {:?}", raw_size);
        }
        
        
        let staging_memory_flags = MPF::HOST_VISIBLE | MPF::HOST_COHERENT;
        
        let (mut staging, staging_size) = Self::create_buffer(state, p_device, device, raw_size, BUF::TRANSFER_SRC, staging_memory_flags)?;
        
        let memory_ptr = unsafe{device.map_memory(staging.memory, 0, staging_size, vk::MemoryMapFlags::empty())}?;
        let mut vert_align = unsafe{ash::util::Align::new(memory_ptr, align_of::<Vertex>() as u64, staging_size)};
        vert_align.copy_from_slice(&VERTEX_ARR);
        
        unsafe{device.unmap_memory(staging.memory)};
        
        let vertex_buffer_usage = BUF::VERTEX_BUFFER | BUF::TRANSFER_DST;
        let vertex_memory_flags = MPF::HOST_VISIBLE | MPF::HOST_COHERENT;
        
        let (holder, _) = Self::create_buffer(state, p_device, device, raw_size, vertex_buffer_usage, vertex_memory_flags)?;
        
        Self::copy_buffer(state, device, command.staging_buffer, &staging, &holder, raw_size);
        
        if state.v_exp() {
            println!("deleting stage buffer");
        }
        staging.silent_drop(device);
        Ok(holder)
    }
    
    pub fn create_index(
        state:&State, 
        p_device:&PDevice, 
        device:&Device, 
        command:&CommandControl,
    ) -> VkResult<Self> {
        use vk::BufferUsageFlags as BUF;
        use vk::MemoryPropertyFlags as MPF;
        
        let raw_size = u64::try_from(size_of::<u16>()*VERTEX_INDEX.len()).expect("index buffer size should fit in u64");
        
        if state.v_exp() {
            println!("\nCREATING:\tINDEX BUFFER");
            println!("index_buffer size in bytes {:?}", raw_size);
        }
        
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
        
        Self::copy_buffer(state, device, command.staging_buffer, &staging, &holder, raw_size);
        
        if state.v_exp() {
            println!("deleting stage buffer");
        }
        staging.silent_drop(device);
        Ok(holder)
    }
    
    
    fn create_buffer(
        state: &State, 
        p_device: &PDevice, 
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


impl UniformBuffers {
    
    pub fn create(
        state:&State, 
        p_device:&PDevice, 
        device:&Device, 
    ) -> VkResult<Self> {
        
        use constants::fif;
        
        if state.v_exp() {
            println!("\nCREATING:\tUNIFORM BUFFERS");
        }
        
        /*
        TODO: maybe unit would be a better way of implementing this since it wouldnt force the user to handle the errors in this function
        for this #96097 would need to be stabilized, maybe in a near future it can be achived, until then this solution is kind of elegant,
        */
        
        /*
        let mut holder:[MaybeUninit<UniformBuffer>; constants::FIF] = from_fn(|_| MaybeUninit::uninit());
        let mut test:MaybeUninit<[UniformBuffer; constants::FIF]> = MaybeUninit::zeroed();
        //  ^
        // /|\
        //  |
        //  |____ with this you can't write to a particular field in the array, maybe with pointers you can work arround this limitation but I would rather not do that
        
        
        for index in 0..constants::FIF {
            if state.v_exp() {
                println!("creating sync objects for frame {}", index);
            }
            let (uniform, uniform_size) = Buffer::create_buffer(state, p_device, device, raw_size, BUF::UNIFORM_BUFFER, staging_memory_flags)?;
            
            let uniform_ptr = unsafe{device.map_memory(uniform.memory, 0, uniform_size, vk::MemoryMapFlags::empty())}?;
            
            let uniform_align:ash::util::Align<UniformBufferObject> = unsafe{ash::util::Align::new(uniform_ptr, align_of::<UniformBufferObject>() as u64, uniform_size)};
            
            holder[index].write(UniformBuffer{
                buffer: uniform, 
                map: uniform_ptr,
                align: uniform_align,
            });
            
        }
        let holder = holder.array_assume_init();
        */
        
        /*
        TODO: acording to the documentation .map on [] is very ineficient memory wise 
        */
        
        let mut index = 0;
        let holder = [(); fif::USIZE].map(|_| {
            
            if state.v_exp() {
                println!("creating sync objects for frame {}", index);
            }
            index += 1;
            UniformBuffer::create(state, p_device, device).expect("should not fail")
        });
        
        Ok(Self{
            buffers:holder,
        })
    }
    
    /*
    pub fn update_buffer(&mut self, state:&State, frame:u32) {
        println!("{:?} {:?}", state.time, state.time.elapsed());
        
        
    }
    */
    
}

impl UniformBuffer {

    pub fn create(
        state:&State, 
        p_device:&PDevice, 
        device:&Device, 
    ) -> VkResult<Self> {
        
        use vk::BufferUsageFlags as BUF;
        use vk::MemoryPropertyFlags as MPF;
        
        if state.v_dmp() {
            println!("\nCREATING:\tUNIFORM BUFFERS");
        }
        
        let raw_size = u64::try_from(size_of::<UniformBufferObject>()).expect("index buffer size should fit in u64");
        
        let staging_memory_flags = MPF::HOST_VISIBLE | MPF::HOST_COHERENT;
        
        let (uniform, uniform_size) = Buffer::create_buffer(state, p_device, device, raw_size, BUF::UNIFORM_BUFFER, staging_memory_flags).expect("should not fail");
        
        let uniform_ptr = unsafe{device.map_memory(uniform.memory, 0, uniform_size, vk::MemoryMapFlags::empty())}.expect("should not fail");
        
        let uniform_align:ash::util::Align<UniformBufferObject> = unsafe{ash::util::Align::new(uniform_ptr, align_of::<UniformBufferObject>() as u64, uniform_size)};
        
        Ok(UniformBuffer{
            buffer: uniform, 
            map: uniform_ptr,
            align: uniform_align,
        })
    }
}

impl Deref for UniformBuffer {
    type Target = Buffer;
    fn deref(&self) -> &Self::Target {
        &self.buffer
    }
}
impl DerefMut for UniformBuffer {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.buffer
    }
}

impl DeviceDrop for UniformBuffer {
    fn device_drop(&mut self, state:&State, device:&Device) {
        if state.v_dmp() {
            println!("[0]:deleting buffer");
        }
        
        unsafe{device.unmap_memory(self.buffer.memory)};
        self.buffer.silent_drop(device);
    }
}

impl DeviceDrop for UniformBuffers {
    fn device_drop(&mut self, state:&State, device:&Device) {
        if state.v_nor() {
            println!("[0]:deleting uniform buffers");
        }
        for buffer in self.buffers.iter_mut() {
            buffer.device_drop(state, device);
        }
    }
}

impl Deref for UniformBuffers {
    type Target = [UniformBuffer; constants::fif::USIZE]; 
    fn deref(&self) -> &Self::Target {
        &self.buffers
    }
}

