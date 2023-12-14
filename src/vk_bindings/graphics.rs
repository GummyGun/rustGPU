use ash::{
    vk,
};

use super::{
    VInit,
};


use crate::{
    graphics::{
        Vertex,
        UniformBufferObject,
    },
};

//use nalgebra as na;

use memoffset::offset_of;

use std::{
    mem::size_of,
};

use std::{
    slice::from_ref,
};

impl Vertex {
    
    pub const fn binding_description() -> &'static[vk::VertexInputBindingDescription] {
        if size_of::<Vertex>() > u32::MAX as usize {
            panic!("Vertex size is too big");
        }
        const HOLDER:[vk::VertexInputBindingDescription; 1] = [
            vk::VertexInputBindingDescription{
                binding: 0,
                stride: size_of::<Vertex>() as u32,
                input_rate: vk::VertexInputRate::VERTEX,
            },
        ];
        &HOLDER
    }
    
    pub const fn attribute_description() -> &'static[vk::VertexInputAttributeDescription] {
        if offset_of!(Vertex, position) > u32::MAX as usize {
            panic!("Vertex size is too big");
        }
        if offset_of!(Vertex, color) > u32::MAX as usize {
            panic!("Vertex size is too big");
        }
        
        const HOLDER:[vk::VertexInputAttributeDescription; 2] = [
            vk::VertexInputAttributeDescription{
                binding: 0,
                location: 0,
                format: vk::Format::R32G32_SFLOAT,
                offset: offset_of!(Vertex, position) as u32
            },
            vk::VertexInputAttributeDescription{
                binding: 0,
                location: 1,
                format: vk::Format::R32G32B32_SFLOAT,
                offset: offset_of!(Vertex, color) as u32
            }
        ];
        &HOLDER
    }
}

impl VInit {
    
    pub fn draw_frame(&mut self) {
        
        let cf = self.current_frame;
        
        unsafe{self.device.wait_for_fences(from_ref(&self.sync_objects.in_flight_fence[cf]), true, u64::MAX)}.expect("waiting for fence should not fail");
        unsafe{self.device.reset_fences(from_ref(&self.sync_objects.in_flight_fence[cf]))}.expect("waiting for fence should not fail");
        
        let (image_index, _invalid_surface) = unsafe{
            self.swapchain.acquire_next_image(
                self.swapchain.swapchain, u64::MAX, 
                self.sync_objects.image_available_semaphore[self.current_frame], 
                vk::Fence::null()
            ).expect("next image should not fail")
        };
        
        unsafe{self.device.reset_command_buffer(self.command_control.buffer[self.current_frame], vk::CommandBufferResetFlags::empty())}.expect("reseting command should not fail");
        
        self.command_control.record_command_buffer(
            &self.state, 
            &self.device, 
            &self.swapchain, 
            &self.render_pass, 
            &self.pipeline, 
            &self.vertex_buffer, 
            &self.index_buffer, 
            &self.descriptor_control,
            image_index, 
            self.current_frame
        );
        
        let wait_stages = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
        
        let submit_info = [
            vk::SubmitInfo::builder()
                //.wait_semaphores(&self.sync_objects.image_available_semaphore[self.frame_range()])
                .wait_semaphores(from_ref(&self.sync_objects.image_available_semaphore[cf]))
                .wait_dst_stage_mask(&wait_stages[..])
                .command_buffers(from_ref(&self.command_control.buffer[cf]))
                .signal_semaphores(from_ref(&self.sync_objects.render_finished_semaphore[cf]))
                .build()
        ];
        
        unsafe{self.device.queue_submit(self.device.queue_handles.graphics, &submit_info[..], self.sync_objects.in_flight_fence[self.current_frame])}.expect("should not fail");
        
        let present_info = vk::PresentInfoKHR::builder()
            .wait_semaphores(from_ref(&self.sync_objects.render_finished_semaphore[cf]))
            .swapchains(from_ref(&self.swapchain.swapchain))
            .image_indices(from_ref(&image_index));
        
        unsafe{self.swapchain.queue_present(self.device.queue_handles.presentation, &present_info)}.expect("present should not fail");
        
        self.frame_update();
    }
    
    pub fn tick(&mut self) {
        use nalgebra as na;
        
        //self.uniform_buffers.update_buffer(&self.state, 0);
        let delta = self.state.time.elapsed().unwrap().as_secs_f32();
        
        
        let rotation:f32 = na::RealField::frac_pi_4();
        let rotation:f32 = rotation * delta;
        let axis = na::Vector3::<f32>::new(0.0,0.0,1.0);
        let norm_axis = na::Unit::new_normalize(axis);
        let rotation = na::Matrix4::from_axis_angle(&norm_axis, rotation);
        
        let eye = na::Point3::<f32>::new(2.0, 2.0, 2.0);
        let center = na::Point3::<f32>::new(0.0, 0.0, 0.0);
        let up = na::Vector3::<f32>::new(0.0, 0.0, 1.0);
        
        let lookat = na::Matrix4::look_at_rh(&eye, &center, &up);
        
        let mut perspective = na::Matrix4::new_perspective(na::RealField::frac_pi_4(), self.swapchain.extent.width as f32/self.swapchain.extent.width as f32, 0.1, 10.0);
        //let mut perspective = na::Matrix4::new_perspective(1f32, 1.0f32, 0.1, 10.0);
        
        perspective[5] *= -1f32;
        
        let current_ubo = [
            UniformBufferObject{
                model: rotation,
                view: lookat,
                proj: perspective,
            },
        ];
        
        self.uniform_buffers.buffers[self.current_frame].align.copy_from_slice(&current_ubo[..]);
    }
    
    
    /*
    pub fn test(&self) {
        //let vector = Vector3::new(1, 2, 3);
        println!("{:?}", size_of::<na::Vector3<f32>>());
        println!("{:?}", size_of::<na::Vector2<f32>>());
        println!("{:?}", size_of::<Vertex>());
        println!("{:?}", size_of::<[Vertex; 3]>());
        println!("{:?}", Vertex::attribute_description());
    }
    */
    
}

