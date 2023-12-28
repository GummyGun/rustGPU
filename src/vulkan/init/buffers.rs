use ash::{
    vk, 
    prelude::VkResult,
};

use super::{
    memory,
    DeviceDestroy,
    device::Device,
    p_device::PDevice,
    command::CommandControl,
};

use crate::{
    State,
    graphics::{
        Vertex,
        //VERTEX_ARR,
        //VERTEX_INDEX,
        UniformBufferObject,
    },
    constants,
};

use std::{
    mem::{
        size_of,
        align_of,
    },
    ffi::{
        c_void,
    },
    ops::{
        Deref,
    },
};

#[derive(Default)]
pub struct Buffer {
    pub buffer: vk::Buffer,
    pub memory: vk::DeviceMemory,
}

pub struct UniformBuffer {
    pub buffer: vk::Buffer,
    pub memory: vk::DeviceMemory,
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
        vertices:&[Vertex],
    ) -> VkResult<Self> {
        use vk::BufferUsageFlags as BUF;
        use vk::MemoryPropertyFlags as MPF;
        
        let raw_size = u64::try_from(size_of::<Vertex>()*vertices.len()).expect("vertex buffer size should fit in u64");
        if state.v_exp() {
            println!("\nCREATING:\tVERTEX BUFFER");
            println!("vertex_buffer size in bytes {:?}", raw_size);
        }
        
        
        let staging_memory_flags = MPF::HOST_VISIBLE | MPF::HOST_COHERENT;
        
        let (staging, staging_size) = Self::create_buffer(state, p_device, device, raw_size, BUF::TRANSFER_SRC, staging_memory_flags)?;
        
        let memory_ptr = unsafe{device.map_memory(staging.memory, 0, staging_size, vk::MemoryMapFlags::empty())}?;
        let mut vert_align = unsafe{ash::util::Align::new(memory_ptr, align_of::<Vertex>() as u64, raw_size)};
        vert_align.copy_from_slice(&vertices);
        
        unsafe{device.unmap_memory(staging.memory)};
        
        let buffer_usage = BUF::VERTEX_BUFFER | BUF::TRANSFER_DST;
        let memory_flags = MPF::HOST_VISIBLE | MPF::HOST_COHERENT;
        
        let (mut holder, _) = Self::create_buffer(state, p_device, device, raw_size, buffer_usage, memory_flags)?;
        
        memory::copy_buffer_2_buffer(state, device, command, &staging, &mut holder, raw_size);
        
        staging.staging_drop(state, device);
        Ok(holder)
    }
    
    pub fn create_index(
        state:&State, 
        p_device:&PDevice, 
        device:&Device, 
        command:&CommandControl,
        indices:&[u32],
    ) -> VkResult<Self> {
        use vk::BufferUsageFlags as BUF;
        use vk::MemoryPropertyFlags as MPF;
        
        let raw_size = u64::try_from(size_of::<u32>()*indices.len()).expect("index buffer size should fit in u64");
        
        if state.v_exp() {
            println!("\nCREATING:\tINDEX BUFFER");
            println!("index_buffer size in bytes {:?}", raw_size);
        }
        
        let staging_memory_flags = MPF::HOST_VISIBLE | MPF::HOST_COHERENT;
        
        let (staging, staging_size) = Self::create_buffer(state, p_device, device, raw_size, BUF::TRANSFER_SRC, staging_memory_flags)?;
        
        let memory_ptr = unsafe{device.map_memory(staging.memory, 0, staging_size, vk::MemoryMapFlags::empty())}?;
        
        let mut vert_align = unsafe{ash::util::Align::new(memory_ptr, align_of::<u16>() as u64, raw_size)};
        
        vert_align.copy_from_slice(&indices);
        /*
        let tmp = unsafe{from_raw_parts(memory_ptr as *const u16, VERTEX_INDEX .len())};
        println!("hola {:#?}", tmp);
        */
        unsafe{device.unmap_memory(staging.memory)};
        
        let buffer_usage = BUF::INDEX_BUFFER | BUF::TRANSFER_DST;
        let memory_flags = MPF::HOST_VISIBLE | MPF::HOST_COHERENT;
        
        
        let (mut holder, _) = Self::create_buffer(state, p_device, device, raw_size, buffer_usage, memory_flags)?;
        
        memory::copy_buffer_2_buffer(state, device, command, &staging, &mut holder, raw_size);
        
        staging.staging_drop(state, device);
        Ok(holder)
    }
    
    
    pub fn create_buffer(
        state: &State, 
        p_device: &PDevice, 
        device: &Device, 
        size: u64, 
        usage_flags: vk::BufferUsageFlags, 
        memory_flags: vk::MemoryPropertyFlags,
    ) -> VkResult<(Self, u64)> {
        
        let create_info = vk::BufferCreateInfo::builder()
            .size(size)
            .usage(usage_flags)
            .sharing_mode(vk::SharingMode::EXCLUSIVE);
        
        let buffer = unsafe{device.create_buffer(&create_info, None)}?;
        
        let memory_requirements = unsafe{device.get_buffer_memory_requirements(buffer)};
        
        let index_holder = memory::find_memory_type_index(state, p_device, &memory_requirements, memory_flags).expect("required memory type is not present");
        
        let allocate_info = ash::vk::MemoryAllocateInfo::builder()
            .allocation_size(memory_requirements.size)
            .memory_type_index(index_holder);
        
        let memory = unsafe{device.allocate_memory(&allocate_info, None)}?;
        
        unsafe{device.bind_buffer_memory(buffer, memory, 0)}?;
        
        Ok((Self{buffer:buffer, memory:memory}, memory_requirements.size))
    }
    
    pub fn staging_drop(mut self, state:&State, device:&Device) {
        if state.v_exp() {
            println!("deleting staging buffer")
        }
        self.drop_internal(device);
    }
    
    fn drop_internal(&mut self, device:&Device) {
        unsafe{device.destroy_buffer(self.buffer, None)}
        unsafe{device.free_memory(self.memory, None)}
    }
}

impl DeviceDestroy for Buffer {
    fn device_drop(&mut self, state:&State, device:&Device) {
        if state.v_nor() {
            println!("[0]:deleting buffer");
        }
        self.drop_internal(device);
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
        
        Ok(UniformBuffer::from((uniform, uniform_ptr, uniform_align)))
    }
}


impl From<(Buffer, *mut c_void, ash::util::Align<UniformBufferObject>)> for UniformBuffer {   
    fn from(base:(Buffer, *mut c_void, ash::util::Align<UniformBufferObject>)) -> Self {
        Self{
            buffer:base.0.buffer, 
            memory:base.0.memory,
            map:base.1,
            align:base.2,
        }
    }
}

impl DeviceDestroy for UniformBuffer {
    fn device_drop(&mut self, state:&State, device:&Device) {
        if state.v_dmp() {
            println!("[0]:deleting buffer");
        }
        
        unsafe{device.unmap_memory(self.memory)};
        unsafe{device.destroy_buffer(self.buffer, None)}
        unsafe{device.free_memory(self.memory, None)}
    }
}

impl DeviceDestroy for UniformBuffers {
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

