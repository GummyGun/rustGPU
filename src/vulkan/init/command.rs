use ash::{
    vk,
    prelude::VkResult,
};

use super::{
    DeviceDestroy,
    device::Device,
    p_device::PDevice,
    swapchain::Swapchain,
    render_pass::RenderPass,
    pipeline::Pipeline,
    descriptor::DescriptorControl,
    
};

use super::super::{
    Model,
};

use crate::{
    State,
    constants,
};

use std::slice::from_ref;

pub struct CommandControl{
    pub pool: vk::CommandPool,
    pub buffer: [vk::CommandBuffer; constants::fif::USIZE],
    s_u_buffer: vk::CommandBuffer,
}


impl CommandControl {
    pub fn create(state:&State, p_device:&PDevice, device:&Device) -> VkResult<Self> {
        use constants::fif;
        
        if state.v_exp() {
            println!("\nCREATING:\tCOMMAND CONTROL STRUCTURES");
        }
        let create_info = vk::CommandPoolCreateInfo::builder()
            .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
            .queue_family_index(p_device.queues.graphics_family);
        let command_pool = unsafe{device.create_command_pool(&create_info, None)}?;
        
        if state.v_exp() {
            println!("\nALLOCATING:\tCOMMAND BUFFERS");
            println!("allocating graphics command buffers");
        }
        let create_info = vk::CommandBufferAllocateInfo::builder()
            .command_pool(command_pool)
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_buffer_count(fif::U32);
        
        let buffer_vec = unsafe{device.allocate_command_buffers(&create_info)}?;
        
        if state.v_exp() {
            println!("allocating staging command buffer");
        }
        let sb_create_info = vk::CommandBufferAllocateInfo::builder()
            .command_pool(command_pool)
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_buffer_count(1);
        
        let s_u_buffer = unsafe{device.allocate_command_buffers(&sb_create_info)}?;
        
        let mut buffer_arr = [vk::CommandBuffer::null(); fif::USIZE];
        for (index, buffer) in buffer_vec.into_iter().enumerate() {
            buffer_arr[index] = buffer;
        }
        
        Ok(Self{
            pool: command_pool,
            buffer: buffer_arr,
            s_u_buffer: s_u_buffer[0],
        })
    }
    
    pub fn setup_su_buffer(&self, device:&Device) -> vk::CommandBuffer {
        
        unsafe{device.reset_command_buffer(self.s_u_buffer, vk::CommandBufferResetFlags::empty())}.expect("reseting buffer should not fail");
        
        let begin_info = ash::vk::CommandBufferBeginInfo::builder()
            .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);
        
        unsafe{device.begin_command_buffer(self.s_u_buffer, &begin_info)}.expect("should not fail");
        
        self.s_u_buffer
    }
    
    pub fn submit_su_buffer(&self, device:&Device) {
        
        unsafe{device.end_command_buffer(self.s_u_buffer)}.expect("should not fail");
        
        let submit_info = [
            vk::SubmitInfo::builder()
                .command_buffers(from_ref(&self.s_u_buffer))
                .build(),
        ];
        
        unsafe{device.queue_submit(device.queue_handles.graphics, &submit_info[..], vk::Fence::null())}.expect("should not fail");
        unsafe{device.device_wait_idle()}.expect("waiting for iddle should not fail");
    }
    
    pub fn record_command_buffer(
        &mut self, 
        state:&State, 
        device:&Device, 
        swapchain:&Swapchain, 
        render_pass:&RenderPass, 
        pipeline:&Pipeline, 
        //vertex_buffer:&Buffer, 
        //index_buffer:&Buffer,
        descriptor_control:&DescriptorControl,
        image_index:u32, 
        frame_index:usize,
        //triangles_to_draw:u32,
        model_vec:&[Model],
    ) {
        
        if state.v_dmp() {
            println!("\nFILLING:\tCOMMAND BUFFER");
        }
        let image_index_usize = usize::try_from(image_index).unwrap();
        
        let command_buffer_begin = vk::CommandBufferBeginInfo::builder();
        
        unsafe{device.begin_command_buffer(self.buffer[frame_index], &command_buffer_begin)}.unwrap();
        
        let scissor = vk::Rect2D::builder()
                .offset(*vk::Offset2D::builder().x(0).y(0))
                .extent(swapchain.extent)
                .build();
        
        let viewport = ash::vk::Viewport::builder()
            .x(0f32)
            .y(0f32)
            .width(swapchain.extent.width as f32)
            .height(swapchain.extent.height as f32)
            .min_depth(0f32)
            .max_depth(1f32)
            .build();
        
        let clear_color = [
            vk::ClearValue{
                color: vk::ClearColorValue{float32:[0.0f32; 4]},
            },
            vk::ClearValue{
                depth_stencil: vk::ClearDepthStencilValue::builder()
                    .depth(1f32)
                    .stencil(0)
                    .build()
            },
        ];
        
        let render_pass_begin = vk::RenderPassBeginInfo::builder()
            .render_pass(render_pass.as_inner())
            .framebuffer(swapchain.framebuffers[image_index_usize])
            .render_area(scissor)
            .clear_values(&clear_color[..]);
        
        
        //initialize the command buffer
        unsafe{device.cmd_begin_render_pass(self.buffer[frame_index], &render_pass_begin, vk::SubpassContents::INLINE)};
        //bind command buffer to graphics pipeline
        
        unsafe{device.cmd_bind_pipeline(self.buffer[frame_index], vk::PipelineBindPoint::GRAPHICS, pipeline.as_inner())};
        
        
        unsafe{device.cmd_set_viewport(self.buffer[frame_index], 0, from_ref(&viewport))};
        unsafe{device.cmd_set_scissor(self.buffer[frame_index], 0, from_ref(&scissor))};
        unsafe{device.cmd_bind_descriptor_sets(self.buffer[frame_index], vk::PipelineBindPoint::GRAPHICS, pipeline.layout, 0, from_ref(&descriptor_control.sets[frame_index]), &[])};
        
        /*
        unsafe{device.cmd_bind_vertex_buffers(self.buffer[frame_index], 0, from_ref(&vertex_buffer.buffer), &[0])};
        unsafe{device.cmd_bind_index_buffer(self.buffer[frame_index], index_buffer.buffer, 0, vk::IndexType::UINT32)};
        unsafe{device.cmd_draw_indexed(
            self.buffer[frame_index], 
            triangles_to_draw, 
            1, 0, 0, 0
        )};
        */
        
        for model in model_vec {
            let (vertex_buffer, index_buffer, texture_descriptor, index_count) = model.render(state);
            
            unsafe{device.cmd_bind_vertex_buffers(self.buffer[frame_index], 0, from_ref(&vertex_buffer), &[0])};
            unsafe{device.cmd_bind_index_buffer(self.buffer[frame_index], index_buffer, 0, vk::IndexType::UINT32)};
            
            unsafe{device.cmd_bind_descriptor_sets(self.buffer[frame_index], vk::PipelineBindPoint::GRAPHICS, pipeline.layout, 1, from_ref(&texture_descriptor), &[])};
            
            unsafe{device.cmd_draw_indexed(
                self.buffer[frame_index],
                index_count, 
                1, 0, 0, 0
            )};
        }
        
        unsafe{device.cmd_end_render_pass(self.buffer[frame_index])};
        
        unsafe{device.end_command_buffer(self.buffer[frame_index])}.unwrap();
    }
    
}

impl DeviceDestroy for CommandControl {
    fn device_drop(&mut self, state:&State, device:&Device) {
        if state.v_nor() {
            println!("[0]:deallocating command buffer");
        }
        if state.v_nor() {
            println!("[0]:deleting command pool");
        }
        unsafe{device.destroy_command_pool(self.pool, None)};
    }
}
