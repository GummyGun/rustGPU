use crate::errors::messages::RESOURCE_FILE;
use crate::AAError;
use crate::constants;

use super::Device;

use std::fs::File;

use ash::vk;


pub fn create_shader_module(device:&mut Device, file:&str) -> Result<vk::ShaderModule, AAError> {
    let mut spv_file = File::open(file).unwrap();
    let spv = ash::util::read_spv(&mut spv_file).expect(RESOURCE_FILE);
    let create_info = vk::ShaderModuleCreateInfo::builder()
        .code(&spv);
    unsafe{device.create_shader_module(&create_info, None)}.map_err(|err|err.into())
    
}

pub fn create_shader_stage_create_info(stage:vk::ShaderStageFlags, module:vk::ShaderModule) -> vk::PipelineShaderStageCreateInfo {
    vk::PipelineShaderStageCreateInfo::builder()
        .stage(stage)
        .module(module)
        .name(constants::SHADER_START)
        .build()
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

