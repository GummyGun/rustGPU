use crate::AAError;
use crate::errors::messages::GRANTED;
use crate::constants;

use super::logger::pipeline as logger;

use super::VkDestructorArguments;
use super::VkDestructor;
use super::Device;
use super::DescriptorLayout;

use std::fs::File;
use std::slice::from_ref;
use std::ffi::CStr;

use ash::vk;


pub struct ComputePipeline {
    pub layout: vk::PipelineLayout,
    pub pipeline: vk::Pipeline,
}

pub fn init_pipeline(device:&mut Device, ds_layout:&DescriptorLayout) -> ComputePipeline {
    logger::init();
    let pipeline_name = unsafe{CStr::from_bytes_with_nul_unchecked(b"main\0")};
    ComputePipeline::create(device, ds_layout, constants::comp::COMP_SHADER, constants::comp::COMP_START).unwrap()
}


impl ComputePipeline {
    pub fn create(device:&mut Device, ds_layout:&DescriptorLayout, file:&str, name:&CStr) -> Result<Self, AAError> {
        logger::compute::create(true);
        let layout_create_info = vk::PipelineLayoutCreateInfo::builder()
            .set_layouts(from_ref(ds_layout));
        let layout = unsafe{device.create_pipeline_layout(&layout_create_info, None)}?;
        
        let compute_module = Self::create_shader_module(device, file)?;
        
        let compute_shader_stage = vk::PipelineShaderStageCreateInfo::builder()
            .stage(vk::ShaderStageFlags::COMPUTE)
            .module(compute_module)
            .name(name)
            .build();
        
            //.name(unsafe{CStr::from_bytes_with_nul_unchecked(b"background\0")});
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

