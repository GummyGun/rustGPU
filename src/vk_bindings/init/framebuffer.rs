use ash::{
    vk,
    prelude::VkResult,
};

use super::{
    DeviceDrop,
    Device,
    swapchain::Swapchain,
    render_pass::RenderPass,
};

use crate::{
    State
};

use std::{
    ops::Deref,
};



pub struct SCFramebuffers(Vec<vk::Framebuffer>);

impl SCFramebuffers {
    pub fn create(state:&State, device:&Device, swapchain:&Swapchain, render_pass:&RenderPass) -> VkResult<SCFramebuffers> {
        if  state.v_exp() {
            println!("\nCREATING:\tSWAPCHAIN");
        }
        
        let mut framebuffer_holder:Vec<vk::Framebuffer> = Vec::with_capacity(swapchain.image_views.len());
        
        for image_view in &swapchain.image_views {
            
            let attachments = [*image_view];
            let create_info = vk::FramebufferCreateInfo::builder()
                .render_pass(render_pass.as_inner())
                .attachments(&attachments[..])
                .width(swapchain.extent.width)
                .height(swapchain.extent.height)
                .layers(1);
            let holder = unsafe{device.create_framebuffer(&create_info, None)}?;
            framebuffer_holder.push(holder);
        }
        
        Ok(Self(framebuffer_holder))
    }
}

impl Deref for SCFramebuffers {
    type Target = Vec<vk::Framebuffer>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DeviceDrop for SCFramebuffers {
    fn device_drop(&mut self, state:&State, device:&Device) {
        if state.v_nor() {
            println!("[0]:deleting Swapchain framebuffers");
        }
        for framebuffer in self.iter() {
            unsafe{device.destroy_framebuffer(*framebuffer, None)};
        }
        
    }
}
