use super::*;

use crate::logger;
use crate::errors::messages::RESOURCE_REFERENCED;

use super::VkDestructor;
use super::VkDestructorArguments;

use std::rc::Rc;

pub struct MetalicMaterial {
    opaque_pipeline: vk::Pipeline,
    transparent_pipeline: vk::Pipeline,
    pipeline_layout: vk::PipelineLayout,
    descriptor_writer: DescriptorWriter,
    descriptor_layout: DescriptorLayout,
    
    dispatchable_opaque: Rc<DispatchableGPipeline>,
    dispatchable_transparent: Rc<DispatchableGPipeline>,
}


impl MetalicMaterial {
    
    pub fn write_material(
        &mut self, 
        device: &mut Device, 
        descriptor_allocator: &mut GDescriptorAllocator, 
        material_pass: MaterialPass, 
        material_resources: &MaterialResources
    ) -> Result<MaterialInstance, AAError> {
        
        let pipeline_holder = match material_pass {
            MaterialPass::MainColor => {
                self.dispatchable_opaque.clone()
            }
            MaterialPass::Transparent => {
                self.dispatchable_transparent.clone()
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
    
    pub fn build_pipelines(device:&mut Device, canvas:&Canvas, scene_descriptor:&DescriptorLayout) -> Result<Self, AAError> {
        logger::create!("metalic_material");
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
        
        let dispatchable_opaque = Rc::new(DispatchableGPipeline{
            pipeline: opaque_pipeline,
            layout: pipeline_layout,
        });
        
        let dispatchable_transparent = Rc::new(DispatchableGPipeline{
            pipeline: transparent_pipeline,
            layout: pipeline_layout,
        });
        
        Ok(Self{
            opaque_pipeline,
            transparent_pipeline,
            pipeline_layout,
            descriptor_writer,
            descriptor_layout: material_descriptor_layout,
            
            dispatchable_opaque,
            dispatchable_transparent,
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
    

    fn internal_destroy(mut self, device: &mut Device) {
        logger::destruct!("metalic_material");
        unsafe{device.destroy_pipeline(self.opaque_pipeline, None)};
        unsafe{device.destroy_pipeline(self.transparent_pipeline, None)};
        unsafe{device.destroy_pipeline_layout(self.pipeline_layout, None)};
        self.descriptor_layout.destruct(VkDestructorArguments::Dev(device));
        
        let count = Rc::strong_count(&self.dispatchable_opaque);
        if count != 1 {
            panic!("{}: {} times", RESOURCE_REFERENCED, count);
        }
        
        let count = Rc::strong_count(&self.dispatchable_transparent);
        if count != 1 {
            panic!("{}: {} times", RESOURCE_REFERENCED, count);
        }
        
    }
}


impl VkDestructor for MetalicMaterial {
    fn destruct(self, mut args:VkDestructorArguments) {
        let device = args.unwrap_dev();
        self.internal_destroy(device);
    }
}

