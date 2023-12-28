use nalgebra::{
    Vector3,
    Point3,
};

#[allow(dead_code)]
pub struct Camera {
    pub location: Point3<f32>,
    pub direction: Vector3<f32>,
    
}

#[allow(dead_code)]
pub enum Movement {
    Foreward,
    Backward,
    Left,
    Rigth,
    Up,
    Down,
}


