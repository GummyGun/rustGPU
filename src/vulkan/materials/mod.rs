mod metalic;
pub use metalic::*;

use crate::AAError;
use crate::constants;
use crate::errors::messages::COMPILETIME_ASSERT;

use super::graphics::*;
use super::init::*;
use super::objects::*;

use std::rc::Rc;
use std::slice::from_ref;
use std::mem::size_of;

use ash::vk;
use gpu_allocator as gpu_all;
use nalgebra as na;

pub struct Materials {
    pub metalic: metalic::MetalicMaterial,
    pub metalic_instance: MaterialInstance,
}


pub struct MaterialInstance {
    pub pipeline: Rc<DispatchableGPipeline>,
    pub descriptor_set: vk::DescriptorSet,
    pub pass_type: MaterialPass,
}

impl Clone for MaterialInstance {
    fn clone(&self) -> Self {
        Self{
            pipeline: self.pipeline.clone(),
            descriptor_set: self.descriptor_set,
            pass_type: self.pass_type,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum MaterialPass {
    MainColor,
    Transparent,
    Other,
}

pub struct MaterialResources<'a> {
    pub color_image: &'a Image,
    pub color_sampler: &'a Sampler,
    pub metal_image: &'a Image,
    pub metal_sampler: &'a Sampler,
    pub buffer: Buffer,
    pub buffer_offset: u64,
}

#[repr(C)]
#[derive(Default, Clone, Copy)]
pub struct MaterialConstants {
    pub color_factors: na::Vector4<f32>,
    pub metal_rough_factors: na::Vector4<f32>,
    pub extra: [na::Vector4<f32>;14],
}

const _:u64 = MaterialConstants::size_u64();
impl MaterialConstants {
    #[allow(dead_code)]
    pub const fn size_u64() -> u64 {
        if size_of::<Self>() > u64::MAX as usize {
            panic!("{}", COMPILETIME_ASSERT);
        }
        size_of::<Self>() as u64
    }
}



pub fn init_material(
    device: &mut Device, 
    allocator:&mut Allocator, 
    canvas:&Canvas, 
    ds_pool:&mut GDescriptorAllocator, 
    destruction_stack: &mut DestructionStack,
    scene_descriptor:&DescriptorLayout, 
    white_texture: &Image,
    linear_sampler: &Sampler,
    
) -> Result<Materials, AAError> {
    
    let mut metalic = MetalicMaterial::build_pipelines(device, canvas, scene_descriptor).unwrap();
    let mut buffer = Buffer::create(device, allocator, Some("Metalic material"), MaterialConstants::size_u64(), vk::BufferUsageFlags::UNIFORM_BUFFER, gpu_all::MemoryLocation::CpuToGpu).unwrap();
    let holder = MaterialConstants{
        color_factors: na::Vector4::new(1f32,1f32,1f32,1f32),
        metal_rough_factors: na::Vector4::new(1f32,0.5f32,0f32,0f32),
        ..MaterialConstants::default()
    };
    destruction_stack.push(buffer.defered_destruct());
    {
        let mut align = buffer.get_align::<MaterialConstants>(0, MaterialConstants::size_u64()).unwrap();
        align.copy_from_slice(from_ref(&holder));
    }
    
    let material_resources = MaterialResources{
        buffer,
        buffer_offset: 0,
        metal_image: white_texture,
        metal_sampler: linear_sampler,
        color_image: white_texture,
        color_sampler: linear_sampler,
    };
    
    let metalic_instance = metalic.write_material(device, ds_pool, MaterialPass::MainColor, &material_resources).unwrap();
    
    Ok(Materials{
        metalic_instance, 
        metalic,
    })
    
}

impl Materials {
    pub fn get_default(&self) -> &MaterialInstance {
        &self.metalic_instance
    }
}

impl VkDestructor for Materials {
    fn destruct(self, mut args:VkDestructorArguments) {
        use std::mem::drop;
        let device = args.unwrap_dev();
        let Materials{
            metalic_instance,
            metalic,
        } = self;
        drop(metalic_instance);
        metalic.destruct(VkDestructorArguments::Dev(device));
    }
}

