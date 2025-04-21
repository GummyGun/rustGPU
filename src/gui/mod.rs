#[cfg(feature = "vulkan")]
mod vk_gui;
#[cfg(feature = "vulkan")]
pub use vk_gui::Gui;

use crate::window::Window;
use crate::graphics::ComputePushConstants;

use nalgebra as na;
use na::Vector3;



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

impl Gui{
    
    pub fn handle_events(
        &mut self,
        window: &Window,
    ) {
        let (context_holder, platform_holder, _ui_data) = self.get_common_mut();
        platform_holder.prepare_frame(context_holder.io_mut(), window.underlying(), &window.event_pump().mouse_state());
    }
    
    pub fn draw_ui<C, CC:Fn(&C)->&str, D, DD:Fn(&D)->&str>(
        &mut self,
        window: &mut Window,
        args: (&[C], &[D]),
        transform: (CC, DD),
        parameters: (&mut usize, &mut ComputePushConstants, &mut usize, &mut Vector3<f32>, &mut f32),
    ) {
        
        let (compute_effects_name, mesh_assets_metadata) = args;
        let (c_transform, d_transform) = transform;
        let (compute_effect_index, compute_push_constant, mesh_index, near_far, downscale_coheficient) = parameters;
        
        let (context, platform, ui_data) = self.get_common_mut();
        let ui = context.new_frame();
        
        let _background = Self::get_next_window(&ui, "Background", [0,0]).build(||{
            
            let _disabled_token = ui.begin_disabled(false);
            
            ui.text("Compute shader");
            
            for (index, effect) in compute_effects_name.into_iter().enumerate() {
                ui.radio_button(c_transform(effect), compute_effect_index, index);
            }
            
            //ui_data.push_constants = compute_effect_metadata[ui_data.background_index].data.clone();
            
            for (index, line) in compute_push_constant.0.iter_mut().enumerate() {
                let _ = ui.input_float4(PUSH_CONSTANT_FIELD_TEXT[index], line)
                    .build();
            }
            ui.text("Dangerous button");
            
        });
        
        let _global = Self::get_next_window(&ui, "Global", [0,1]).build(||{
            let _disabled_token = ui.begin_disabled(false);
            
            ui.text("Render scale");
            ui.slider("Scale", 0.1, 1.0, downscale_coheficient);
            
        });
        
        let _model = Self::get_next_window(&ui, "Model", [0,2]).build(||{
            let _disabled_token = ui.begin_disabled(false);
            ui.text("Select Model");
            for (index, mesh) in mesh_assets_metadata.into_iter().enumerate() {
                ui.radio_button(d_transform(mesh), mesh_index, index);
            }
        });
        
        let mut _window = Self::get_next_window(&ui, "Field of View(FOV)", [0,3]).build(||{
            let _disabled_token = ui.begin_disabled(false);
            //ui_data.push_constants = compute_effect_metadata[ui_data.background_index].data.clone();
            let _ = ui.slider(FOV_FIELD_TEXT[0], 0.0, 10000.0, &mut near_far[0]);
            let _ = ui.slider(FOV_FIELD_TEXT[1], 0.0, 10.0, &mut near_far[1]);
            let _ = ui.slider(FOV_FIELD_TEXT[2], 1.0, 180.0, &mut near_far[2]);
            ui.text("Dangerous button");
            
        });
        
        platform.prepare_render(&ui, window.underlying());
    }
    

    fn get_next_window<'a>(ui:&'a imgui::Ui, name:&'a str, position:[u8; 2]) -> imgui::Window<'a, 'a, &'a str> {
        let position = [32.0+position[0] as f32*256.0, 32.0+position[1] as f32*32.0];
        ui.window(name)
            .collapsed(true, imgui::Condition::Once)
            .position(position, imgui::Condition::Once)
            .movable(false)
            .size([1.5*256.0, 256.0], imgui::Condition::Once)
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
