use crate::AAError;
use crate::constants;
use crate::logger;
use crate::errors::messages::GRANTED;

use super::super::graphics as vk_graphics;
use vk_graphics::ComputePushConstants;
use vk_graphics::ComputeEffectMetadata;

use super::VkDestructorArguments;
use super::VkDestructor;
use super::Device;
use super::DescriptorLayout;
use super::pipeline;

use std::slice::from_ref;

use ash::vk;
use nalgebra::Vector4;
use arrayvec::ArrayString;
use derivative::Derivative;


#[derive(Clone)]
pub struct CPipeline {
    pub layout: vk::PipelineLayout,
    pub pipeline: vk::Pipeline,
}

#[derive(Derivative)]
#[derivative(Debug)]
pub struct ComputeEffects {
    pub metadatas: Vec<ComputeEffectMetadata>,
    #[derivative(Debug="ignore")]
    pub pipelines: Vec<CPipeline>,
}



pub fn init_pipelines(device:&mut Device, ds_layout:&DescriptorLayout) -> ComputeEffects {
    
    logger::various_log!("compute_pipeline",
        (logger::Warn, "Instancing simple compute effects pipeline")
    );
    
    let mut metadatas = Vec::new();
    let mut pipelines = Vec::new();
    let mut effect_name = ArrayString::new();
    
    let gradient = CPipeline::create(device, ds_layout, constants::comp::GRADIENT_SHADER).unwrap();
    effect_name.push_str("gradient");
    logger::various_log!("compute_pipeline",
        (logger::Warn, "Instancing {} compute pipeline", effect_name)
    );
    metadatas.push(ComputeEffectMetadata{
        name:effect_name,
        data: ComputePushConstants([
            Vector4::new(0.4,0.4,0.4,1.0),
            Vector4::new(0.4,0.4,0.4,1.0),
            Vector4::new(0.0,0.0,0.0,0.0),
            Vector4::new(0.0,0.0,0.0,0.0),
        ]),
    });
    pipelines.push(gradient);
    effect_name.clear();
    
    
    let gradient = CPipeline::create(device, ds_layout, constants::comp::COMP_SHADER).unwrap();
    effect_name.push_str("square fade");
    logger::various_log!("compute_pipeline",
        (logger::Warn, "Instancing {} compute pipeline", effect_name)
    );
    metadatas.push(ComputeEffectMetadata{
        name:effect_name,
        data: ComputePushConstants([
            Vector4::new(0.0,0.0,0.0,0.0),
            Vector4::new(0.0,0.0,0.0,0.0),
            Vector4::new(0.0,0.0,0.0,0.0),
            Vector4::new(0.0,0.0,0.0,0.0),
        ]),
    });
    pipelines.push(gradient);
    effect_name.clear();
    
    
    effect_name.push_str("sky 2.0");
    logger::various_log!("compute_pipeline",
        (logger::Warn, "Instancing {} compute pipeline", effect_name)
    );
    let sky = CPipeline::create(device, ds_layout, constants::comp::SKY_SHADER).unwrap();
    metadatas.push(ComputeEffectMetadata{
        name:effect_name,
        data: ComputePushConstants([
            //Vector4::new(0.1, 0.2, 0.4 ,0.97),
            Vector4::new(0.0,0.0,0.0,0.0),
            Vector4::new(0.0,0.0,0.0,0.0),
            Vector4::new(0.0,0.0,0.0,0.0),
            Vector4::new(0.0,0.0,0.0,0.0),
        ]),
    });
    pipelines.push(sky);
    
    
    ComputeEffects{
        metadatas,
        pipelines,
    }
}


impl VkDestructor for ComputeEffects {
    fn destruct(self, mut args:VkDestructorArguments) {
        logger::destruct!("compute_effects");
        let device = args.unwrap_dev();
        for pipeline in self.pipelines.into_iter() {
            pipeline.destruct(VkDestructorArguments::Dev(device));
        }
    }
}


impl CPipeline {
    pub fn create(device:&mut Device, ds_layout:&DescriptorLayout, file:&str) -> Result<Self, AAError> {
        logger::create!("compute_pipeline");
        
        let push_constant_description = vk::PushConstantRange::builder()
            .size(vk_graphics::ComputePushConstants::size_u32())
            .stage_flags(vk::ShaderStageFlags::COMPUTE);
        
        
        let layout_create_info = vk::PipelineLayoutCreateInfo::builder()
            .set_layouts(from_ref(ds_layout))
            .push_constant_ranges(from_ref(&push_constant_description));
        
        let layout = unsafe{device.create_pipeline_layout(&layout_create_info, None)}?;
        
        let compute_module = pipeline::create_shader_module(device, file)?;
        let compute_shader_stage = pipeline::create_shader_stage_create_info(vk::ShaderStageFlags::COMPUTE, compute_module);
        
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
    
    
}


impl VkDestructor for CPipeline {
    fn destruct(self, mut args:VkDestructorArguments) {
        let device = args.unwrap_dev();
        logger::destruct!("command_pipeline");
        unsafe{device.destroy_pipeline_layout(self.layout, None)}
        unsafe{device.destroy_pipeline(self.pipeline, None)};
    }
}

