use super::DrawContext;
use super::VkGeoSurface;
use super::VkMeshBuffers;
use super::VkMeshAsset;
use super::MaterialInstance;


use std::rc::Rc;

use nalgebra as na;
use arrayvec::ArrayString;
use derivative::Derivative;
use ash::vk;

#[derive(Derivative)]
#[derivative(Debug)]
pub struct RenderObject {
    pub index_count: u32,
    pub first_index: u32,
    
    pub index_buffer: vk::Buffer,
    pub vertex_buffer_address: vk::DeviceAddress,
    
    #[derivative(Debug="ignore")]
    pub material: Option<MaterialInstance>,
    
    pub transform: na::Matrix4<f32>,
}


pub enum RenderableNode {
    Node(Node),
    MeshNode(MeshNode),
}


struct Node {
    world_transform: na::Matrix4<f32>,
    sons: Vec<Rc<RenderableNode>>,
}

struct MeshNode {
    mesh: Rc<VkMeshAsset>,
    world_transform: na::Matrix4<f32>,
    sons: Vec<Rc<RenderableNode>>,
}

#[derive(Debug, Default)]
pub struct MeshAssetMetadata {
    pub name: ArrayString<64>,
    pub surfaces: Vec<VkGeoSurface>,
    pub meshes: Vec<VkMeshBuffers>,
}



// base class for a renderable dynamic object
pub trait IRenderable {
    fn draw(&self, top_matrix:&na::Matrix4<f32>, ctx:&mut DrawContext);
}

impl IRenderable for VkMeshAsset {
    fn draw(&self, top_matrix:&na::Matrix4<f32>, ctx:&mut DrawContext) {
        
        for (index, geo_surface) in self.surfaces.iter().enumerate() {
            let first_index = geo_surface.start_index;
            let index_count = geo_surface.count;
            let index_buffer = self.meshes[index].index_buffer.underlying();
            let vertex_buffer_address = self.meshes[index].vertex_buffer_address;
            let material = geo_surface.material.clone();
            
            let render_object_holder = RenderObject{
                first_index,
                index_count,
                index_buffer,
                vertex_buffer_address,
                material,
                transform: top_matrix.clone(),
            };
            ctx.push(render_object_holder);
        }
    }
}

impl IRenderable for Node {
    fn draw(&self, top_matrix:&na::Matrix4<f32>, ctx:&mut DrawContext) {
        panic!();
    }
}


impl IRenderable for MeshNode {
    fn draw(&self, top_matrix:&na::Matrix4<f32>, ctx:&mut DrawContext) {
        let node_matrix = top_matrix * self.world_transform;
        self.mesh.draw(top_matrix, ctx);
        for son in &self.sons{
            son.draw(top_matrix, ctx);
        }
    }
}

impl RenderableNode {
    fn unwrap(&self) -> &dyn IRenderable {
        match self {
            RenderableNode::Node(node) => node,
            RenderableNode::MeshNode(node) => node,
        }
    }
}

impl IRenderable for RenderableNode {
    fn draw(&self, top_matrix:&na::Matrix4<f32>, ctx:&mut DrawContext) {
        self.unwrap().draw(top_matrix, ctx);
    }
}


