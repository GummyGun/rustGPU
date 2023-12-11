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
    pub p_device: PhysicalDevice,
    pub device: VkObj<Device>,
    pub render_pass: VkObjDevDep<RenderPass>,
    pub swapchain: VkObjDevDep<Swapchain>,
    pub pipeline: VkObjDevDep<Pipeline>,
    pub command_control: VkObjDevDep<CommandControl>,
    pub sync_objects: VkObjDevDep<SyncObjects>,
    pub vertex_buffer: VkObjDevDep<Buffer>,
    pub index_buffer: VkObjDevDep<Buffer>,
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
        let p_device = vk_create_interpreter(state, PhysicalDevice::chose(&state, &instance, &surface), "p_device selected"); 
        let device = vk_create_interpreter(state, Device::create(&state, &instance, &p_device), "device"); 
        let swapchain_basic = vk_create_interpreter(state, SwapchainBasic::create(&state, &window, &instance, &surface, &p_device, &device), "swapchain");
        let render_pass = vk_create_interpreter(state, RenderPass::create(&state, &device, &swapchain_basic), "render_pass");
        let swapchain = vk_create_interpreter(state, Swapchain::complete(&state, &device, swapchain_basic, &render_pass), "framebuffer");
        let pipeline = vk_create_interpreter(state, Pipeline::create(&state, &device, &render_pass), "pipeline");
        let command_control = vk_create_interpreter(state, CommandControl::create(&state, &p_device, &device), "command_control");
        let sync_objects = vk_create_interpreter(state, SyncObjects::create(&state, &device), "sync_objects");
        let vertex_buffer = vk_create_interpreter(state, Buffer::create_vertex(&state, &p_device, &device, &command_control), "vertex_buffer");
        let index_buffer = vk_create_interpreter(state, Buffer::create_index(&state, &p_device, &device, &command_control), "index_buffer");
        
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
        }
    }
    
    pub fn draw_frame(&mut self) {
        
        unsafe{self.device.wait_for_fences(&self.sync_objects.in_flight_fence[self.frame_range()], true, u64::MAX)}.expect("waiting for fence should not fail");
        unsafe{self.device.reset_fences(&self.sync_objects.in_flight_fence[self.frame_range()])}.expect("waiting for fence should not fail");
        
        let (image_index, _) = unsafe{self.swapchain.acquire_next_image(self.swapchain.swapchain, u64::MAX, self.sync_objects.image_available_semaphore[self.current_frame], vk::Fence::null()).expect("next image should not fail")};
        
        unsafe{self.device.reset_command_buffer(self.command_control.buffer[self.current_frame], vk::CommandBufferResetFlags::empty())}.expect("reseting command should not fail");
        self.command_control.record_command_buffer(&self.state, &self.device, &self.swapchain, &self.render_pass, &self.pipeline, &self.vertex_buffer, &self.index_buffer, image_index, self.current_frame);
        
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
        self.current_frame = (self.current_frame + 1) % constants::FIF;
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


