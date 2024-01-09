use ash::{
    vk,
    prelude::VkResult,
};

use super::{
    //memory,
    DeviceDestroy,
    device::Device,
    p_device::PDevice,
    //command::CommandControl,
    //buffers::Buffer,
};


use crate::{
    State,
    //errors::Error as AAError,
};

pub struct Sampler{
    pub sampler: vk::Sampler,
}

impl Sampler{
    pub fn create(
        state: &State,
        p_device: &PDevice,
        device: &Device,
    ) -> VkResult<Self> {
        if state.v_exp() {
            println!("\nCREATING:\tSAMPLER");
        }
        let create_info = vk::SamplerCreateInfo::builder()
            .mag_filter(vk::Filter::LINEAR)
            .min_filter(vk::Filter::LINEAR)
            .address_mode_u(vk::SamplerAddressMode::REPEAT)
            .address_mode_v(vk::SamplerAddressMode::REPEAT)
            .address_mode_w(vk::SamplerAddressMode::REPEAT)
            .anisotropy_enable(true)
            .max_anisotropy(p_device.properties.limits.max_sampler_anisotropy)
            .unnormalized_coordinates(false)
            .compare_enable(false)
            .compare_op(vk::CompareOp::ALWAYS)
            .mipmap_mode(vk::SamplerMipmapMode::LINEAR)
            .mip_lod_bias(0f32)
            .min_lod(0f32)
            .max_lod(0f32);
        
        let sampler = unsafe{device.create_sampler(&create_info, None)}?;
        
        Ok(Self{
            sampler:sampler
        })
    }
}

impl DeviceDestroy for Sampler {
    fn device_drop(&mut self, state:&State, device:&Device) {
        if state.v_nor() {
            println!("[0]:deleting sampler");
        }
        unsafe{device.destroy_sampler(self.sampler, None)};
    }
}
