mod types;
mod model;
pub use model::Model;

use ash::vk;

use super::{
    VInit,
};


use crate::{
    graphics::{
        UniformBufferObject,
    },
};


use std::{
    slice::from_ref,
};


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
            //&self.vertex_buffer, 
            //&self.index_buffer, 
            &self.descriptor_control,
            image_index, 
            self.current_frame,
            //u32::try_from(self.model.indices.len()).unwrap(),
            &self.model_vec,
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
        let delta = self.state.secs_from_start();
        
        
        let rotation:f32 = na::RealField::frac_pi_4();
        let rotation:f32 = rotation * delta;
        let axis = na::Vector3::<f32>::new(0.0,0.0,1.0);
        let norm_axis = na::Unit::new_normalize(axis);
        let rotation_mat = na::Matrix4::from_axis_angle(&norm_axis, rotation);
        /*
        
        let quat = na::Unit::from_axis_angle(&norm_axis, rotation);
        println!("quat\t{:?}", quat);
        
        let mat_quat = na::Matrix4::from(quat);
        println!("rm  \t{:?}", rotation_mat);
        println!("mq  \t{:?}", mat_quat);
        
        let vector = na::Vector4::<f32>::new(1.0,2.0,3.0,0.0);
        
        let result =  rotation_mat * vector;
        
        println!("{:?}", result);
        
        let quat_conj = quat.conjugate();
        let vector_quat = na::Quaternion::from([1.0f32,2.0,3.0,0.0]);
        
        let result_mq = quat.quaternion() * vector_quat * quat_conj.quaternion();
        
        println!("{:?}", result_mq);
        
        panic!();
        */
        
        let eye = na::Point3::<f32>::new(2.0, 0.0, 2.0);
        let center = na::Point3::<f32>::new(0.0, 0.0, 0.0);
        let up = na::Vector3::<f32>::new(0.0, 0.0, 1.0);
        
        /*
        let eye = na::Point3::<f32>::new(2.0, 2.0, 2.0);
        let helper = na::Vector3::<f32>::new((delta*1.5).sin(), (delta*1.5).cos(), -0.0);
        let center = eye + helper;
        println!("{:?}", center);
        let up = na::Vector3::<f32>::new(0.0, 0.0, 1.0);
        */
        
        let lookat = na::Matrix4::look_at_rh(&eye, &center, &up);
        
        let mut perspective = na::Matrix4::new_perspective(na::RealField::frac_pi_2(), self.swapchain.extent.width as f32/self.swapchain.extent.height as f32, 0.1, 10.0);
        
        //println!("{:?}", perspective);
        
        //println!("{}", self.swapchain.extent.width as f32/self.swapchain.extent.height as f32);
        //println!("{:?}", perspective);
        //let mut perspective = na::Matrix4::new_perspective(1f32, 1.0f32, 0.1, 10.0);
        
        perspective[5] *= -1f32;
        
        let current_ubo = [
            UniformBufferObject{
                model: rotation_mat,
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

