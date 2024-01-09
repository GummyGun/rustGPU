use nalgebra as na;

use na::{
    Vector2,
    Vector3,
};

use super::{
    Model,
    Vertex,
    FileType,
    LoadTransformation,
    LoadSizeTransformation,
};

use crate::{
    State,
    errors::Error as AAError,
};

use std::{
    collections::HashMap,
};

use easy_gltf::model::Vertex as GLTFVertex;


#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
struct DedupHelper {
    pub position: Vector3<i32>,
    pub color: Vector3<i32>,
    pub texcoord: Vector2<i32>,
}

impl Model {
    
    pub fn load(
        state: &State,
        metadata: (&'static str, &'static str, FileType),
        transformation: LoadTransformation,
    ) -> Result<Self, AAError> {
        match metadata.2 {
            FileType::Obj => Self::load_obj(state, metadata.0, metadata.1),
            FileType::Gltf => Self::load_gltf(state, metadata.0, metadata.1, transformation),
        }
    }
    
    pub fn load_gltf(
        state: &State,
        obj_file:&'static str,
        texture_file:&'static str,
        transformation: LoadTransformation,
    ) -> Result<Self, AAError> {
        use image::io::Reader as ImageReader;
        
        if state.v_exp() {
            println!("loading model {} with texture {}", obj_file, texture_file);
            
        }
        
        let scenes = easy_gltf::load(obj_file).expect("Failed to load glTF");
        if scenes.len() > 1 {
            return Err(AAError::ComplexGltf);
        }
        /*
        for scene in &scenes {
            println!(
                "Cameras: #{}  Lights: #{}  Models: #{}",
                scene.cameras.len(),
                scene.lights.len(),
                scene.models.len()
            )
        }
        */
        
        let model = scenes[0].models[0].triangles().unwrap();
        println!("triangle count: {:#?}", model.len());
        
        let iterable = model.iter().map(|a|a.iter()).flatten().map(|raw_vertex|Self::apply_transform(raw_vertex, &transformation));
        let (vertex_vec, index_vec) = Self::dedup_vertices(iterable);//::<std::slice::Iter<'_, u32>, &u32, _>(iterable);
        
        println!("deduped triangle_count: {:#?}", index_vec.len()/3);
        
        let image_holder = ImageReader::open(texture_file).unwrap().decode().map_err(|_| AAError::DecodeError).unwrap().into_rgba8();
        
        Ok(Self{
            vertices: vertex_vec,
            indices: index_vec,
            image: image_holder
        })
    }
    
    pub fn load_obj(
        state:&State, 
        obj_file:&'static str, 
        texture_file:&'static str
    ) -> Result<Self, AAError> {
        use image::io::Reader as ImageReader;
        
        if state.v_exp() {
            println!("loading model {} with texture {}", obj_file, texture_file);
            
        }
        
        let (mut model_vec, _materials) = tobj::load_obj(
            obj_file,
            &tobj::GPU_LOAD_OPTIONS
        )?;
        
        if model_vec.len() != 1 {
            return Err(AAError::ComplexObj);
        }
        
        let model = model_vec.pop().unwrap();
        let mesh = model.mesh;
        
        println!("triangle count: {:#?}", mesh.indices.len());
        
        let iterable = mesh.indices.iter().map(|current_u32|{
            let mut vertex = Vertex::default();
            let current = usize::try_from(*current_u32).unwrap();
            vertex.position = Self::vector3_from_index(&mesh.positions, current);
            vertex.texcoord = Self::vector2_from_index(&mesh.texcoords, current);
            vertex.color = Vector3::new(1f32, 1f32, 1f32);
            vertex
        });
        
        let (vertex_vec, index_vec) = Self::dedup_vertices(iterable);//::<std::slice::Iter<'_, u32>, &u32, _>(iterable);
        println!("deduped triangle_count: {:#?}", index_vec.len()/3);
        
        
        let image_holder = ImageReader::open(texture_file).unwrap().decode().map_err(|_| AAError::DecodeError).unwrap().into_rgba8();
        
        
        if state.v_exp() {
            println!("amount of indices: {}", index_vec.len());
            println!("amount of vertices:  {}", vertex_vec.len());
        }
        
        
        Ok(Self{
            vertices: vertex_vec,
            indices: index_vec,
            image: image_holder
        })
    }
    
    fn dedup_vertices<U,X,T,V>(iter:std::iter::Map<U, T>) -> (Vec<Vertex>, Vec<u32>) 
    where 
        U: Iterator<Item = X>,
        T: FnMut(<U as Iterator>::Item) -> V,
        Vertex: From<V>,
    {
        let mut deduping_vertex_helper:HashMap<DedupHelper, u32> = HashMap::new();
        let mut index_vec:Vec<u32> = Vec::new();
        let mut vertex_vec:Vec<Vertex> = Vec::new();
        
        for vertex in iter {
            let next_len = u32::try_from(vertex_vec.len()).unwrap();
            let vertex = Vertex::from(vertex);
            
            let dedup = DedupHelper::from(&vertex);
            
            match deduping_vertex_helper.get(&dedup) {
                Some(vertex_index) => {
                    index_vec.push(*vertex_index);
                }
                None => {
                    deduping_vertex_helper.insert(dedup, next_len);
                    index_vec.push(next_len);
                    vertex_vec.push(vertex);
                }
            }
        }
        //println!("{} {}", vertex_vec.len(), index_vec.len());
        (vertex_vec, index_vec)
    }
    
    
    #[inline(always)]
    fn vector3_from_index(vec:&Vec<f32>, index:usize) -> Vector3<f32> {
        Vector3::new(
            vec[index*3+0],
            vec[index*3+1],
            vec[index*3+2],
        )
    }
    
    #[inline(always)]
    fn vector2_from_index(vec:&Vec<f32>, index:usize) -> Vector2<f32> {
        
        Vector2::new(
            vec[index*2+0],
            1f32 - vec[index*2+1],
        )
    }
    
    fn apply_transform(vertex:&GLTFVertex, transform:&LoadTransformation) -> Vertex {
        let mut holder = Vertex::from(vertex);
        
        let quaternion = match transform.size {
            Some(LoadSizeTransformation::Enlarge(factor)) => holder.position*factor,   
            Some(LoadSizeTransformation::Shrink(factor)) => holder.position/factor,   
            None => holder.position,
        };
        
        let quat_holder = na::Quaternion::from_parts(0f32, quaternion);
        
        let quat_conj = transform.rotation_translation.rotation().conjugate();
        
        let tmp = transform.rotation_translation.rotation().quaternion() * quat_holder * quat_conj.quaternion();
        
        holder .position = tmp.imag() + transform.rotation_translation.translation().vector;
        
        holder 
    }
    
}

impl From<&Vertex> for DedupHelper {
    fn from(base:&Vertex) -> Self {
        let holder = base.position*10_000_000f32;
        let position = Vector3::new(
            holder[0] as i32,
            holder[1] as i32,
            holder[2] as i32,
        );
        let holder = base.color*10_000_000f32;
        let color = Vector3::new(
            holder[0] as i32,
            holder[1] as i32,
            holder[2] as i32,
        );
        let holder = base.texcoord*10_000_000f32;
        let texcoord = Vector2::new(
            holder[0] as i32,
            holder[1] as i32,
        );
        Self{
            texcoord:texcoord,
            color:color,
            position:position,
        }
    }
}

impl From<&GLTFVertex> for Vertex {
    
    fn from(base:&GLTFVertex) -> Self {
        
        let position = Vector3::new(
            base.position[0],
            base.position[1],
            base.position[2],
        );
        let color = Vector3::new(0f32, 0f32, 0f32);
        let texcoord = Vector2::new(
            base.tex_coords[0],
            base.tex_coords[1],
        );
        Self{
            texcoord:texcoord,
            color:color,
            position:position,
        }
    }

}

