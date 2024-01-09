use ash::{
    vk,
    prelude::VkResult
};

use super::{
    DeviceDestroy,
    device::Device,
    swapchain::SwapchainBasic,
    depth_buffer::DepthBuffer,
};

use crate::{
    State,
};

use std::{
    slice::from_ref,
};


pub struct RenderPass(vk::RenderPass);

impl RenderPass {
    pub fn create(
        state:&State, 
        device:&Device, 
        swapchain:&SwapchainBasic, 
        depth:&DepthBuffer
    ) -> VkResult<Self> {
        
        use vk::PipelineStageFlags as PSF;
        use vk::AccessFlags as AF;
        
        
        if state.v_exp() {
            println!("\nCREATING:\tRENDER PASS");
        }
        
        let subpass_dependency = vk::SubpassDependency::builder()
            .src_subpass(vk::SUBPASS_EXTERNAL)
            .dst_subpass(0)
            .src_access_mask(AF::empty())
            .src_stage_mask(PSF::COLOR_ATTACHMENT_OUTPUT | PSF::EARLY_FRAGMENT_TESTS)
            .dst_access_mask(AF::COLOR_ATTACHMENT_WRITE | AF::DEPTH_STENCIL_ATTACHMENT_WRITE)
            .dst_stage_mask(PSF::COLOR_ATTACHMENT_OUTPUT | PSF::EARLY_FRAGMENT_TESTS);
        
        if state.v_exp() {
            println!("initial layout:\tundefined");
            println!("final layout:  \tpresent");
        }
        
        
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
                .build(),
            vk::AttachmentDescription::builder()
                .format(depth.format)
                .final_layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL)
                .samples(vk::SampleCountFlags::TYPE_1)
                .load_op(vk::AttachmentLoadOp::CLEAR)
                .store_op(vk::AttachmentStoreOp::DONT_CARE)
                .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
                .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
                .initial_layout(vk::ImageLayout::UNDEFINED)
                .final_layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL)
                .build(),
        ];
        
        
        let color_attachment_reference = vk::AttachmentReference::builder()
            .attachment(0)
            .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL);
        
        let depth_attachment_reference = vk::AttachmentReference::builder()
            .attachment(1)
            .layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL);
        
        let subpass_description = vk::SubpassDescription::builder()
            .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
            .color_attachments(from_ref(&color_attachment_reference))
            .depth_stencil_attachment(&depth_attachment_reference);
        
        let create_info = vk::RenderPassCreateInfo::builder()
            .attachments(&attachment_description[..])
            .subpasses(from_ref(&subpass_description))
            .dependencies(from_ref(&subpass_dependency));
        
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

impl DeviceDestroy for RenderPass {
    fn device_drop(&mut self, state:&State, device:&Device) {
        if state.v_nor() {
            println!("[0]:deleting render pass");
        }
        unsafe{device.destroy_render_pass(self.0, None)}
    }
}

