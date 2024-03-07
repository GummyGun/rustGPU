/*
use super::VInit;
use super::Instance;
use super::DMessenger;
use super::Surface;
use super::Device;
use super::Allocator;
use super::Swapchain;
use super::CommandControl;
use super::Image;
use super::DescriptorLayout;
use super::GDescriptorAllocator;
use super::ComputeEffects;
use super::GPipeline;
use super::MeshAssets;
use super::graphics::FramesData;

use super::VkDestructor;
use super::VkDestructorArguments;
use super::VkDestructorType;


type Objects = (
    Instance, 
    Option<DMessenger>,
    Surface,
    Device,
    Allocator,
    Swapchain,
    CommandControl,
    Image,
    Image,
    DescriptorLayout,
    GDescriptorAllocator,
    ComputeEffects,
    GPipeline,
    MeshAssets,
    FramesData,
);

impl VInit {
    
    pub(super) fn destructure(&mut self) -> Objects {
        let instance = self.instance.take(); 
        let messenger = self.messenger.as_mut().map(|messenger|messenger.take());
        let surface = self.surface.take();
        let device = self.device.take();
        let allocator = self.allocator.take();
        let swapchain = self.swapchain.take();
        let command_control = self.command_control.take();
        
        let render_image = self.render_image.take();
        let depth_image = self.depth_image.take();
        
        
        let ds_layout = self.ds_layout.take();
        let ds_pool = self.ds_pool.take();
        
        let compute_effects = self.compute_effects.take();
        let mesh_pipeline = self.mesh_pipeline.take();
        let mesh_assets = self.mesh_assets.take();
        
        let frames_data = self.frames_data.take();
        (
            instance,
            messenger,
            surface,
            device,
            allocator,
            swapchain,
            command_control,
            render_image,
            depth_image,
            ds_layout,
            ds_pool,
            compute_effects,
            mesh_pipeline,
            mesh_assets,
            frames_data,
        )
    }
}
*/
