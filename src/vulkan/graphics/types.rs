use super::MaterialInstance;
use super::GeoSurface;
use super::RenderObject;


use derivative::Derivative;

#[derive(Derivative, Default)]
#[derivative(Debug)]
pub struct VkGeoSurface {
    pub start_index: u32,
    pub count: u32,
    #[derivative(Debug="ignore")]
    pub material: Option<MaterialInstance>,
}

impl From<VkGeoSurface> for GeoSurface {
    fn from(base:VkGeoSurface) -> GeoSurface {
        GeoSurface{
            start_index: base.start_index,
            count: base.count,
        }
    }
}

/*
#[derive(Default, Debug)]
pub struct DrawContext {
    pub context: Vec<RenderObject>,
}
*/

pub type DrawContext = Vec<RenderObject>;


