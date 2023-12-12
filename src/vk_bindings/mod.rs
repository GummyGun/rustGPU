use ash::vk;
mod init;
pub use init::*;

mod graphics;

mod objects;
use super::{
    window::{
        Window,
    },
    constants,
    State,
};

use crate::{
    graphics::UniformBufferObject,
};

use objects::{
    VkObj,
    VkObjDevDep,
    DeviceDrop,
    ActiveDrop,
};

use std::{
    ptr::addr_of,
    ops::RangeInclusive,
    slice::from_raw_parts,
};

pub struct VInit {
    state: State,
    current_frame:usize,
    pub instance: VkObj<Instance>,
    pub messenger: Option<VkObj<DMessenger>>,
    pub surface: VkObj<Surface>,
    pub p_device: PDevice,
    pub device: VkObj<Device>,
    pub render_pass: VkObjDevDep<RenderPass>,
    pub swapchain: VkObjDevDep<Swapchain>,
    pub pipeline: VkObjDevDep<Pipeline>,
    pub command_control: VkObjDevDep<CommandControl>,
    pub sync_objects: VkObjDevDep<SyncObjects>,
    pub vertex_buffer: VkObjDevDep<Buffer>,
    pub index_buffer: VkObjDevDep<Buffer>,
    pub uniform_buffers: VkObjDevDep<UniformBuffers>,
    pub descriptor_control: VkObjDevDep<DescriptorControl>,
}


impl VInit {
    pub fn init(state:State, window:&Window) -> VInit {
        
        let instance = vk_create_interpreter(state, Instance::create(&state, window), "instance"); 
        
        let messenger = if constants::VALIDATION {
            Some(match DMessenger::create(&state, &instance) {
                Ok(messenger) => {
                    if state.v_nor() {
                        println!("[0]:messenger");
                    }
                    messenger
                }
                Err(err) => {panic!("{:?}", err);}
            })
        } else {
            println!("[X]:messenger");
            None
        };
        
        let surface =  vk_create_interpreter(state, Surface::create(&state, &window, &instance), "surface"); 
        let p_device = vk_create_interpreter(state, PDevice::chose(&state, &instance, &surface), "p_device selected"); 
        let device = vk_create_interpreter(state, Device::create(&state, &instance, &p_device), "device"); 
        let swapchain_basic = vk_create_interpreter(state, Swapchain::create(&state, &window, &instance, &surface, &p_device, &device), "swapchain");
        let render_pass = vk_create_interpreter(state, RenderPass::create(&state, &device, &swapchain_basic), "render_pass");
        let swapchain = vk_create_interpreter(state, Swapchain::complete(&state, &device, swapchain_basic, &render_pass), "framebuffer");
        
        let layout = vk_create_interpreter(state, DescriptorControl::create(&state, &device), "descriptor_set_layout");
        
        let pipeline = vk_create_interpreter(state, Pipeline::create(&state, &device, &render_pass, &layout), "pipeline");
        let command_control = vk_create_interpreter(state, CommandControl::create(&state, &p_device, &device), "command_control");
        let sync_objects = vk_create_interpreter(state, SyncObjects::create(&state, &device), "sync_objects");
        let vertex_buffer = vk_create_interpreter(state, Buffer::create_vertex(&state, &p_device, &device, &command_control), "vertex_buffer");
        let index_buffer = vk_create_interpreter(state, Buffer::create_index(&state, &p_device, &device, &command_control), "index_buffer");
        let uniform_buffers = vk_create_interpreter(state, UniformBuffers::create(&state, &p_device, &device), "uniform_buffer");
        
        let descriptor_control = vk_create_interpreter(state, DescriptorControl::complete(&state, &device, layout, &uniform_buffers), "descriptor_control");
        
        
        VInit{
            state: state,
            current_frame: 0,
            instance: VkObj::new(instance),
            messenger: match messenger {
                Some(holder) => {Some(VkObj::new(holder))}
                None => None
            },
            p_device: p_device,
            surface: VkObj::new(surface),
            device: VkObj::new(device),
            render_pass: VkObjDevDep::new(render_pass),
            pipeline: VkObjDevDep::new(pipeline),
            swapchain: VkObjDevDep::new(swapchain),
            command_control: VkObjDevDep::new(command_control),
            sync_objects: VkObjDevDep::new(sync_objects),
            vertex_buffer: VkObjDevDep::new(vertex_buffer),
            index_buffer: VkObjDevDep::new(index_buffer),
            uniform_buffers: VkObjDevDep::new(uniform_buffers),
            descriptor_control: VkObjDevDep::new(descriptor_control),
        }
    }
    
    pub fn draw_frame(&mut self) {
        
        unsafe{self.device.wait_for_fences(&self.sync_objects.in_flight_fence[self.frame_range()], true, u64::MAX)}.expect("waiting for fence should not fail");
        unsafe{self.device.reset_fences(&self.sync_objects.in_flight_fence[self.frame_range()])}.expect("waiting for fence should not fail");
        
        let (image_index, _invalid_surface) = unsafe{self.swapchain.acquire_next_image(self.swapchain.swapchain, u64::MAX, self.sync_objects.image_available_semaphore[self.current_frame], vk::Fence::null()).expect("next image should not fail")};
        
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
                .wait_semaphores(&self.sync_objects.image_available_semaphore[self.frame_range()])
                .wait_dst_stage_mask(&wait_stages[..])
                .command_buffers(&self.command_control.buffer[self.frame_range()])
                .signal_semaphores(&self.sync_objects.render_finished_semaphore[self.frame_range()])
                .build()
        ];
        
        unsafe{self.device.queue_submit(self.device.queue_handles.graphics, &submit_info[..], self.sync_objects.in_flight_fence[self.current_frame])}.expect("should not fail");
        
        let swapchain_slice = unsafe{from_raw_parts(addr_of!(self.swapchain.swapchain), 1)};
        let image_index_slice = unsafe{from_raw_parts(addr_of!(image_index), 1)};
        
        let present_info = vk::PresentInfoKHR::builder()
            .wait_semaphores(&self.sync_objects.render_finished_semaphore[self.frame_range()])
            .swapchains(swapchain_slice)
            .image_indices(image_index_slice);
        
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
        
        /*
        println!("{:?}", rotation);
        println!("{:?}", lookat);
        println!("{:?}", perspective);
        */
        //panic!("\n\n\n\n\n\n\n");
        
        /*
        */
        
        //perspective[5] *= -1.0;
        
        /*
        println!("{:?}", perspective);
        */
        
        let current_ubo = [
            UniformBufferObject{
                model: rotation,
                view: lookat,
                proj: perspective,
            },
        ];
        
        self.uniform_buffers.buffers[self.current_frame].align.copy_from_slice(&current_ubo[..]);
        
        /*
        let slice = unsafe{from_raw_parts(self.uniform_buffers.buffers[self.current_frame].map as *const UniformBufferObject, 1)};
        println!("{:#?}", slice);
        */
    }
    
    #[inline(always)]
    pub fn wait_idle(&self) {
        unsafe{self.device.device_wait_idle()}.expect("waiting for iddle should not fail");
    }
    
    #[inline(always)]
    fn frame_range(&self) -> RangeInclusive<usize> {
        self.current_frame..=self.current_frame
    }
    
    #[inline(always)]
    fn frame_update(&mut self) {
        use constants::fif;
        self.current_frame = (self.current_frame + 1) % fif::USIZE;
    }
    
}



#[inline]
fn vk_create_interpreter<T, A:std::fmt::Debug>(state:State, result:Result<T, A>, name:&'static str) -> T {
    match result {
        Ok(device) => {
            if state.v_nor() {
                println!("[0]:{}", name);
            }
            device
        }
        Err(err) => {panic!("error in {} {:?}", name, err);}
    }
}

impl Drop for VInit {
    fn drop(&mut self) {
        
        self.descriptor_control.device_drop(&self.state, &self.device);
        self.uniform_buffers.device_drop(&self.state, &self.device);
        self.index_buffer.device_drop(&self.state, &self.device);
        self.vertex_buffer.device_drop(&self.state, &self.device);
        self.sync_objects.device_drop(&self.state, &self.device);
        self.command_control.device_drop(&self.state, &self.device);
        self.pipeline.device_drop(&self.state, &self.device);
        self.render_pass.device_drop(&self.state, &self.device);
        self.swapchain.device_drop(&self.state, &self.device);
        self.device.active_drop(&self.state);
        self.surface.active_drop(&self.state);
        
        match &mut self.messenger {
            Some(ref mut messenger) => {
                messenger.active_drop(&self.state);
            }
            None => {
                if self.state.v_nor() {
                    println!("No Messenger to delete");
                }
            }
        }
        
        self.instance.active_drop(&self.state);
    }
}


