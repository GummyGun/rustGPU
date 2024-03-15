#[cfg(feature = "vulkan")]
mod vk_imgui;
#[cfg(feature = "vulkan")]
pub use vk_imgui::Imgui;

use crate::window::Window;
use crate::graphics::ComputePushConstants;
use crate::graphics::MeshAssetMetadata;

use nalgebra as na;
use na::Vector3;
use arrayvec::ArrayString;


#[derive(Default, Debug)]
pub struct InputData {
    pub background_index: usize,
    pub push_constants: ComputePushConstants,
    
    /*
    pub mesh_index: usize,
    pub perspectives: na::Vector3<f32>,
    */
    
}

const PUSH_CONSTANT_FIELD_TEXT:[&str; 4] = [
    "push_constant 1",
    "push_constant 2",
    "push_constant 3",
    "push_constant 4",
];

const FOV_FIELD_TEXT:[&str; 3] = [
    "near",
    "far",
    "angle",
];

impl Imgui{
    
    pub fn handle_events(
        &mut self,
        window: &Window,
    ) {
        let (context_holder, platform_holder, _ui_data) = self.get_common_mut();
        platform_holder.prepare_frame(context_holder.io_mut(), window.underlying(), &window.event_pump().mouse_state());
    }
    
    pub fn draw_ui(
        &mut self,
        window: &mut Window,
        args: (&[ArrayString<64>], &[MeshAssetMetadata], ),
        parameters: (&mut usize, &mut ComputePushConstants, &mut usize, &mut Vector3<f32>, &mut f32),
    ) {
        
        let (compute_effects_name, mesh_assets_metadata) = args;
        
        let (compute_effect_index, compute_push_constant, mesh_index, near_far, downscale_coheficient) = parameters;
        
        let (context, platform, ui_data) = self.get_common_mut();
        let ui = context.new_frame();
        
        let _background = ui.window("Background").build(||{
            let _disabled_token = ui.begin_disabled(false);
            ui.text("Compute shader");
            
            for (index, effect) in compute_effects_name.into_iter().enumerate() {
                ui.radio_button(effect, compute_effect_index, index);
            }
            
            //ui_data.push_constants = compute_effect_metadata[ui_data.background_index].data.clone();
            
            for (index, line) in compute_push_constant.0.iter_mut().enumerate() {
                let _ = ui.input_float4(PUSH_CONSTANT_FIELD_TEXT[index], line)
                    .build();
            }
            ui.text("Dangerous button");
            
        });
        
        let _global = ui.window("Global").build(||{
            let _disabled_token = ui.begin_disabled(false);
            
            ui.text("Render scale");
            ui.slider("Scale", 0.1, 1.0, downscale_coheficient);
            
        });
        
        let _model = ui.window("Model").build(||{
            let _disabled_token = ui.begin_disabled(false);
            ui.text("Select Model");
            for (index, mesh) in mesh_assets_metadata.into_iter().enumerate() {
                ui.radio_button(mesh.name, mesh_index, index);
            }
        });
        
        let _window = ui.window("Field of View(FOV)").build(||{
            
            let _disabled_token = ui.begin_disabled(false);
            ui.text("graphics");
            
            //ui_data.push_constants = compute_effect_metadata[ui_data.background_index].data.clone();
            let _ = ui.slider(FOV_FIELD_TEXT[0], 0.0, 10000.0, &mut near_far[0]);
            let _ = ui.slider(FOV_FIELD_TEXT[1], 0.0, 10.0, &mut near_far[1]);
            let _ = ui.slider(FOV_FIELD_TEXT[2], 1.0, 180.0, &mut near_far[2]);
            ui.text("Dangerous button");
            
        });
        /*
        */
        
        platform.prepare_render(&ui, window.underlying());
    }
}

/*
impl InputData {
    pub fn initial_data() -> Self {
        InputData{
            background_index: 0,
            push_constants:ComputePushConstants([
            Vector4::new(3.1, 3.2, 0.4 ,0.97),
            Vector4::new(0.0, 0.0, 0.0 ,0.0),
            Vector4::new(0.0, 0.0, 0.0 ,0.0),
            Vector4::new(0.0, 0.0, 0.0 ,0.0),
            ]),
        }
    }
}
*/
