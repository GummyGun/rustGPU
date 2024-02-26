use crate::AAError;
use crate::logger;
use crate::errors::messages::VK_CAST;
use crate::errors::messages::GRANTED;
use crate::errors::messages::MODEL_DENSITY;

use super::Vertex;
use super::MeshAssetMetadata;

use super::GeoSurface;
use super::VkDestructor;
use super::VkDestructorArguments;
use super::super::Device;
use super::super::Allocator;
use super::super::CommandControl;
use super::super::Buffer;
use super::super::memory;

use std::mem::size_of_val;
use std::path::Path;
use std::fs; 
use std::io;

use ash::vk;
use nalgebra as na;
use na::Vector3;
use na::Vector4;
use derivative::Derivative;

#[derive(Derivative, Default)]
#[derivative(Debug)]
pub struct MeshAssets {
    pub metadatas: Vec<MeshAssetMetadata>,
    #[derivative(Debug="ignore")]
    pub meshes: Vec<GPUMeshBuffers>,
}


pub struct GPUMeshBuffers {
    pub index_buffer: Buffer,
    pub vertex_buffer: Buffer,
    pub vertex_buffer_address: vk::DeviceAddress,
}


pub fn load_gltf<P: AsRef<Path>>(
    device: &mut Device,
    allocator: &mut Allocator,
    command_control: &mut CommandControl,
    path: P,
) -> Result<MeshAssets, AAError> {
    let file = fs::File::open(path)?;
    let reader = io::BufReader::new(file);
    let gltf = gltf::Gltf::from_reader(reader)?;
    let mut holder = MeshAssets::default();
    
    let mut indices_vec:Vec<u32> = Vec::new();
    let mut vertices_vec:Vec<Vertex> = Vec::new();
    
    let meshes = gltf.meshes();
    
    logger::various_log!("mesh",
        (logger::Trace, "amount of meshes {}", meshes.len())
    );
    
    //println!("{}", meshes.len());
    for mesh in meshes {
        let mut metadata_holder = MeshAssetMetadata::default();
        indices_vec.clear();
        vertices_vec.clear();
        
        
        //println!("{:?}", mesh.name());
        match mesh.name() {
            Some(name) => {
                logger::various_log!("mesh",
                    (logger::Trace, "mesh name {}", name)
                );
                metadata_holder.name.push_str(name);
            }
            None => {
                logger::various_log!("mesh",
                    (logger::Trace, "no mesh name")
                );
                metadata_holder.name.push_str("empty");
            }
        }
        
        
        let primitives = mesh.primitives();
        
        logger::various_log!("mesh",
            (logger::Trace, "primitives_count {}", &primitives.len())
        );
        //println!("{}", &primitives.len());
        for primitive in primitives {
            
            let mut surface = GeoSurface::default();
            surface.start_index = u32::try_from(indices_vec.len()).expect(MODEL_DENSITY);
            let reader = primitive.reader(|_primitive|{Some(&gltf.blob.as_ref().unwrap()[..])});
            
            let indices = reader.read_indices().unwrap();
            //println!("indices count");
            use gltf::mesh::util::ReadIndices;
            match indices {
                ReadIndices::U8(indices) => {
                    logger::various_log!("mesh",
                        (logger::Trace, "indices count u8 {}", indices.len())
                    );
                    for index in indices {
                        indices_vec.push(u32::from(index));
                    }
                }
                ReadIndices::U16(indices) => {
                    logger::various_log!("mesh",
                        (logger::Trace, "indices count u16 {}", indices.len())
                    );
                    for index in indices {
                        indices_vec.push(u32::from(index));
                    }
                }
                ReadIndices::U32(indices) => {
                    logger::various_log!("mesh",
                        (logger::Trace, "indices count u32 {}", indices.len())
                    );
                    for index in indices {
                        indices_vec.push(u32::from(index));
                    }
                }
            }
            
            
            let positions = reader.read_positions().unwrap();
            logger::various_log!("mesh",
                (logger::Trace, "vertex count {}", positions.len())
            );
            for pos in positions {
                let mut vertex_holder = Vertex::default();
                vertex_holder.position = Vector3::from(pos);
                vertices_vec.push(vertex_holder);
            }
            
            let normals = reader.read_normals().unwrap();
            logger::various_log!("mesh",
                (logger::Trace, "normals count {}", normals.len())
            );
            for (index, norm) in normals.enumerate() {
                vertices_vec[index].normal = Vector3::from(norm);
                vertices_vec[index].color = Vector4::new(norm[0], norm[1], norm[2], 1.0);
            }
            
            
            
            
            surface.count = u32::try_from(indices_vec.len()).expect(MODEL_DENSITY);
            metadata_holder.surfaces.push(surface);
        }
        holder.metadatas.push(metadata_holder);
        holder.meshes.push(GPUMeshBuffers::upload_mesh(device, allocator, command_control, &indices_vec, &vertices_vec[..]).unwrap());
        /*
        println!("{:?}", holder.metadatas);
        println!("{:?}", indices_vec);
        for vertex in &vertices_vec {
            println!("{:?}", vertex);
            
        }
        */
        
    }
    
    /*
    println!("surface");
    for metadata in &holder.metadatas {
        println!("{:?}", metadata);
    }
    */
    

    Ok(holder)
    
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
        let indices_size = indices.len() * size_of_val(&indices[0]);
        let indices_size_u64 = u64::try_from(indices_size).expect(VK_CAST);
        let vertices_size = vertices.len() * size_of_val(&vertices[0]);
        let vertices_size_u64 = u64::try_from(vertices_size).expect(VK_CAST);
        
        use vk::BufferUsageFlags as buf;
        let mut vertex_buffer = Buffer::create(device, allocator, Some("mesh vertex buffer"), vertices_size_u64, buf::VERTEX_BUFFER|buf::STORAGE_BUFFER|buf::SHADER_DEVICE_ADDRESS|buf::TRANSFER_DST, memory::GpuOnly)?;
        let vertex_buffer_address = vertex_buffer.get_device_address(device);
        
        let mut index_buffer = Buffer::create(device, allocator, Some("mesh index buffer"), indices_size_u64, buf::INDEX_BUFFER|buf::TRANSFER_DST, memory::GpuOnly)?;
        
        let mut staging_buffer = Buffer::create(device, allocator, Some("mesh staging buffer"), indices_size_u64+vertices_size_u64, buf::TRANSFER_SRC, memory::CpuToGpu)?;
        
        
        let mut vert_align = staging_buffer.get_align::<Vertex>(0, vertices_size_u64).expect(GRANTED);
        vert_align.copy_from_slice(vertices);
        
        let mut index_align = staging_buffer.get_align::<u32>(vertices_size, indices_size_u64).expect(GRANTED);
        index_align.copy_from_slice(indices);
        
        memory::copy_buffer_2_buffer(device, command, &staging_buffer, 0, &mut vertex_buffer, 0, vertices_size_u64);
        memory::copy_buffer_2_buffer(device, command, &staging_buffer, vertices_size_u64, &mut index_buffer, 0, indices_size_u64);
        
        staging_buffer.destruct(VkDestructorArguments::DevAll(device, allocator));
        
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

impl VkDestructor for MeshAssets {
    fn destruct(self, mut args:VkDestructorArguments) {
        logger::destruct!("mesh_assets");
        let (device, allocator) = args.unwrap_dev_all();
        for mesh in self.meshes.into_iter() {
            mesh.destruct(VkDestructorArguments::DevAll(device, allocator));
        }
        
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
