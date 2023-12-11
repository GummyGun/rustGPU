use ash::{
    vk,
    prelude::VkResult
};

use super::{
    DeviceDrop,
    device::Device,
    render_pass::RenderPass,
};

use crate::{
    State,
    graphics::Vertex,
};

use std::{
    fs::File,
    ffi::CStr,
};


pub struct Pipeline {
    pub layout: vk::PipelineLayout,
    pub pipeline: vk::Pipeline,
}

impl Pipeline {
    
    pub fn create(state:&State, device:&Device, render_pass:&RenderPass) -> VkResult<Self> {
        if  state.v_exp() {
            println!("\nCREATING:\tPIPELINE");
        }
        
        let frag_sm = Self::create_shader_module(state, device, "ssrc/sh.frag.spv").unwrap();
        let vert_sm = Self::create_shader_module(state, device, "ssrc/sh.vert.spv").unwrap();
        
        let shader_name = unsafe{CStr::from_bytes_with_nul_unchecked(b"main\0")};
        
        let sm_create_info = [
            vk::PipelineShaderStageCreateInfo::builder()
                .stage(vk::ShaderStageFlags::VERTEX)
                .module(vert_sm)
                .name(shader_name)
                .build(),
            vk::PipelineShaderStageCreateInfo::builder()
                .stage(vk::ShaderStageFlags::FRAGMENT)
                .module(frag_sm)
                .name(shader_name)
                .build()
        ];
        
        let dynamic_states = [vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR];
        
        let dynamic_state_create_info = vk::PipelineDynamicStateCreateInfo::builder()
            .dynamic_states(&dynamic_states);
        
        
        let vertex_input_state_create_info = vk::PipelineVertexInputStateCreateInfo::builder()
            .vertex_binding_descriptions(Vertex::binding_description())
            .vertex_attribute_descriptions(Vertex::attribute_description());
        
        let input_assembly_create_info = ash::vk::PipelineInputAssemblyStateCreateInfo::builder()
            .topology(vk::PrimitiveTopology::TRIANGLE_LIST)
            .primitive_restart_enable(false);
        
        let viewport_state_create_info = vk::PipelineViewportStateCreateInfo::builder()
            .viewport_count(1)
            .scissor_count(1);
        
        let rasterization_state_create_info = vk::PipelineRasterizationStateCreateInfo::builder()
            .depth_clamp_enable(false)
            .rasterizer_discard_enable(false)
            .polygon_mode(vk::PolygonMode::FILL)
            .line_width(1f32)
            .cull_mode(vk::CullModeFlags::BACK)
            .front_face(vk::FrontFace::CLOCKWISE)
            .depth_bias_enable(false);
            //.depth_bias_slope_factor(0f32);
        
        let multisample_state_create_info = vk::PipelineMultisampleStateCreateInfo::builder()
            .sample_shading_enable(false)
            .rasterization_samples(vk::SampleCountFlags::TYPE_1);
        
        if state.v_exp() {
            println!("enabling RGBA color attachment");
        }
        
        let color_blend_attachment = [
            vk::PipelineColorBlendAttachmentState::builder()
                .color_write_mask(
                    vk::ColorComponentFlags::R | 
                    vk::ColorComponentFlags::G | 
                    vk::ColorComponentFlags::B | 
                    vk::ColorComponentFlags::A
                )
                .build()
        ];
        
        let color_blend_state_create_info = vk::PipelineColorBlendStateCreateInfo::builder()
            .logic_op_enable(false)
            .logic_op(vk::LogicOp::COPY)
            .attachments(&color_blend_attachment[..]);
        
        let pipeline_layout_create_info = vk::PipelineLayoutCreateInfo::builder();
        
        let pipeline_layout = unsafe{device.create_pipeline_layout(&pipeline_layout_create_info, None)?};
        

        let create_info = [
            vk::GraphicsPipelineCreateInfo::builder()
                .stages(&sm_create_info[..])
                .vertex_input_state(&vertex_input_state_create_info)
                .input_assembly_state(&input_assembly_create_info)
                .viewport_state(&viewport_state_create_info)
                .rasterization_state(&rasterization_state_create_info)
                .multisample_state(&multisample_state_create_info)
                .color_blend_state(&color_blend_state_create_info)
                .dynamic_state(&dynamic_state_create_info)
                .layout(pipeline_layout)
                .render_pass(render_pass.as_inner())
                .base_pipeline_index(-1)
                .build()
        ];
        
        let pipeline_vec = match unsafe{device.create_graphics_pipelines(vk::PipelineCache::null(), &create_info[..], None)} {
            Ok(vec) => vec,
            Err(_) => {panic!();}
        };
        
        let pipeline = pipeline_vec[0];
        
        unsafe{device.destroy_shader_module(vert_sm, None)};
        unsafe{device.destroy_shader_module(frag_sm, None)};
        Ok(Self{
            layout:pipeline_layout,
            pipeline:pipeline,
        })
    }
    
    pub fn as_inner(&self) -> vk::Pipeline {
        self.pipeline
    }

    fn create_shader_module(state:&State, device:&Device, file:&str) -> VkResult<vk::ShaderModule> {
        if state.v_exp() {
            println!("creating shader module from {}", file);
        }
        let mut spv_file = File::open(file).unwrap();
        let spv = ash::util::read_spv(&mut spv_file).expect("should be in file");
        let create_info = vk::ShaderModuleCreateInfo::builder()
            .code(&spv);
        unsafe{device.create_shader_module(&create_info, None)}
    }
}

impl DeviceDrop for Pipeline {
    fn device_drop(&mut self, state:&State, device:&Device) {
        if state.v_nor() {
            println!("[0]:deleting pipeline");
        }
        unsafe{device.destroy_pipeline(self.pipeline, None)};
        
        if state.v_nor() {
            println!("[0]:deleting pipeline layout");
        }
        unsafe{device.destroy_pipeline_layout(self.layout, None)}
    }
}
