use crate::AAError;
use crate::errors::messages::GRANTED;
use crate::constants;

use super::logger::pipeline as logger;

use super::super::graphics;
use super::super::ComputeEffect;
use super::super::graphics::ComputePushConstants;

use super::VkDestructorArguments;
use super::VkDestructor;
use super::Device;
use super::DescriptorLayout;

use std::fs::File;
use std::slice::from_ref;
use std::ffi::CStr;

use ash::vk;
use nalgebra::Vector4;
use arrayvec::ArrayString;


#[derive(Clone)]
pub struct ComputePipeline {
    pub layout: vk::PipelineLayout,
    pub pipeline: vk::Pipeline,
}

/*
pub fn init_pipeline(device:&mut Device, ds_layout:&DescriptorLayout) -> ComputePipeline {
    logger::init();
    ComputePipeline::create(device, ds_layout, constants::comp::COMP_SHADER, constants::comp::COMP_START).unwrap()
}
*/

pub fn init_pipelines(device:&mut Device, ds_layout:&DescriptorLayout) -> Vec<ComputeEffect> {
    logger::init();
    let mut holder = Vec::new();
    let gradient = ComputePipeline::create(device, ds_layout, constants::comp::GRADIENT_SHADER, constants::comp::COMP_START).unwrap();
    let mut effect_name = ArrayString::new();
    effect_name.push_str("gradient");
    holder.push(ComputeEffect{
        name: effect_name,
        pipeline: gradient,
        data: ComputePushConstants{
            data1: Vector4::new(1.0,0.0,0.0,1.0),
            data2: Vector4::new(1.0,0.0,1.0,1.0),
            ..Default::default()
        },
    });
    
    let sky = ComputePipeline::create(device, ds_layout, constants::comp::SKY_SHADER, constants::comp::COMP_START).unwrap();
    effect_name.clear();
    effect_name.push_str("sky");
    holder.push(ComputeEffect{
        name: effect_name,
        pipeline: sky,
        data: ComputePushConstants{
            data1: Vector4::new(0.1, 0.2, 0.4 ,0.97),
            ..Default::default()
        },
    });
    
    holder
}



impl ComputePipeline {
    pub fn create(device:&mut Device, ds_layout:&DescriptorLayout, file:&str, name:&CStr) -> Result<Self, AAError> {
        logger::compute::create(true);
        
        let push_constant_description = vk::PushConstantRange::builder()
            .size(graphics::ComputePushConstants::size_u32())
            .stage_flags(vk::ShaderStageFlags::COMPUTE);
        
        
        let layout_create_info = vk::PipelineLayoutCreateInfo::builder()
            .set_layouts(from_ref(ds_layout))
            .push_constant_ranges(from_ref(&push_constant_description));
        
        let layout = unsafe{device.create_pipeline_layout(&layout_create_info, None)}?;
        
        let compute_module = Self::create_shader_module(device, file)?;
        
        let compute_shader_stage = vk::PipelineShaderStageCreateInfo::builder()
            .stage(vk::ShaderStageFlags::COMPUTE)
            .module(compute_module)
            .name(name)
            .build();
        let compute_pipeline_create_info = vk::ComputePipelineCreateInfo::builder()
            .layout(layout)
            .stage(compute_shader_stage);
        
        let pipeline = match unsafe{device.create_compute_pipelines(vk::PipelineCache::null(), from_ref(&compute_pipeline_create_info), None)} {
            Ok(mut pipeline) => {
                pipeline.pop().expect(GRANTED)
            }
            Err(_) => {
                panic!("Error creating compute pipeline");
            }
        };
        
        unsafe{device.destroy_shader_module(compute_module, None)};
        
        Ok(Self{
            layout,
            pipeline
        })
    }
    
    
    fn create_shader_module(device:&mut Device, file:&str) -> Result<vk::ShaderModule, AAError> {
        /*
        if state.v_exp() {
            println!("creating shader module from {}", file);
        }
        */
        let mut spv_file = File::open(file).unwrap();
        let spv = ash::util::read_spv(&mut spv_file).expect("should be in file");
        let create_info = vk::ShaderModuleCreateInfo::builder()
            .code(&spv);
        unsafe{device.create_shader_module(&create_info, None)}.map_err(|err|err.into())
        
    }
}

impl VkDestructor for ComputePipeline {
    fn destruct(self, mut args:VkDestructorArguments) {
        let device = args.unwrap_dev();
        logger::compute::destruct();
        unsafe{device.destroy_pipeline_layout(self.layout, None)}
        unsafe{device.destroy_pipeline(self.pipeline, None)};
    }
}


pub fn rendering_attachment_info(
    view: vk::ImageView,
    clear_value: Option<vk::ClearValue>,
    layout: vk::ImageLayout,
) -> vk::RenderingAttachmentInfo {
    let mut holder = ash::vk::RenderingAttachmentInfo::default();
    holder.image_view = view;
    holder.image_layout = layout;
    holder.store_op = vk::AttachmentStoreOp::STORE;
    match clear_value{
        Some(clear) => {
            holder.load_op = vk::AttachmentLoadOp::CLEAR;
            holder.clear_value = clear;
        }
        None => {
            holder.load_op = vk::AttachmentLoadOp::LOAD;
        }
    }
    holder
}

pub fn rendering_info(
    extent: vk::Extent2D,
    color_attachment: &vk::RenderingAttachmentInfo,
    depth_attachment: Option<&vk::RenderingAttachmentInfo>,
) -> vk::RenderingInfo {
    let mut holder = vk::RenderingInfo::builder()
        .render_area(vk::Rect2D::from(extent))
        .layer_count(1)
        .color_attachments(std::slice::from_ref(color_attachment));
    match depth_attachment {
        Some(att) => {
            holder = holder.depth_attachment(att);
        }
        None => {}
    }
    holder.build()
}

