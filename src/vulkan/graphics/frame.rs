use crate::constants;
use crate::AAError;

use crate::errors::messages::GRANTED;

use crate::logger;


use super::VkDestructor;
use super::VkDestructorArguments;
use super::super::PDevice;
use super::super::Device;
use super::super::GDescriptorAllocator;
use super::super::DescriptorLayoutBuilder;


use ash::vk;
use arrayvec::ArrayVec;
use derivative::Derivative;

#[derive(Derivative)]
#[derivative(Debug)]
pub struct FrameData {
    pub image_available_semaphore: vk::Semaphore,
    pub render_finished_semaphore: vk::Semaphore,
    pub inflight_fence: vk::Fence,
    
    pub cmd_pool: vk::CommandPool,
    pub cmd_buffer: vk::CommandBuffer,
    
    #[derivative(Debug="ignore")]
    pub descriptor_allocator: GDescriptorAllocator,
}


pub struct FrameDatas (
    [FrameData; constants::fif::USIZE],
);

impl FrameData {
    pub fn create(p_device:&PDevice, device:&mut Device) -> Result<Self, AAError> {
        
        let semaphore_create_info = vk::SemaphoreCreateInfo::builder();
        let fence_create_info = vk::FenceCreateInfo::builder()
            .flags(vk::FenceCreateFlags::SIGNALED);
        
        let image_available_semaphore = unsafe{device.create_semaphore(&semaphore_create_info, None)}?;
        let render_finished_semaphore = unsafe{device.create_semaphore(&semaphore_create_info, None)}?;
        let inflight_fence =  unsafe{device.create_fence(&fence_create_info, None)}?;
        
        let create_info = vk::CommandPoolCreateInfo::builder()
            .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
            .queue_family_index(p_device.queues.graphics_family);
        
        let cmd_pool = unsafe{device.create_command_pool(&create_info, None)}?;
        
        let create_info = vk::CommandBufferAllocateInfo::builder()
            .command_pool(cmd_pool)
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_buffer_count(1);
        
        let buffer_vec = unsafe{device.allocate_command_buffers(&create_info)}?;
        let cmd_buffer = buffer_vec[0];
        
        let mut ds_layout_builder = DescriptorLayoutBuilder::create().unwrap();
        ds_layout_builder.add_binding(0, vk::DescriptorType::STORAGE_IMAGE, 3);
        ds_layout_builder.add_binding(0, vk::DescriptorType::STORAGE_BUFFER, 3);
        ds_layout_builder.add_binding(0, vk::DescriptorType::UNIFORM_BUFFER, 3);
        ds_layout_builder.add_binding(0, vk::DescriptorType::COMBINED_IMAGE_SAMPLER, 4);
        let descriptor_counts = ds_layout_builder.assemble();
        
        let descriptor_allocator:GDescriptorAllocator = GDescriptorAllocator::create(device, descriptor_counts).unwrap();
        
        Ok(Self{
            image_available_semaphore,
            render_finished_semaphore,
            inflight_fence,
            cmd_pool,
            cmd_buffer,
            descriptor_allocator,
        })
    }
}

impl FrameDatas {
    pub fn create(p_device:&PDevice, device:&mut Device) -> Result<Self, AAError> {
        let mut holder:ArrayVec<FrameData, {constants::fif::USIZE}> = ArrayVec::new();
        for index in 0..constants::fif::USIZE {
            let frame_data = FrameData::create(p_device, device)?;
            holder.push(frame_data);
        }
        let holder = holder.into_inner().expect(GRANTED);
        Ok(Self(holder))
    }
}


impl VkDestructor for FrameData {
    fn destruct(self, mut args:VkDestructorArguments) {
        logger::destruct!("frame_data");
        logger::destruct!("command_control");
        let device = args.unwrap_dev();
        unsafe{device.destroy_command_pool(self.cmd_pool, None)};
        
        logger::destruct!("sync_objects");
        unsafe{device.destroy_semaphore(self.image_available_semaphore, None)};
        unsafe{device.destroy_semaphore(self.render_finished_semaphore, None)};
        unsafe{device.destroy_fence(self.inflight_fence, None)};
        self.descriptor_allocator.destruct(VkDestructorArguments::Dev(device));
    }
}

impl VkDestructor for FrameDatas {
    fn destruct(self, mut args:VkDestructorArguments) {
        logger::destruct!("frame_datas");
        let device = args.unwrap_dev();
        for frame_data in self.0 {
            frame_data.destruct(VkDestructorArguments::Dev(device));
        }
    }
}

