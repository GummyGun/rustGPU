/*
mod types;
mod model;
pub use model::Model;
*/

//use crate::graphics::UniformBufferObject;
use crate::AAError;
use crate::errors::messanges::SIMPLE_VK_FN;

use super::VInit;
use super::Device;
use super::Allocator;
use super::Image2;



use std::slice::from_ref;

use ash::vk;

pub struct Graphics {
    /*
    render_image: Image,
    render_extent: vk::Extent2D,
    */
    
}

impl Graphics {
    pub fn new(_device:&mut Device, _allocator:&Allocator) -> Result<Self, AAError> {
        
        Ok(Self{})
    }
}


impl VInit {
    
    
//----
    pub fn draw_frame2(&mut self) {
        self.frame_update();
        let cf = self.get_frame();
        let frame_count = self.get_frame_count();
        
        let VInit{render_image, command_control, sync_objects, swapchain, device, ..} = self;
        let cmd = command_control.buffers[cf];
        
        let (image_avaliable_semaphore, render_finished_semaphore, inflight_fence) = sync_objects.get_frame(cf);
        let (p_image_handle, image_index) = swapchain.get_next_image(image_avaliable_semaphore);
        
        unsafe{device.wait_for_fences(from_ref(&inflight_fence), true, u64::MAX)}.expect(SIMPLE_VK_FN);
        unsafe{device.reset_fences(from_ref(&inflight_fence))}.expect(SIMPLE_VK_FN);
        unsafe{device.reset_command_buffer(cmd, vk::CommandBufferResetFlags::empty())}.expect(SIMPLE_VK_FN);
        unsafe{device.reset_command_buffer(cmd, vk::CommandBufferResetFlags::empty())}.expect(SIMPLE_VK_FN);
        
        let begin_info = vk::CommandBufferBeginInfo::builder()
            .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);
        
        unsafe{device.begin_command_buffer(cmd, &begin_info)}.expect(SIMPLE_VK_FN);
        
        
        let r_image_handle = render_image.underlying();
        
        Image2::transition_image(device, cmd, r_image_handle, vk::ImageLayout::UNDEFINED, vk::ImageLayout::GENERAL);
        
        Self::draw_background(device, cmd, r_image_handle, frame_count);
        
        Image2::transition_image(device, cmd, r_image_handle, vk::ImageLayout::GENERAL, vk::ImageLayout::TRANSFER_SRC_OPTIMAL);
        Image2::transition_image(device, cmd, p_image_handle, vk::ImageLayout::UNDEFINED, vk::ImageLayout::TRANSFER_DST_OPTIMAL);
        
        Image2::raw_copy_image_to_image(device, cmd, r_image_handle, render_image.extent, p_image_handle, vk::Extent3D::from(swapchain.extent));
        
        Image2::transition_image(device, cmd, p_image_handle, vk::ImageLayout::TRANSFER_DST_OPTIMAL, vk::ImageLayout::PRESENT_SRC_KHR);
        
        
        unsafe{device.end_command_buffer(cmd)}.expect(SIMPLE_VK_FN);
        
        
        let wait_semaphore_submit_info = vk::SemaphoreSubmitInfo::builder()
            .stage_mask(vk::PipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT)
            .semaphore(image_avaliable_semaphore);
        
        let signal_semaphore_submit_info = vk::SemaphoreSubmitInfo::builder()
            .stage_mask(vk::PipelineStageFlags2::ALL_GRAPHICS)
            .semaphore(render_finished_semaphore);
        
        let command_submit_info = vk::CommandBufferSubmitInfo::builder()
            .command_buffer(cmd);
        
        let submit_info = vk::SubmitInfo2::builder()
            .command_buffer_infos(from_ref(&command_submit_info))
            .wait_semaphore_infos(from_ref(&wait_semaphore_submit_info))
            .signal_semaphore_infos(from_ref(&signal_semaphore_submit_info));
        
        unsafe{device.queue_submit2(device.queue_handles.graphics, from_ref(&submit_info), inflight_fence)}.expect(SIMPLE_VK_FN);
        
        let present_info = vk::PresentInfoKHR::builder()
            .swapchains(from_ref(&swapchain.swapchain))
            .image_indices(from_ref(&image_index))
            .wait_semaphores(from_ref(&render_finished_semaphore));
        
        
        unsafe{swapchain.queue_present(device.queue_handles.presentation, &present_info)}.expect(SIMPLE_VK_FN);
        
    }
    
//----
    pub fn draw_background(device:&mut Device, cmd:vk::CommandBuffer, image:vk::Image, frame:usize) {
        
        //let rgba_component = (self.get_frame_count()%100) as f32/100 as f32;
        let rgba_component = (frame%100) as f32/100 as f32;
        
        let clear_color = vk::ClearColorValue{float32:[rgba_component; 4]};
        
        let subresource = Self::subresource_range(vk::ImageAspectFlags::COLOR);
        
        unsafe{device.cmd_clear_color_image(cmd, image, vk::ImageLayout::GENERAL, &clear_color, from_ref(&subresource))};
        
    }
    
//----
    pub fn subresource_range(aspect:vk::ImageAspectFlags) -> vk::ImageSubresourceRange {
        let mut holder = vk::ImageSubresourceRange::default();
        holder.aspect_mask = aspect;
        holder.level_count = vk::REMAINING_MIP_LEVELS;
        holder.layer_count = vk::REMAINING_ARRAY_LAYERS;
        holder
    }
    


    /*
    pub fn draw_frame(&mut self) {
        
        let cf = self.get_frame();
        
        unsafe{self.device.wait_for_fences(from_ref(&self.sync_objects.inflight_fence[cf]), true, u64::MAX)}.expect("waiting for fence should not fail");
        unsafe{self.device.reset_fences(from_ref(&self.sync_objects.inflight_fence[cf]))}.expect("waiting for fence should not fail");
        
        let (image_index, _invalid_surface) = unsafe{
            self.swapchain.acquire_next_image(
                self.swapchain.swapchain, u64::MAX, 
                self.sync_objects.image_available_semaphore[cf], 
                vk::Fence::null()
            )
        }.expect("next image should not fail");
        
        unsafe{self.device.reset_command_buffer(
            self.command_control.buffers[cf], 
            vk::CommandBufferResetFlags::empty())
        }.expect("reseting command should not fail");
        
        self.command_control.record_command_buffer(
            &self.state, 
            &self.device, 
            &self.swapchain, 
            &self.render_pass, 
            &self.pipeline, 
            image_index, 
            self.command_control.buffers[cf],
            from_ref(&self.descriptor_control.sets[cf]),
            &self.model_vec,
        );
        
        let wait_stages = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
        
        let submit_info = 
            vk::SubmitInfo::builder()
                .wait_semaphores(from_ref(&self.sync_objects.image_available_semaphore[cf]))
                .wait_dst_stage_mask(&wait_stages[..])
                .command_buffers(from_ref(&self.command_control.buffers[cf]))
                .signal_semaphores(from_ref(&self.sync_objects.render_finished_semaphore[cf]));
        
        unsafe{self.device.queue_submit(
            self.device.queue_handles.graphics, 
            from_ref(&submit_info), 
            self.sync_objects.inflight_fence[cf]
        )}.expect("should not fail");
        
        let present_info = vk::PresentInfoKHR::builder()
            .wait_semaphores(from_ref(&self.sync_objects.render_finished_semaphore[cf]))
            .swapchains(from_ref(&self.swapchain.swapchain))
            .image_indices(from_ref(&image_index));
        
        unsafe{self.swapchain.queue_present(self.device.queue_handles.presentation, &present_info)}.expect("present should not fail");
        
        self.frame_update();
    }
    */
    
    /*
    pub fn tick(&mut self) {
        use nalgebra as na;
        
        let cf = self.get_frame();
        
        //self.uniform_buffers.update_buffer(&self.state, 0);
        let delta = self.state.secs_from_start();
        
        
        let rotation:f32 = na::RealField::frac_pi_4();
        let rotation:f32 = rotation * delta;
        let axis = na::Vector3::<f32>::new(0.0,0.0,1.0);
        let norm_axis = na::Unit::new_normalize(axis);
        let rotation_mat = na::Matrix4::from_axis_angle(&norm_axis, rotation);
        
        /*
        TODO: DELLETE
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
        
        let result_mq = quat.quaternion() * vector_quat;// * quat_conj.quaternion();
        
        println!("result mq: - : {:?}", result_mq);
        
        panic!();
        */
        
        let eye = na::Point3::<f32>::new(3.0, 0.0, 0.0);
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
        
        self.uniform_buffers.buffers[cf].align.copy_from_slice(&current_ubo[..]);
        
    }
    */
    
    
}

