use crate::AAError;
use crate::logger;
use crate::errors::messages::GRANTED;
use crate::errors::messages::VK_CAST;

use super::Vertex;

use super::VkDestructor;
use super::VkDestructorArguments;
use super::super::Device;
use super::super::Allocator;
use super::super::CommandControl;
use super::super::Buffer;
use super::super::memory;

use std::mem::size_of_val;
use std::mem::align_of;

use ash::vk;
use nalgebra as na;
use na::Matrix4;
use na::Vector3;
use na::Vector4;



pub struct GPUMeshBuffers {
    pub index_buffer: Buffer,
    pub vertex_buffer: Buffer,
    pub vertex_buffer_address: vk::DeviceAddress,
}

pub fn init_square_mesh(
        device: &mut Device,
        allocator: &mut Allocator,
        command_control: &mut CommandControl,
    ) -> GPUMeshBuffers {
    let mut indices = vec![1u32;6];
    let mut vertices = vec![super::Vertex::default();4];
    
    
    /*
    indices[0] = 0;
	indices[1] = 1;
	indices[2] = 2;
	indices[3] = 2;
	indices[4] = 1;
	indices[5] = 3;
    */

    
    indices[0] = 0;
	indices[1] = 1;
	indices[2] = 2;
	indices[3] = 2;
	indices[4] = 1;
	indices[5] = 3;

    vertices[0].position = Vector3::new(0.5, -0.5, 0.0);
    vertices[1].position = Vector3::new(0.5, 0.5, 0.0);
    vertices[2].position = Vector3::new(-0.5, -0.5, 0.0);
    vertices[3].position = Vector3::new(-0.5, 0.5, 0.0);
    
    
    vertices[0].color = Vector4::new(0.0, 0.0, 0.0, 1.0);
    vertices[1].color = Vector4::new(0.5, 0.5, 0.5, 1.0);
    vertices[2].color = Vector4::new(1.0, 0.0, 0.0, 1.0);
    vertices[3].color = Vector4::new(0.0, 1.0, 0.0, 1.0);
    
    /*
    vertices[0].position = Vector3::new(-0.5, -0.5, 0.0);
    vertices[0].normal = Vector3::new(1.0, 0.0, 0.0);
    vertices[0].color = Vector4::new(1.0, 0.0, 0.0, 1.0);
    
    vertices[1].position = Vector3::new(0.5, 0.5, 0.0);
    vertices[1].normal = Vector3::new(0.0, 1.0, 0.0);
    vertices[1].color = Vector4::new(0.5, 0.5, 0.5, 1.0);
    
    vertices[2].position = Vector3::new(-0.5, 0.5, 0.0);
    vertices[2].normal = Vector3::new(0.0, 0.0, 1.0);
    vertices[2].color = Vector4::new(0.0, 1.0, 0.0, 1.0);
    
    */
    
    let mesh = GPUMeshBuffers::upload_mesh(device, allocator, command_control, &indices[..], &vertices[..]).unwrap();
    
    mesh
}

impl GPUMeshBuffers {
    pub fn upload_mesh(
        device: &mut Device,
        allocator: &mut Allocator,
        command: &mut CommandControl,
        indices: &[u32],
        vertices: &[Vertex],
    ) -> Result<Self, AAError> {
        logger::create!("mesh");
        if indices.is_empty() || vertices.is_empty() {
            return Err(AAError::EmptyMesh);
        }
        let indices_size = u64::try_from(indices.len() * size_of_val(&indices[0])).expect(GRANTED);
        let vertices_size = u64::try_from(vertices.len() * size_of_val(&vertices[0])).expect(GRANTED);
        
        /*
        println!("index size: {}", size_of_val(&indices[0]));
        println!("index count: {} \tindex_buf size: {}", indices.len(), indices_size);
        
        println!("vertex size: {}", size_of_val(&vertices[0]));
        println!("vertex count: {}\tvertex_buf size: {}", vertices.len(), vertices_size);
        */
        
        use vk::BufferUsageFlags as buf;
        let mut vertex_buffer = Buffer::create(device, allocator, Some("mesh vertex buffer"), vertices_size, buf::VERTEX_BUFFER|buf::STORAGE_BUFFER|buf::SHADER_DEVICE_ADDRESS|buf::TRANSFER_DST, memory::GpuOnly)?;
        let vertex_buffer_address = vertex_buffer.get_device_address(device);
        
        let mut index_buffer = Buffer::create(device, allocator, Some("mesh index buffer"), indices_size, buf::INDEX_BUFFER|buf::TRANSFER_DST, memory::GpuOnly)?;
        
        let staging_buffer = Buffer::create(device, allocator, Some("mesh staging buffer"), indices_size+vertices_size, buf::TRANSFER_SRC, memory::CpuToGpu)?;
        let staging_mem_ptr = staging_buffer.allocation.mapped_ptr().unwrap();
        
        /*
        let tmp:Vec<u8> = vertices.into_iter().map(|vertex| unsafe {crate::any_as_u8_slice(vertex).iter().map(|a|*a)}).flatten().collect();
        println!("{:?}",tmp.len());
        
        for slice in tmp[..].chunks(48) {
            println!("{:?}", slice);
        }
        
        for (buffer, slice) in vertex_buffer.get_slice_mut().unwrap().iter_mut().zip(tmp.iter()) {
            *buffer = *slice;
        }
        
        
        let mut vert_align:ash::util::Align<Vertex> = unsafe{ash::util::Align::new(vertex_buffer.allocation.mapped_ptr().unwrap().as_ptr(), align_of::<Vertex>() as u64, vertices_size)};
        vert_align.copy_from_slice(vertices);
        
        for elem in vertex_buffer.get_slice_mut().unwrap().chunks(48) {
            println!("{:?}", elem);
        }
        
        let mut index_align:ash::util::Align<u32> = unsafe{ash::util::Align::new(index_buffer.allocation.mapped_ptr().unwrap().as_ptr(), align_of::<u32>() as u64, indices_size)};
        index_align.copy_from_slice(indices);
        */
        
        let staging_mem_ptr = staging_buffer.allocation.mapped_ptr().unwrap();
        let mut vert_align:ash::util::Align<Vertex> = unsafe{ash::util::Align::new(staging_mem_ptr.as_ptr(), align_of::<Vertex>() as u64, vertices_size)};
        vert_align.copy_from_slice(vertices);
        
        let staging_index_ptr = unsafe{staging_mem_ptr.as_ptr().byte_add(usize::try_from(vertices_size).expect(VK_CAST))};
        let mut index_align:ash::util::Align<u32> = unsafe{ash::util::Align::new(staging_index_ptr, align_of::<u32>() as u64, indices_size)};
        index_align.copy_from_slice(indices);
        
        memory::copy_buffer_2_buffer(device, command, &staging_buffer, 0, &mut vertex_buffer, 0, vertices_size);
        
        memory::copy_buffer_2_buffer(device, command, &staging_buffer, vertices_size, &mut index_buffer, 0, indices_size);
        
        staging_buffer.destruct(VkDestructorArguments::DevAll(device, allocator));
        /*
        */
        
        Ok(Self{
            vertex_buffer,
            vertex_buffer_address,
            index_buffer,
        })
    }
}


impl VkDestructor for GPUMeshBuffers {
    fn destruct(self, mut args:VkDestructorArguments) {
        logger::destruct!("mesh");
        let (device, allocator) = args.unwrap_dev_all();
        
        self.index_buffer.destruct(VkDestructorArguments::DevAll(device, allocator));
        self.vertex_buffer.destruct(VkDestructorArguments::DevAll(device, allocator));
    }
}

/*
    indices[0] = 0;
	indices[1] = 1;
	indices[2] = 2;
	indices[3] = 2;
	indices[4] = 1;
	indices[5] = 3;

    vertices[0].position = Vector3::new(0.5, -0.5, 0.0);
    vertices[1].position = Vector3::new(0.5, 0.5, 0.0);
    vertices[2].position = Vector3::new(-0.5, -0.5, 0.0);
    vertices[3].position = Vector3::new(-0.5, 0.5, 0.0);
    
    
    vertices[0].color = Vector4::new(0.0, 0.0, 0.0, 1.0);
    vertices[1].color = Vector4::new(0.5, 0.5, 0.5, 1.0);
    vertices[2].color = Vector4::new(1.0, 0.0, 0.0, 1.0);
    vertices[3].color = Vector4::new(0.0, 1.0, 0.0, 1.0);
*/
