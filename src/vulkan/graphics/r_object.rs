use crate::AAError;
use crate::constants;

use super::super::Device;
use super::super::pipeline;
use super::super::GPipeline;
use super::super::GPipelineBuilder;
use super::super::DispatchableGPipeline;
use super::super::Image;
use super::super::Sampler;
use super::super::Buffer;
use super::super::DescriptorWriter;
use super::super::DescriptorLayout;
use super::super::DescriptorLayoutBuilder;
use super::super::GDescriptorAllocator;

use super::GPUDrawPushConstants;
use super::Canvas;

use std::slice::from_ref;

use ash::vk;
use nalgebra as na;

#[derive(Debug)]
pub struct RenderObject {
    index_count: u32,
    first_index: u32,
    index_buffers: Buffer,
    
    material_instance: i32,
    
    traunsform: na::Matrix4<f32>,
    device_address: vk::DeviceAddress,
}


pub struct MaterialInstance {
    pipeline: DispatchableGPipeline,
    descriptor_set: vk::DescriptorSet,
    pass_type: MaterialPass,
}

pub enum MaterialPass {
    MainColor,
    Transparent,
    Other,
}

pub struct MaterialResources {
    color_image: Image,
    color_sampler: Sampler,
    metal_image: Image,
    metal_sampler: Sampler,
    buffer: Buffer,
    buffer_offset: u64,
}


struct MaterialConstants {
    color_factors: na::Vector4<f32>,
    metal_rough_factors: na::Vector4<f32>,
    extra: [na::Vector4<f32>;14],
}

pub struct GLTFMetalic {
    opaque_pipeline: vk::Pipeline,
    transparent_pipeline: vk::Pipeline,
    pipeline_layout: vk::PipelineLayout,
    descriptor_writer: DescriptorWriter,
    descriptor_layout: DescriptorLayout,
    
}

pub struct DrawContext<'a> {
    context: Vec<&'a dyn IRenderable>,
}

// base class for a renderable dynamic object
pub trait IRenderable {
    fn draw(&self, top_matrix:&na::Matrix4<f32>, ctx:&DrawContext);
}


impl GLTFMetalic {
    
    pub fn write_material(&mut self, device:&mut Device, descriptor_allocator: &mut GDescriptorAllocator, material_pass:MaterialPass, material_resources:&MaterialResources) -> Result<MaterialInstance, AAError> {
        
        
        let pipeline_holder = match material_pass {
            MaterialPass::MainColor => {
                DispatchableGPipeline{
                    pipeline: self.opaque_pipeline, 
                    layout: self.pipeline_layout
                }
            }
            MaterialPass::Transparent => {
                DispatchableGPipeline{
                    pipeline: self.transparent_pipeline, 
                    layout: self.pipeline_layout
                }
            }
            _ => {
                panic!("Invalid material pass");
            }
        };
        
        let descriptor_holder = descriptor_allocator.allocate(device, &self.descriptor_layout)?;
        
        let Self{
            descriptor_writer: writer,
            ..
        } = self;
        
        writer.clear();
        writer.write_buffer(0, material_resources.buffer.underlying(), std::mem::size_of::<MaterialConstants>() as u64, material_resources.buffer_offset, vk::DescriptorType::UNIFORM_BUFFER);
        writer.write_image(1, material_resources.color_image.view, material_resources.color_sampler.underlying(), vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL, vk::DescriptorType::COMBINED_IMAGE_SAMPLER);
        writer.write_image(2, material_resources.metal_image.view, material_resources.metal_sampler.underlying(), vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL, vk::DescriptorType::COMBINED_IMAGE_SAMPLER);
        
        writer.update_set(device, descriptor_holder);
        
        Ok(MaterialInstance{
            pipeline: pipeline_holder,
            descriptor_set: descriptor_holder,
            pass_type: material_pass,
        })
    }
    
    pub fn build_pipelines(device:&mut Device, canvas:&Canvas, scene_descriptor:&DescriptorLayout, ) -> Result<Self, AAError> {
        let vert_module = pipeline::create_shader_module(device, constants::graph::MESH_VERT)?;
        let frag_module = pipeline::create_shader_module(device, constants::graph::MESH_FRAG)?;
        
        let push_constant_description = vk::PushConstantRange::builder()
            .size(GPUDrawPushConstants::size_u32())
            .stage_flags(vk::ShaderStageFlags::VERTEX);
        
        let mut layout_builder = DescriptorLayoutBuilder::create();
        layout_builder.add_binding(0, vk::DescriptorType::UNIFORM_BUFFER, 1);
        layout_builder.add_binding(1, vk::DescriptorType::COMBINED_IMAGE_SAMPLER, 1);
        layout_builder.add_binding(2, vk::DescriptorType::COMBINED_IMAGE_SAMPLER, 1);
        
        let (material_descriptor_layout, material_types_in_layout) = layout_builder.build(device, vk::ShaderStageFlags::FRAGMENT|vk::ShaderStageFlags::VERTEX)?;
        
        let descriptor_layouts = [scene_descriptor.underlying(), material_descriptor_layout.underlying()];
        
        let layout_ci = vk::PipelineLayoutCreateInfo::builder()
            .push_constant_ranges(from_ref(&push_constant_description))
            .set_layouts(&descriptor_layouts[..]);
        
        let pipeline_layout = unsafe{device.create_pipeline_layout(&layout_ci, None)}.unwrap();
        
        let opaque_pipeline = Self::create_opaque_pipeline(device, canvas, pipeline_layout, vert_module, frag_module)?;
        let transparent_pipeline = Self::create_transparent_pipeline(device, canvas, pipeline_layout, vert_module, frag_module)?;
        
        let descriptor_writer = DescriptorWriter::default();
        
        unsafe{device.destroy_shader_module(vert_module, None)};
        unsafe{device.destroy_shader_module(frag_module, None)};
        
        Ok(Self{
            opaque_pipeline,
            transparent_pipeline,
            pipeline_layout,
            descriptor_writer,
            descriptor_layout: material_descriptor_layout,
        })
    }
    
    pub fn clear_resources() {
        
    }
    
    pub fn create_opaque_pipeline(device:&mut Device, canvas:&Canvas, pipeline_layout:vk::PipelineLayout, vert_module:vk::ShaderModule, frag_module:vk::ShaderModule) -> Result<vk::Pipeline, AAError> {
        let mut builder = GPipelineBuilder::new();
        let (color_format, depth_format) = canvas.get_formats();
        
        builder.set_pipeline_layout(pipeline_layout)
            .set_shaders(vert_module, frag_module)
            .set_input_topology(vk::PrimitiveTopology::TRIANGLE_LIST)
            .set_polygon_mode(vk::PolygonMode::FILL)
            .set_cull_mode(vk::CullModeFlags::NONE, vk::FrontFace::CLOCKWISE)
            .set_multisampling_none()
            .set_blending_disabled()
            .set_depthtest_enable()
            .set_color_attachment_format(color_format)
            .set_depth_format(depth_format);
        
        builder.build_raw(device)
    }
    
    pub fn create_transparent_pipeline(device:&mut Device, canvas:&Canvas, pipeline_layout:vk::PipelineLayout, vert_module:vk::ShaderModule, frag_module:vk::ShaderModule) -> Result<vk::Pipeline, AAError> {
        let mut builder = GPipelineBuilder::new();
        let (color_format, depth_format) = canvas.get_formats();
        
        builder.set_pipeline_layout(pipeline_layout)
            .set_shaders(vert_module, frag_module)
            .set_input_topology(vk::PrimitiveTopology::TRIANGLE_LIST)
            .set_polygon_mode(vk::PolygonMode::FILL)
            .set_cull_mode(vk::CullModeFlags::NONE, vk::FrontFace::CLOCKWISE)
            .set_multisampling_none()
            .set_blending_additive()
            .set_depthtest_none()
            .set_color_attachment_format(color_format)
            .set_depth_format(depth_format);
        
        builder.build_raw(device)
    }
    
}
