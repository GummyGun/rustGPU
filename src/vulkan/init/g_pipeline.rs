use crate::AAError;
use crate::macros;
use crate::constants;
use crate::logger;

use super::VkDestructorArguments;
use super::VkDestructor;
use super::Device;
use super::Image;
use super::pipeline;

use super::super::graphics as vk_graphics;

use std::slice::from_ref;

use arrayvec::ArrayVec;
use ash::vk;

pub struct GPipeline {
    pub layout: vk::PipelineLayout,
    pipeline: vk::Pipeline,
}

macros::impl_underlying!(GPipeline, vk::Pipeline, pipeline);

#[derive(Default, Debug)]
pub struct GPipelineBuilder {
    shader_stages: ArrayVec<vk::PipelineShaderStageCreateInfo, 16>,
    
    input_assembly: vk::PipelineInputAssemblyStateCreateInfo,
    rasterizer: vk::PipelineRasterizationStateCreateInfo,
    color_blend_attachment: vk::PipelineColorBlendAttachmentState,
    multisampling: vk::PipelineMultisampleStateCreateInfo,
    layout: Option<vk::PipelineLayout>,
    depth_stencil: vk::PipelineDepthStencilStateCreateInfo,
    rendering_ci: vk::PipelineRenderingCreateInfo,
    color_attachment_format: vk::Format,
    vertex_input_state:vk::PipelineVertexInputStateCreateInfo,
}


/*
pub fn init_pipeline(device:&mut Device, render_image:&Image, depth_image:&Image) -> GPipeline {
    
    logger::various_log!("graphics_pipeline",
        (logger::Warn, "Instancing simple triangle graphics pipeline")
    );
    
    let vert_shader = pipeline::create_shader_module(device, constants::graph::TRIANGLE_VERT).unwrap();
    let frag_shader = pipeline::create_shader_module(device, constants::graph::TRIANGLE_FRAG).unwrap();
    
    let layout_ci = vk::PipelineLayoutCreateInfo::builder();
    
    let layout = unsafe{device.create_pipeline_layout(&layout_ci, None)}.unwrap();
    
    let mut builder = GPipelineBuilder::new();
    builder.set_pipeline_layout(layout)
        .set_shaders(vert_shader, frag_shader)
        .set_input_topology(vk::PrimitiveTopology::TRIANGLE_LIST)
        .set_polygon_mode(vk::PolygonMode::FILL)
        .set_cull_mode(vk::CullModeFlags::NONE, vk::FrontFace::CLOCKWISE)
        .set_multisampling_none()
        .set_blending_disabled()
        .set_color_attachment_format(render_image.format)
        .set_depth_format(depth_image.format)
        //.set_depthtest_enable();
        .set_depthtest_none();
    
    let triangle_pipeline = builder.build(device);
    
    unsafe{device.destroy_shader_module(vert_shader, None)};
    unsafe{device.destroy_shader_module(frag_shader, None)};
    
    triangle_pipeline.unwrap()
}
*/

pub fn init_mesh_pipeline(device:&mut Device, render_image:&Image, depth_image:&Image) -> GPipeline {
    
    logger::various_log!("graphics_pipeline",
        (logger::Warn, "Instancing mesh pipeline")
    );
    
    let vert_shader = pipeline::create_shader_module(device, constants::graph::MESH_VERT).unwrap();
    let frag_shader = pipeline::create_shader_module(device, constants::graph::TRIANGLE_FRAG).unwrap();
    
    
    let push_constant_description = vk::PushConstantRange::builder()
        .size(vk_graphics::GPUDrawPushConstants::size_u32())
        .stage_flags(vk::ShaderStageFlags::VERTEX);
    
    
    let layout_ci = vk::PipelineLayoutCreateInfo::builder()
        .push_constant_ranges(from_ref(&push_constant_description));
    
    let layout = unsafe{device.create_pipeline_layout(&layout_ci, None)}.unwrap();
    
    let mut builder = GPipelineBuilder::new();
    builder.set_pipeline_layout(layout)
        .set_shaders(vert_shader, frag_shader)
        .set_input_topology(vk::PrimitiveTopology::TRIANGLE_LIST)
        .set_polygon_mode(vk::PolygonMode::FILL)
        .set_cull_mode(vk::CullModeFlags::NONE, vk::FrontFace::CLOCKWISE)
        .set_multisampling_none()
        .set_blending_disabled()
        .set_color_attachment_format(render_image.format)
        .set_depth_format(depth_image.format)
        .set_depthtest_enable()
        .set_blending_additive();
        //.set_vertex_input_state(vk_graphics::Vertex::binding_description(), vk_graphics::Vertex::attribute_description());
    
    let mesh_pipeline = builder.build(device);
    
    unsafe{device.destroy_shader_module(vert_shader, None)};
    unsafe{device.destroy_shader_module(frag_shader, None)};
    
    mesh_pipeline .unwrap()
}

impl GPipelineBuilder {
    
//----
    pub fn new() -> Self {
        //logger::creating_builder();
        
        Self::default()
    }
    
//----
    pub fn build(mut self, device:&mut Device) -> Result<GPipeline, AAError> {
        
        logger::create!("graphics_pipeline");
        logger::various_log!("graphics_pipeline",
            (logger::Trace, "{:#?}", &self)
        );
        
        let layout = match self.layout {
            Some(layout) => layout,
            None => {return Err(AAError::LayoutNotSet)}
        };
        
        let viewport_state = vk::PipelineViewportStateCreateInfo::builder()
            .viewport_count(1)
            .scissor_count(1);
        
        let color_blend_sci = vk::PipelineColorBlendStateCreateInfo::builder()
            .logic_op_enable(false)
            .logic_op(vk::LogicOp::COPY)
            .attachments(from_ref(&self.color_blend_attachment));
        
        let dynamic_state = [vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR];
        let dynamic_sci = vk::PipelineDynamicStateCreateInfo::builder()
            .dynamic_states(&dynamic_state[..]);
        
        let create_info = vk::GraphicsPipelineCreateInfo::builder()
            .push_next(&mut self.rendering_ci)
            .vertex_input_state(&self.vertex_input_state)
            .viewport_state(&viewport_state)
            .stages(&self.shader_stages[..])
            .input_assembly_state(&self.input_assembly)
            .rasterization_state(&self.rasterizer)
            .multisample_state(&self.multisampling)
            .color_blend_state(&color_blend_sci)
            .depth_stencil_state(&self.depth_stencil)
            .layout(layout)
            .dynamic_state(&dynamic_sci);
        
        let pipeline_holder = unsafe{device.create_graphics_pipelines(vk::PipelineCache::null(), from_ref(&create_info), None)}.unwrap();
        
        Ok(GPipeline{
            layout,
            pipeline: pipeline_holder[0],
        })
        
    }
    
//----
    pub fn clear(&mut self) -> &mut Self {
        *self = Self::default();
        self
    }
    
//----
    pub fn set_shaders(&mut self, vertex_shader:vk::ShaderModule, fragment_shader:vk::ShaderModule) -> &mut Self {
        self.shader_stages.clear();
        self.shader_stages.push(
            pipeline::create_shader_stage_create_info(vk::ShaderStageFlags::VERTEX, vertex_shader)
        );
        self.shader_stages.push(
            pipeline::create_shader_stage_create_info(vk::ShaderStageFlags::FRAGMENT, fragment_shader)
        );
        self
    }
    
//----
    pub fn set_input_topology(&mut self, topology:vk::PrimitiveTopology) -> &mut Self {
        self.input_assembly.topology = topology;
        self.input_assembly.primitive_restart_enable = vk::FALSE;
        self
    }
    
//----
    pub fn set_polygon_mode(&mut self, mode:vk::PolygonMode) -> &mut Self {
        self.rasterizer.polygon_mode = mode;
        self.rasterizer.line_width = 1f32;
        self
    }
    
//----
    pub fn set_cull_mode(&mut self, mode:vk::CullModeFlags, face:vk::FrontFace) -> &mut Self {
        self.rasterizer.cull_mode = mode;
        self.rasterizer.front_face = face;
        self
    }
    
//----
    pub fn set_multisampling_none(&mut self) -> &mut Self {
        let Self{ multisampling, .. } = self;
        multisampling.sample_shading_enable = vk::FALSE;
        multisampling.rasterization_samples = vk::SampleCountFlags::TYPE_1;
        multisampling.min_sample_shading = 1f32;
        multisampling.alpha_to_coverage_enable = vk::FALSE;
        multisampling.alpha_to_one_enable = vk::FALSE;
        self
    }
    
//----
    pub fn set_blending_disabled(&mut self) -> &mut Self {
        self.color_blend_attachment.color_write_mask = vk::ColorComponentFlags::RGBA;
        self.color_blend_attachment.blend_enable = vk::FALSE;
        self
    }
    
//----
    pub fn set_color_attachment_format(&mut self, format:vk::Format) -> &mut Self {
        self.color_attachment_format = format;
        self.rendering_ci.p_color_attachment_formats = &self.color_attachment_format as *const _;
        self.rendering_ci.color_attachment_count = 1;
        self
    }
    
//----
    pub fn set_depth_format(&mut self, format:vk::Format) -> &mut Self {
        self.rendering_ci.depth_attachment_format = format;
        self
    }
    
//----
    pub fn set_depthtest_none(&mut self) -> &mut Self {
        let Self{ depth_stencil, .. } = self;
        depth_stencil.depth_test_enable = vk::FALSE;
        depth_stencil.depth_write_enable = vk::FALSE;
        depth_stencil.depth_compare_op = vk::CompareOp::NEVER;
        depth_stencil.depth_bounds_test_enable = vk::FALSE;
        depth_stencil.stencil_test_enable = vk::FALSE;
        depth_stencil.front = vk::StencilOpState::default();
        depth_stencil.back = vk::StencilOpState::default();
        depth_stencil.min_depth_bounds = 0f32;
        depth_stencil.max_depth_bounds = 1f32;
        self
    }

    pub fn set_depthtest_enable(&mut self) -> &mut Self {
        let Self{ depth_stencil, .. } = self;
        depth_stencil.depth_test_enable = vk::TRUE;
        depth_stencil.depth_write_enable = vk::TRUE;
        depth_stencil.depth_compare_op = vk::CompareOp::GREATER_OR_EQUAL;
        depth_stencil.depth_bounds_test_enable = vk::FALSE;
        depth_stencil.stencil_test_enable = vk::FALSE;
        depth_stencil.front = vk::StencilOpState::default();
        depth_stencil.back = vk::StencilOpState::default();
        depth_stencil.min_depth_bounds = 0f32;
        depth_stencil.max_depth_bounds = 1f32;
        self
    }
    
//----
    pub fn set_vertex_input_state(&mut self, input_binding:&[vk::VertexInputBindingDescription], vertex_attribute:&[vk::VertexInputAttributeDescription]) -> &mut Self {
        let tmp = vk::PipelineVertexInputStateCreateInfo::builder()
            .vertex_binding_descriptions(input_binding)
            .vertex_attribute_descriptions(vertex_attribute);
        self.vertex_input_state.clone_from(&tmp);
        self
    }
    
    
//----
    pub fn set_pipeline_layout(&mut self, layout:vk::PipelineLayout) -> &mut Self {
        self.layout = Some(layout);
        self
    }

//----
    pub fn set_blending_additive(&mut self) -> &mut Self {
        let Self{ color_blend_attachment, .. } = self;
        color_blend_attachment.color_write_mask = vk::ColorComponentFlags::RGBA;
        color_blend_attachment.blend_enable = vk::TRUE;
        color_blend_attachment.src_color_blend_factor = vk::BlendFactor::ONE;
        color_blend_attachment.dst_color_blend_factor = vk::BlendFactor::DST_ALPHA;
        color_blend_attachment.color_blend_op = vk::BlendOp::ADD;
        color_blend_attachment.src_alpha_blend_factor = vk::BlendFactor::ONE;
        color_blend_attachment.dst_alpha_blend_factor = vk::BlendFactor::ZERO;
        color_blend_attachment.alpha_blend_op = vk::BlendOp::ADD;
        self
    }
    
//----
    pub fn set_blending_alphablend(&mut self) -> &mut Self {
        let Self{ color_blend_attachment, .. } = self;
        color_blend_attachment.color_write_mask = vk::ColorComponentFlags::RGBA;
        color_blend_attachment.blend_enable = vk::TRUE;
        color_blend_attachment.src_color_blend_factor = vk::BlendFactor::ONE_MINUS_DST_ALPHA;
        color_blend_attachment.dst_color_blend_factor = vk::BlendFactor::DST_ALPHA;
        color_blend_attachment.color_blend_op = vk::BlendOp::ADD;
        color_blend_attachment.src_alpha_blend_factor = vk::BlendFactor::ONE;
        color_blend_attachment.dst_alpha_blend_factor = vk::BlendFactor::ZERO;
        color_blend_attachment.alpha_blend_op = vk::BlendOp::ADD;
        self
    }
    
}


impl VkDestructor for GPipeline {
    fn destruct(self, mut args:VkDestructorArguments) {
        let device = args.unwrap_dev();
        logger::destruct!("graphics_pipeline");
        unsafe{device.destroy_pipeline_layout(self.layout, None)}
        unsafe{device.destroy_pipeline(self.pipeline, None)};
    }
}

