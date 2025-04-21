mod mesh;
pub use mesh::load_gltf;
pub use mesh::VkMeshBuffers;
pub use mesh::VkMeshAsset;
pub use mesh::VkMeshAssets;

mod frame;
pub use frame::FramesData;

mod r_object;
pub use r_object::RenderObject;
pub use r_object::IRenderable;

mod types;
pub use types::*;

use crate::AAError;
use crate::gui::Gui;
use crate::errors::messages::SIMPLE_VK_FN;
use crate::errors::messages::COMPILETIME_ASSERT;
use crate::errors::messages::CPU_ACCESIBLE;

pub use crate::graphics::GeoSurface;
pub use crate::graphics::ComputePushConstants;
pub use crate::graphics::Vertex;
pub use crate::graphics::GPUSceneData;


use super::VkDestructor;
use super::VkDeferedDestructor;
use super::VkDestructorArguments;
use super::VInit;
use super::Device;
use super::Allocator;
use super::Buffer;
use super::Image;
use super::CPipeline;
use super::pipeline;
use super::image;

use super::DescriptorWriter;

use super::materials::MaterialInstance;

use std::slice::from_ref;
use std::mem::size_of;
use std::cmp::min;

use memoffset::offset_of;
use ash::vk;
use nalgebra as na;
use nalgebra_glm as glm;
use na::Matrix4;
use gpu_allocator as gpu_all;


#[repr(C)]
#[derive(Debug, Clone)]
pub struct GPUDrawPushConstants {
    world_matrix: Matrix4<f32>,
    vertex_buffer: vk::DeviceAddress,
}


pub struct Canvas {
    render_image: Image,
    depth_image: Image,
}

impl Canvas {
    pub fn new(device:&mut Device, allocator:&mut Allocator, extent:vk::Extent3D) -> Result<Self, AAError> {
        
        let render_image = Image::create(device, allocator, extent, image::RENDER, None)?;
        
        let depth_image = Image::create(device, allocator, extent, image::DEPTH, None)?;
        
        Ok(Self{
            render_image,
            depth_image,
        })
    }
    
    pub fn get_images(&mut self) -> (&mut Image, &mut Image) {
        (&mut self.render_image, &mut self.depth_image)
    }
    
    pub fn get_color(&self) -> &Image {
        &self.render_image
    }
    
    pub fn get_formats(&self) -> (vk::Format, vk::Format) {
        (self.render_image.format, self.depth_image.format)
    }
    
}

impl VkDestructor for Canvas {
    fn destruct(self, mut args:VkDestructorArguments) {
        let (device, allocator) = args.unwrap_dev_all();
        self.render_image.destruct(VkDestructorArguments::DevAll(device, allocator));
        self.depth_image.destruct(VkDestructorArguments::DevAll(device, allocator));
    }
}



impl VInit {
    
    
//----
    pub fn draw_frame(
        &mut self,
        imgui: &mut Gui,
        
    ) {
        self.frame_update();
        let cf = self.get_frame();
        
        let VInit{
            resize_required,
            compute_effects, 
            compute_effect_index, 
            background_image_ds, 
            
            canvas,
            main_draw_context,
            materials,
            
            swapchain, 
            device, 
            allocator,
            
            mesh_assets,
            mesh_index,
            
            field_of_view,
            
            frames_data,
            downscale_coheficient,
            
            error_texture: texture,
            pixelated_sampler,
            texture_descriptor_layout,
            
            gpu_scene_layout,
            scene_data,
            ..
        } = self;
        
        let compute_effect_index = compute_effect_index.clone();
        let cmd = frames_data.get_frame_command_buffer(cf);
        
        let (image_avaliable_semaphore, render_finished_semaphore, inflight_fence) = frames_data.get_frame_sync(cf);
        
        let destruction_stack = frames_data.get_destruction_stack(cf);
        
        unsafe{device.wait_for_fences(from_ref(&inflight_fence), true, u64::MAX)}.expect(SIMPLE_VK_FN);
        
        destruction_stack.dispatch(device, allocator);
        main_draw_context.clear();
        
        let mut gpu_scene_buffer = Buffer::create(device, allocator, Some("per_frame_buffer"), GPUSceneData::size_u64(), vk::BufferUsageFlags::UNIFORM_BUFFER, gpu_all::MemoryLocation::CpuToGpu).unwrap();//TODO:changet this unwrap
        {
            let mut align = gpu_scene_buffer.get_align::<GPUSceneData>(0, GPUSceneData::size_u64()).expect(CPU_ACCESIBLE);
            align.copy_from_slice(from_ref(&scene_data));
        }
        destruction_stack.push(gpu_scene_buffer.defered_destruct());
        
        let descriptor_allocator = frames_data.get_descriptor_allocator(cf);
        descriptor_allocator.clear_pools(device);
        let scene_descriptor = descriptor_allocator.allocate(device, gpu_scene_layout).unwrap();
        
        let mut writer = DescriptorWriter::default();
        writer.write_buffer(0, gpu_scene_buffer.underlying(), GPUSceneData::size_u64(), 0, vk::DescriptorType::UNIFORM_BUFFER);
        writer.update_set(device, scene_descriptor);
        
        
        let (p_image_handle, p_image_view, image_index) = match swapchain.get_next_image(image_avaliable_semaphore){
            Ok(holder) => {holder}
            Err(()) => {
                *resize_required = true;
                return;
            }
        };
        
        unsafe{device.reset_fences(from_ref(&inflight_fence))}.expect(SIMPLE_VK_FN);
        unsafe{device.reset_command_buffer(cmd, vk::CommandBufferResetFlags::empty())}.expect(SIMPLE_VK_FN);
        
        let begin_info = vk::CommandBufferBeginInfo::builder()
            .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);
        
        unsafe{device.begin_command_buffer(cmd, &begin_info)}.expect(SIMPLE_VK_FN);
        
        let (render_image, depth_image) = canvas.get_images();
        
        let r_image_handle = render_image.underlying();
        let d_image_handle = depth_image.underlying();
        
        
        let extent = Self::calculate_extent(render_image.extent_2d, swapchain.extent, *downscale_coheficient);
        
        Image::transition_image(device, cmd, r_image_handle, vk::ImageLayout::UNDEFINED, vk::ImageLayout::GENERAL);
        
        Self::draw_background(device, cmd, render_image, *background_image_ds, &compute_effects.pipelines[compute_effect_index], &compute_effects.push_constants[compute_effect_index]);
        
        Image::transition_image(device, cmd, d_image_handle, vk::ImageLayout::UNDEFINED, vk::ImageLayout::DEPTH_ATTACHMENT_OPTIMAL);
        Image::transition_image(device, cmd, r_image_handle, vk::ImageLayout::GENERAL, vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL);
        
        let default_material = materials.get_default();
        mesh_assets[*mesh_index].draw(&na::Matrix4::<f32>::identity(), main_draw_context);
        //mesh_assets[2].draw(&(na::Matrix4::<f32>::identity().append_translation(&na::Vector3::new(1.3,0.4,0.0))), main_draw_context);
        /*
        mesh_assets[*mesh_index].draw(&(na::Matrix4::<f32>::identity().append_translation(&na::Vector3::new(-1.0,-1.0,-1.0))), main_draw_context);
        mesh_assets[*mesh_index].draw(&(na::Matrix4::<f32>::identity().append_translation(&na::Vector3::new(1.0,-1.0,-1.0))), main_draw_context);
        mesh_assets[*mesh_index].draw(&(na::Matrix4::<f32>::identity().append_translation(&na::Vector3::new(1.0,1.0,-1.0))), main_draw_context);
        */
        //Self::draw_geometry(device, cmd, extent, canvas, mesh_assets, *mesh_index, field_of_view, main_draw_context, default_material, scene_descriptor);
        Self::draw_geometry(device, cmd, extent, canvas, /*mesh_assets, *mesh_index, */field_of_view, main_draw_context, default_material, scene_descriptor);
        
        Image::transition_image(device, cmd, r_image_handle, vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL, vk::ImageLayout::TRANSFER_SRC_OPTIMAL);
        Image::transition_image(device, cmd, p_image_handle, vk::ImageLayout::UNDEFINED, vk::ImageLayout::TRANSFER_DST_OPTIMAL);
        
        Image::raw_copy_image_to_image(device, cmd, r_image_handle, vk::Extent3D::from(extent), p_image_handle, vk::Extent3D::from(swapchain.extent));
        
        Image::transition_image(device, cmd, p_image_handle, vk::ImageLayout::TRANSFER_DST_OPTIMAL, vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL);
        
        imgui.render(device, cmd, swapchain.extent, p_image_view);
        
        Image::transition_image(device, cmd, p_image_handle, vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL, vk::ImageLayout::PRESENT_SRC_KHR);
        
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
        
        match unsafe{swapchain.queue_present(device.queue_handles.presentation, &present_info)}{
            Ok(_) => {}
            Err(_error) => {
                *resize_required = true;
            }
        }
    }
    
//----
    pub fn draw_background(device:&mut Device, cmd:vk::CommandBuffer, image:&Image, background_image_ds:vk::DescriptorSet, cp_pipeline:&CPipeline, push_constants:&ComputePushConstants) {
        
        unsafe{device.cmd_bind_pipeline(cmd, vk::PipelineBindPoint::COMPUTE, cp_pipeline.pipeline)};
        unsafe{device.cmd_bind_descriptor_sets(cmd, vk::PipelineBindPoint::COMPUTE, cp_pipeline.layout, 0, from_ref(&background_image_ds), &[])};
        
        let push_constants_slice = unsafe{crate::any_as_u8_slice(push_constants)};
        unsafe{device.cmd_push_constants(cmd, cp_pipeline.layout, vk::ShaderStageFlags::COMPUTE, 0, push_constants_slice)};
        
        unsafe{device.cmd_dispatch(cmd, image.extent.width/16+1, image.extent.height/16+1, 1)};
        
    }


//----
    pub fn draw_geometry(
        device: &mut Device, 
        cmd: vk::CommandBuffer, 
        extent: vk::Extent2D, 
        canvas: &mut Canvas,
        /*
        image: &Image, 
        depth: &Image, 
        */
        
        //mesh_pipeline: &GPipeline, 
        
        /*
        meshes: &VkMeshAssets, 
        mesh_selector: usize, 
        */
        
        field_of_view: &na::Vector3<f32>,
        draw_context: &mut DrawContext,
        
        default_material: &MaterialInstance,
        scene_descriptor: vk::DescriptorSet,
        /*
        texture_descriptor_layout: &DescriptorLayout,
        descriptor_allocator: &mut GDescriptorAllocator,
        texture: &Image,
        sampler: &Sampler
        */
    ) {
        
        let (image, depth) = canvas.get_images();
        let color_attachment_info = pipeline::rendering_attachment_info(image.view, None, vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL);
        let depth_attachment_info = pipeline::depth_attachment_info(depth.view, vk::ImageLayout::DEPTH_ATTACHMENT_OPTIMAL);
        let rendering_info = pipeline::rendering_info(extent, &color_attachment_info, Some(&depth_attachment_info));
        
        
        let viewport = vk::Viewport::builder()
            .width(extent.width as f32)
            .height(extent.height as f32)
            .min_depth(0f32)
            .max_depth(1f32);
        
        let scissor = vk::Rect2D::from(extent);
        
        unsafe{device.cmd_begin_rendering(cmd, &rendering_info)};
        //unsafe{device.cmd_bind_pipeline(cmd, vk::PipelineBindPoint::GRAPHICS, mesh_pipeline.underlying())};
        unsafe{device.cmd_set_viewport(cmd, 0, from_ref(&viewport))};
        unsafe{device.cmd_set_scissor(cmd, 0, from_ref(&scissor))};
        
        for render_object in draw_context.iter() {
            
            let material = match render_object.material.as_ref() {
                Some(material) => {material}
                None => {default_material}
            };
            
            unsafe{device.cmd_bind_pipeline(cmd, vk::PipelineBindPoint::GRAPHICS, material.pipeline.underlying())};
            //TODO add cmd_descriptor_1
            unsafe{device.cmd_bind_descriptor_sets(cmd, vk::PipelineBindPoint::GRAPHICS, material.pipeline.layout, 0, from_ref(&scene_descriptor), &[])};
            unsafe{device.cmd_bind_descriptor_sets(cmd, vk::PipelineBindPoint::GRAPHICS, material.pipeline.layout, 1, from_ref(&material.descriptor_set), &[])};
            unsafe{device.cmd_bind_index_buffer(cmd, render_object.index_buffer, 0, vk::IndexType::UINT32)};
            
            let mut push_constant_tmp = GPUDrawPushConstants::default();
            push_constant_tmp.vertex_buffer = render_object.vertex_buffer_address;
            push_constant_tmp.world_matrix = Self::tmp_perspective_matrix(extent, field_of_view)*render_object.transform;
            let push_constants_slice = unsafe{crate::any_as_u8_slice(&push_constant_tmp)};
            
            unsafe{device.cmd_push_constants(cmd, material.pipeline.layout, vk::ShaderStageFlags::VERTEX, 0, push_constants_slice)};
            
            unsafe{device.cmd_draw_indexed(cmd, render_object.index_count, 1, render_object.first_index, 0, 0)};
        }
        
        /*
        let texture_set = descriptor_allocator.allocate(device, texture_descriptor_layout).unwrap();//TODO move this unwrap
        {
            let mut writer = DescriptorWriter::default();
            writer.write_image(0, texture.view, sampler.underlying(), vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL, vk::DescriptorType::COMBINED_IMAGE_SAMPLER);
            writer.update_set(device, texture_set);
        }
        
        unsafe{device.cmd_bind_descriptor_sets(cmd, vk::PipelineBindPoint::GRAPHICS, mesh_pipeline.layout, 0, from_ref(&texture_set), &[])};
        
        let mut push_constant_tmp = GPUDrawPushConstants::default();
        push_constant_tmp.vertex_buffer = meshes.meshes[mesh_selector].vertex_buffer_address;
        push_constant_tmp.world_matrix = Self::tmp_perspective_matrix(extent, field_of_view);
        let push_constants_slice = unsafe{crate::any_as_u8_slice(&push_constant_tmp)};
        
        
        unsafe{device.cmd_push_constants(cmd, mesh_pipeline.layout, vk::ShaderStageFlags::VERTEX, 0, push_constants_slice)};
        unsafe{device.cmd_bind_index_buffer(cmd, meshes.meshes[mesh_selector].index_buffer.underlying(), 0, vk::IndexType::UINT32)};
        unsafe{device.cmd_draw_indexed(cmd, meshes.metadatas[mesh_selector].surfaces[0].count, 1, meshes.metadatas[mesh_selector].surfaces[0].start_index, 0, 0)};
        
        /*
        push_constant_tmp.world_matrix = Self::tmp_perspective_matrix(extent, field_of_view, 1.0);
        let push_constants_slice = unsafe{crate::any_as_u8_slice(&push_constant_tmp)};
        unsafe{device.cmd_push_constants(cmd, mesh_pipeline.layout, vk::ShaderStageFlags::VERTEX, 0, push_constants_slice)};
        unsafe{device.cmd_draw_indexed(cmd, meshes.metadatas[mesh_selector].surfaces[0].count, 1, meshes.metadatas[mesh_selector].surfaces[0].start_index, 0, 0)};
        */
        */
        unsafe{device.cmd_end_rendering(cmd)};
    }
    
//----
    pub fn draw_scene(
        device: &mut Device, 
        cmd: vk::CommandBuffer, 
        extent: vk::Extent2D, 
        canvas: &mut Canvas,
    ) -> Result<(), ()> {
        let (image, depth) = canvas.get_images();
        let color_attachment_info = pipeline::rendering_attachment_info(image.view, None, vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL);
        let depth_attachment_info = pipeline::depth_attachment_info(depth.view, vk::ImageLayout::DEPTH_ATTACHMENT_OPTIMAL);
        let rendering_info = pipeline::rendering_info(extent, &color_attachment_info, Some(&depth_attachment_info));
        
        
        let viewport = vk::Viewport::builder()
            .width(extent.width as f32)
            .height(extent.height as f32)
            .min_depth(0f32)
            .max_depth(1f32);
        
        let scissor = vk::Rect2D::from(extent);
        
        
        Ok(())
    }
    
//----
    pub fn tmp_perspective_matrix(extent:vk::Extent2D, field_of_view:&na::Vector3<f32>) -> na::Matrix4<f32> {
        let mut view = Matrix4::<f32>::identity();
        view.prepend_translation_mut(&na::Vector3::new(-1.5,1.5,-5.0));
        
        //let mut projection = Matrix4::new_perspective(extent.width as f32/extent.height as f32, 70.0/180.0*std::f32::consts::PI, 10000.0, 0.1);       
        let mut projection = glm::perspective_zo(extent.width as f32/extent.height as f32, field_of_view[2]/180.0*std::f32::consts::PI, field_of_view[0], field_of_view[1]);
        
        /*
        let mut projection = Matrix4::new_perspective(extent.width as f32/extent.height as f32, field_of_view[2]/180.0*std::f32::consts::PI, field_of_view[0], field_of_view[1]);
        
        let mut projection2 = nalgebra_glm::perspective_zo(extent.width as f32/extent.height as f32, field_of_view[2]/180.0*std::f32::consts::PI, field_of_view[0], field_of_view[1]);
        
        todo!("\n{:?}\n{:?}\n", projection, projection2);
        */
        
        //let mut projection = Matrix4::new_perspective(extent.width as f32/extent.height as f32, 70.0/180.0*std::f32::consts::PI, 830.3, 1.2);
        
        //let mut projection = Matrix4::new_perspective(extent.width as f32/extent.height as f32, 70.0/180.0*std::f32::consts::PI, 100.0, 1.0);
        /*
         *
        println!("{:?}", projection);
        projection[(2,2)] /= 2.0;
        projection[(2,2)] /= 2.0;
        */
        
        projection[(1,1)] *= -1.0;
        let holder = projection*view;
        
        
        /*
        let test_vec = na::Vector4::new(1.0, 1.0, 1.0, 1.0);
        println!("moved \t{:?}", view*test_vec);
        let test_vec_m = view*test_vec;
        
        panic!("{:?}", projection);
        
        println!(" = FrontUpRigh\t{:?}", projection*test_vec_m);
        */
        
        holder
        /*
        let mut projection = Matrix4::new_perspective(extent.width as f32/extent.height as f32, 70.0/180.0*std::f32::consts::PI, 0.1, 10.0);
        projection
        */
        /*
        panic!("{:?}", field_of_view);
        //field_of_view[(1,1)] *= -1.0;
        //field_of_view.transpose()
        (holder*field_of_view).transpose()
        */
        //nalgebra::Matrix4::new(-0.8526403, -0.5224984, 0.0, 0.0, 0.5224984, -0.8526403, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0).transpose()
        //nalgebra::Matrix4::new(2.3306494, 0.0, 0.0, 0.0, 0.0, 1.8304877, 0.0, 0.0, 0.0, 0.0, -1.020202, -1.0, 0.0, 0.0, -0.20202021, 0.0)
        //let holder = (field_of_view /* holder*/).transpose();
    }
    
    
//----
    pub fn subresource_range(aspect:vk::ImageAspectFlags) -> vk::ImageSubresourceRange {
        let mut holder = vk::ImageSubresourceRange::default();
        holder.aspect_mask = aspect;
        holder.level_count = vk::REMAINING_MIP_LEVELS;
        holder.layer_count = vk::REMAINING_ARRAY_LAYERS;
        holder
    }
    

//----
    pub fn calculate_extent(render_extent:vk::Extent2D, swapchain_extent:vk::Extent2D, downscale_coheficient:f32) -> vk::Extent2D {
        
        let vk::Extent2D{width:render_width, height:render_height} = render_extent;
        let vk::Extent2D{width:swapchain_width, height:swapchain_height} = swapchain_extent;
        let final_height = (min(render_height, swapchain_height) as f32) * downscale_coheficient;
        let final_width = (min(render_width, swapchain_width) as f32) * downscale_coheficient;
        vk::Extent2D{width: final_width as u32, height: final_height as u32}
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
        
        let mut field_of_view = na::Matrix4::new_perspective(na::RealField::frac_pi_2(), self.swapchain.extent.width as f32/self.swapchain.extent.height as f32, 0.1, 10.0);
        
        //println!("{:?}", field_of_view);
        
        //println!("{}", self.swapchain.extent.width as f32/self.swapchain.extent.height as f32);
        //println!("{:?}", field_of_view);
        //let mut field_of_view = na::Matrix4::new_perspective(1f32, 1.0f32, 0.1, 10.0);
        
        field_of_view[5] *= -1f32;
        
        let current_ubo = [
            UniformBufferObject{
                model: rotation_mat,
                view: lookat,
                proj: field_of_view,
            },
        ];
        
        self.uniform_buffers.buffers[cf].align.copy_from_slice(&current_ubo[..]);
        
    }
    */
    
    
}


#[allow(dead_code)]
impl Vertex {
    pub const fn binding_description() -> &'static[vk::VertexInputBindingDescription] {
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
        
        const HOLDER:[vk::VertexInputAttributeDescription; 5] = [
            vk::VertexInputAttributeDescription{
                binding: 0,
                location: 0,
                format: vk::Format::R32G32B32_SFLOAT,
                offset: offset_of!(Vertex, position) as u32
            },
            vk::VertexInputAttributeDescription{
                binding: 0,
                location: 1,
                format: vk::Format::R32_SFLOAT,
                offset: offset_of!(Vertex, uv_x) as u32
            },
            vk::VertexInputAttributeDescription{
                binding: 0,
                location: 2,
                format: vk::Format::R32G32B32_SFLOAT,
                offset: offset_of!(Vertex, normal) as u32
            },
            vk::VertexInputAttributeDescription{
                binding: 0,
                location: 3,
                format: vk::Format::R32_SFLOAT,
                offset: offset_of!(Vertex, uv_y) as u32
            },
            vk::VertexInputAttributeDescription{
                binding: 0,
                location: 4,
                format: vk::Format::R32G32B32A32_SFLOAT,
                offset: offset_of!(Vertex, color) as u32
            },
        ];
        
        &HOLDER
    }

}

impl Default for GPUDrawPushConstants {
    fn default() -> Self {
        Self{
            world_matrix:Matrix4::<f32>::identity(),
            vertex_buffer:vk::DeviceAddress::default(),
        }
    }
}

const _:u32 = GPUDrawPushConstants::size_u32();
impl GPUDrawPushConstants {
    #[allow(dead_code)]
    pub const fn size_u32() -> u32 {
        if size_of::<Self>() > u32::MAX as usize {
            panic!("{}", COMPILETIME_ASSERT);
        }
        size_of::<Self>() as u32
    }
}




