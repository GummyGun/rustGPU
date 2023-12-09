use ash::{
    vk,
    prelude::VkResult
};

use super::{
    DeviceDrop,
    device::Device,
    swapchain::SwapchainBasic,
    /*
    instance::Instance,
    surface::Surface,
    p_device::PhysicalDevice,
    */
};

use crate::{
    State,
};


pub struct RenderPass(vk::RenderPass);

impl RenderPass {
    pub fn create(state:&State, device:&Device, swapchain:&SwapchainBasic) -> VkResult<Self> {
        if state.v_exp() {
            println!("\nCREATING:\tRENDER PASS");
        }
        
        let subpass_dependency = [
            vk::SubpassDependency::builder()
                .src_subpass(vk::SUBPASS_EXTERNAL)
                .dst_subpass(0)
                .src_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
                .src_access_mask(vk::AccessFlags::empty())
                .dst_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
                .dst_access_mask(vk::AccessFlags::COLOR_ATTACHMENT_WRITE)
                .build()
        ];
        
        let attachment_description = [
            vk::AttachmentDescription::builder()
                .format(swapchain.surface_format.format)
                .samples(vk::SampleCountFlags::TYPE_1)
                .load_op(vk::AttachmentLoadOp::CLEAR)
                .store_op(vk::AttachmentStoreOp::STORE)
                .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
                .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
                .initial_layout(vk::ImageLayout::UNDEFINED)
                .final_layout(vk::ImageLayout::PRESENT_SRC_KHR)
                .build()
        ];
            
        
        let color_attachment_reference = [
            vk::AttachmentReference::builder()
                .attachment(0)
                .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
                .build()
        ];
        
        let subpass_description = [
            vk::SubpassDescription::builder()
                .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
                .color_attachments(&color_attachment_reference[..])
            .build()
        ];
        
        let create_info = vk::RenderPassCreateInfo::builder()
            .attachments(&attachment_description[..])
            .subpasses(&subpass_description[..])
            .dependencies(&subpass_dependency[..]);
        
        let render_pass = unsafe{device.create_render_pass(&create_info, None)?};
        
        Ok(Self(render_pass))
    }
    
    pub fn as_inner(&self) -> vk::RenderPass {
        self.0
    }
}

/*
impl Deref for RenderPass {
    type Target = vk::RenderPass;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
*/

impl DeviceDrop for RenderPass {
    fn device_drop(&mut self, state:&State, device:&Device) {
        if state.v_nor() {
            println!("[0]:deleting render pass");
        }
        unsafe{device.destroy_render_pass(self.0, None)}
    }
}

