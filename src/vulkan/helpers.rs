use super::VInit;
use super::Instance;
use super::DMessenger;
use super::Surface;
use super::Device;
use super::Allocator;
use super::Swapchain;
use super::SyncObjects;
use super::CommandControl;
use super::Image;
use super::DescriptorLayoutBuilder;
use super::DescriptorLayout;
use super::DescriptorPoolAllocator;
use super::ComputeEffects;
use super::GPipeline;
use super::MeshAssets;


type Objects = (
    Instance, 
    Option<DMessenger>,
    Surface,
    Device,
    Allocator,
    Swapchain,
    SyncObjects,
    CommandControl,
    Image,
    DescriptorLayoutBuilder,
    DescriptorLayout,
    DescriptorPoolAllocator,
    ComputeEffects,
    GPipeline,
    GPipeline,
    MeshAssets,
);

impl VInit {
    
    pub(super) fn destructure(&mut self) -> Objects {
        let instance = self.instance.take(); 
        let messenger = self.messenger.as_mut().map(|messenger|messenger.take());
        let surface = self.surface.take();
        let device = self.device.take();
        let allocator = self.allocator.take();
        let swapchain = self.swapchain.take();
        let sync_objects = self.sync_objects.take();
        let command_control = self.command_control.take();
        let render_image = self.render_image.take();
        
        
        let ds_layout_builder = self.ds_layout_builder.take();
        let ds_layout = self.ds_layout.take();
        let ds_pool = self.ds_pool.take();
        
        let compute_effects = self.compute_effects.take();
        let graphics_pipeline = self.graphics_pipeline.take();
        let mesh_pipeline = self.mesh_pipeline.take();
        let mesh_assets = self.mesh_assets.take();
        
        (
            instance,
            messenger,
            surface,
            device,
            allocator,
            swapchain,
            sync_objects,
            command_control,
            render_image,
            ds_layout_builder,
            ds_layout,
            ds_pool,
            compute_effects,
            graphics_pipeline,
            mesh_pipeline,
            mesh_assets,
        )
    }
    
}


